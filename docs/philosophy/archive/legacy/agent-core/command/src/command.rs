// crabjar/agent-core/command/src/command.rs

use crate::filesystem::{FileSystem, InMemoryFs, MountableFs, OverlayFs, ReadWriteFs};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

/// Command representation
#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    pub stdin: Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub cwd: PathBuf,
    pub env: HashMap<String, String>,
}

impl Command {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            args: Vec::new(),
            stdin: None,
            stdout: None,
            stderr: None,
            cwd: PathBuf::from("/home/user"),
            env: HashMap::new(),
        }
    }

    pub fn args(mut self, args: &[impl Into<String>]) -> Self {
        self.args = args.iter().map(|a| a.into()).collect();
        self
    }

    pub fn cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = cwd.into();
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    pub fn stdin(mut self, stdin: impl Into<String>) -> Self {
        self.stdin = Some(stdin.into());
        self
    }

    pub fn stdout(mut self, stdout: impl Into<String>) -> Self {
        self.stdout = Some(stdout.into());
        self
    }

    pub fn stderr(mut self, stderr: impl Into<String>) -> Self {
        self.stderr = Some(stderr.into());
        self
    }
}

/// Command execution result
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
    pub execution_time: Duration,
}

impl CommandResult {
    pub fn new(stdout: String, stderr: String, exit_code: i32) -> Self {
        Self {
            stdout,
            stderr,
            exit_code,
            success: exit_code == 0,
            execution_time: Duration::ZERO,
        }
    }
}

/// Command execution limits
#[derive(Debug, Clone)]
pub struct CommandLimits {
    pub max_execution_time: Duration,
    pub max_output_size: usize,
    pub max_input_size: usize,
    pub max_args: usize,
    pub max_env_vars: usize,
    pub allow_network_commands: bool,
}

impl Default for CommandLimits {
    fn default() -> Self {
        Self {
            max_execution_time: Duration::from_secs(30),
            max_output_size: 10 * 1024 * 1024, // 10MB
            max_input_size: 1 * 1024 * 1024,   // 1MB
            max_args: 100,
            max_env_vars: 1000,
            allow_network_commands: false,
        }
    }
}

/// Command executor
pub struct CommandExecutor {
    fs: Arc<Mutex<Box<dyn FileSystem>>>,
    limits: CommandLimits,
    execution_count: Arc<Mutex<usize>>,
    network_config: Option<NetworkConfig>,
}

impl CommandExecutor {
    pub fn new(fs: Box<dyn FileSystem>) -> Self {
        Self {
            fs: Arc::new(Mutex::new(fs)),
            limits: CommandLimits::default(),
            execution_count: Arc::new(Mutex::new(0)),
            network_config: None,
        }
    }

    pub fn with_limits(mut self, limits: CommandLimits) -> Self {
        self.limits = limits;
        self
    }

    pub fn with_network_config(mut self, network_config: NetworkConfig) -> Self {
        self.network_config = Some(network_config);
        self
    }

    pub async fn execute(&mut self, cmd: &Command) -> Result<CommandResult, CommandError> {
        let mut execution_count = self.execution_count.lock().await;
        *execution_count += 1;

        if *execution_count > 1000 {
            return Err(CommandError::MaxExecutionCountExceeded);
        }

        // Validate command
        self.validate_command(cmd)?;

        // Check execution time limit
        let start_time = std::time::Instant::now();

        // Execute command
        let result = self.run_command(cmd).await;

        let execution_time = start_time.elapsed();

        // Check output size limit
        if result.stdout.len() > self.limits.max_output_size
            || result.stderr.len() > self.limits.max_output_size
        {
            return Err(CommandError::OutputSizeExceeded);
        }

        let mut result = result;
        result.execution_time = execution_time;

        Ok(result)
    }

