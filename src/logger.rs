use colored::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;

#[derive(Clone)]
pub struct Logger {
    pub no_color: bool,
    pub verbose: bool,
    multi_progress: Arc<MultiProgress>,
}

impl Logger {
    pub fn new(no_color: bool, verbose: bool) -> Self {
        Self {
            no_color,
            verbose,
            multi_progress: Arc::new(MultiProgress::new()),
        }
    }

    pub fn print_banner(&self, username: &str) {
        if self.no_color {
            println!("\nInvestigating {} on:", username);
        } else {
            println!("\n🔎 Investigating {} on:", username.bright_green().bold());
        }
    }

    pub fn print_found(&self, site: &str, url: &str) {
        if self.no_color {
            println!("[+] {}: {}", site, url);
        } else {
            println!("[{}] {}: {}", "+".bright_green(), site.bright_white(), url);
        }
    }

    pub fn print_not_found(&self, site: &str) {
        if !self.verbose {
            return;
        }

        if self.no_color {
            println!("[-] {}: Not Found!", site);
        } else {
            println!(
                "[{}] {}: {}",
                "-".bright_red(),
                site,
                "Not Found!".bright_yellow()
            );
        }
    }

    pub fn print_error(&self, site: &str, error: &str) {
        if !self.verbose {
            return;
        }

        if self.no_color {
            println!("[!] {}: ERROR: {}", site, error);
        } else {
            println!(
                "[{}] {}: {}: {}",
                "!".bright_red(),
                site,
                "ERROR".bright_magenta(),
                error.bright_red()
            );
        }
    }

    pub fn print_info(&self, message: &str) {
        if self.no_color {
            println!("[*] {}", message);
        } else {
            println!("[{}] {}", "*".bright_blue(), message);
        }
    }

    pub fn print_success(&self, message: &str) {
        if self.no_color {
            println!("[✓] {}", message);
        } else {
            println!("[{}] {}", "✓".bright_green(), message.bright_white());
        }
    }

    pub fn print_warning(&self, message: &str) {
        if self.no_color {
            println!("[!] {}", message);
        } else {
            println!("[{}] {}", "!".bright_yellow(), message.yellow());
        }
    }

    pub fn create_progress_bar(&self, total: u64, message: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new(total));

        let style = if self.no_color {
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40} {pos}/{len} {msg}")
                .unwrap()
        } else {
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("█▓▒░")
        };

        pb.set_style(style);
        pb.set_message(message.to_string());
        pb
    }

    pub fn print_summary(&self, found: usize, total: usize, elapsed: std::time::Duration) {
        println!();
        if self.no_color {
            println!("═══════════════════════════════════════");
            println!("  SCAN COMPLETE");
            println!("═══════════════════════════════════════");
            println!("  Found:    {}", found);
            println!("  Checked:  {}", total);
            println!("  Time:     {:.2}s", elapsed.as_secs_f64());
            println!("═══════════════════════════════════════");
        } else {
            println!(
                "{}",
                "═══════════════════════════════════════".bright_cyan()
            );
            println!("  {}", "🧠 SCAN COMPLETE".bright_white().bold());
            println!(
                "{}",
                "═══════════════════════════════════════".bright_cyan()
            );
            println!(
                "  {}: {}",
                "Found".bright_green(),
                found.to_string().bright_white().bold()
            );
            println!(
                "  {}: {}",
                "Checked".bright_blue(),
                total.to_string().bright_white()
            );
            println!(
                "  {}: {}",
                "Time".bright_yellow(),
                format!("{:.2}s", elapsed.as_secs_f64()).bright_white()
            );
            println!(
                "{}",
                "═══════════════════════════════════════".bright_cyan()
            );
        }
    }
}

pub fn init_tracing(verbose: bool) {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = if verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();
}
