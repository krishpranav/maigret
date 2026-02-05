use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "maigret",
    version,
    about = "Professional OSINT Username Scanner",
    long_about = "maigret - User OSINT Across Social Networks.\n\nA powerful tool for investigating usernames across 2000+ social networks and websites."
)]
pub struct Cli {
    #[arg(required = true)]
    pub usernames: Vec<String>,

    #[arg(long = "no-color")]
    pub no_color: bool,

    #[arg(long = "update")]
    pub update: bool,

    #[arg(short = 't', long = "tor")]
    pub tor: bool,

    #[arg(short = 's', long = "screenshot")]
    pub screenshot: bool,

    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    #[arg(short = 'd', long = "download")]
    pub download: bool,

    #[arg(long = "database", value_name = "DATABASE")]
    pub database: Option<String>,

    #[arg(long = "site", value_name = "SITE")]
    pub site: Option<String>,

    #[arg(long = "test")]
    pub test: bool,
}

impl Cli {
    pub fn parse_args() -> Self {
        Cli::parse()
    }

    pub fn max_workers(&self) -> usize {
        if self.screenshot {
            8
        } else {
            32
        }
    }

    pub fn database_path(&self) -> String {
        self.database
            .clone()
            .unwrap_or_else(|| "data.json".to_string())
    }
}
