// crabjar/agent-core/sandbox/src/sandbox.rs

use crate::filesystem::{FileSystem, InMemoryFs, OverlayFs, ReadWriteFs, MountableFs};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;

/// Execution result containing stdout, stderr, and exit code
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub metadata: Option<ExecutionMetadata>,
}

/// Metadata about the execution
#[derive(Debug, Clone)]
pub struct ExecutionMetadata {
    pub command: String,
    pub execution_time: u64,
    pub filesystem_changes: Vec<String>,
}

/// Configuration for execution limits
#[derive(Debug, Clone)]
pub struct ExecutionLimits {
    pub max_call_depth: usize,
    pub max_command_count: usize,
    pub max_loop_iterations: usize,
    pub max_recursion_depth: usize,
    pub max_file_size: u64,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            max_call_depth: 100,
            max_command_count: 10000,
            max_loop_iterations: 10000,
            max_recursion_depth: 50,
            max_file_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// Configuration for network access
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub allowed_url_prefixes: Vec<String>,
    pub allowed_methods: Vec<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            allowed_url_prefixes: Vec::new(),
            allowed_methods: vec!["GET".to_string(), "HEAD".to_string()],
        }
    }
}

/// Sandbox environment for safe command execution
pub struct Sandbox {
    fs: Arc<Mutex<Box<dyn FileSystem>>>,
    env: HashMap<String, String>,
    cwd: PathBuf,
    limits: ExecutionLimits,
    network: Option<NetworkConfig>,
    execution_count: Arc<Mutex<usize>>,
}

impl Sandbox {
    /// Create a new sandbox with an in-memory filesystem
    pub fn new() -> Self {
        Self::with_fs(InMemoryFs::new())
    }

