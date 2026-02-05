use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "maigret",
    version,
    about = "Professional OSINT Username Scanner",
    long_about = "maigret - User OSINT Across Social Networks.\n\nA powerful tool for investigating usernames across 2000+ social networks and websites."
)]
pub struct Cli {
    /// One or more usernames to investigate
    #[arg(required = true)]
    pub usernames: Vec<String>,

    /// Disable colored stdout output
    #[arg(long = "no-color")]
    pub no_color: bool,

    /// Update database before run from Sherlock repository
    #[arg(long = "update")]
    pub update: bool,

    /// Use Tor proxy (requires Tor running on 127.0.0.1:9050)
    #[arg(short = 't', long = "tor")]
    pub tor: bool,

    /// Take a screenshot of each matched URL
    #[arg(short = 's', long = "screenshot")]
    pub screenshot: bool,

    /// Verbose output (show not found sites)
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Download the contents of site if available
    #[arg(short = 'd', long = "download")]
    pub download: bool,

    /// Use custom database file
    #[arg(long = "database", value_name = "DATABASE")]
    pub database: Option<String>,

    /// Specific site to investigate
    #[arg(long = "site", value_name = "SITE")]
    pub site: Option<String>,

    /// Run validation tests on all sites
    #[arg(long = "test")]
    pub test: bool,
}

impl Cli {
    pub fn parse_args() -> Self {
        Cli::parse()
    }

    pub fn max_workers(&self) -> usize {
        if self.screenshot {
            8 // Reduce concurrency for screenshot mode
        } else {
            32 // Default high concurrency
        }
    }

    pub fn database_path(&self) -> String {
        self.database.clone().unwrap_or_else(|| "data.json".to_string())
    }
}
