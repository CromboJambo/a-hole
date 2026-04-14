//! Abstract Syntax Tree (AST) implementation for CrabJar agent command system
//!
//! This module defines the AST structures that represent commands, arguments,
//! and execution context for the CrabJar agent tooling system.
//!
//! The AST provides a structured representation of user commands that can be
//! analyzed, transformed, and executed safely within the agent environment.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents a command in the AST with its name and arguments
#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
}

impl Command {
    /// Create a new command with a name and optional arguments
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            args: Vec::new(),
        }
    }

    /// Add one or more arguments to the command
    pub fn with_args(mut self, args: &[impl Into<String>]) -> Self {
        self.args = args.iter().map(|a| a.into()).collect();
        self
    }

    /// Check if this command has any arguments
    pub fn has_args(&self) -> bool {
        !self.args.is_empty()
    }

    /// Get the first argument, if it exists
    pub fn first_arg(&self) -> Option<&String> {
        self.args.first()
    }

    /// Convert to a string representation for debugging
    pub fn to_string(&self) -> String {
        let args_str = if self.has_args() {
            format!(" {}", self.args.join(" "))
        } else {
            "".to_string()
        };
        format!("{}{}", self.name, args_str)
    }
}

/// Represents an expression in the AST that can be evaluated
#[derive(Debug, Clone)]
pub enum Expression {
    /// A literal string value
    Literal(String),

    /// A variable reference (e.g., $HOME, $PATH)
    Variable(String),

    /// A command with its arguments
    Command(Command),

    /// A pipeline of expressions connected by pipes
    Pipeline(Vec<Expression>),

    /// A conditional expression (if/then/else)
    Conditional {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Option<Box<Expression>>,
    },
}

impl Expression {
    /// Create a literal string expression
    pub fn literal(s: impl Into<String>) -> Self {
        Expression::Literal(s.into())
    }

    /// Create a variable reference expression
    pub fn variable(name: impl Into<String>) -> Self {
        Expression::Variable(name.into())
    }

    /// Create a command expression
    pub fn command(cmd: Command) -> Self {
        Expression::Command(cmd)
    }

    /// Create a pipeline of expressions
    pub fn pipeline(expressions: Vec<Expression>) -> Self {
        Expression::Pipeline(expressions)
    }

    /// Create a conditional expression (if condition then expr else optional_expr)
    pub fn conditional(
        condition: Expression,
        then_branch: Expression,
        else_branch: Option<Expression>,
    ) -> Self {
        Expression::Conditional {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        }
    }

    /// Get the type of this expression
    pub fn kind(&self) -> &'static str {
        match self {
            Expression::Literal(_) => "literal",
            Expression::Variable(_) => "variable",
            Expression::Command(_) => "command",
            Expression::Pipeline(_) => "pipeline",
            Expression::Conditional { .. } => "conditional",
        }
    }

    /// Convert to a string representation for debugging
    pub fn to_string(&self) -> String {
        match self {
            Expression::Literal(s) => format!("\"{}\"", s),
            Expression::Variable(name) => format!("${}", name),
            Expression::Command(cmd) => cmd.to_string(),
            Expression::Pipeline(exprs) => {
                let expr_strs: Vec<String> = exprs.iter().map(|e| e.to_string()).collect();
                expr_strs.join(" | ")
            }
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_str = condition.to_string();
                let then_str = then_branch.to_string();
                match else_branch {
                    Some(else_expr) => format!(
                        "if {} then {} else {}",
                        cond_str,
                        then_str,
                        else_expr.to_string()
                    ),
                    None => format!("if {} then {}", cond_str, then_str),
                }
            }
        }
    }
}

/// Represents the execution context for an AST node
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub cwd: PathBuf,
    pub env: HashMap<String, String>,
    pub stdin: Option<String>,
    pub stdout: Option<PathBuf>,
    pub stderr: Option<PathBuf>,
    pub timeout: Option<std::time::Duration>,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            cwd: PathBuf::from("/home/user"),
            env: HashMap::new(),
            stdin: None,
            stdout: None,
            stderr: None,
            timeout: Some(std::time::Duration::from_secs(30)),
        }
    }
}

