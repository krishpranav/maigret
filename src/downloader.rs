use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tracing::{info, warn};

type DownloaderFn =
    fn(&str, &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>;

pub struct DownloaderRegistry {
    downloaders: HashMap<String, DownloaderFn>,
}

impl DownloaderRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            downloaders: HashMap::new(),
        };

        registry.register("instagram", download_instagram_wrapper);

        registry
    }

    pub fn register(&mut self, site: &str, downloader: DownloaderFn) {
        self.downloaders.insert(site.to_lowercase(), downloader);
    }

    pub async fn download(&self, site: &str, url: &str, username: &str) -> Result<()> {
        let site_lower = site.to_lowercase();

        if let Some(downloader) = self.downloaders.get(&site_lower) {
            info!("Downloading content from {} for {}", site, username);
            downloader(url, username).await?;
            Ok(())
        } else {
            warn!("No downloader available for {}", site);
            Ok(())
        }
    }

    pub fn list_available(&self) -> Vec<String> {
        self.downloaders.keys().cloned().collect()
    }
}

fn download_instagram_wrapper(
    url: &str,
    username: &str,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> {
    let url = url.to_string();
    let username = username.to_string();
    Box::pin(download_instagram(url, username))
}

async fn download_instagram(url: String, username: String) -> Result<()> {
    let output_dir = PathBuf::from("downloads").join(&username).join("instagram");
    fs::create_dir_all(&output_dir).context("Failed to create download directory")?;

    let api_url = format!("{}?__a=1", url);
    let client = reqwest::Client::new();

    let response = client
        .get(&api_url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .send()
        .await
        .context("Failed to fetch Instagram profile")?;

    if !response.status().is_success() {
        anyhow::bail!("Instagram API returned status: {}", response.status());
    }

    let data: Value = response
        .json()
        .await
        .context("Failed to parse Instagram JSON")?;

    let mut download_urls = Vec::new();

    if let Some(profile_pic) = data
        .get("graphql")
        .and_then(|g| g.get("user"))
        .and_then(|u| u.get("profile_pic_url_hd"))
        .and_then(|p| p.as_str())
    {
        download_urls.push(profile_pic.to_string());
    }

    if let Some(edges) = data
        .get("graphql")
        .and_then(|g| g.get("user"))
        .and_then(|u| u.get("edge_owner_to_timeline_media"))
        .and_then(|e| e.get("edges"))
        .and_then(|e| e.as_array())
    {
        for edge in edges {
            if let Some(node) = edge.get("node") {
                let url = if node
                    .get("is_video")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    node.get("video_url").and_then(|v| v.as_str())
                } else {
                    node.get("display_url").and_then(|v| v.as_str())
                };

                if let Some(url_str) = url {
                    download_urls.push(url_str.to_string());
                }
            }
        }
    }

    let tasks: Vec<_> = download_urls
        .into_iter()
        .enumerate()
        .map(|(i, url)| {
            let output_dir = output_dir.clone();
            tokio::spawn(async move { download_file(&url, &output_dir, i).await })
        })
        .collect();

    for task in tasks {
        if let Err(e) = task.await {
            warn!("Download task failed: {}", e);
        }
    }

    info!("Instagram download complete for {}", username);
    Ok(())
}

async fn download_file(url: &str, output_dir: &PathBuf, index: usize) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;

    let ext = url
        .split('?')
        .next()
        .and_then(|s| s.split('.').last())
        .unwrap_or("jpg");

    let file_path = output_dir.join(format!("{}.{}", index, ext));
    let bytes = response.bytes().await?;

    let mut file = tokio::fs::File::create(&file_path).await?;
    file.write_all(&bytes).await?;

    info!("Downloaded: {:?}", file_path);
    Ok(())
}
