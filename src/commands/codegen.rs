use crate::commands::constants::{DEVNET_RPC_URL, MAINNET_RPC_URL, PROGRAM_CLIENT_DIR, PROGRAM_IDL_DIR, SOLANA_PROGRAMS_FILE, SYSTEM_PROGRAM_ID};
use crate::commands::types::{Idl, IdlInstruction, IdlSeed, Program, SolanaPrograms};
use crate::error::{Result, SolanaPmError};
use crate::utils::CliStyle;
use std::collections::HashSet;
use std::fs;

/// Generates TypeScript client code for all installed Solana programs.
/// 
/// This function reads the SolanaPrograms.json configuration file and generates
/// TypeScript client files for each program by:
/// 1. Reading IDL files for each program dependency
/// 2. Generating TypeScript wrapper functions for each instruction
/// 3. Creating PDA (Program Derived Address) helper functions
/// 4. Setting up proper imports and network connections
/// 
/// The generated client files are saved in the `program/client/` directory with
/// the naming convention `{ProgramName}Client.ts`.
/// 
/// # Returns
/// 
/// Returns `Ok(())` on success, or an error if configuration files are missing,
/// IDL files cannot be read, or file generation fails.
/// 
/// # Errors
/// 
/// * `SolanaPmError::ConfigNotFound` - If SolanaPrograms.json doesn't exist
/// * `SolanaPmError::InvalidPath` - If required IDL files are missing
/// * File I/O errors during client file generation
pub fn generate_typescript_client() -> Result<()> {
    // Check if SolanaPrograms.json exists
    if !std::path::Path::new(SOLANA_PROGRAMS_FILE).exists() {
        return Err(SolanaPmError::ConfigNotFound(format!("{} not found. Run 'solpm add <program>' first.", SOLANA_PROGRAMS_FILE)));
    }
    
    // Read SolanaPrograms.json
    let solana_programs_content = fs::read_to_string(SOLANA_PROGRAMS_FILE)?;
    let solana_programs: SolanaPrograms = serde_json::from_str(&solana_programs_content)?;
    
    // Create client directory
    std::fs::create_dir_all(PROGRAM_CLIENT_DIR)?;
    
    println!("{}", CliStyle::header("TypeScript Client Generation"));
    println!();
    
    let mut generated_count = 0;
    
    // Process all programs (regular and dev)
    let all_programs = solana_programs.programs.iter()
        .chain(solana_programs.dev_programs.iter());
    
    for (program_name, program_info) in all_programs {
        // Determine IDL file path
        let default_idl_path = format!("{}/{}.json", PROGRAM_IDL_DIR, program_name);
        let idl_file_path = program_info.idl_path.as_deref().unwrap_or(&default_idl_path);
        
        // Check if IDL file exists
        if !std::path::Path::new(idl_file_path).exists() {
            return Err(SolanaPmError::InvalidPath(
                format!("IDL file not found for '{}': {}\nRun {} to fetch missing IDL files.", 
                program_name, idl_file_path, CliStyle::command("solpm install"))
            ));
        }
        
        println!("{}", CliStyle::codegen(&format!(
            "Generating client for {} ({}) from {}...", 
            CliStyle::package(program_name),
            CliStyle::highlight(&program_info.network),
            CliStyle::path(idl_file_path)
        )));
        
        // Read and parse IDL
        let idl_content = fs::read_to_string(idl_file_path)?;
        let idl: Idl = serde_json::from_str(&idl_content)?;
        
        // Generate TypeScript code
        let ts_code = generate_ts_code(&idl, program_name, program_info)?;
        
        // Write client file
        let client_file_name = format!("{}Client.ts", snake_to_pascal(program_name));
        let client_file_path = format!("{}/{}", PROGRAM_CLIENT_DIR, client_file_name);
        fs::write(&client_file_path, ts_code)?;
        
        generated_count += 1;
        println!("{}", CliStyle::success(&format!(
            "Generated {}", 
            CliStyle::path(&client_file_path)
        )));
    }
    
    if generated_count == 0 {
        println!("{}", CliStyle::warning("No client files generated. Make sure IDL files are available."));
    } else {
        println!("\n{}", CliStyle::success(&format!(
            "🎉 Generated {} client{}!", 
            generated_count, 
            if generated_count == 1 { "" } else { "s" }
        )));
    }
    
    Ok(())
}


