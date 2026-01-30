use clap::Parser;
use skill_manager::cli::Cli;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() {
    // Initialize tracing with RUST_LOG env filter
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    if let Err(e) = cli.run() {
        eprintln!("Error: {}", e);

        // Print error chain
        let mut source = std::error::Error::source(&e);
        while let Some(s) = source {
            eprintln!("  Caused by: {}", s);
            source = std::error::Error::source(s);
        }

        std::process::exit(1);
    }
}
