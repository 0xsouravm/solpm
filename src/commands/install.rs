//! # Dependency Installation Module
//!
//! This module implements the `install` command which fetches and installs
//! all program dependencies defined in the SolanaPrograms.json configuration file.
//!
//! Features:
//! - Batch installation of all project dependencies
//! - IDL file fetching and local caching
//! - Support for both regular and development dependencies
//! - Network-specific program resolution
//! - Optional TypeScript client code generation
//! - Progress reporting and error handling
//! - Incremental installation (skips existing dependencies)
//!
//! The installation process downloads IDL files from the registry and saves them
//! locally for use in development and code generation workflows.

use crate::commands::constants::{GET_PROGRAM_URL, PROGRAM_IDL_DIR, SOLANA_PROGRAMS_FILE};
use crate::commands::types::{Program, ProgramResponse, SolanaPrograms};
use crate::commands::codegen;
use crate::error::{Result, SolanaPmError};
use crate::utils::{CliProgress, CliStyle, generate_project_hash};
use std::fs;
use serde_json::json;

/// Installs all program dependencies defined in SolanaPrograms.json.
/// 
/// This function reads the SolanaPrograms.json configuration file and installs
/// all program dependencies by:
/// 1. Checking if IDL files already exist locally (skipping if they do)
/// 2. Fetching program metadata and IDL files from the registry API
/// 3. Saving IDL files to the configured paths
/// 4. Updating the configuration with IDL paths if needed
/// 5. Optionally generating TypeScript client code if the codegen flag is enabled
/// 
/// The function processes both regular and development dependencies, displaying
/// progress information and handling errors gracefully by continuing with remaining
/// dependencies.
/// 
/// # Arguments
/// 
/// * `codegen` - Whether to generate TypeScript client code after installing programs
/// 
/// # Returns
/// 
/// Returns `Ok(())` on success, or an error if the configuration file is not found,
/// cannot be parsed, or critical file operations fail.
/// 
/// # Errors
/// 
/// * `SolanaPmError::ConfigNotFound` - If SolanaPrograms.json doesn't exist
/// * File I/O errors during configuration reading/writing
/// * Network errors when fetching from the registry (continues with other dependencies)
/// 
/// # Examples
/// 
/// ```rust
/// // Install all dependencies from SolanaPrograms.json
/// install_dependencies(false).await?;
/// 
/// // Install dependencies and generate TypeScript client code
/// install_dependencies(true).await?;
/// ```
pub async fn install_dependencies(codegen: bool) -> Result<()> {
    // Check if SolanaPrograms.json exists
    if !std::path::Path::new(SOLANA_PROGRAMS_FILE).exists() {
        return Err(SolanaPmError::ConfigNotFound(format!("{} not found. Run 'solpm add <program>' first.", SOLANA_PROGRAMS_FILE)));
    }
    
    // Read SolanaPrograms.json
    let content = fs::read_to_string(SOLANA_PROGRAMS_FILE)?;
    let mut solana_programs: SolanaPrograms = serde_json::from_str(&content)?;
    
    let client = reqwest::Client::new();
    let mut installed_count = 0;
    let mut total_count = 0;
    let mut programs_updated = false;
    
    // Count total programs for progress bar
    let all_programs_count = solana_programs.programs.len() + solana_programs.dev_programs.len();
    let progress_bar = if all_programs_count > 1 {
        Some(CliProgress::new_progress_bar(all_programs_count as u64, "Installing dependencies"))
    } else {
        None
    };
    
    // Process regular programs
    let regular_programs: Vec<(String, Program)> = solana_programs.programs.clone().into_iter().collect();
    for (package_name, mut program_info) in regular_programs {
        total_count += 1;
        let default_path = format!("{}/{}.json", PROGRAM_IDL_DIR, package_name);
        let idl_file_path = program_info.idl_path.as_deref().unwrap_or(&default_path);
        
        // Check if IDL already exists
        if std::path::Path::new(idl_file_path).exists() {
            // Ensure the path is stored in the config
            if program_info.idl_path.is_none() {
                program_info.idl_path = Some(idl_file_path.to_string());
                solana_programs.programs.insert(package_name.clone(), program_info);
                programs_updated = true;
            }
            continue;
        }
        
        println!("{}", CliStyle::progress(&format!("Installing {} {}...", 
            CliStyle::package(&package_name), 
            CliStyle::version(&program_info.version)
        )));
        
        // Install program using backend API with download tracking
        let project_hash = generate_project_hash();
        let url = format!("{}/{}/latest/install", GET_PROGRAM_URL, package_name);
        
        // Create request body with network and project hash for download tracking
        let request_body = json!({
            "network": program_info.network,
            "project_hash": project_hash
        });
        
        let response = client
            .post(&url)
            .json(&request_body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            if let Some(ref pb) = progress_bar {
                CliProgress::finish_with_error(pb.clone(), &format!("Failed to fetch {}", package_name));
            } else {
                eprintln!("{}", CliStyle::error(&format!("Failed to fetch {}: {}", package_name, response.status())));
            }
            continue;
        }
        
        let program_response: ProgramResponse = response.json().await?;
        
        // Create directory for IDL file
        if let Some(parent) = std::path::Path::new(idl_file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Save IDL file
        let idl_content = serde_json::to_string_pretty(&program_response.idl)?;
        fs::write(idl_file_path, idl_content)?;
        
        // Update program info with IDL path
        program_info.idl_path = Some(idl_file_path.to_string());
        let version = program_info.version.clone();
        solana_programs.programs.insert(package_name.clone(), program_info);
        programs_updated = true;
        
        installed_count += 1;
        if let Some(ref pb) = progress_bar {
            pb.inc(1);
        } else {
            println!("{}", CliStyle::success(&format!(
                "{} {} - installed successfully",
                CliStyle::package(&package_name),
                CliStyle::version(&version)
            )));
        }
    }
    
    // Process dev programs
    let dev_programs: Vec<(String, Program)> = solana_programs.dev_programs.clone().into_iter().collect();
    for (package_name, mut program_info) in dev_programs {
        total_count += 1;
        let default_path = format!("{}/{}.json", PROGRAM_IDL_DIR, package_name);
        let idl_file_path = program_info.idl_path.as_deref().unwrap_or(&default_path);
        
        // Check if IDL already exists
        if std::path::Path::new(idl_file_path).exists() {
            // Ensure the path is stored in the config
            if program_info.idl_path.is_none() {
                program_info.idl_path = Some(idl_file_path.to_string());
                solana_programs.dev_programs.insert(package_name.clone(), program_info);
                programs_updated = true;
            }
            continue;
        }
        
        println!("{}", CliStyle::progress(&format!("Installing {} {}...", 
            CliStyle::package(&package_name), 
            CliStyle::version(&program_info.version)
        )));
        
        // Install program using backend API with download tracking
        let project_hash = generate_project_hash();
        let url = format!("{}/{}/latest/install", GET_PROGRAM_URL, package_name);
        
        // Create request body with network and project hash for download tracking
        let request_body = json!({
            "network": program_info.network,
            "project_hash": project_hash
        });
        
        let response = client
            .post(&url)
            .json(&request_body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            if let Some(ref pb) = progress_bar {
                CliProgress::finish_with_error(pb.clone(), &format!("Failed to fetch {}", package_name));
            } else {
                eprintln!("{}", CliStyle::error(&format!("Failed to fetch {}: {}", package_name, response.status())));
            }
            continue;
        }
        
        let program_response: ProgramResponse = response.json().await?;
        
        // Create directory for IDL file
        if let Some(parent) = std::path::Path::new(idl_file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Save IDL file
        let idl_content = serde_json::to_string_pretty(&program_response.idl)?;
        fs::write(idl_file_path, idl_content)?;
        
        // Update program info with IDL path
        program_info.idl_path = Some(idl_file_path.to_string());
        let version = program_info.version.clone();
        solana_programs.dev_programs.insert(package_name.clone(), program_info);
        programs_updated = true;
        
        installed_count += 1;
        if let Some(ref pb) = progress_bar {
            pb.inc(1);
        } else {
            println!("{}", CliStyle::success(&format!(
                "{} {} - installed successfully",
                CliStyle::package(&package_name),
                CliStyle::version(&version)
            )));
        }
    }
    
    // Write back updated SolanaPrograms.json if any programs were updated
    if programs_updated {
        let json = serde_json::to_string_pretty(&solana_programs)?;
        fs::write(SOLANA_PROGRAMS_FILE, json)?;
    }
    
    // Finish progress bar and print summary
    if let Some(pb) = progress_bar {
        if installed_count > 0 {
            CliProgress::finish_with_message(pb, &format!(
                "Installed {} program{}", 
                installed_count, 
                if installed_count == 1 { "" } else { "s" }
            ));
        } else {
            CliProgress::finish_with_message(pb, "All programs up to date");
        }
    } else {
        if total_count == 0 {
            println!("{}", CliStyle::warning(&format!("No programs found in {}", SOLANA_PROGRAMS_FILE)));
        } else if installed_count == 0 {
            println!("{}", CliStyle::info(&format!(
                "Up to date, {} program{} installed", 
                total_count, 
                if total_count == 1 { "" } else { "s" }
            )));
        } else {
            println!("{}", CliStyle::success(&format!(
                "Added {} program{}, {} program{} total", 
                installed_count, if installed_count == 1 { "" } else { "s" },
                total_count, if total_count == 1 { "" } else { "s" }
            )));
        }
    }
    
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