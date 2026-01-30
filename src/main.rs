use clap::Parser;
use skill_manager::cli::Cli;

fn main() {
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