/// Generates the complete TypeScript client code for a single Solana program.
/// 
/// This function creates a comprehensive TypeScript client by:
/// 1. Setting up imports for Anchor and Solana Web3.js
/// 2. Adding program ID and network configuration
/// 3. Creating connection and program instance helpers
/// 4. Generating PDA (Program Derived Address) functions
/// 5. Creating wrapper functions for each program instruction
/// 
/// # Arguments
/// 
/// * `idl` - The parsed IDL (Interface Definition Language) for the program
/// * `program_name` - The name of the program
/// * `program_info` - Program metadata including network and program ID
/// 
/// # Returns
/// 
/// Returns the complete TypeScript code as a string, or an error if code
/// generation fails.
fn generate_ts_code(idl: &Idl, program_name: &str, program_info: &Program) -> Result<String> {
    let mut code = String::new();
    
    // Imports
    code.push_str("import * as anchor from '@coral-xyz/anchor';\n");
    code.push_str("import { Connection, PublicKey } from '@solana/web3.js';\n");
    
    // Generate correct IDL import path relative to the client file location
    let default_idl_path = format!("../idl/{}.json", program_name);
    let idl_path = if let Some(custom_path) = &program_info.idl_path {
        // Convert absolute/relative custom path to relative from program/client/
        if custom_path.starts_with("./") {
            format!("../../{}", &custom_path[2..])
        } else if custom_path.starts_with("/") {
            custom_path.clone()
        } else {
            format!("../../{}", custom_path)
        }
    } else {
        default_idl_path
    };
    code.push_str(&format!("import idl from '{}';\n\n", idl_path));
    
    // Constants
    code.push_str(&format!("// Your deployed program ID\n"));
    code.push_str(&format!("const PROGRAM_ID = new PublicKey('{}');\n\n", program_info.program_id));
    
    // Connection and getProgram
    let (network_comment, rpc_url) = match program_info.network.as_str() {
        "mainnet" => ("// Mainnet connection", MAINNET_RPC_URL),
        "devnet" => ("// Devnet connection", DEVNET_RPC_URL),
        _ => ("// Unknown network, defaulting to devnet", DEVNET_RPC_URL),
    };
    code.push_str(&format!("{}\n", network_comment));
    code.push_str(&format!("const connection = new Connection('{}', 'confirmed');\n\n", rpc_url));
    code.push_str("// Get program instance\n");
    code.push_str("const getProgram = (wallet) => {\n");
    code.push_str("  const provider = new anchor.AnchorProvider(connection, wallet, {\n");
    code.push_str("    commitment: 'confirmed',\n");
    code.push_str("  });\n");
    code.push_str("  \n");
    code.push_str("  return new anchor.Program(idl, provider);\n");
    code.push_str("};\n\n");
    
    // Generate PDA helper functions
    generate_pda_functions(&mut code, idl)?;
    
    // Generate instruction wrapper functions
    for instruction in &idl.instructions {
        generate_instruction_function(&mut code, instruction, idl)?;
    }
    
    Ok(code)
}

