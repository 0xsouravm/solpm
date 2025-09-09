//! # Add Command Implementation
//!
//! This module implements the `add` command which allows users to add Solana program
//! dependencies to their project. It supports:
//!
//! - Adding programs by name (latest version) or name@version (specific version)
//! - Installing as regular or development dependencies
//! - Custom IDL file paths
//! - Network selection (mainnet/devnet)
//! - Optional TypeScript client code generation
//!
//! The command fetches program metadata and IDL files from the registry,
//! saves them locally, and updates the project's SolanaPrograms.json configuration.

use crate::commands::constants::{GET_PROGRAM_URL, PROGRAM_IDL_DIR, SOLANA_PROGRAMS_FILE};
use crate::commands::types::{Program, ProgramResponse, SolanaPrograms};
use crate::commands::codegen;
use crate::cli::Network;
use crate::error::{Result, SolanaPmError};
use crate::utils::{CliProgress, CliStyle, generate_project_hash, parse_package_spec};
use std::collections::HashMap;
use std::fs;
use serde_json::json;

/// Adds a Solana program dependency to the current project.
/// 
/// This function first checks if the program already exists locally to avoid unnecessary API calls.
/// If the program doesn't exist locally, it fetches the program metadata and IDL from the registry API,
/// then saves the IDL file locally and updates the SolanaPrograms.json configuration. Optionally
/// generates TypeScript client code if the codegen flag is enabled.
/// 
/// # Arguments
/// 
/// * `package_spec` - The package specification (name or name@version) to add
/// * `is_dev` - Whether to add as a development dependency
/// * `custom_path` - Optional custom path for the IDL file
/// * `network` - The target network (mainnet or devnet) to fetch from
/// * `codegen` - Whether to generate TypeScript client code after adding the program
/// 
/// # Returns
/// 
/// Returns `Ok(())` on success, or an error if the program is not found, network request fails,
/// or file operations fail.
/// 
/// # Examples
/// 
/// ```rust
/// // Add a regular dependency (latest version) from devnet
/// add_program("my-program", false, None, &Network::Dev, false).await?;
/// 
/// // Add a specific version as dev dependency with custom IDL path and generate client code
/// add_program("my-program@1.0.0", true, Some("./custom/path.json"), &Network::Main, true).await?;
/// ```
pub async fn add_program(package_spec: &str, is_dev: bool, custom_path: Option<&str>, network: &Network, codegen: bool) -> Result<()> {
    // Parse package specification
    let parsed_spec = parse_package_spec(package_spec);
    let package_name = &parsed_spec.name;
    
    // Convert network enum to string
    let network_str = match network {
        Network::Main => "mainnet",
        Network::Dev => "devnet",
    };
    
    // Read existing SolanaPrograms.json or create new one
    let mut solana_programs = if fs::metadata(SOLANA_PROGRAMS_FILE).is_ok() {
        let content = fs::read_to_string(SOLANA_PROGRAMS_FILE)?;
        serde_json::from_str(&content)?
    } else {
        SolanaPrograms {
            programs: HashMap::new(),
            dev_programs: HashMap::new(),
        }
    };
    
    // Check if program already exists locally first
    let already_exists = if is_dev {
        solana_programs.dev_programs.contains_key(package_name)
    } else {
        solana_programs.programs.contains_key(package_name)
    };
    
    if already_exists {
        let dependency_type = if is_dev { "dev dependency" } else { "dependency" };
        println!("{}", CliStyle::warning(&format!(
            "Program {} already exists as {}. Skipping.",
            CliStyle::package(package_name),
            dependency_type
        )));
        return Ok(());
    }
    
    // Only fetch from API if program doesn't exist locally
    let spinner = CliProgress::new_spinner(&format!("Installing {} from {}...", CliStyle::package(package_name), CliStyle::highlight(network_str)));

    let client = reqwest::Client::new();
    let project_hash = generate_project_hash();
    
    // Build URL based on whether a specific version was requested
    let url = if let Some(version) = &parsed_spec.version {
        format!("{}/{}/{}/install", GET_PROGRAM_URL, package_name, version)
    } else {
        format!("{}/{}/latest/install", GET_PROGRAM_URL, package_name)
    };
    
    // Create request body with network and project hash for download tracking
    let request_body = json!({
        "network": network_str,
        "project_hash": project_hash
    });
    
    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await?;

    spinner.finish_and_clear();
    
    if !response.status().is_success() {
        if response.status().as_u16() == 404 {
            return Err(SolanaPmError::ProgramNotFound(package_name.to_string()));
        } else {
            let error_text = response.text().await?;
            return Err(SolanaPmError::UploadFailed(error_text));
        }
    }
    
    let program_response: ProgramResponse = response.json().await?;
    
    // Determine IDL file path
    let idl_file_path = if let Some(path) = custom_path {
        path.to_string()
    } else {
        format!("{}/{}.json", PROGRAM_IDL_DIR, package_name)
    };
    
    // Convert API response to our Program struct  
    let program_info = Program {
        version: program_response.version,
        program_id: program_response.program_id,
        network: network_str.to_string(),
        idl_path: Some(idl_file_path.clone()),
    };
    
    // Create directory for IDL file
    if let Some(parent) = std::path::Path::new(&idl_file_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            SolanaPmError::InvalidPath(format!("Failed to create directory {}: {}", parent.display(), e))
        })?;
    }
    
    // Save IDL file
    let idl_content = serde_json::to_string_pretty(&program_response.idl)?;
    fs::write(&idl_file_path, idl_content).map_err(|e| {
        SolanaPmError::InvalidPath(format!("Failed to write IDL file {}: {}", idl_file_path, e))
    })?;
    
    // Add program to appropriate section
    if is_dev {
        solana_programs.dev_programs.insert(package_name.to_string(), program_info.clone());
        println!("{}", CliStyle::success(&format!(
            "Added {} {} as dev dependency",
            CliStyle::package(package_name),
            CliStyle::version(&program_info.version)
        )));
    } else {
        solana_programs.programs.insert(package_name.to_string(), program_info.clone());
        println!("{}", CliStyle::success(&format!(
            "Added {} {} as dependency",
            CliStyle::package(package_name),
            CliStyle::version(&program_info.version)
        )));
    }
    
    // Write back to SolanaPrograms.json
    let json = serde_json::to_string_pretty(&solana_programs)?;
    fs::write(SOLANA_PROGRAMS_FILE, json)?;
    
    // Generate TypeScript client code if requested
    if codegen {
        println!("\n{}", CliStyle::info("Generating TypeScript client code..."));
        if let Err(e) = codegen::generate_typescript_client() {
            println!("{}", CliStyle::warning(&format!(
                "Failed to generate TypeScript client: {}",
                e
            )));
        }
    }
    
    Ok(())
}