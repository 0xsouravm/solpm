//! # Authentication Module
//!
//! This module handles user authentication and credential management for the
//! Solana Program Manager. It provides secure storage and retrieval of
//! authentication tokens using AES-256-GCM encryption with PBKDF2 key derivation.
//!
//! Features:
//! - Secure token storage with password-based encryption
//! - Token verification with the registry API
//! - Login/logout functionality
//! - Credential persistence across sessions
//! - Safe handling of sensitive authentication data
//!
//! All credentials are stored encrypted in the user's configuration directory
//! (~/.solpm) and require password verification for access.

use crate::commands::constants::AUTH_VERIFY_URL;
use crate::error::{Result, SolanaPmError};
use crate::utils::{CliStyle, prompt_input};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::{Aead, OsRng};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use rand::RngCore;
use base64::{Engine as _, engine::general_purpose};

#[derive(Serialize, Deserialize, Debug)]
struct Credentials {
    encrypted_token: String,
    salt: String,
    nonce: String,
}

#[derive(Deserialize)]
struct AuthVerifyResponse {
    valid: bool,
    permissions: Vec<String>,
}

/// Gets the file path for storing encrypted credentials.
/// 
/// Creates the configuration directory (~/.solpm) if it doesn't exist
/// and returns the path to the credentials.json file.
/// 
/// # Returns
/// 
/// Returns the PathBuf to the credentials file, or an error if the home
/// directory cannot be found or the config directory cannot be created.
fn get_credentials_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| SolanaPmError::InvalidPath("Could not find home directory".to_string()))?;
    
    let config_dir = home_dir.join(".solpm");
    
    // Create config directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }
    
    Ok(config_dir.join("credentials.json"))
}

/// Derives a 32-byte encryption key from a password using PBKDF2.
/// 
/// Uses PBKDF2 with SHA-256 and 100,000 iterations for secure key derivation.
/// 
/// # Arguments
/// 
/// * `password` - The password to derive the key from
/// * `salt` - Random salt bytes for key derivation
/// 
/// # Returns
/// 
/// Returns a 32-byte key suitable for AES-256-GCM encryption.
fn derive_key_from_password(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut key);
    key
}

/// Encrypts an API token using AES-256-GCM with a password-derived key.
/// 
/// Generates random salt and nonce for each encryption operation to ensure
/// security. The encrypted data is base64 encoded for storage.
/// 
/// # Arguments
/// 
/// * `token` - The API token to encrypt
/// * `password` - The password to derive the encryption key from
/// 
/// # Returns
/// 
/// Returns a tuple of (encrypted_token, salt, nonce) all base64 encoded,
/// or an error if encryption fails.
fn encrypt_token(token: &str, password: &str) -> Result<(String, String, String)> {
    // Generate random salt and nonce
    let mut salt = [0u8; 16];
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce_bytes);
    
    // Derive key from password
    let key_bytes = derive_key_from_password(password, &salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    
    // Encrypt token
    let nonce = Nonce::from_slice(&nonce_bytes);
    let encrypted = cipher.encrypt(nonce, token.as_bytes())
        .map_err(|e| SolanaPmError::InvalidPath(format!("Encryption failed: {}", e)))?;
    
    // Encode to base64
    let encrypted_b64 = general_purpose::STANDARD.encode(&encrypted);
    let salt_b64 = general_purpose::STANDARD.encode(&salt);
    let nonce_b64 = general_purpose::STANDARD.encode(&nonce_bytes);
    
    Ok((encrypted_b64, salt_b64, nonce_b64))
}

/// Decrypts an API token using AES-256-GCM with a password-derived key.
/// 
/// Takes base64 encoded encrypted data and decrypts it back to the original token.
/// 
/// # Arguments
/// 
/// * `encrypted_token` - Base64 encoded encrypted token
/// * `salt` - Base64 encoded salt used for key derivation
/// * `nonce` - Base64 encoded nonce used for encryption
/// * `password` - The password to derive the decryption key from
/// 
/// # Returns
/// 
/// Returns the decrypted token string, or an error if decryption fails
/// (usually indicating an incorrect password).
fn decrypt_token(encrypted_token: &str, salt: &str, nonce: &str, password: &str) -> Result<String> {
    // Decode from base64
    let encrypted = general_purpose::STANDARD.decode(encrypted_token)
        .map_err(|e| SolanaPmError::InvalidPath(format!("Invalid encrypted token: {}", e)))?;
    let salt_bytes = general_purpose::STANDARD.decode(salt)
        .map_err(|e| SolanaPmError::InvalidPath(format!("Invalid salt: {}", e)))?;
    let nonce_bytes = general_purpose::STANDARD.decode(nonce)
        .map_err(|e| SolanaPmError::InvalidPath(format!("Invalid nonce: {}", e)))?;
    
    // Derive key from password
    let key_bytes = derive_key_from_password(password, &salt_bytes);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    
    // Decrypt token
    let nonce = Nonce::from_slice(&nonce_bytes);
    let decrypted = cipher.decrypt(nonce, encrypted.as_slice())
        .map_err(|_| SolanaPmError::InvalidPath("Decryption failed. Incorrect password?".to_string()))?;
    
    String::from_utf8(decrypted)
        .map_err(|e| SolanaPmError::InvalidPath(format!("Invalid token data: {}", e)))
}

