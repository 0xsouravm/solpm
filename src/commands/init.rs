//! # Project Initialization Module
//!
//! This module implements the `init` command which initializes a new Solana program
//! project with the necessary configuration files and directory structure.
//!
//! Features:
//! - Creates SolanaPrograms.toml configuration file
//! - Auto-detects existing program information from project files
//! - Supports network selection (mainnet/devnet)
//! - Validates project structure and dependencies
//! - Provides interactive setup with confirmation prompts
//! - Attempts to discover GitHub repository information
//!
//! The initialization process creates a standardized project structure that
//! enables dependency management and program publishing through the registry.

use crate::commands::types::{SolanaProgramsConfig, ProgramConfig};
use crate::cli::Network;
use crate::error::{Result, SolanaPmError};
use crate::utils::{CliStyle, CliProgress, confirm_action};
use std::fs;
use std::path::Path;

const SOLANA_PROGRAMS_TOML: &str = "SolanaPrograms.toml";
const IDL_PATHS: &[&str] = &["target/idl", "idl", "target/deploy"];

/// Attempts to get the GitHub repository URL from git remote origin.
/// 
/// # Returns
/// 
/// Returns `Some(String)` with the GitHub repository URL if found,
/// or `None` if not in a git repository or no GitHub remote found.
fn get_github_repository_url() -> Option<String> {
    use std::process::Command;
    
    // Try to get the git remote origin URL
    let output = Command::new("git")
        .args(&["config", "--get", "remote.origin.url"])
        .output()
        .ok()?;
    
    if !output.status.success() {
        return None;
    }
    
    let url = String::from_utf8(output.stdout).ok()?.trim().to_string();
    
    // Check if it's a GitHub URL
    if url.contains("github.com") {
        Some(normalize_github_url(url))
    } else {
        None
    }
}

/// Normalizes GitHub URLs to a consistent format.
/// 
/// Converts both SSH and HTTPS GitHub URLs to a consistent format.
/// 
/// # Arguments
/// 
/// * `url` - The GitHub URL to normalize
/// 
/// # Returns
/// 
/// Returns a normalized GitHub URL string.
fn normalize_github_url(url: String) -> String {
    // Convert SSH format to HTTPS for consistency
    if url.starts_with("git@github.com:") {
        let repo_path = url.trim_start_matches("git@github.com:").trim_end_matches(".git");
        format!("https://github.com/{}", repo_path)
    } else if url.starts_with("https://github.com/") {
        url.trim_end_matches(".git").to_string()
    } else {
        url
    }
}