/// Generates TypeScript functions for deriving Program Derived Addresses (PDAs).
/// 
/// This function analyzes all instructions in the IDL to find accounts that use PDAs
/// and generates corresponding helper functions for deriving those addresses.
/// Each PDA function handles seed parsing and buffer conversion appropriately.
/// 
/// # Arguments
/// 
/// * `code` - Mutable string to append the generated PDA functions to
/// * `idl` - The IDL containing account definitions with PDA specifications
/// 
/// # Returns
/// 
/// Returns `Ok(())` on success, or an error if PDA seed parsing fails.
fn generate_pda_functions(code: &mut String, idl: &Idl) -> Result<()> {
    let mut generated_pdas = HashSet::new();
    
    // Collect all unique PDA patterns from all instructions
    for instruction in &idl.instructions {
        for account in &instruction.accounts {
            if let Some(pda) = &account.pda {
                let pda_name = &account.name;
                if generated_pdas.contains(pda_name) {
                    continue;
                }
                generated_pdas.insert(pda_name.clone());
                
                let function_name = format!("get{}PDA", snake_to_pascal(&account.name));
                
                // Parse seeds to determine function parameters
                let (params, seed_buffers) = parse_pda_seeds(&pda.seeds, &instruction.args)?;
                
                code.push_str(&format!("// Get {} PDA\n", account.name));
                code.push_str(&format!("export const {} = ({}) => {{\n", function_name, params.join(", ")));
                code.push_str("  return PublicKey.findProgramAddressSync(\n");
                code.push_str("    [\n");
                
                for seed_buffer in seed_buffers {
                    code.push_str(&format!("      {},\n", seed_buffer));
                }
                
                code.push_str("    ],\n");
                code.push_str("    PROGRAM_ID\n");
                code.push_str("  );\n");
                code.push_str("};\n\n");
            }
        }
    }
    
    Ok(())
}

/// Generates a TypeScript wrapper function for a single Solana program instruction.
/// 
/// This function creates a complete wrapper that:
/// 1. Derives required PDAs for accounts that need them
/// 2. Sets up the proper accounts object with signers, writeable accounts, etc.
/// 3. Handles argument passing and type conversion
/// 4. Returns transaction signature and any derived PDAs
/// 
/// # Arguments
/// 
/// * `code` - Mutable string to append the generated function to
/// * `instruction` - The IDL instruction definition to generate code for
/// * `_idl` - The complete IDL (unused but available for future enhancements)
/// 
/// # Returns
/// 
/// Returns `Ok(())` on success, or an error if function generation fails.
fn generate_instruction_function(code: &mut String, instruction: &IdlInstruction, _idl: &Idl) -> Result<()> {
    let function_name = snake_to_camel(&instruction.name);
    
    code.push_str(&format!("// {} on-chain\n", function_name));
    code.push_str(&format!("export const {} = async (wallet", function_name));
    
    // Collect all parameters needed for this instruction
    let mut all_params = Vec::new();
    
    // Add instruction args as parameters
    for arg in &instruction.args {
        all_params.push(arg.name.clone());
    }
    
    // Add PDA-derived parameters
    for account in &instruction.accounts {
        if let Some(pda) = &account.pda {
            let (pda_params, _) = parse_pda_seeds(&pda.seeds, &instruction.args)?;
            for param in pda_params {
                if !all_params.contains(&param) && param != "creator" {
                    all_params.push(param);
                }
            }
        }
    }
    
    // Add all parameters to function signature
    for param in &all_params {
        code.push_str(&format!(", {}", param));
    }
    
    code.push_str(") => {\n");
    code.push_str("  const program = getProgram(wallet);\n");
    
    // Generate PDA derivations for accounts that need them
    let mut pda_variables = Vec::new();
    for account in &instruction.accounts {
        if let Some(pda) = &account.pda {
            let pda_function_name = format!("get{}PDA", snake_to_pascal(&account.name));
            let pda_var_name = format!("{}Pda", snake_to_camel(&account.name));
            
            let (pda_params, _) = parse_pda_seeds(&pda.seeds, &instruction.args)?;
            
            let mut call_params = Vec::new();
            for param in pda_params {
                if param == "creator" {
                    call_params.push("wallet.publicKey".to_string());
                } else {
                    call_params.push(param);
                }
            }
            
            code.push_str(&format!("  const [{}] = {}({});\n", 
                pda_var_name, pda_function_name, call_params.join(", ")));
            
            pda_variables.push((account.name.clone(), pda_var_name));
        }
    }
    
    code.push_str("  \n");
    code.push_str("  const tx = await program.methods\n");
    code.push_str(&format!("    .{}(", snake_to_camel(&instruction.name)));
    
    // Add method arguments
    for (i, arg) in instruction.args.iter().enumerate() {
        if i > 0 { code.push_str(", "); }
        code.push_str(&arg.name);
    }
    
    code.push_str(")\n");
    code.push_str("    .accounts({\n");
    
    // Generate accounts object - completely generic
    for account in &instruction.accounts {
        let account_camel = snake_to_camel(&account.name);
        let writable_comment = if account.is_writable() { " // writable" } else { "" };
        let signer_comment = if account.is_signer_account() { " // signer" } else { "" };
        
        // Check if this account has a PDA
        if let Some((_, pda_var)) = pda_variables.iter().find(|(name, _)| name == &account.name) {
            code.push_str(&format!("      {}: {},{}{}  \n", account_camel, pda_var, writable_comment, signer_comment));
        }
        // Check if it's a signer (typically wallet.publicKey) 
        else if account.is_signer_account() {
            code.push_str(&format!("      {}: wallet.publicKey,{}{}\n", account_camel, writable_comment, signer_comment));
        }
        // Check if it has a fixed address
        else if let Some(address) = &account.address {
            // Special case for system program
            if address == SYSTEM_PROGRAM_ID {
                code.push_str(&format!("      {}: anchor.web3.SystemProgram.programId,{}{}\n", account_camel, writable_comment, signer_comment));
            } else {
                code.push_str(&format!("      {}: new PublicKey('{}'),{}{}\n", account_camel, address, writable_comment, signer_comment));
            }
        }
        // Default case - parameter or TODO
        else {
            code.push_str(&format!("      {}: {}, // TODO: Add proper account{}{}\n", account_camel, account_camel, writable_comment, signer_comment));
        }
    }
    
    code.push_str("    })\n");
    code.push_str("    .rpc();\n");
    code.push_str("    \n");
    
    // Return appropriate value based on whether we have PDAs
    if pda_variables.is_empty() {
        code.push_str("  return tx;\n");
    } else {
        let primary_pda = &pda_variables[0].1; // Use first PDA as primary return
        code.push_str(&format!("  return {{ tx, pda: {} }};\n", primary_pda));
    }
    
    code.push_str("};\n\n");
    
    Ok(())
}