impl ExecutionContext {
    /// Create a new execution context with a specified working directory
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        Self {
            cwd: cwd.into(),
            ..Default::default()
        }
    }

    /// Set the current working directory
    pub fn with_cwd(mut self, path: impl Into<PathBuf>) -> Self {
        self.cwd = path.into();
        self
    }

    /// Add an environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set stdin input for the command
    pub fn with_stdin(mut self, content: impl Into<String>) -> Self {
        self.stdin = Some(content.into());
        self
    }

    /// Redirect stdout to a file path
    pub fn with_stdout(mut self, path: impl Into<PathBuf>) -> Self {
        self.stdout = Some(path.into());
        self
    }

    /// Redirect stderr to a file path
    pub fn with_stderr(mut self, path: impl Into<PathBuf>) -> Self {
        self.stderr = Some(path.into());
        self
    }

    /// Set execution timeout
    pub fn with_timeout(mut self, duration: std::time::Duration) -> Self {
        self.timeout = Some(duration);
        self
    }
}

/// Represents a parsed AST node that can be executed
#[derive(Debug, Clone)]
pub struct AstNode {
    pub expression: Expression,
    pub context: ExecutionContext,
}

impl AstNode {
    /// Create a new AST node with an expression and execution context
    pub fn new(expression: Expression) -> Self {
        Self {
            expression,
            context: ExecutionContext::default(),
        }
    }

    /// Create a new AST node with an expression and custom context
    pub fn with_context(expression: Expression, context: ExecutionContext) -> Self {
        Self {
            expression,
            context,
        }
    }

    /// Get the current working directory from the context
    pub fn cwd(&self) -> &PathBuf {
        &self.context.cwd
    }

    /// Get environment variables from the context
    pub fn env(&self) -> &HashMap<String, String> {
        &self.context.env
    }

    /// Convert to a string representation for debugging
    pub fn to_string(&self) -> String {
        format!(
            "{} @ {}",
            self.expression.to_string(),
            self.context.cwd.display()
        )
    }
}

/// AST Parser that converts text input into an AST node
pub struct AstParser;

impl AstParser {
    /// Parse a command string into an AST node
    pub fn parse_command(input: &str) -> Result<AstNode, ParseError> {
        // Simple parser for basic commands
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            return Err(ParseError::EmptyCommand);
        }

        let name = parts[0].to_string();
        let args = parts[1..].iter().map(|s| s.to_string()).collect();

        let command = Command { name, args };
        let expression = Expression::Command(command);

        Ok(AstNode::new(expression))
    }

    /// Parse a more complex expression with pipes and redirection
    pub fn parse_expression(input: &str) -> Result<AstNode, ParseError> {
        // This is a simplified parser - in a real implementation,
        // this would use proper tokenization and parsing grammar

        if input.contains('|') {
            let parts: Vec<&str> = input.split('|').collect();
            let expressions: Vec<Expression> = parts
                .iter()
                .map(|part| {
                    let trimmed = part.trim();
                    if trimmed.is_empty() {
                        Expression::Literal("".to_string())
                    } else {
                        let cmd_parts: Vec<&str> = trimmed.split_whitespace().collect();
                        if cmd_parts.is_empty() {
                            Expression::Literal("".to_string())
                        } else {
                            let name = cmd_parts[0].to_string();
                            let args = cmd_parts[1..].iter().map(|s| s.to_string()).collect();
                            Expression::Command(Command { name, args })
                        }
                    }
                })
                .collect();

            Ok(AstNode::new(Expression::Pipeline(expressions)))
        } else {
            Self::parse_command(input)
        }
    }
}

/// AST errors
#[derive(Debug)]
pub enum ParseError {
    EmptyCommand,
    InvalidSyntax(String),
    VariableNotFound(String),
    FileNotFound(PathBuf),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::EmptyCommand => write!(f, "Empty command"),
            ParseError::InvalidSyntax(msg) => write!(f, "Invalid syntax: {}", msg),
            ParseError::VariableNotFound(var) => write!(f, "Variable not found: {}", var),
            ParseError::FileNotFound(path) => write!(f, "File not found: {}", path.display()),
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_creation() {
        let cmd = Command::new("echo");
        assert_eq!(cmd.name, "echo");
        assert!(cmd.args.is_empty());

        let cmd_with_args = cmd.with_args(&["hello", "world"]);
        assert_eq!(cmd_with_args.name, "echo");
        assert_eq!(cmd_with_args.args, vec!["hello", "world"]);
    }