    /// Create a sandbox with a custom filesystem
    pub fn with_fs(fs: Box<dyn FileSystem>) -> Self {
        Self {
            fs: Arc::new(Mutex::new(fs)),
            env: HashMap::new(),
            cwd: PathBuf::from("/home/user"),
            limits: ExecutionLimits::default(),
            network: None,
            execution_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Create a sandbox with an overlay filesystem
    pub fn with_overlay(base_path: PathBuf) -> Self {
        let base_fs = Box::new(ReadWriteFs::new(base_path));
        let overlay_fs = Box::new(OverlayFs::new(base_fs));
        Self::with_fs(overlay_fs)
    }

    /// Create a sandbox with a mountable filesystem
    pub fn with_mountable(
        base_fs: Box<dyn FileSystem>,
        mounts: Vec<(PathBuf, Box<dyn FileSystem>)>,
    ) -> Self {
        let mut mountable_fs = MountableFs::new(base_fs);
        for (mount_point, fs) in mounts {
            mountable_fs.mount(mount_point, fs);
        }
        Self::with_fs(Box::new(mountable_fs))
    }

    /// Set the working directory
    pub fn set_cwd(&mut self, cwd: impl Into<PathBuf>) {
        self.cwd = cwd.into();
    }

    /// Set environment variables
    pub fn set_env(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.env.insert(key.into(), value.into());
    }

    /// Set execution limits
    pub fn set_limits(&mut self, limits: ExecutionLimits) {
        self.limits = limits;
    }

    /// Set network configuration
    pub fn set_network(&mut self, network: NetworkConfig) {
        self.network = Some(network);
    }

    /// Execute a command in the sandbox
    pub async fn exec(&mut self, command: &str) -> Result<ExecutionResult, ExecutionError> {
        let mut execution_count = self.execution_count.lock().await;
        *execution_count += 1;

        if *execution_count > self.limits.max_command_count {
            return Err(ExecutionError::MaxCommandCountExceeded);
        }

        // Parse command and execute
        let result = self.execute_command(command).await?;
        Ok(result)
    }

    /// Execute a command with custom options
    pub async fn exec_with_options(
        &mut self,
        command: &str,
        options: ExecOptions,
    ) -> Result<ExecutionResult, ExecutionError> {
        let mut execution_count = self.execution_count.lock().await;
        *execution_count += 1;

        if *execution_count > self.limits.max_command_count {
            return Err(ExecutionError::MaxCommandCountExceeded);
        }

        // Apply custom options
        if let Some(new_cwd) = options.cwd {
            self.cwd = new_cwd;
        }

        if let Some(new_env) = options.env {
            for (key, value) in new_env {
                self.env.insert(key, value);
            }
        }

        // Parse command and execute
        let result = self.execute_command(command).await?;
        Ok(result)
    }

    /// Execute a command and return metadata
    async fn execute_command(&mut self, command: &str) -> Result<ExecutionResult, ExecutionError> {
        let start_time = std::time::Instant::now();
        let mut filesystem_changes = Vec::new();

        // Parse command
        let parsed = self.parse_command(command)?;

        // Check recursion depth
        if parsed.recursion_depth > self.limits.max_recursion_depth {
            return Err(ExecutionError::MaxRecursionDepthExceeded);
        }

        // Execute based on command type
        let result = match parsed {
            CommandParse::Simple(cmd) => self.execute_simple_command(&cmd, &mut filesystem_changes).await,
            CommandParse::Pipeline(pipeline) => self.execute_pipeline(pipeline, &mut filesystem_changes).await,
            CommandParse::Redirection(redir) => self.execute_redirection(redir, &mut filesystem_changes).await,
        };

        let execution_time = start_time.elapsed().as_millis() as u64;

        let metadata = if execution_time > 0 {
            Some(ExecutionMetadata {
                command: command.to_string(),
                execution_time,
                filesystem_changes,
            })
        } else {
            None
        };

        match result {
            Ok((stdout, stderr, exit_code)) => {
                Ok(ExecutionResult {
                    stdout,
                    stderr,
                    exit_code,
                    metadata,
                })
            }
            Err(e) => Err(e),
        }
    }

    /// Execute a simple command
    async fn execute_simple_command(
        &self,
        cmd: &str,
        filesystem_changes: &mut Vec<String>,
    ) -> Result<(String, String, i32), ExecutionError> {
        // Check for built-in commands
        if let Some(result) = self.execute_builtin(cmd)? {
            return Ok(result);
        }

        // External command
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return Err(ExecutionError::EmptyCommand);
        }

        let command = parts[0];
        let args: Vec<&str> = parts.iter().skip(1).cloned().collect();

        // Check network access for curl
        if command == "curl" && self.network.is_some() {
            self.check_network_access(&args)?;
        }

        // Execute command
        let mut cmd_obj = Command::new(command);
        cmd_obj.args(&args);
        cmd_obj.current_dir(&self.cwd);
        cmd_obj.envs(&self.env);

        let output = cmd_obj.output().await.map_err(|e| {
            ExecutionError::CommandFailed {
                command: command.to_string(),
                message: e.to_string(),
            }
        })?;

        // Track filesystem changes
        if command == "echo" && args.contains(&">") {
            // File operations
        }

        Ok((
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
            output.status.code().unwrap_or(-1),
        ))
    }

    /// Execute a command pipeline
    async fn execute_pipeline(
        &self,
        pipeline: Vec<CommandPart>,
        filesystem_changes: &mut Vec<String>,
    ) -> Result<(String, String, i32), ExecutionError> {
        if pipeline.is_empty() {
            return Err(ExecutionError::EmptyPipeline);
        }

        // Execute first command
        let mut result = self.execute_command_part(&pipeline[0], filesystem_changes).await?;

        // Chain through remaining commands
        for part in pipeline.iter().skip(1) {
            let input = result.stdout.clone();
            let mut part_result = self.execute_command_part(part, filesystem_changes).await?;
            part_result.stdout = input;
            result = part_result;
        }

        Ok(result)
    }

    /// Execute a single command part
    async fn execute_command_part(
        &self,
        part: &CommandPart,
        filesystem_changes: &mut Vec<String>,
    ) -> Result<(String, String, i32), ExecutionError> {
        match part {
            CommandPart::Simple(cmd) => self.execute_simple_command(cmd, filesystem_changes).await,
            CommandPart::Pipeline(parts) => self.execute_pipeline(parts.clone(), filesystem_changes).await,
            CommandPart::Redirection(redir) => self.execute_redirection(redir, filesystem_changes).await,
        }
    }

    /// Execute a command with redirections
    async fn execute_redirection(
        &self,
        redir: &Redirection,
        filesystem_changes: &mut Vec<String>,
    ) -> Result<(String, String, i32), ExecutionError> {
        // Parse redirection
        let parts: Vec<&str> = redir.command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(ExecutionError::EmptyCommand);
        }

        let command = parts[0];
        let args: Vec<&str> = parts.iter().skip(1).cloned().collect();

        // Handle output redirection
        if let Some(output) = &redir.output {
            if output.mode == "append" {
                let file_path = self.cwd.join(&output.path);
                if let Some(content) = self.fs.lock().unwrap().read(&file_path).ok() {
                    filesystem_changes.push(format!("Appended to {}", output.path));
                }
            } else {
                filesystem_changes.push(format!("Wrote to {}", output.path));
            }
        }

        // Check network access
        if command == "curl" && self.network.is_some() {
            self.check_network_access(&args)?;
        }

        // Execute command
        let mut cmd_obj = Command::new(command);
        cmd_obj.args(&args);
        cmd_obj.current_dir(&self.cwd);
        cmd_obj.envs(&self.env);

        let output = cmd_obj.output().await.map_err(|e| {
            ExecutionError::CommandFailed {
                command: command.to_string(),
                message: e.to_string(),
            }
        })?;

        Ok((
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
            output.status.code().unwrap_or(-1),
        ))
    }

