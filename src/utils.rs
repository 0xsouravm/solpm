//! # Utility Functions Module
//!
//! This module provides common utility functions and types used throughout the
//! Solana Program Manager application. It includes:
//!
//! - CLI styling and formatting utilities
//! - Progress indicators and spinners
//! - User input and confirmation prompts
//! - Project identification and hashing
//! - Package specification parsing
//! - ASCII art banner display
//!
//! The utilities are designed to provide a consistent user experience across
//! all commands with proper error handling and user feedback.

use colored::*;
use console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use sha2::{Sha256, Digest};

/// Represents a parsed package specification with name and optional version.
/// 
/// This struct holds the parsed components of a package specification string,
/// separating the package name from an optional version specifier.
/// 
/// # Fields
/// 
/// * `name` - The package name
/// * `version` - Optional version string (Some if @version was specified, None otherwise)
/// 
/// # Examples
/// 
/// ```rust
/// let spec = PackageSpec {
///     name: "feedana".to_string(),
///     version: Some("0.1.0".to_string()),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct PackageSpec {
    /// The name of the package
    pub name: String,
    /// Optional version specification
    pub version: Option<String>,
}

pub struct CliStyle;

impl CliStyle {
    // Success messages
    /// Formats a success message with a green checkmark.
    /// 
    /// # Arguments
    /// 
    /// * `msg` - The message to format
    /// 
    /// # Returns
    /// 
    /// Returns a formatted string with green checkmark and text.
    pub fn success(msg: &str) -> String {
        format!("{} {}", "✓".green().bold(), msg.green())
    }

    // Warning messages
    /// Formats a warning message with a yellow warning symbol.
    /// 
    /// # Arguments
    /// 
    /// * `msg` - The message to format
    /// 
    /// # Returns
    /// 
    /// Returns a formatted string with yellow warning symbol and text.
    pub fn warning(msg: &str) -> String {
        format!("{} {}", "⚠".yellow().bold(), msg.yellow())
    }

    // Error messages
    /// Formats an error message with a red X symbol.
    /// 
    /// # Arguments
    /// 
    /// * `msg` - The message to format
    /// 
    /// # Returns
    /// 
    /// Returns a formatted string with red X symbol and text.
    pub fn error(msg: &str) -> String {
        format!("{} {}", "✗".red().bold(), msg.red())
    }

    // Info messages
    /// Formats an informational message with a blue info symbol.
    /// 
    /// # Arguments
    /// 
    /// * `msg` - The message to format
    /// 
    /// # Returns
    /// 
    /// Returns a formatted string with blue info symbol and text.
    pub fn info(msg: &str) -> String {
        format!("{} {}", "ℹ".blue().bold(), msg.blue())
    }

    // Progress messages
    /// Formats a progress message with a cyan download symbol.
    /// 
    /// # Arguments
    /// 
    /// * `msg` - The message to format
    /// 
    /// # Returns
    /// 
    /// Returns a formatted string with cyan download symbol and text.
    pub fn progress(msg: &str) -> String {
        format!("{} {}", "⬇".cyan().bold(), msg.cyan())
    }

    // Code generation
    /// Formats a code generation message with a magenta refresh symbol.
    /// 
    /// # Arguments
    /// 
    /// * `msg` - The message to format
    /// 
    /// # Returns
    /// 
    /// Returns a formatted string with magenta refresh symbol and text.
    pub fn codegen(msg: &str) -> String {
        format!("{} {}", "🔄".magenta().bold(), msg.magenta())
    }

    // Package/program names
    /// Formats a package name with bold text.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The package name to format
    /// 
    /// # Returns
    /// 
    /// Returns the package name formatted in bold.
    pub fn package(name: &str) -> String {
        name.bold().to_string()
    }

    // Version numbers
    /// Formats a version string with 'v' prefix and dimmed styling.
    /// 
    /// # Arguments
    /// 
    /// * `version` - The version string to format
    /// 
    /// # Returns
    /// 
    /// Returns the version formatted as "v{version}" with dimmed styling.
    pub fn version(version: &str) -> String {
        format!("v{}", version.dimmed())
    }

    // File paths
    /// Formats a file path with cyan color.
    /// 
    /// # Arguments
    /// 
    /// * `path` - The file path to format
    /// 
    /// # Returns
    /// 
    /// Returns the path formatted in cyan color.
    pub fn path(path: &str) -> String {
        path.cyan().to_string()
    }

    // Commands
    /// Formats a command with backticks and yellow bold styling.
    /// 
    /// # Arguments
    /// 
    /// * `cmd` - The command to format
    /// 
    /// # Returns
    /// 
    /// Returns the command formatted as `cmd` in yellow bold.
    pub fn command(cmd: &str) -> String {
        format!("`{}`", cmd.yellow().bold())
    }

    // Headers/titles
    /// Formats a header with bold and underlined text.
    /// 
    /// # Arguments
    /// 
    /// * `msg` - The header message to format
    /// 
    /// # Returns
    /// 
    /// Returns the message formatted in bold and underlined.
    pub fn header(msg: &str) -> String {
        msg.bold().underline().to_string()
    }

    // Highlight important text
    /// Formats text with cyan bold highlighting.
    /// 
    /// # Arguments
    /// 
    /// * `msg` - The message to format
    /// 
    /// # Returns
    /// 
    /// Returns the message formatted in cyan bold for highlighting.
    pub fn highlight(msg: &str) -> String {
        msg.cyan().bold().to_string()
    }
}

pub struct CliProgress;

