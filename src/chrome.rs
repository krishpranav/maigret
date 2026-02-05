use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, warn};

#[cfg(feature = "screenshots")]
use headless_chrome::{Browser, LaunchOptionsBuilder};

pub struct Chrome {
    pub resolution: String,
    pub timeout: u64,
    pub user_agent: String,
    path: Option<String>,
}

impl Chrome {
    pub fn new(resolution: String, timeout: u64, user_agent: String) -> Self {
        Self {
            resolution,
            timeout,
            user_agent,
            path: None,
        }
    }

    pub fn setup(&mut self) -> Result<()> {
        self.locate_chrome()?;
        Ok(())
    }

    fn locate_chrome(&mut self) -> Result<()> {
        let paths = vec![
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
            "/usr/bin/google-chrome-stable",
            "/usr/bin/google-chrome",
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
            "C:/Program Files (x86)/Google/Chrome/Application/chrome.exe",
        ];

        for path in paths {
            if Path::new(path).exists() {
                debug!("Found Chrome at: {}", path);

                if self.check_version(path)? {
                    self.path = Some(path.to_string());
                    info!("Using Chrome: {}", path);
                    return Ok(());
                }
            }
        }

        anyhow::bail!(
            "Unable to locate Chrome/Chromium v60+. Please install Google Chrome or specify path."
        )
    }

    fn check_version(&self, chrome_path: &str) -> Result<bool> {
        let output = Command::new(chrome_path)
            .arg("-version")
            .output()
            .context("Failed to check Chrome version")?;

        let version_str = String::from_utf8_lossy(&output.stdout);
        debug!("Chrome version output: {}", version_str);

        let re = regex::Regex::new(r"\d+(\.\d+)+").unwrap();
        if let Some(captures) = re.find(&version_str) {
            let version = captures.as_str();
            debug!("Chrome version: {}", version);

            if let Some(major) = version.split('.').next() {
                if let Ok(major_num) = major.parse::<u32>() {
                    return Ok(major_num >= 60);
                }
            }
        }

        warn!("Could not determine Chrome version");
        Ok(false)
    }

    #[cfg(feature = "screenshots")]
    pub fn screenshot_url(&self, url: &str, output_path: &Path) -> Result<()> {
        info!("Taking screenshot: {} -> {:?}", url, output_path);

        let launch_options = LaunchOptionsBuilder::default()
            .headless(true)
            .window_size(Some((1024, 768)))
            .build()
            .context("Failed to build Chrome launch options")?;

        let browser = Browser::new(launch_options).context("Failed to launch Chrome")?;

        let tab = browser.new_tab().context("Failed to create new tab")?;

        tab.navigate_to(url).context("Failed to navigate to URL")?;

        tab.wait_until_navigated()
            .context("Failed to wait for navigation")?;

        std::thread::sleep(std::time::Duration::from_secs(2));

        let screenshot_data = tab
            .capture_screenshot(
                headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
                None,
                None,
                true,
            )
            .context("Failed to capture screenshot")?;

        std::fs::write(output_path, screenshot_data).context("Failed to write screenshot")?;

        info!("Screenshot saved: {:?}", output_path);
        Ok(())
    }

    #[cfg(not(feature = "screenshots"))]
    pub fn screenshot_url(&self, _url: &str, _output_path: &Path) -> Result<()> {
        anyhow::bail!("Screenshot feature not enabled. Rebuild with --features screenshots")
    }
}

pub fn take_screenshot(username: &str, site: &str, url: &str, chrome: &Chrome) -> Result<()> {
    let folder_path = PathBuf::from("screenshots").join(username);
    std::fs::create_dir_all(&folder_path).context("Failed to create screenshot directory")?;

    let url_parts = reqwest::Url::parse(url)?;
    let host = url_parts.host_str().unwrap_or("unknown");
    let output_path = folder_path.join(format!("{}.png", host));

    chrome.screenshot_url(url, &output_path)?;

    Ok(())
}