/// Authenticates with the registry API and stores encrypted credentials.
/// 
/// This function performs the complete login flow:
/// 1. Prompts for or accepts an API token
/// 2. Validates the token format and permissions with the registry
/// 3. Prompts for an encryption password to secure the token locally
/// 4. Encrypts and stores the credentials in ~/.solpm/credentials.json
/// 
/// # Arguments
/// 
/// * `token_arg` - Optional API token to use (if None, prompts user)
/// 
/// # Returns
/// 
/// Returns `Ok(())` on successful authentication and storage, or an error
/// if token validation fails, encryption fails, or file operations fail.
/// 
/// # Examples
/// 
/// ```rust
/// // Login with prompt for token
/// login(None).await?;
/// 
/// // Login with provided token
/// login(Some("spr_your_token_here")).await?;
/// ```
pub async fn login(token_arg: Option<&str>) -> Result<()> {
    println!("\n{}", CliStyle::header("Registry API Token Required"));
    println!("To use Solana Program Manager to publish programs, you need an API token from the registry.");
    println!("Follow these steps to get an API token:");
    println!("1. Go to: {}", CliStyle::highlight("http://localhost:3000/auth/github"));
    println!("2. Sign in with GitHub");
    println!("3. Go to: {}", CliStyle::highlight("http://localhost:3000/api-tokens"));
    println!("4. Create a new token with {} permissions", CliStyle::package("publish:programs"));
    println!("5. Copy the generated token (starts with 'spr_')\n");
    
    // Get token from argument or prompt
    let token = if let Some(t) = token_arg {
        t.trim().to_string()
    } else {
        match prompt_input("Enter your Registry API Token", None) {
            Some(t) if !t.trim().is_empty() => t.trim().to_string(),
            _ => return Err(SolanaPmError::InvalidPath("Token is required".to_string())),
        }
    };
    
    // Validate token format (should start with 'spr_')
    if !token.starts_with("spr_") {
        return Err(SolanaPmError::UploadFailed(
            "Invalid API token format. Registry API tokens should start with 'spr_'.".to_string()
        ));
    }
    
    // Validate token by making a test request to the auth/verify endpoint
    let client = reqwest::Client::new();
    
    println!("{}", CliStyle::progress("Validating token..."));
    
    let response = client
        .get(AUTH_VERIFY_URL)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| SolanaPmError::UploadFailed(format!("Failed to connect to registry server: {}. Make sure the server is running at {}", e, AUTH_VERIFY_URL)))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(SolanaPmError::UploadFailed(format!("API token validation failed ({}): {}. Make sure your token is correct and the server is running.", status, error_text)));
    }
    
    // Parse the verification response
    let auth_response: AuthVerifyResponse = response.json().await?;
    
    if !auth_response.valid {
        return Err(SolanaPmError::UploadFailed("Token verification failed. Please check your token and try again.".to_string()));
    }
    
    // Check for required permissions
    if !auth_response.permissions.contains(&"publish:programs".to_string()) {
        return Err(SolanaPmError::UploadFailed("Token does not have required 'publish:programs' permission.".to_string()));
    }
    
    // Prompt for encryption password
    println!("\n{}", CliStyle::header("Encryption Password Setup"));
    println!("To secure your API token, please create an encryption password.");
    println!("You will need this password when publishing programs (not for other operations).");
    
    let password = rpassword::prompt_password("Enter encryption password: ")
        .map_err(|e| SolanaPmError::InvalidPath(format!("Failed to read password: {}", e)))?;
    
    if password.trim().is_empty() {
        return Err(SolanaPmError::InvalidPath("Password cannot be empty".to_string()));
    }
    
    let confirm_password = rpassword::prompt_password("Confirm encryption password: ")
        .map_err(|e| SolanaPmError::InvalidPath(format!("Failed to read password: {}", e)))?;
    
    if password != confirm_password {
        return Err(SolanaPmError::InvalidPath("Passwords do not match".to_string()));
    }
    
    // Encrypt and save credentials
    let (encrypted_token, salt, nonce) = encrypt_token(&token, &password)?;
    let credentials = Credentials {
        encrypted_token,
        salt,
        nonce,
    };
    
    let credentials_path = get_credentials_path()?;
    let credentials_json = serde_json::to_string_pretty(&credentials)?;
    fs::write(&credentials_path, credentials_json)?;
    
    let permissions_str = auth_response.permissions.join(", ");
    println!("\n{}", CliStyle::success("Successfully authenticated with API token"));
    println!("Token permissions: {}", CliStyle::package(&permissions_str));
    println!("Encrypted credentials saved to: {}", CliStyle::path(&credentials_path.display().to_string()));
    println!("{}", CliStyle::info("Remember your encryption password - you'll need it when publishing programs!"));
    
    Ok(())
}

