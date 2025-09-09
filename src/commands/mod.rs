//! # Commands Module
//!
//! This module contains all the command implementations for the Solana Program Manager.
//! Each submodule handles a specific CLI command:
//!
//! - `add`: Add program dependencies to a project
//! - `auth`: Authentication and credential management
//! - `codegen`: TypeScript client code generation
//! - `constants`: API URLs and configuration constants
//! - `init`: Project initialization and configuration
//! - `install`: Install program dependencies from existing file
//! - `publish`: Program publishing to the registry
//! - `types`: Shared data structures and types
//!
//! All commands follow a consistent pattern of input validation, API communication,
//! file management, and user feedback.

pub mod add;
pub mod auth;
pub mod codegen;
pub mod constants;
pub mod init;
pub mod install;
pub mod publish;
pub mod types;