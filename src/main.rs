mod chrome;
mod cli;
mod core;
mod downloader;
mod logger;
mod scraper;

use anyhow::Result;
use cli::Cli;
use colored::Colorize;
use core::{filter_sites, load_site_data};
use downloader::DownloaderRegistry;
use logger::Logger;
use scraper::Scraper;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let args = Cli::parse_args();

    // Initialize logging
    logger::init_tracing(args.verbose);

    // Print banner
    print_banner();

    // Handle --download flag with no usernames (list downloaders)
    if args.download && args.usernames.is_empty() {
        let registry = DownloaderRegistry::new();
        println!("List of sites that can download userdata:");
        for site in registry.list_available() {
            if args.no_color {
                println!("[+] {}", site);
            } else {
                println!("[{}] {}", "+".bright_green(), site.bright_white());
            }
        }
        return Ok(());
    }

    // Load site database
    let database = load_site_data(&args.database_path(), args.update).await?;
    info!("Loaded {} sites from database", database.len());

    // Handle test mode
    if args.test {
        run_tests(&args, &database).await?;
        return Ok(());
    }

    // Run normal scan mode
    for username in &args.usernames {
        scan_username(username, &args, &database).await?;
    }

    Ok(())
}

async fn scan_username(username: &str, args: &Cli, database: &core::SiteDatabase) -> Result<()> {
    let logger = Logger::new(args.no_color, args.verbose);
    logger.print_banner(username);

    // Filter sites if specific site requested
    let sites = filter_sites(database, args.site.as_deref());

    if sites.is_empty() {
        logger.print_error("site", "No matching sites found");
        return Ok(());
    }

    // Initialize scraper
    let scraper = Arc::new(Scraper::new(args.tor)?);

    // Initialize Chrome if screenshots enabled
    let chrome = if args.screenshot {
        let mut chrome = chrome::Chrome::new(
            "1024x768".to_string(),
            60,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
        );
        chrome.setup()?;
        Some(Arc::new(chrome))
    } else {
        None
    };

    // Initialize downloader registry
    let downloader_registry = if args.download {
        Some(Arc::new(DownloaderRegistry::new()))
    } else {
        None
    };

    // Create semaphore for concurrency control
    let semaphore = Arc::new(Semaphore::new(args.max_workers()));

    // Track results
    let found_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let start_time = Instant::now();

    // Spawn tasks for each site
    let mut tasks = Vec::new();

    for (site_name, site_data) in sites.iter() {
        let username = username.to_string();
        let site_name = site_name.clone();
        let site_data = site_data.clone();
        let scraper = Arc::clone(&scraper);
        let semaphore = Arc::clone(&semaphore);
        let found_count = Arc::clone(&found_count);
        let chrome = chrome.clone();
        let downloader_registry = downloader_registry.clone();
        let args = args.clone();
        let logger = logger.clone();

        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            let result = scraper::check_username_with_retry(
                &scraper, &username, &site_name, &site_data, args.tor, 2, // max retries
            )
            .await;

            // Print result
            if result.exist {
                found_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                logger.print_found(&site_name, &result.link);

                // Take screenshot if enabled
                if let Some(chrome) = chrome {
                    if let Err(e) =
                        chrome::take_screenshot(&username, &site_name, &result.link, &chrome)
                    {
                        logger
                            .print_warning(&format!("Screenshot failed for {}: {}", site_name, e));
                    }
                }

                // Download content if enabled
                if let Some(registry) = downloader_registry {
                    if let Err(e) = registry.download(&site_name, &result.link, &username).await {
                        logger.print_warning(&format!("Download failed for {}: {}", site_name, e));
                    }
                }
            } else if result.error {
                logger.print_error(&site_name, &result.error_msg);
            } else {
                logger.print_not_found(&site_name);
            }
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    for task in tasks {
        let _ = task.await;
    }

    // Print summary
    let elapsed = start_time.elapsed();
    let found = found_count.load(std::sync::atomic::Ordering::SeqCst);
    logger.print_summary(found, sites.len(), elapsed);

    Ok(())
}

async fn run_tests(args: &Cli, database: &core::SiteDatabase) -> Result<()> {
    let logger = Logger::new(args.no_color, args.verbose);

    logger.print_info("maigret is activated for checking site validity.");

    if args.screenshot {
        logger.print_warning("Taking screenshot is not available in test mode. Aborted.");
        return Ok(());
    }

    let scraper = Arc::new(Scraper::new(args.tor)?);
    let semaphore = Arc::new(Semaphore::new(args.max_workers()));
    let failed_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let mut tasks = Vec::new();

    for (site_name, site_data) in database.iter() {
        let site_name = site_name.clone();
        let site_data = site_data.clone();
        let scraper = Arc::clone(&scraper);
        let semaphore = Arc::clone(&semaphore);
        let failed_count = Arc::clone(&failed_count);
        let logger = logger.clone();
        let use_tor = args.tor;

        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            let used_result = scraper
                .check_username(&site_data.username_claimed, &site_name, &site_data, use_tor)
                .await;

            let unused_result = scraper
                .check_username(
                    &site_data.username_unclaimed,
                    &site_name,
                    &site_data,
                    use_tor,
                )
                .await;

            if used_result.exist && !unused_result.exist {
                // Site works correctly
            } else {
                failed_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                let mut error_msg = String::new();
                if used_result.error {
                    error_msg.push_str(&format!("[{}]", used_result.error_msg));
                }
                if unused_result.error {
                    error_msg.push_str(&format!("[{}]", unused_result.error_msg));
                }

                if !error_msg.is_empty() {
                    logger.print_error(&site_name, &format!("Failed with error {}", error_msg));
                } else {
                    let msg = format!(
                        "Not working ({}: expected true, but {}, {}: expected false, but {})",
                        site_data.username_claimed,
                        used_result.exist,
                        site_data.username_unclaimed,
                        unused_result.exist
                    );
                    logger.print_warning(&format!("{}: {}", site_name, msg));
                }
            }
        });

        tasks.push(task);
    }

    for task in tasks {
        let _ = task.await;
    }

    logger.print_success("Done");

    let failed = failed_count.load(std::sync::atomic::Ordering::SeqCst);
    println!(
        "\nThese {} sites are not compatible with the Sherlock database.",
        failed
    );

    Ok(())
}

fn print_banner() {
    println!(
        r#"
    ███╗   ███╗ █████╗ ██╗ ██████╗ ██████╗ ███████╗████████╗
    ████╗ ████║██╔══██╗██║██╔════╝ ██╔══██╗██╔════╝╚══██╔══╝
    ██╔████╔██║███████║██║██║  ███╗██████╔╝█████╗     ██║   
    ██║╚██╔╝██║██╔══██║██║██║   ██║██╔══██╗██╔══╝     ██║   
    ██║ ╚═╝ ██║██║  ██║██║╚██████╔╝██║  ██║███████╗   ██║   
    ╚═╝     ╚═╝╚═╝  ╚═╝╚═╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝   ╚═╝   
    
    🔎 Professional OSINT Username Scanner - Rust Edition
    "#
    );
}
