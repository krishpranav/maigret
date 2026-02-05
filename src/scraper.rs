use crate::core::{ScanResult, SiteData};
use anyhow::Result;
use fancy_regex::Regex;
use reqwest::{Client, Proxy};
use std::time::Duration;
use tracing::{debug, warn};

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.0.4430.93 Safari/537.36";
const TOR_PROXY: &str = "socks5://127.0.0.1:9050";

pub struct Scraper {
    client: Client,
    tor_client: Option<Client>,
}

impl Scraper {
    pub fn new(use_tor: bool) -> Result<Self> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(60))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()?;

        let tor_client = if use_tor {
            let proxy = Proxy::all(TOR_PROXY)?;
            Some(
                Client::builder()
                    .user_agent(USER_AGENT)
                    .timeout(Duration::from_secs(60))
                    .proxy(proxy)
                    .redirect(reqwest::redirect::Policy::limited(10))
                    .build()?,
            )
        } else {
            None
        };

        Ok(Self { client, tor_client })
    }

    pub async fn check_username(
        &self,
        username: &str,
        site: &str,
        data: &SiteData,
        use_tor: bool,
    ) -> ScanResult {
        let mut result = ScanResult::new(username.to_string(), site.to_string());
        result.proxied = use_tor;

        // Build URLs
        let url = data.url.replace("{}", username);
        let url_probe = if !data.url_probe.is_empty() {
            data.url_probe.replace("{}", username)
        } else {
            url.clone()
        };

        result.url = data.url.clone();
        result.url_probe = data.url_probe.clone();

        // Check regex validation
        if !data.regex_check.is_empty() {
            match Regex::new(&data.regex_check) {
                Ok(re) => {
                    if let Ok(is_match) = re.is_match(username) {
                        if !is_match {
                            debug!("Username {} doesn't match regex for {}", username, site);
                            return result.not_found(url);
                        }
                    }
                }
                Err(e) => {
                    warn!("Invalid regex for {}: {}", site, e);
                }
            }
        }

        // Select client
        let client = if use_tor && self.tor_client.is_some() {
            self.tor_client.as_ref().unwrap()
        } else {
            &self.client
        };

        // Make request
        let response = match client.get(&url_probe).send().await {
            Ok(resp) => resp,
            Err(e) => {
                return result.with_error(e.to_string());
            }
        };

        // Check based on error type
        match data.error_type.as_str() {
            "status_code" => {
                if response.status().is_success() {
                    result.found(url.clone(), url.clone())
                } else {
                    result.not_found(url)
                }
            }
            "message" => {
                let body = match response.text().await {
                    Ok(text) => text,
                    Err(e) => return result.with_error(e.to_string()),
                };

                if !body.contains(&data.error_msg) {
                    result.found(url.clone(), url.clone())
                } else {
                    result.not_found(url)
                }
            }
            "response_url" => {
                let final_url = response.url().to_string();
                let status = response.status();

                if status.is_success() && final_url == url {
                    result.found(url.clone(), url.clone())
                } else {
                    result.not_found(url)
                }
            }
            _ => result.with_error(format!("Unsupported error type: {}", data.error_type)),
        }
    }
}

pub async fn check_username_with_retry(
    scraper: &Scraper,
    username: &str,
    site: &str,
    data: &SiteData,
    use_tor: bool,
    max_retries: u32,
) -> ScanResult {
    let mut retries = 0;

    loop {
        let result = scraper.check_username(username, site, data, use_tor).await;

        if !result.error || retries >= max_retries {
            return result;
        }

        retries += 1;
        debug!("Retrying {} (attempt {}/{})", site, retries, max_retries);
        tokio::time::sleep(Duration::from_millis(500 * retries as u64)).await;
    }
}