    fn validate_command(&self, cmd: &Command) -> Result<(), CommandError> {
        // Check command name
        if cmd.name.is_empty() {
            return Err(CommandError::InvalidCommand(
                "Empty command name".to_string(),
            ));
        }

        // Check argument count
        if cmd.args.len() > self.limits.max_args {
            return Err(CommandError::TooManyArguments {
                count: cmd.args.len(),
                max: self.limits.max_args,
            });
        }

        // Check environment variables
        if cmd.env.len() > self.limits.max_env_vars {
            return Err(CommandError::TooManyEnvironmentVariables {
                count: cmd.env.len(),
                max: self.limits.max_env_vars,
            });
        }

        // Check input size
        if let Some(stdin) = &cmd.stdin {
            if stdin.len() > self.limits.max_input_size {
                return Err(CommandError::InputSizeExceeded {
                    size: stdin.len(),
                    max: self.limits.max_input_size,
                });
            }
        }

        // Check for network commands
        if self.network_config.is_none() {
            let network_commands = ["curl", "wget", "ssh", "scp", "rsync"];
            if network_commands.iter().any(|nc| cmd.name.starts_with(nc)) {
                return Err(CommandError::NetworkCommandNotAllowed);
            }
        }

        Ok(())
    }

    async fn run_command(&self, cmd: &Command) -> CommandResult {
        // Handle built-in commands
        if let Some(result) = self.run_builtin(cmd) {
            return result;
        }

        // External command
        let mut cmd_obj = Command::new(&cmd.name);
        cmd_obj.args(&cmd.args);
        cmd_obj.current_dir(&cmd.cwd);
        cmd_obj.envs(&cmd.env);

        // Set timeout
        let timeout = Some(self.limits.max_execution_time);
        cmd_obj.timeout(timeout);

        // Set stdin
        if let Some(stdin) = &cmd.stdin {
            cmd_obj.stdin(std::process::Stdio::piped());
        }

        // Capture output
        cmd_obj.stdout(std::process::Stdio::piped());
        cmd_obj.stderr(std::process::Stdio::piped());

        // Execute
        let output = match cmd_obj.output() {
            Ok(output) => output,
            Err(e) => {
                return CommandResult::new(String::new(), format!("Command failed: {}", e), -1);
            }
        };

        CommandResult::new(
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
            output.status.code().unwrap_or(-1),
        )
    }

    fn run_builtin(&self, cmd: &Command) -> Option<CommandResult> {
        match cmd.name.as_str() {
            "echo" => {
                let output = cmd.args.join(" ");
                Some(CommandResult::new(output, String::new(), 0))
            }
            "pwd" => {
                let output = cmd.cwd.to_string_lossy().to_string();
                Some(CommandResult::new(output, String::new(), 0))
            }
            "ls" => {
                let entries = self.fs.lock().unwrap().read_dir(&cmd.cwd).ok()?;
                let output = entries
                    .iter()
                    .map(|e| e.name.clone())
                    .collect::<Vec<_>>()
                    .join("\n");
                Some(CommandResult::new(output, String::new(), 0))
            }
            "cd" => {
                if cmd.args.is_empty() {
                    return None;
                }

                let new_path = cmd.cwd.join(&cmd.args[0]);
                if !self.fs.lock().unwrap().exists(&new_path) {
                    return None;
                }

                Some(CommandResult::new(String::new(), String::new(), 0))
            }
            "cat" => {
                let output = String::new();
                for arg in &cmd.args {
                    let file_path = cmd.cwd.join(arg);
                    if let Ok(content) = self.fs.lock().unwrap().read(&file_path) {
                        output.push_str(&content);
                        output.push('\n');
                    }
                }
                Some(CommandResult::new(output, String::new(), 0))
            }
            "exit" => Some(CommandResult::new(String::new(), String::new(), 0)),
            "true" => Some(CommandResult::new(String::new(), String::new(), 0)),
            "false" => Some(CommandResult::new(String::new(), String::new(), 1)),
            _ => None,
        }
    }

