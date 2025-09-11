//! # Program Publishing Module
//!
//! This module implements the `publish` command which uploads Solana programs
//! to the registry for sharing and distribution.
//!
//! Features:
//! - Secure program publishing with authentication
//! - IDL file validation and upload
//! - Digital signature verification for program authenticity  
//! - Program metadata extraction from configuration files
//! - Authority keypair validation and signing
//! - Comprehensive error handling and user feedback
//! - Support for custom IDL and keypair file paths
//!
//! The publishing process ensures program integrity through cryptographic
//! signatures and validates all required metadata before submission.

use crate::commands::auth::ensure_authenticated;
use crate::commands::constants::PUBLISH_PROGRAM_URL;
use crate::commands::types::{UploadProgramRequest, SolanaProgramsConfig};
use crate::error::{Result, SolanaPmError};
use crate::utils::{CliProgress, CliStyle};
use solana_sdk::signature::{Keypair, Signer};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/// Expands tilde (~) in file paths to the user's home directory.
/// 
/// # Arguments
/// 
/// * `path` - The file path that may contain a tilde prefix
/// 
/// # Returns
/// 
/// Returns the expanded path as a string, or the original path if
/// expansion fails or no tilde is present.
fn expand_path(path: &str) -> String {
    if path.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            return path.replacen('~', &home.to_string_lossy(), 1);
        }
    }
    path.to_string()
}

/// Loads a Solana keypair from a file.
/// 
/// Supports both JSON array format (Solana CLI standard) and raw 64-byte format.
/// Automatically expands tilde paths to the user's home directory.
/// 
/// # Arguments
/// 
/// * `path` - The file path to the keypair file (supports ~/ prefix)
/// 
/// # Returns
/// 
/// Returns a Solana Keypair on success, or an error if the file cannot be
/// read or the keypair format is invalid.
/// 
/// # Examples
/// 
/// ```rust
/// // Load from standard Solana CLI location
/// let keypair = load_keypair_from_file("~/.config/solana/id.json")?;
/// 
/// // Load from custom path
/// let keypair = load_keypair_from_file("./my-keypair.json")?;
/// ```
fn load_keypair_from_file(path: &str) -> Result<Keypair> {
    let expanded_path = expand_path(path);
    let keypair_bytes = fs::read(&expanded_path)
        .map_err(|e| SolanaPmError::InvalidPath(format!("Failed to read keypair file '{}': {}", expanded_path, e)))?;
    
    // Try to parse as JSON first (Solana CLI format)
    if let Ok(keypair_json) = serde_json::from_slice::<Vec<u8>>(&keypair_bytes) {
        if keypair_json.len() == 64 {
            return Keypair::from_bytes(&keypair_json)
                .map_err(|e| SolanaPmError::InvalidPath(format!("Invalid keypair format: {}", e)));
        }
    }
    
    // Try to parse as raw bytes
    if keypair_bytes.len() == 64 {
        return Keypair::from_bytes(&keypair_bytes)
            .map_err(|e| SolanaPmError::InvalidPath(format!("Invalid keypair format: {}", e)));
    }
    
    Err(SolanaPmError::InvalidPath(format!(
        "Invalid keypair file format. Expected 64-byte keypair in JSON array or raw bytes format."
    )))
}



const SOLANA_PROGRAMS_TOML: &str = "SolanaPrograms.toml";

/// Publishes a Solana program to the registry.
/// 
/// This function performs the complete program publishing flow:
/// 1. Ensures user authentication with stored credentials
/// 2. Reads and validates the SolanaPrograms.toml configuration
/// 3. Locates and parses the program's IDL file
/// 4. Loads the authority keypair for cryptographic verification
/// 5. Generates a signed challenge for program ownership proof
/// 6. Uploads the program metadata and IDL to the registry
/// 
/// The function requires:
/// - Valid authentication (run `solpm login` first)
/// - A properly configured SolanaPrograms.toml file
/// - An IDL file in standard locations (target/idl, idl, target/deploy)
/// - Access to the authority keypair specified in the config
/// 
/// /// # Arguments
/// 
/// * `token_arg` - Optional API token to use (if None, prompts user)