impl CliProgress {
    /// Creates a new animated spinner progress indicator.
    /// 
    /// The spinner displays a rotating animation with the provided message
    /// and updates every 80 milliseconds.
    /// 
    /// # Arguments
    /// 
    /// * `msg` - The message to display next to the spinner
    /// 
    /// # Returns
    /// 
    /// Returns a configured ProgressBar with spinner animation.
    pub fn new_spinner(msg: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠁", "⠂", "⠄", "⡀", "⢀", "⠠", "⠐", "⠈"])
                .template("{spinner:.blue} {msg}")
                .unwrap(),
        );
        pb.set_message(msg.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));
        pb
    }

    /// Creates a new progress bar with specified length.
    /// 
    /// The progress bar shows completion percentage, current position,
    /// and total length with a visual progress indicator.
    /// 
    /// # Arguments
    /// 
    /// * `len` - The total number of items to track
    /// * `msg` - The message to display with the progress bar
    /// 
    /// # Returns
    /// 
    /// Returns a configured ProgressBar for tracking progress.
    pub fn new_progress_bar(len: u64, msg: &str) -> ProgressBar {
        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} {percent}%")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );
        pb.set_message(msg.to_string());
        pb
    }

    /// Finishes a progress bar with a success message.
    /// 
    /// # Arguments
    /// 
    /// * `pb` - The progress bar to finish
    /// * `msg` - The success message to display
    pub fn finish_with_message(pb: ProgressBar, msg: &str) {
        pb.finish_with_message(CliStyle::success(msg));
    }

    /// Finishes a progress bar with an error message.
    /// 
    /// # Arguments
    /// 
    /// * `pb` - The progress bar to finish
    /// * `msg` - The error message to display
    pub fn finish_with_error(pb: ProgressBar, msg: &str) {
        pb.finish_with_message(CliStyle::error(msg));
    }
}

/// Prints the Solana Program Manager ASCII art banner to stdout.
/// 
/// Displays a colorized ASCII art banner if the terminal supports colors,
/// otherwise displays a simple text version.
pub fn print_banner() {
    let term = Term::stdout();
    if term.features().colors_supported() {
        println!("{}", r#"
            _            
  ___  ___ | |_ __  _ __ 
 / __|/ _ \| |  _ \| '  \
 \__ \ (_) | | |_) | | | |
 |___/\___/|_| .__/|_|_|_|
             |_|        "#.cyan().bold());
        println!("  {}\n", "Solana Program Manager".bold().white());
    } else {
        println!("solpm - Solana Program Manager");
    }
}

/// Prompts the user for a yes/no confirmation.
/// 
/// Uses an interactive prompt with the provided message and defaults to 'no'.
/// 
/// # Arguments
/// 
/// * `msg` - The confirmation prompt message
/// 
/// # Returns
/// 
/// Returns `true` if user confirms, `false` if they decline or on error.
pub fn confirm_action(msg: &str) -> bool {
    use dialoguer::Confirm;
    
    Confirm::new()
        .with_prompt(msg)
        .default(false)
        .interact()
        .unwrap_or(false)
}

/// Prompts the user for text input.
/// 
/// Displays an interactive text input prompt with an optional default value.
/// 
/// # Arguments
/// 
/// * `msg` - The input prompt message
/// * `default` - Optional default value to use if user provides no input
/// 
/// # Returns
/// 
/// Returns `Some(String)` with the user's input, or `None` if input fails.
pub fn prompt_input(msg: &str, default: Option<&str>) -> Option<String> {
    use dialoguer::Input;
    
    let mut input = Input::<String>::new().with_prompt(msg);
    
    if let Some(def) = default {
        input = input.default(def.to_string());
    }
    
    input.interact().ok()
}

/// Generates a unique project hash for download tracking.
/// 
/// Creates a hash based on GitHub repository URL if available, otherwise falls back
/// to the current working directory path to uniquely identify this project for 
/// download deduplication purposes.
/// 
/// Priority order:
/// 1. GitHub repository URL (from git remote origin)
/// 2. Current working directory path
/// 
/// # Returns
/// 
/// Returns a hex-encoded SHA-256 hash of the project identifier.
/// 
/// # Examples
/// 
/// ```rust
/// let project_hash = generate_project_hash();
/// println!("Project hash: {}", project_hash);
/// ```
pub fn generate_project_hash() -> String {
    let mut hasher = Sha256::new();
    
    // Try to get GitHub repository URL first
    if let Some(repo_url) = get_github_repository_url() {
        hasher.update(repo_url.as_bytes());
    } else {
        // Fallback to current directory path
        let current_dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."));
        hasher.update(current_dir.to_string_lossy().as_bytes());
    }
    
    format!("{:x}", hasher.finalize())
}

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

/// Normalizes GitHub URLs to a consistent format for hashing.
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

/// Parses a package specification string into name and optional version.
/// 
/// Supports the following formats:
/// - `package_name` - Uses latest version
/// - `package_name@version` - Uses specific version
/// 
/// # Arguments
/// 
/// * `package_spec` - The package specification string to parse
/// 
/// # Returns
/// 
/// Returns a `PackageSpec` with the parsed name and optional version.
/// 
/// # Examples
/// 
/// ```rust
/// let spec = parse_package_spec("feedana@0.1.0");
/// assert_eq!(spec.name, "feedana");
/// assert_eq!(spec.version, Some("0.1.0".to_string()));
/// 
/// let spec = parse_package_spec("feedana");
/// assert_eq!(spec.name, "feedana");
/// assert_eq!(spec.version, None);
/// ```
pub fn parse_package_spec(package_spec: &str) -> PackageSpec {
    if let Some(at_pos) = package_spec.find('@') {
        let name = package_spec[..at_pos].to_string();
        let version = package_spec[at_pos + 1..].to_string();
        PackageSpec {
            name,
            version: Some(version),
        }
    } else {
        PackageSpec {
            name: package_spec.to_string(),
            version: None,
        }
    }
}