/// Verifies an API token with the registry server.
/// 
/// Makes a request to the auth/verify endpoint to check if the token is
/// valid and has the required 'publish:programs' permission.
/// 
/// # Arguments
/// 
/// * `token` - The API token to verify
/// 
/// # Returns
/// 
/// Returns `Ok(true)` if the token is valid and has required permissions,
/// `Ok(false)` if invalid, or an error if the request fails.
pub async fn verify_token(token: &str) -> Result<bool> {
    let client = reqwest::Client::new();
    
    let response = client
        .get(AUTH_VERIFY_URL)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| SolanaPmError::UploadFailed(format!("Failed to connect to registry server: {}", e)))?;
    
    if !response.status().is_success() {
        return Ok(false);
    }
    
    let auth_response: AuthVerifyResponse = response.json().await
        .map_err(|e| SolanaPmError::UploadFailed(format!("Failed to parse server response: {}", e)))?;
    Ok(auth_response.valid && auth_response.permissions.contains(&"publish:programs".to_string()))
}

/// Logs out by removing stored credentials from the local system.
/// 
/// Deletes the encrypted credentials file from ~/.solpm/credentials.json
/// if it exists.
/// 
/// # Returns
/// 
/// Returns `Ok(())` on success, or an error if file deletion fails.
pub fn logout() -> Result<()> {
    let credentials_path = get_credentials_path()?;
    
    if credentials_path.exists() {
        fs::remove_file(&credentials_path)?;
        println!("{}", CliStyle::success("Successfully logged out"));
        println!("Credentials removed from: {}", credentials_path.display());
    } else {
        println!("{}", CliStyle::info("Already logged out"));
    }
    
    Ok(())
}

/// Retrieves and decrypts a stored API token.
/// 
/// Prompts for the encryption password and decrypts the stored token.
/// This function should only be called when the token is actually needed
/// to avoid unnecessary password prompts.
/// 
/// # Returns
/// 
/// Returns `Some(token)` if credentials exist and decryption succeeds,
/// `None` if no credentials are stored, or an error if decryption fails.
pub fn get_stored_token() -> Result<Option<String>> {
    let credentials_path = get_credentials_path()?;
    
    if !credentials_path.exists() {
        return Ok(None);
    }
    
    let credentials_content = fs::read_to_string(&credentials_path)?;
    let credentials: Credentials = serde_json::from_str(&credentials_content)?;
    
    // Prompt for password to decrypt token only when needed
    println!("{}", CliStyle::progress("Authentication required"));
    let password = rpassword::prompt_password("Enter your encryption password: ")
        .map_err(|e| SolanaPmError::InvalidPath(format!("Failed to read password: {}", e)))?;
    
    let decrypted_token = decrypt_token(
        &credentials.encrypted_token,
        &credentials.salt,
        &credentials.nonce,
        &password
    )?;
    
    Ok(Some(decrypted_token))
}

/// Checks if encrypted credentials exist without decrypting them.
/// 
/// This is useful for checking authentication status without prompting
/// for a password.
/// 
/// # Returns
/// 
/// Returns `true` if credentials file exists, `false` otherwise, or an
/// error if the credentials path cannot be determined.
pub fn has_stored_credentials() -> Result<bool> {
    let credentials_path = get_credentials_path()?;
    Ok(credentials_path.exists())
}

/// Ensures the user is authenticated and returns a valid API token.
/// 
/// This function:
/// 1. Checks if credentials exist locally
/// 2. Prompts for decryption password if needed
/// 3. Verifies the token is still valid with the registry
/// 4. Returns the token if everything is valid
/// 
/// # Returns
/// 
/// Returns a valid API token, or an error if not authenticated,
/// decryption fails, or token verification fails.
/// 
/// # Examples
/// 
/// ```rust
/// let token = ensure_authenticated().await?;
/// // Use token for API calls
/// ```
pub async fn ensure_authenticated() -> Result<String> {
    // First check if credentials exist without prompting for password
    if !has_stored_credentials()? {
        return Err(SolanaPmError::ConfigNotFound(
            "Not logged in. Please run 'solpm login' first.".to_string()
        ));
    }
    
    // Only prompt for password when we actually need the token
    match get_stored_token()? {
        Some(token) => {
            // Verify token is still valid
            if verify_token(&token).await? {
                Ok(token)
            } else {
                Err(SolanaPmError::ConfigNotFound(
                    "Token is invalid or expired. Please run 'solpm login' again.".to_string()
                ))
            }
        },
        None => Err(SolanaPmError::ConfigNotFound(
            "Failed to decrypt stored token. Please run 'solpm login' again.".to_string()
        ))
    }
}