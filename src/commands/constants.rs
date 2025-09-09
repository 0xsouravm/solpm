//! # Constants Module
//!
//! This module defines all configuration constants used throughout the
//! Solana Program Manager application. It centralizes:
//!
//! - Backend API URLs and endpoints
//! - File and directory paths for project structure
//! - Network RPC endpoints for Solana clusters
//! - System program identifiers and addresses
//!
//! These constants ensure consistency across all modules and provide
//! a single location for configuration management.

// Backend API URLs
pub const BACKEND_BASE_URL: &str = "https://solpm-registry-production.up.railway.app";
pub const PUBLISH_PROGRAM_URL: &str = "https://solpm-registry-production.up.railway.app/programs";
pub const GET_PROGRAM_URL: &str = "https://solpm-registry-production.up.railway.app/programs";
pub const AUTH_VERIFY_URL: &str = "https://solpm-registry-production.up.railway.app/auth/verify";

// File paths
pub const SOLANA_PROGRAMS_FILE: &str = "SolanaPrograms.json";
pub const PROGRAM_CLIENT_DIR: &str = "./program/client";
pub const PROGRAM_IDL_DIR: &str = "./program/idl";

// Network RPC URLs
pub const MAINNET_RPC_URL: &str = "https://api.mainnet-beta.solana.com";
pub const DEVNET_RPC_URL: &str = "https://api.devnet.solana.com";

// System Program ID
pub const SYSTEM_PROGRAM_ID: &str = "11111111111111111111111111111111";