/// # Returns
/// 
/// Returns `Ok(())` on successful publication, or an error if any step fails.
/// 
/// # Errors
/// 
/// * `SolanaPmError::ConfigNotFound` - If not authenticated or config missing
/// * `SolanaPmError::DataMissing` - If required config fields are empty
/// * `SolanaPmError::InvalidPath` - If files cannot be read or keypair is invalid
/// * `SolanaPmError::UploadFailed` - If registry upload fails
/// 
/// # Examples
/// 
/// ```rust
/// // Publish the program configured in SolanaPrograms.toml
/// publish_program(None).await?;
/// 
/// // Publish using a specific authority keypair file
/// publish_program(Some("./path/to/keypair.json")).await?;
/// ```
pub async fn publish_program(authority_keypair_arg: Option<&str>) -> Result<()> {
    // Ensure user is authenticated
    let token = ensure_authenticated().await?;
    
    // Read TOML configuration
    let spinner = CliProgress::new_spinner("Reading SolanaPrograms.toml...");
    
    if !std::path::Path::new(SOLANA_PROGRAMS_TOML).exists() {
        spinner.finish_and_clear();
        return Err(SolanaPmError::ConfigNotFound(
            "SolanaPrograms.toml not found. Run 'solpm init' first.".to_string()
        ));
    }
    
    let toml_content = fs::read_to_string(SOLANA_PROGRAMS_TOML)
        .map_err(|e| SolanaPmError::InvalidPath(format!("Failed to read SolanaPrograms.toml: {}", e)))?;
    
    let config: SolanaProgramsConfig = toml::from_str(&toml_content)
        .map_err(|e| SolanaPmError::InvalidPath(format!("Invalid TOML format: {}", e)))?;
    
    spinner.finish_and_clear();
    
    // Validate required fields
    if config.program.description.trim().is_empty() {
        return Err(SolanaPmError::DataMissing(
            "Description is required. Please fill in the 'description' field in SolanaPrograms.toml".to_string()
        ));
    }
    
    if config.program.repository.trim().is_empty() {
        return Err(SolanaPmError::DataMissing(
            "Repository is required. Please fill in the 'repository' field in SolanaPrograms.toml".to_string()
        ));
    }
    
    // Find and read IDL file
    let spinner = CliProgress::new_spinner("Finding IDL file...");
    let idl_file_path = find_idl_file()?;
    let idl_content = fs::read_to_string(&idl_file_path)
        .map_err(|e| SolanaPmError::InvalidPath(format!("Failed to read IDL file: {}", e)))?;
    
    let idl_json: serde_json::Value = serde_json::from_str(&idl_content)
        .map_err(|e| SolanaPmError::InvalidIdl(format!("Invalid JSON in IDL: {}", e)))?;
    
    spinner.finish_and_clear();
    
    // Load authority keypair
    let spinner = CliProgress::new_spinner("Loading authority keypair...");
    let authority_keypair = if let Some(ak) = authority_keypair_arg {
        load_keypair_from_file(ak.trim())?
    } else {
        load_keypair_from_file(&config.program.authority_keypair)?
    };
    spinner.finish_and_clear();
    
    // Generate challenge and sign it
    let spinner = CliProgress::new_spinner("Generating cryptographic proof...");
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| SolanaPmError::InvalidPath(format!("System time error: {}", e)))?
        .as_secs();
    
    let challenge = format!("Publish program {} to {} registry at {}", 
        config.program.program_id, config.program.network, timestamp);
    let signature = authority_keypair.sign_message(challenge.as_bytes());
    let authority_pubkey = authority_keypair.pubkey();
    
    spinner.finish_and_clear();
    
    println!("{}", CliStyle::progress(&format!(
        "Publishing {} {} to {} with authority {}...", 
        CliStyle::package(&config.program.name), 
        CliStyle::version(&config.program.version),
        CliStyle::highlight(&config.program.network),
        CliStyle::highlight(&authority_pubkey.to_string())
    )));
    
    // Create upload request with cryptographic proof
    let upload_request = UploadProgramRequest {
        name: config.program.name.clone(),
        version: config.program.version.clone(),
        program_id: config.program.program_id.clone(),
        network: config.program.network.clone(),
        idl: idl_json,
        description: config.program.description.clone(),
        repository: config.program.repository.clone(),
        // Cryptographic verification fields
        challenge,
        signature: bs58::encode(signature.as_ref()).into_string(),
        authority_pubkey: bs58::encode(authority_pubkey.as_ref()).into_string(),
    };
    
    // Upload to registry
    let spinner = CliProgress::new_spinner("Publishing to registry...");
    
    let client = reqwest::Client::new();
    let publish_response = client
        .post(PUBLISH_PROGRAM_URL)
        .header("Authorization", format!("Bearer {}", token))
        .json(&upload_request)
        .send()
        .await?;
    
    spinner.finish_and_clear();
    
    if publish_response.status().is_success() {
        println!("{}", CliStyle::success(&format!(
            "Successfully published {} {} to {}",
            CliStyle::package(&config.program.name),
            CliStyle::version(&config.program.version),
            CliStyle::highlight(&config.program.network)
        )));
    } else {
        let status = publish_response.status();
        let error_text = publish_response.text().await?;
        return Err(SolanaPmError::UploadFailed(format!(
            "Failed to publish program ({}): {}", status, error_text
        )));
    }
    
    Ok(())
}

/// Searches for an IDL file in standard Solana project directories.
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
    const IDL_PATHS: &[&str] = &["target/idl", "idl", "target/deploy"];
    
    for idl_dir in IDL_PATHS {
        let dir_path = std::path::Path::new(idl_dir);
        if dir_path.exists() && dir_path.is_dir() {
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