    /// Execute a command and return metadata
    pub fn execute_with_metadata(
        &mut self,
        cmd: &Command,
    ) -> Result<(CommandResult, CommandMetadata), CommandError> {
        let result = self.execute(cmd).await?;
        let metadata = CommandMetadata {
            command: cmd.name.clone(),
            args: cmd.args.clone(),
            execution_time: result.execution_time,
            exit_code: result.exit_code,
        };
        Ok((result, metadata))
    }
}

/// Command metadata
#[derive(Debug, Clone)]
pub struct CommandMetadata {
    pub command: String,
    pub args: Vec<String>,
    pub execution_time: Duration,
    pub exit_code: i32,
}

/// Network configuration
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

/// Command errors
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Invalid command: {message}")]
    InvalidCommand { message: String },

    #[error("Too many arguments: {count} > {max}")]
    TooManyArguments { count: usize, max: usize },

    #[error("Too many environment variables: {count} > {max}")]
    TooManyEnvironmentVariables { count: usize, max: usize },

    #[error("Input size exceeded: {size} > {max}")]
    InputSizeExceeded { size: usize, max: usize },

    #[error("Output size exceeded: {size} > {max}")]
    OutputSizeExceeded { size: usize, max: usize },

    #[error("Network command not allowed")]
    NetworkCommandNotAllowed,

    #[error("Max execution count exceeded")]
    MaxExecutionCountExceeded,

    #[error("Command timed out")]
    CommandTimedOut,
}

/// Command registry
pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn Fn(&Command) -> Result<CommandResult, CommandError>>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn register<F>(&mut self, name: impl Into<String>, handler: F)
    where
        F: Fn(&Command) -> Result<CommandResult, CommandError> + 'static,
    {
        self.commands.insert(name.into(), Box::new(handler));
    }

    pub fn execute(&self, cmd: &Command) -> Result<CommandResult, CommandError> {
        let handler = self
            .commands
            .get(&cmd.name)
            .ok_or_else(|| CommandError::InvalidCommand {
                message: format!("Command not found: {}", cmd.name),
            })?;

        handler(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_creation() {
        let cmd = Command::new("echo")
            .args(&["hello", "world"])
            .cwd("/tmp")
            .env("TEST", "value");

        assert_eq!(cmd.name, "echo");
        assert_eq!(cmd.args, vec!["hello", "world"]);
        assert_eq!(cmd.cwd, PathBuf::from("/tmp"));
        assert_eq!(cmd.env.get("TEST"), Some(&"value".to_string()));
    }

    #[test]
    fn test_command_executor() {
        let fs = Box::new(InMemoryFs::new());
        let mut executor = CommandExecutor::new(fs);

        let cmd = Command::new("echo").args(&["test"]);
        let result = executor.execute(&cmd).unwrap();

        assert_eq!(result.stdout, "test");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_builtin_commands() {
        let fs = Box::new(InMemoryFs::new());
        let mut executor = CommandExecutor::new(fs);

        // Test echo
        let cmd = Command::new("echo").args(&["hello"]);
        let result = executor.execute(&cmd).unwrap();
        assert_eq!(result.stdout, "hello");

        // Test pwd
        let cmd = Command::new("pwd");
        let result = executor.execute(&cmd).unwrap();
        assert!(result.stdout.contains("/home/user"));

        // Test exit
        let cmd = Command::new("exit");
        let result = executor.execute(&cmd).unwrap();
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_command_registry() {
        let mut registry = CommandRegistry::new();

        registry.register("custom", |cmd| {
            Ok(CommandResult::new(
                format!("Custom command: {:?}", cmd.args),
                String::new(),
                0,
            ))
        });

        let cmd = Command::new("custom").args(&["arg1", "arg2"]);
        let result = registry.execute(&cmd).unwrap();

        assert!(result.stdout.contains("Custom command"));
    }

    #[test]
    fn test_error_handling() {
        let fs = Box::new(InMemoryFs::new());
        let mut executor = CommandExecutor::new(fs);

        // Test non-existent file
        let cmd = Command::new("cat").args(&["nonexistent.txt"]);
        let result = executor.execute(&cmd).unwrap();
        assert_ne!(result.exit_code, 0);
    }
}
