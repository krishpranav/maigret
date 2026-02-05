use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Custom deserializer for errorMsg that can be either a string or array of strings
fn deserialize_error_msg<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        String(String),
        Vec(Vec<String>),
    }

    match StringOrVec::deserialize(deserializer)? {
        StringOrVec::String(s) => Ok(s),
        StringOrVec::Vec(v) => Ok(v.join("|")), // Join multiple error messages with |
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteData {
    #[serde(rename = "errorType")]
    pub error_type: String,

    #[serde(
        rename = "errorMsg",
        default,
        deserialize_with = "deserialize_error_msg"
    )]
    pub error_msg: String,

    pub url: String,

    #[serde(rename = "urlMain")]
    pub url_main: String,

    #[serde(rename = "urlProbe", default)]
    pub url_probe: String,

    #[serde(rename = "errorUrl", default)]
    pub error_url: String,

    #[serde(rename = "username_claimed")]
    pub username_claimed: String,

    #[serde(rename = "username_unclaimed")]
    pub username_unclaimed: String,

    #[serde(rename = "regexCheck", default)]
    pub regex_check: String,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub username: String,
    pub site: String,
    pub url: String,
    pub url_probe: String,
    pub link: String,
    pub exist: bool,
    pub proxied: bool,
    pub error: bool,
    pub error_msg: String,
}

impl ScanResult {
    pub fn new(username: String, site: String) -> Self {
        Self {
            username,
            site,
            url: String::new(),
            url_probe: String::new(),
            link: String::new(),
            exist: false,
            proxied: false,
            error: false,
            error_msg: String::new(),
        }
    }

    pub fn with_error(mut self, error_msg: String) -> Self {
        self.error = true;
        self.error_msg = error_msg;
        self
    }

    pub fn found(mut self, url: String, link: String) -> Self {
        self.exist = true;
        self.url = url;
        self.link = link;
        self
    }

    pub fn not_found(mut self, url: String) -> Self {
        self.exist = false;
        self.url = url;
        self
    }
}

pub type SiteDatabase = HashMap<String, SiteData>;

pub async fn load_site_data(path: &str, update: bool) -> Result<SiteDatabase> {
    if update || !Path::new(path).exists() {
        update_database(path).await?;
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read database file: {}", path))?;

    let data: SiteDatabase =
        serde_json::from_str(&content).with_context(|| "Failed to parse database JSON")?;

    Ok(data)
}

async fn update_database(path: &str) -> Result<()> {
    use colored::Colorize;

    println!(
        "[{}] Update database: {}",
        "!".bright_blue(),
        "Downloading...".bright_yellow()
    );

    let url = "https://raw.githubusercontent.com/sherlock-project/sherlock/master/sherlock/resources/data.json";

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download database: HTTP {}", response.status());
    }

    let content = response.text().await?;

    // Remove existing file if it exists
    if Path::new(path).exists() {
        fs::remove_file(path)?;
    }

    fs::write(path, content)?;

    println!(" [{}]", "Done".green());

    Ok(())
}

pub fn filter_sites(database: &SiteDatabase, site_filter: Option<&str>) -> SiteDatabase {
    if let Some(site_name) = site_filter {
        let site_lower = site_name.to_lowercase();
        database
            .iter()
            .filter(|(name, _)| name.to_lowercase() == site_lower)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    } else {
        database.clone()
    }
}
