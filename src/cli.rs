//! # Command Line Interface Module
//!
//! This module defines the CLI structure and commands for the Solana Program Manager.
//! It uses the `clap` crate for command-line argument parsing and provides:
//!
//! - Network selection (mainnet/devnet)
//! - All supported subcommands with their options
//! - Help text and examples for each command
//!
//! The CLI supports the following commands:
//! - `init`: Initialize a new Solana project
//! - `add`: Add program dependencies
//! - `install`: Install all dependencies
//! - `codegen`: Generate TypeScript client code
//! - `login`: Authenticate with the registry
//! - `logout`: Clear stored credentials
//! - `publish`: Publish programs to the registry

use clap::{Parser, Subcommand, ValueEnum};

/// Represents the target Solana network for operations.
/// 
/// This enum defines the supported network environments where programs
/// can be published or from which they can be installed.
#[derive(Clone, ValueEnum)]
pub enum Network {
    /// Solana mainnet-beta (production network)
    #[value(name = "mainnet")]
    Main,
    /// Solana devnet (development/testing network)
    #[value(name = "devnet")]
    Dev,
}

/// Main CLI application structure for the Solana Program Manager.
/// 
/// This struct defines the root command structure and global configuration
/// for the solpm CLI application.
#[derive(Parser)]
#[command(name = "solpm")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A Solana program manager for anchor program publishing and management")]
#[command(long_about = "Solana Program Manager (solpm) helps you publish your own Solana programs from GitHub repositories, \ninstall published program as dependencies, and generate TypeScript clients.")]
pub struct Cli {
    /// The subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands for the Solana Program Manager.
/// 
/// This enum defines all the subcommands supported by the solpm CLI tool.
/// Each command corresponds to a specific functionality like initializing projects,
/// adding dependencies, installing programs, or publishing to the registry.
#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new Solana project with package configuration
    #[command(alias = "i")]
    Init {
        /// Target network for the project
        /// 
        /// Examples:
        ///   solpm init --network mainnet
        ///   solpm init --network devnet
        #[arg(long, value_enum, default_value = "devnet")]
        network: Network,
    },
    
    /// Add a program dependency to the current project  
    #[command(alias = "a")]
    Add {
        /// Package specification (name or name@version) to add
        package: String,
        /// Add as development dependency
        /// 
        /// Examples:
        ///   solpm add my-program --dev
        ///   solpm add my-program@1.0.0 --dev --path ./custom/path.json
        #[arg(long)]
        dev: bool,
        /// Custom path for the IDL file
        /// 
        /// Examples:
        ///   solpm add my-program --path ./custom/idl/program.json
        ///   solpm add my-program@1.0.0 --path ./dev/idls/program.json
        #[arg(long)]
        path: Option<String>,
        /// Target network to fetch from
        /// 
        /// Examples:
        ///   solpm add my-program --network devnet
        ///   solpm add my-program@1.0.0 --network mainnet
        #[arg(long, value_enum, default_value = "devnet")]
        network: Network,
        /// Generate TypeScript client code after adding the program
        /// 
        /// Examples:
        ///   solpm add my-program --codegen
        ///   solpm add my-program@1.0.0 --dev --codegen
        #[arg(long)]
        codegen: bool,
    },
    
    /// Install all program dependencies from SolanaPrograms.json
    #[command(alias = "in")]
    Install {
        /// Generate TypeScript client code after installing programs
        /// 
        /// Examples:
        ///   solpm install --codegen
        #[arg(long)]
        codegen: bool,
    },
    
    /// Generate TypeScript client code for installed programs
    #[command(alias = "gen")]
    Codegen,
    
    /// Authenticate with Registry API Token
    #[command(alias = "l")]  
    Login {
        /// Registry API Token (starts with 'spr_')
        /// 
        /// Examples:
        ///   solpm login --token spr_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
        ///   solpm login (interactive prompt for token)
        #[arg(long)]
        token: Option<String>,
    },
    
    /// Clear stored Registry credentials
    #[command(alias = "lo")]
    Logout,
    
    /// Publish program to the registry
    #[command(alias = "p")]
    Publish {
        /// Path to the authority keypair file
        /// 
        /// Examples:
        ///   solpm publish --authority-keypair ./path/to/keypair.json
        ///   solpm publish (uses authority_keypair from SolanaPrograms.toml)
        #[arg(long)]
        authority_keypair: Option<String>,
    }
    
}