/// Initializes a new Solana project with package configuration.
/// 
/// This function creates a `SolanaPrograms.toml` configuration file by:
/// 1. Checking if a configuration already exists (with overwrite confirmation)
/// 2. Auto-detecting GitHub repository URL if available
/// 3. Searching for IDL files in common locations (target/idl, idl, target/deploy)
/// 4. Extracting metadata from the IDL file (name, version, program ID)
/// 5. Creating a configuration template with detected/specified values
/// 
/// # Arguments
/// 
/// * `network` - The target network (mainnet or devnet) for the project
/// 
/// # Returns
/// 
/// Returns `Ok(())` on success, or an error if IDL files are not found,
/// file operations fail, or IDL parsing fails.
/// 
/// # Examples
/// 
/// ```rust
/// // Initialize project configuration for devnet
/// init_project(&Network::Dev)?;
/// 
/// // Initialize project configuration for mainnet
/// init_project(&Network::Main)?;
/// ```
pub fn init_project(network: &Network) -> Result<()> {
    // Check if config already exists and ask for confirmation
    if Path::new(SOLANA_PROGRAMS_TOML).exists() {
        println!("{}", CliStyle::warning(&format!("{} already exists.", SOLANA_PROGRAMS_TOML)));
        if !confirm_action("Do you want to overwrite it?") {
            println!("{}", CliStyle::info("Initialization cancelled."));
            return Ok(());
        }
    }

    println!("{}", CliStyle::info("Initializing Solana program configuration..."));
    
    // Find IDL file
    let spinner = CliProgress::new_spinner("Looking for IDL files...");
    let idl_file_path = find_idl_file()?;
    spinner.finish_and_clear();
    
    println!("{}", CliStyle::success(&format!("Found IDL file: {}", idl_file_path)));
    
    // Read and parse IDL
    let spinner = CliProgress::new_spinner("Reading IDL metadata...");
    let idl_content = fs::read_to_string(&idl_file_path)
        .map_err(|e| SolanaPmError::InvalidPath(format!("Failed to read IDL file: {}", e)))?;
    
    let idl_json: serde_json::Value = serde_json::from_str(&idl_content)
        .map_err(|e| SolanaPmError::InvalidIdl(format!("Invalid JSON in IDL: {}", e)))?;
    
    spinner.finish_and_clear();
    
    // Extract metadata
    let name = idl_json["metadata"]["name"]
        .as_str()
        .ok_or_else(|| SolanaPmError::InvalidIdl("Program name not found in IDL metadata".to_string()))?
        .to_string();
        
    let version = idl_json["metadata"]["version"]
        .as_str()
        .ok_or_else(|| SolanaPmError::InvalidIdl("Program version not found in IDL metadata".to_string()))?
        .to_string();
    
    let program_id = idl_json["address"]
        .as_str()
        .unwrap_or("PLACEHOLDER_PROGRAM_ID")
        .to_string();
    
    // Convert network enum to string
    let network_str = match network {
        Network::Main => "mainnet",
        Network::Dev => "devnet",
    };
    
    // Detect GitHub repository URL if available
    let repository_url = get_github_repository_url().unwrap_or_else(|| "".to_string());
    
    if !repository_url.is_empty() {
        println!("{}", CliStyle::success(&format!(
            "Detected GitHub repository: {}",
            CliStyle::highlight(&repository_url)
        )));
    }
    
    // Create config structure
    let config = SolanaProgramsConfig {
        program: ProgramConfig {
            name,
            version,
            program_id,
            network: network_str.to_string(),
            description: "".to_string(), // Left blank for user to fill
            repository: repository_url.clone(),
            authority_keypair: "~/.config/solana/id.json".to_string(),
        },
    };
    
    // Write TOML file
    let toml_content = toml::to_string_pretty(&config)
        .map_err(|e| SolanaPmError::InvalidPath(format!("Failed to serialize TOML: {}", e)))?;
    
    fs::write(SOLANA_PROGRAMS_TOML, toml_content)?;
    
    println!("{}", CliStyle::success(&format!(
        "Created {} for {} network",
        SOLANA_PROGRAMS_TOML,
        CliStyle::highlight(network_str)
    )));
    
    if repository_url.is_empty() {
        println!("{}", CliStyle::info("Please fill in the 'description' and 'repository' fields before publishing."));
    } else {
        println!("{}", CliStyle::info("Please fill in the 'description' field before publishing."));
    }
    
    Ok(())
}

/// Searches for an IDL file in common Solana project directories.
/// 
/// This function looks for `.json` IDL files in the following directories (in order):
/// - `target/idl` - Standard Anchor build output
/// - `idl` - Custom IDL directory
/// - `target/deploy` - Alternative build output location
/// 
/// # Returns
/// 
/// Returns the path to the first IDL file found, or an error if no IDL files
/// are found in any of the searched directories.
/// 
/// # Errors
/// 
/// Returns `SolanaPmError::InvalidPath` if no IDL file is found or if
/// directory reading fails.
fn find_idl_file() -> Result<String> {
    for idl_dir in IDL_PATHS {
        let dir_path = Path::new(idl_dir);
        if dir_path.exists() && dir_path.is_dir() {
            // Look for .json files in this directory
            let entries = fs::read_dir(dir_path)
                .map_err(|e| SolanaPmError::InvalidPath(format!("Failed to read directory {}: {}", idl_dir, e)))?;
            
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "json") {
                    return Ok(path.to_string_lossy().to_string());
                }
            }
        }
    }
    
    Err(SolanaPmError::InvalidPath(
        "No IDL file found. Please build/deploy your program first. Searched paths: target/idl, idl, target/deploy".to_string()
    ))
}