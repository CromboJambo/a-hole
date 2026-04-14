//! Tool implementation pattern for AutoAgents integration
//!
//! This module provides a base implementation pattern for creating tools
//! that can be used with the AutoAgents framework.
//!
//! # Example
//!
//! ```rust
//! use autoagents_derive::tool;
//! use autoagents::core::tool::{ToolCallError, ToolInputT, ToolRuntime, ToolT};
//! use serde::{Deserialize, Serialize};
//! use serde_json::Value;
//! use std::sync::Arc;
//! use async_trait::async_trait;
//! use std::path::Path;
//!
//! #[derive(Serialize, Deserialize, ToolInput, Debug)]
//! pub struct ReadFileArgs {
//!     #[input(description = "Path to the file to read")]
//!     path: String,
//! }
//!
//! #[tool(
//!     name = "read_file",
//!     description = "Read the contents of a file",
//!     input = ReadFileArgs
//! )]
//! struct ReadFileTool;
//!
//! #[async_trait]
//! impl ToolRuntime for ReadFileTool {
//!     async fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
//!         let args: ReadFileArgs = serde_json::from_value(args)?;
//!         // Implementation here
//!         Ok(json!({
//!             "success": true,
//!             "content": "file content"
//!         }))
//!     }
//! }
//! ```

use async_trait::async_trait;
use autoagents::core::tool::{ToolCallError, ToolInputT, ToolRuntime, ToolT};
use autoagents_derive::tool;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;

// ============================================================================
// File System Tools
// ============================================================================

/// Read file contents
#[derive(Serialize, Deserialize, ToolInput, Debug)]
pub struct ReadFileArgs {
    #[input(description = "Path to the file to read")]
    pub path: String,
}

#[tool(
    name = "read_file",
    description = "Read the contents of a file",
    input = ReadFileArgs
)]
pub struct ReadFileTool;

#[async_trait]
impl ToolRuntime for ReadFileTool {
    async fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let args: ReadFileArgs = serde_json::from_value(args)?;

        // Security check: ensure we're not accessing home directory
        if args.path.starts_with("/home/") {
            return Err(ToolCallError::custom(
                "Access to home directory is prohibited",
            ));
        }

        match fs::read_to_string(&args.path) {
            Ok(content) => Ok(json!({
                "success": true,
                "content": content,
                "path": args.path
            })),
            Err(e) => Ok(json!({
                "success": false,
                "error": e.to_string(),
                "path": args.path
            })),
        }
    }
}

/// Write file contents
#[derive(Serialize, Deserialize, ToolInput, Debug)]
pub struct WriteFileArgs {
    #[input(description = "Path to the file to write")]
    pub path: String,
    #[input(description = "Content to write to the file")]
    pub content: String,
}

#[tool(
    name = "write_file",
    description = "Write content to a file, creating directories if needed",
    input = WriteFileArgs
)]
pub struct WriteFileTool;

#[async_trait]
impl ToolRuntime for WriteFileTool {
    async fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let args: WriteFileArgs = serde_json::from_value(args)?;

        // Security check: ensure we're not accessing home directory
        if args.path.starts_with("/home/") {
            return Err(ToolCallError::custom(
                "Access to home directory is prohibited",
            ));
        }

        // Security check: ensure we're only writing to /jar/workspace
        if let Some(workspace) = std::env::var("JAR_WORKSPACE").ok() {
            let path = Path::new(&args.path);
            if !path.starts_with(&workspace) {
                return Err(ToolCallError::custom("Cannot write outside of workspace"));
            }
        }

        // Create parent directories
        if let Some(parent) = Path::new(&args.path).parent() {
            let _ = fs::create_dir_all(parent);
        }

        match fs::write(&args.path, &args.content) {
            Ok(_) => Ok(json!({
                "success": true,
                "path": args.path,
                "bytes_written": args.content.len()
            })),
            Err(e) => Ok(json!({
                "success": false,
                "error": e.to_string(),
                "path": args.path
            })),
        }
    }
}

/// List directory contents
#[derive(Serialize, Deserialize, ToolInput, Debug)]
pub struct ListDirArgs {
    #[input(description = "Path to the directory to list")]
    pub path: String,
}

#[tool(
    name = "list_directory",
    description = "List the contents of a directory",
    input = ListDirArgs
)]
pub struct ListDirTool;

#[async_trait]
impl ToolRuntime for ListDirTool {
    async fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let args: ListDirArgs = serde_json::from_value(args)?;

        // Security check: ensure we're not accessing home directory
        if args.path.starts_with("/home/") {
            return Err(ToolCallError::custom(
                "Access to home directory is prohibited",
            ));
        }

        match fs::read_dir(&args.path) {
            Ok(entries) => {
                let files: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect();

                Ok(json!({
                    "success": true,
                    "path": args.path,
                    "entries": files,
                    "count": files.len()
                }))
            }
            Err(e) => Ok(json!({
                "success": false,
                "error": e.to_string(),
                "path": args.path
            })),
        }
    }
}

// ============================================================================
// Cargo/Build Tools
// ============================================================================

/// Run cargo command
#[derive(Serialize, Deserialize, ToolInput, Debug)]
pub struct CargoCommandArgs {
    #[input(description = "Cargo subcommand (build, test, check, etc.)")]
    pub command: String,
    #[input(description = "Additional arguments for cargo")]
    pub args: Vec<String>,
    #[input(description = "Working directory")]
    pub working_dir: String,
}

#[tool(
    name = "run_cargo",
    description = "Run a cargo command in a specified directory",
    input = CargoCommandArgs
)]
pub struct CargoTool;

#[async_trait]
impl ToolRuntime for CargoTool {
    async fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let args: CargoCommandArgs = serde_json::from_value(args)?;

        // Security check: ensure we're not accessing home directory
        if args.working_dir.starts_with("/home/") {
            return Err(ToolCallError::custom(
                "Access to home directory is prohibited",
            ));
        }

        let output = Command::new("cargo")
            .arg(&args.command)
            .args(&args.args)
            .current_dir(&args.working_dir)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                Ok(json!({
                    "success": output.status.success(),
                    "exit_code": output.status.code(),
                    "stdout": stdout,
                    "stderr": stderr,
                    "command": format!("cargo {} {}", args.command, args.args.join(" "))
                }))
            }
            Err(e) => Ok(json!({
                "success": false,
                "error": e.to_string(),
                "command": format!("cargo {} {}", args.command, args.args.join(" "))
            })),
        }
    }
}

// ============================================================================
// Search/Discovery Tools
// ============================================================================

/// Search crates.io for libraries
#[derive(Serialize, Deserialize, ToolInput, Debug)]
pub struct SearchCratesArgs {
    #[input(description = "Search query for crates.io")]
    pub query: String,
}

#[tool(
    name = "search_crates",
    description = "Search crates.io for Rust libraries",
    input = SearchCratesArgs
)]
pub struct SearchCratesTool;

#[async_trait]
impl ToolRuntime for SearchCratesTool {
    async fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let args: SearchCratesArgs = serde_json::from_value(args)?;

        // In production, use the crates.io API. For now, provide suggestions.
        Ok(json!({
            "query": args.query,
            "suggestions": [
                "For ML: burn, candle-core, linfa, ndarray",
                "For tensors: burn-tensor, tch, ndarray",
                "For linear algebra: nalgebra, ndarray-linalg",
                "Search manually at: https://crates.io/search?q=" + &args.query
            ]
        }))
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn path() -> Option<&'static str> {
    // Can be used for path validation
    None
}
