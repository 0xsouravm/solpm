//! # Solana Program Manager (solpm)
//!
//! A command-line interface for managing Solana program dependencies and publishing
//! programs to a registry.

use clap::Parser;

mod cli;
mod commands;
mod error;
mod utils;

use cli::{Cli, Commands};
use utils::{CliStyle, print_banner};

/// Main entry point for the Solana Program Manager CLI application.
/// 
/// This function handles command line argument parsing, displays the banner when appropriate,
/// and routes commands to their respective handlers. It also manages error handling and
/// provides appropriate success/error messages to the user.
#[tokio::main]
async fn main() {
    // Check if help was requested
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) || args.contains(&"help".to_string()) {
        print_banner();
        println!();
    }
    
    let cli = Cli::parse();
    

    let result = match &cli.command {
        Commands::Init { network } => {
            commands::init::init_project(network)
        }
        Commands::Add { package, dev, path, network, codegen } => {
            commands::add::add_program(&package, *dev, path.as_deref(), network, *codegen).await
        }
        Commands::Install { codegen } => {
            commands::install::install_dependencies(*codegen).await
        }
        Commands::Codegen => {
            commands::codegen::generate_typescript_client()
        }
        Commands::Login { token } => {
            commands::auth::login(token.as_deref()).await
        }
        Commands::Logout => {
            commands::auth::logout()
        }
        Commands::Publish { authority_keypair }=> {
            commands::publish::publish_program(authority_keypair.as_deref()).await
        }
    };

    if let Err(e) = result {
            eprintln!("{}", CliStyle::error(&format!("{}", e)));
            std::process::exit(1);
    }
}