/// Converts snake_case strings to camelCase.
/// 
/// # Arguments
/// 
/// * `s` - The snake_case string to convert
/// 
/// # Returns
/// 
/// Returns the string converted to camelCase format.
fn snake_to_camel(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    
    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    
    result
}

/// Converts snake_case strings to PascalCase.
/// 
/// # Arguments
/// 
/// * `s` - The snake_case string to convert
/// 
/// # Returns
/// 
/// Returns the string converted to PascalCase format.
fn snake_to_pascal(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

/// Converts byte array to a string representation.
/// 
/// Attempts to convert bytes to UTF-8 string, falling back to hex representation
/// if the bytes are not valid UTF-8.
/// 
/// # Arguments
/// 
/// * `bytes` - The byte array to convert
/// 
/// # Returns
/// 
/// Returns either the UTF-8 string or hex representation.
fn bytes_to_string(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap_or_else(|_| {
        // If not valid UTF-8, return as hex representation
        format!("0x{}", hex::encode(bytes))
    })
}

/// Extracts the parameter name from a dot-separated path.
/// 
/// Used for parsing account paths in PDA seeds like "feedback_board.creator".
/// 
/// # Arguments
/// 
/// * `path` - The dot-separated path string
/// 
/// # Returns
/// 
/// Returns the last component of the path as the parameter name.
fn extract_param_from_path(path: &str) -> String {
    // "feedback_board.creator" → "creator"
    // "creator" → "creator" 
    path.split('.').last().unwrap().to_string()
}

/// Parses PDA seeds to generate function parameters and buffer conversion code.
/// 
/// This function analyzes the seeds used for PDA derivation and generates:
/// 1. Function parameters needed for the PDA function
/// 2. TypeScript buffer conversion code for each seed
/// 
/// It handles different seed types (const, account, arg) and generates appropriate
/// TypeScript code for buffer conversion based on the underlying data types.
/// 
/// # Arguments
/// 
/// * `seeds` - Array of IDL seed definitions
/// * `instruction_args` - Instruction arguments for type information
/// 
/// # Returns
/// 
/// Returns a tuple of (parameters, buffer_conversions) or an error if seed
/// parsing fails.
fn parse_pda_seeds(seeds: &[IdlSeed], instruction_args: &[crate::commands::types::IdlArg]) -> Result<(Vec<String>, Vec<String>)> {
    let mut params = Vec::new();
    let mut seed_buffers = Vec::new();
    
    for seed in seeds {
        match seed.kind.as_str() {
            "const" => {
                if let Some(value_bytes) = &seed.value {
                    let string_value = bytes_to_string(value_bytes);
                    seed_buffers.push(format!("Buffer.from('{}')", string_value));
                }
            }
            "account" => {
                if let Some(path) = &seed.path {
                    let param_name = extract_param_from_path(path);
                    if !params.contains(&param_name) {
                        params.push(param_name.clone());
                    }
                    
                    // ALL account references are PublicKeys, so they need .toBuffer()
                    seed_buffers.push(format!("{}.toBuffer()", param_name));
                }
            }
            "arg" => {
                if let Some(path) = &seed.path {
                    let param_name = path.to_string();
                    if !params.contains(&param_name) {
                        params.push(param_name.clone());
                    }
                    
                    // Check the actual argument type from instruction args
                    let arg_type = instruction_args.iter()
                        .find(|arg| arg.name == param_name)
                        .map(|arg| arg.get_type_string())
                        .unwrap_or_else(|| "string".to_string());
                    
                    // Generate appropriate buffer conversion based on type
                    let buffer_code = match arg_type.as_str() {
                        "string" => format!("Buffer.from({})", param_name),
                        "u8" => format!("Buffer.from([{}])", param_name),
                        "u16" => format!("Buffer.from(new Uint16Array([{}]))", param_name),
                        "u32" => format!("Buffer.from(new Uint32Array([{}]))", param_name),
                        "u64" => format!("Buffer.from(new anchor.BN({}).toArray('le', 8))", param_name),
                        "i8" => format!("Buffer.from([{} < 0 ? {} + 256 : {}])", param_name, param_name, param_name),
                        "i16" => format!("Buffer.from(new Int16Array([{}]))", param_name),
                        "i32" => format!("Buffer.from(new Int32Array([{}]))", param_name),
                        "i64" => format!("Buffer.from(new anchor.BN({}).toArray('le', 8))", param_name),
                        "bool" => format!("Buffer.from([{} ? 1 : 0])", param_name),
                        "bytes" | "Vec<u8>" => format!("Buffer.from({})", param_name),
                        "publicKey" => format!("{}.toBuffer()", param_name),
                        // Handle custom types and pubkey
                        "pubkey" | "Pubkey" | "PublicKey" => format!("{}.toBuffer()", param_name),
                        // Default fallback for unknown types
                        _ => {
                            // If it looks like a number type we missed, treat as u32
                            if arg_type.starts_with('u') || arg_type.starts_with('i') {
                                format!("Buffer.from(new Uint32Array([{}]))", param_name)
                            } else {
                                // Default to string handling with a comment
                                format!("Buffer.from({}) // TODO: Verify type handling for '{}'", param_name, arg_type)
                            }
                        }
                    };
                    
                    seed_buffers.push(buffer_code);
                }
            }
            _ => {
                return Err(SolanaPmError::InvalidIdl(format!("Unknown seed kind: {}", seed.kind)));
            }
        }
    }
    
    Ok((params, seed_buffers))
}