//! CLI tool for PII detection and anonymization
//! Replacement for redactctl

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "redact")]
#[command(about = "PII detection and anonymization CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze text for PII entities
    Analyze {
        /// Text to analyze
        text: String,
    },
    /// Anonymize detected PII
    Anonymize {
        /// Text to anonymize
        text: String,
    },
    /// Show version information
    Version,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Analyze { text }) => {
            println!("Analyzing: {}", text);
            println!("(Placeholder - full implementation coming soon)");
        }
        Some(Commands::Anonymize { text }) => {
            println!("Anonymizing: {}", text);
            println!("(Placeholder - full implementation coming soon)");
        }
        Some(Commands::Version) => {
            println!("redact v{}", env!("CARGO_PKG_VERSION"));
        }
        None => {
            println!("Use --help for usage information");
        }
    }
}