    #[test]
    fn test_command_to_string() {
        let cmd = Command::new("ls").with_args(&["-l", "-a"]);
        assert_eq!(cmd.to_string(), "ls -l -a");

        let cmd_no_args = Command::new("pwd");
        assert_eq!(cmd_no_args.to_string(), "pwd");
    }

    #[test]
    fn test_expression_literals() {
        let expr = Expression::literal("hello world");
        assert_eq!(expr.to_string(), "\"hello world\"");
    }

    #[test]
    fn test_expression_variables() {
        let expr = Expression::variable("HOME");
        assert_eq!(expr.to_string(), "$HOME");
    }

    #[test]
    fn test_expression_command() {
        let cmd = Command::new("echo").with_args(&["test"]);
        let expr = Expression::command(cmd);
        assert_eq!(expr.to_string(), "echo test");
    }

    #[test]
    fn test_expression_pipeline() {
        let expr1 = Expression::command(Command::new("ls"));
        let expr2 = Expression::command(Command::new("grep").with_args(&["txt"]));

        let pipeline = Expression::pipeline(vec![expr1, expr2]);
        assert_eq!(pipeline.to_string(), "ls | grep txt");
    }

    #[test]
    fn test_expression_conditional() {
        let condition = Expression::variable("DEBUG");
        let then_branch = Expression::command(Command::new("echo").with_args(&["debug mode"]));
        let else_branch = Some(Expression::command(
            Command::new("echo").with_args(&["normal mode"]),
        ));

        let conditional =
            Expression::conditional(condition, then_branch.clone(), else_branch.clone());
        assert_eq!(
            conditional.to_string(),
            "if $DEBUG then echo debug mode else echo normal mode"
        );

        // Test without else branch
        let conditional_no_else = Expression::conditional(condition, then_branch, None);
        assert_eq!(
            conditional_no_else.to_string(),
            "if $DEBUG then echo debug mode"
        );
    }

    #[test]
    fn test_execution_context() {
        let ctx = ExecutionContext::new("/tmp")
            .with_env("HOME", "/home/user")
            .with_timeout(std::time::Duration::from_secs(10));

        assert_eq!(ctx.cwd, PathBuf::from("/tmp"));
        assert_eq!(ctx.env.get("HOME"), Some(&"/home/user".to_string()));
        assert_eq!(ctx.timeout, Some(std::time::Duration::from_secs(10)));
    }

    #[test]
    fn test_ast_node() {
        let cmd = Command::new("echo").with_args(&["hello"]);
        let expr = Expression::command(cmd);
        let ctx = ExecutionContext::new("/tmp");

        let node = AstNode::with_context(expr, ctx.clone());

        assert_eq!(node.to_string(), "echo hello @ /tmp");
    }

    #[test]
    fn test_ast_parser_simple_command() {
        let result = AstParser::parse_command("ls -l");
        assert!(result.is_ok());

        let node = result.unwrap();
        if let Expression::Command(cmd) = node.expression {
            assert_eq!(cmd.name, "ls");
            assert_eq!(cmd.args, vec!["-l"]);
        }
    }

    #[test]
    fn test_ast_parser_pipeline() {
        let result = AstParser::parse_expression("ls -a | grep .txt");
        assert!(result.is_ok());

        let node = result.unwrap();
        if let Expression::Pipeline(exprs) = node.expression {
            assert_eq!(exprs.len(), 2);

            if let Expression::Command(cmd1) = &exprs[0] {
                assert_eq!(cmd1.name, "ls");
                assert_eq!(cmd1.args, vec!["-a"]);
            }

            if let Expression::Command(cmd2) = &exprs[1] {
                assert_eq!(cmd2.name, "grep");
                assert_eq!(cmd2.args, vec![".txt"]);
            }
        }
    }
}