    /// Check if command is a built-in
    fn execute_builtin(&self, cmd: &str) -> Result<Option<(String, String, i32)>, ExecutionError> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();

        match parts[0] {
            "echo" => {
                let args: Vec<&str> = parts.iter().skip(1).cloned().collect();
                let output = args.join(" ");
                Ok(Some((output, String::new(), 0)))
            }
            "pwd" => {
                let output = self.cwd.to_string_lossy().to_string();
                Ok(Some((output, String::new(), 0)))
            }
            "ls" => {
                let args: Vec<&str> = parts.iter().skip(1).cloned().collect();
                let entries = self.fs.lock().unwrap().read_dir(&self.cwd)?;
                let output = entries.iter().map(|e| e.name.clone()).collect::<Vec<_>>().join("\n");
                Ok(Some((output, String::new(), 0)))
            }
            "cd" => {
                if parts.len() < 2 {
                    return Err(ExecutionError::InvalidArgument {
                        command: "cd".to_string(),
                        argument: "no directory specified".to_string(),
                    });
                }

                let new_path = self.cwd.join(&parts[1]);
                if !self.fs.lock().unwrap().exists(&new_path) {
                    return Err(ExecutionError::PathNotFound {
                        path: new_path,
                    });
                }

                self.cwd = new_path;
                Ok(Some((String::new(), String::new(), 0)))
            }
            "cat" => {
                let args: Vec<&str> = parts.iter().skip(1).cloned().collect();
                let mut output = String::new();

                for arg in args {
                    let file_path = self.cwd.join(arg);
                    let content = self.fs.lock().unwrap().read(&file_path)?;
                    output.push_str(&content);
                    output.push('\n');
                }

                Ok(Some((output, String::new(), 0)))
            }
            "exit" => {
                Ok(Some((String::new(), String::new(), 0)))
            }
            _ => Ok(None),
        }
    }

    /// Parse command string into structured form
    fn parse_command(&self, command: &str) -> Result<CommandParse, ExecutionError> {
        // Simple parser for demonstration
        // In a real implementation, this would use a proper shell parser

        if command.contains('|') {
            // Pipeline
            let parts: Vec<&str> = command.split('|').collect();
            let pipeline: Vec<CommandPart> = parts
                .iter()
                .map(|p| self.parse_command_part(p))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(CommandParse::Pipeline(pipeline))
        } else if command.contains('&') {
            // Background (not supported in this simple version)
            Err(ExecutionError::UnsupportedFeature {
                feature: "background execution".to_string(),
            })
        } else if command.contains('>') || command.contains('>>') {
            // Output redirection
            let parts: Vec<&str> = command.split('>').collect();
            if parts.len() != 2 {
                return Err(ExecutionError::InvalidSyntax {
                    message: "Invalid redirection syntax".to_string(),
                });
            }

            let mode = if parts[1].contains('>') {
                "append"
            } else {
                "overwrite"
            };

            Ok(CommandParse::Redirection(Redirection {
                command: parts[0].trim().to_string(),
                input: None,
                output: Some(OutputRedirection {
                    path: parts[1].trim().to_string(),
                    mode,
                }),
                append: false,
            }))
        } else {
            // Simple command
            Ok(CommandParse::Simple(command.trim().to_string()))
        }
    }

    /// Parse individual command part
    fn parse_command_part(&self, part: &str) -> Result<CommandPart, ExecutionError> {
        if part.contains('|') {
            let parts: Vec<&str> = part.split('|').collect();
            let pipeline: Vec<CommandPart> = parts
                .iter()
                .map(|p| self.parse_command_part(p))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(CommandPart::Pipeline(pipeline))
        } else if part.contains('>') || part.contains('>>') {
            let parts: Vec<&str> = part.split('>').collect();
            if parts.len() != 2 {
                return Err(ExecutionError::InvalidSyntax {
                    message: "Invalid redirection syntax".to_string(),
                });
            }

            let mode = if parts[1].contains('>') {
                "append"
            } else {
                "overwrite"
            };

            Ok(CommandPart::Redirection(Redirection {
                command: parts[0].trim().to_string(),
                input: None,
                output: Some(OutputRedirection {
                    path: parts[1].trim().to_string(),
                    mode,
                }),
                append: false,
            }))
        } else {
            Ok(CommandPart::Simple(part.trim().to_string()))
        }
    }

    /// Check network access for curl
    fn check_network_access(&self, args: &[&str]) -> Result<(), ExecutionError> {
        if let Some(network) = &self.network {
            for arg in args {
                if arg.starts_with("http://") || arg.starts_with("https://") {
                    let url = arg;
                    let allowed = network.allowed_url_prefixes.iter().any(|prefix| {
                        url.starts_with(prefix)
                    });

                    if !allowed {
                        return Err(ExecutionError::NetworkAccessDenied {
                            url: url.to_string(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Read a file from the filesystem
    pub async fn read_file(&self, path: &Path) -> Result<String, ExecutionError> {
        let content = self.fs.lock().unwrap().read(path)?;
        Ok(content)
    }

    /// Write a file to the filesystem
    pub async fn write_file(&self, path: &Path, content: &str) -> Result<(), ExecutionError> {
        self.fs.lock().unwrap().write(path, content)?;
        Ok(())
    }

    /// Check if a path exists
    pub async fn path_exists(&self, path: &Path) -> bool {
        self.fs.lock().unwrap().exists(path)
    }

    /// Get the current working directory
    pub fn cwd(&self) -> PathBuf {
        self.cwd.clone()
    }

    /// Get environment variables
    pub fn env(&self) -> &HashMap<String, String> {
        &self.env
    }
}

/// Command parsing result
#[derive(Debug, Clone)]
enum CommandParse {
    Simple(String),
    Pipeline(Vec<CommandPart>),
    Redirection(Redirection),
}

/// Command part
#[derive(Debug, Clone)]
enum CommandPart {
    Simple(String),
    Pipeline(Vec<CommandPart>),
    Redirection(Redirection),
}

/// Command with redirections
#[derive(Debug, Clone)]
struct Redirection {
    command: String,
    input: Option<InputRedirection>,
    output: Option<OutputRedirection>,
    append: bool,
}

/// Input redirection
#[derive(Debug, Clone)]
struct InputRedirection {
    path: String,
    mode: String,
}

/// Output redirection
#[derive(Debug, Clone)]
struct OutputRedirection {
    path: String,
    mode: String,
}

/// Execution options
#[derive(Debug, Clone)]
pub struct ExecOptions {
    pub cwd: Option<PathBuf>,
    pub env: Option<Vec<(String, String)>>,
    pub replace_env: bool,
}

impl Default for ExecOptions {
    fn default() -> Self {
        Self {
            cwd: None,
            env: None,
            replace_env: false,
        }
    }
}

/// Execution errors
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Command failed: {command}: {message}")]
    CommandFailed { command: String, message: String },

    #[error("Max command count exceeded")]
    MaxCommandCountExceeded,

    #[error("Max recursion depth exceeded")]
    MaxRecursionDepthExceeded,

    #[error("Empty command")]
    EmptyCommand,

    #[error("Empty pipeline")]
    EmptyPipeline,

    #[error("Invalid syntax: {message}")]
    InvalidSyntax { message: String },

    #[error("Invalid argument: {command}: {argument}")]
    InvalidArgument { command: String, argument: String },

    #[error("Path not found: {path:?}")]
    PathNotFound { path: PathBuf },

    #[error("Network access denied for URL: {url}")]
    NetworkAccessDenied { url: String },

    #[error("Unsupported feature: {feature}")]
    UnsupportedFeature { feature: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_execution() {
        let mut sandbox = Sandbox::new();
        let result = sandbox.exec("echo 'Hello World'").await.unwrap();

        assert_eq!(result.stdout, "Hello World\n");
        assert_eq!(result.exit_code, 0);
    }

    #[tokio::test]
    async fn test_file_operations() {
        let mut sandbox = Sandbox::new();

        // Write file
        sandbox
            .write_file(Path::new("/test.txt"), "test content")
            .await
            .unwrap();

        // Read file
        let content = sandbox.read_file(Path::new("/test.txt")).await.unwrap();
        assert_eq!(content, "test content");

        // Check file exists
        assert!(sandbox.path_exists(Path::new("/test.txt")).await);
    }

    #[tokio::test]
    async fn test_ls_command() {
        let mut sandbox = Sandbox::new();

        // Create directory and file
        sandbox
            .write_file(Path::new("/dir/file.txt"), "content")
            .await
            .unwrap();

        // List directory
        let result = sandbox.exec("ls /dir").await.unwrap();
        assert!(result.stdout.contains("file.txt"));
    }

    #[tokio::test]
    async fn test_env_variables() {
        let mut sandbox = Sandbox::new();
        sandbox.set_env("TEST_VAR", "test_value");

        let result = sandbox.exec("echo $TEST_VAR").await.unwrap();
        assert_eq!(result.stdout, "test_value\n");
    }

    #[tokio::test]
    async fn test_cwd() {
        let mut sandbox = Sandbox::new();
        sandbox.set_cwd("/tmp");

        let result = sandbox.exec("pwd").await.unwrap();
        assert!(result.stdout.contains("/tmp"));
    }

    #[tokio::test]
    async fn test_overlay_fs() {
        let base_path = std::path::Path::new("/tmp/test_base");
        let mut sandbox = Sandbox::with_overlay(base_path.to_path_buf());

        sandbox
            .write_file(Path::new("/test.txt"), "overlay content")
            .await
            .unwrap();

        let content = sandbox.read_file(Path::new("/test.txt")).await.unwrap();
        assert_eq!(content, "overlay content");
    }

    #[tokio::test]
    async fn test_error_handling() {
        let mut sandbox = Sandbox::new();

        // Test non-existent file
        let result = sandbox
            .exec("cat /nonexistent.txt")
            .await
            .unwrap();
        assert_ne!(result.exit_code, 0);
    }
}
