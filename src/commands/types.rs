//! # Type Definitions Module
//!
//! This module contains all the shared data structures and types used across
//! the Solana Program Manager commands. It defines:
//!
//! - Configuration structures for project and program metadata
//! - API request and response types for registry communication
//! - IDL (Interface Definition Language) parsing structures
//! - Program dependency and package management types
//! - Serialization/deserialization traits for JSON and TOML formats
//!
//! These types ensure type safety and consistency across all CLI operations,
//! from project initialization to program publishing and dependency management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct SolanaProgramsConfig {
    pub program: ProgramConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProgramConfig {
    pub name: String,
    pub version: String,
    pub program_id: String,
    pub network: String,
    pub description: String,
    pub repository: String,
    pub authority_keypair: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Program {
    pub version: String,
    pub program_id: String,
    pub network: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idl_path: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SolanaPrograms {
    pub programs: HashMap<String, Program>,
    #[serde(rename = "devPrograms")]
    pub dev_programs: HashMap<String, Program>,
}

#[derive(Serialize, Deserialize)]
pub struct IdlMetadata {
    pub name: String,
    pub version: String,
    pub spec: String,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
pub struct IdlArg {
    pub name: String,
    #[serde(rename = "type")]
    pub arg_type: serde_json::Value,  // Can be string or complex object
}

impl IdlArg {
    pub fn get_type_string(&self) -> String {
        match &self.arg_type {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Object(obj) => {
                // Handle complex types like {"option": "u64"}
                if let Some(option_type) = obj.get("option") {
                    format!("option_{}", option_type.as_str().unwrap_or("unknown"))
                } else if let Some(defined_type) = obj.get("defined") {
                    format!("defined_{}", defined_type.as_str().unwrap_or("unknown"))
                } else {
                    "unknown".to_string()
                }
            }
            _ => "unknown".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct IdlSeed {
    pub kind: String,
    pub value: Option<Vec<u8>>,
    pub path: Option<String>,
    pub account: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct IdlPda {
    pub seeds: Vec<IdlSeed>,
}

#[derive(Serialize, Deserialize)]
pub struct IdlAccount {
    pub name: String,
    // New format
    pub writable: Option<bool>,
    pub signer: Option<bool>,
    // Old format (for backward compatibility)
    #[serde(rename = "isMut")]
    pub is_mut: Option<bool>,
    #[serde(rename = "isSigner")]
    pub is_signer: Option<bool>,
    pub address: Option<String>,
    pub pda: Option<IdlPda>,
}

impl IdlAccount {
    pub fn is_writable(&self) -> bool {
        self.writable.unwrap_or(self.is_mut.unwrap_or(false))
    }
    
    pub fn is_signer_account(&self) -> bool {
        self.signer.unwrap_or(self.is_signer.unwrap_or(false))
    }
}

#[derive(Serialize, Deserialize)]
pub struct IdlInstruction {
    pub name: String,
    pub accounts: Vec<IdlAccount>,
    pub args: Vec<IdlArg>,
}

#[derive(Serialize, Deserialize)]
pub struct Idl {
    pub instructions: Vec<IdlInstruction>,
    pub accounts: Option<Vec<serde_json::Value>>,
    pub events: Option<Vec<serde_json::Value>>,
    pub errors: Option<Vec<serde_json::Value>>,
    pub types: Option<Vec<serde_json::Value>>,
}

#[derive(Serialize)]
pub struct UploadProgramRequest {
    pub name: String,
    pub version: String,
    pub program_id: String,
    pub network: String,
    pub idl: serde_json::Value,
    pub description: String,
    pub repository: String,
    // Cryptographic verification fields
    pub challenge: String,
    pub signature: String,
    pub authority_pubkey: String,
}

#[derive(Deserialize)]
pub struct ProgramResponse {
    pub name: String,
    pub version: String,
    pub program_id: String,
    pub idl: serde_json::Value,
}