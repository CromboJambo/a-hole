// examples/redox_ml_coding_agent.rs
// Autonomous coding agent for generating ML code for Redox OS

use autoagents::prelude::*;
use autoagents_derive::{agent, tool, AgentHooks, AgentOutput, ToolInput};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::process::Command;
use std::sync::Arc;

// ============================================================================
// TOOL DEFINITIONS
// ============================================================================

/// Read file contents
#[derive(Serialize, Deserialize, ToolInput, Debug)]
pub struct FileReadArgs {
    #[input(description = "Path to the file to read")]
    path: String,
}

#[tool(
    name = "read_file",
    description = "Read the contents of a file",
    input = FileReadArgs
)]
struct FileReadTool;

#[async_trait]
impl ToolRuntime for FileReadTool {
    async fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let args: FileReadArgs = serde_json::from_value(args)?;
        match std::fs::read_to_string(&args.path) {
            Ok(content) => Ok(json!({
                "success": true,
                "content": content,
                "path": args.path
            })),
            Err(e) => Ok(json!({
                "success": false,
                "error": e.to_string(),
                "path": args.path
            }))
        }
    }
}

/// Write file contents
#[derive(Serialize, Deserialize, ToolInput, Debug)]
pub struct FileWriteArgs {
    #[input(description = "Path to the file to write")]
    path: String,
    #[input(description = "Content to write to the file")]
    content: String,
}

#[tool(
    name = "write_file",
    description = "Write content to a file, creating directories if needed",
    input = FileWriteArgs
)]
struct FileWriteTool;

#[async_trait]
impl ToolRuntime for FileWriteTool {
    async fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let args: FileWriteArgs = serde_json::from_value(args)?;
        
        // Create parent directories
        if let Some(parent) = std::path::Path::new(&args.path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        
        match std::fs::write(&args.path, &args.content) {
            Ok(_) => Ok(json!({
                "success": true,
                "path": args.path,
                "bytes_written": args.content.len()
            })),
            Err(e) => Ok(json!({
                "success": false,
                "error": e.to_string(),
                "path": args.path
            }))
        }
    }
}

/// List directory contents
#[derive(Serialize, Deserialize, ToolInput, Debug)]
pub struct ListDirArgs {
    #[input(description = "Path to the directory to list")]
    path: String,
}

#[tool(
    name = "list_directory",
    description = "List the contents of a directory",
    input = ListDirArgs
)]
struct ListDirTool;

#[async_trait]
impl ToolRuntime for ListDirTool {
    async fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let args: ListDirArgs = serde_json::from_value(args)?;
        
        match std::fs::read_dir(&args.path) {
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
            }))
        }
    }
}

/// Run cargo command
#[derive(Serialize, Deserialize, ToolInput, Debug)]
pub struct CargoCommandArgs {
    #[input(description = "Cargo subcommand (build, test, check, etc.)")]
    command: String,
    #[input(description = "Additional arguments for cargo")]
    args: Vec<String>,
    #[input(description = "Working directory")]
    working_dir: String,
}

#[tool(
    name = "run_cargo",
    description = "Run a cargo command in a specified directory",
    input = CargoCommandArgs
)]
struct CargoTool;

#[async_trait]
impl ToolRuntime for CargoTool {
    async fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let args: CargoCommandArgs = serde_json::from_value(args)?;
        
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
            }))
        }
    }
}

/// Search crates.io for libraries
#[derive(Serialize, Deserialize, ToolInput, Debug)]
pub struct SearchCratesArgs {
    #[input(description = "Search query for crates.io")]
    query: String,
}

#[tool(
    name = "search_crates",
    description = "Search crates.io for Rust libraries",
    input = SearchCratesArgs
)]
struct SearchCratesTool;

#[async_trait]
impl ToolRuntime for SearchCratesTool {
    async fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let args: SearchCratesArgs = serde_json::from_value(args)?;
        
        // Simple implementation - in production, use the crates.io API
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
// AGENT OUTPUT STRUCTURE
// ============================================================================

#[derive(Debug, Serialize, Deserialize, AgentOutput)]
struct CodingAgentOutput {
    #[output(description = "The generated code or response")]
    code: String,
    
    #[output(description = "Explanation of what was done")]
    explanation: String,
    
    #[output(description = "Files that were created or modified")]
    files_modified: Vec<String>,
    
    #[output(description = "Whether the code compiled successfully")]
    compiled: bool,
}

impl From<ReActAgentOutput> for CodingAgentOutput {
    fn from(out: ReActAgentOutput) -> Self {
        // Try to parse structured output, fall back to text
        serde_json::from_str(&out.response).unwrap_or(CodingAgentOutput {
            code: String::new(),
            explanation: out.response,
            files_modified: vec![],
            compiled: false,
        })
    }
}

// ============================================================================
// AGENT DEFINITION
// ============================================================================

#[agent(
    name = "redox_ml_coder",
    description = "Autonomous coding agent for generating ML code for Redox OS",
    tools = [
        FileReadTool,
        FileWriteTool,
        ListDirTool,
        CargoTool,
        SearchCratesTool
    ],
    output = CodingAgentOutput
)]
#[derive(Clone, AgentHooks, Default)]
struct RedoxMLCoder;

// ============================================================================
// MAIN EXAMPLE
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦞 Redox ML Coding Agent Starting...\n");

    // Check for API key
    let api_key = std::env::var("OPENAI_API_KEY")
        .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
        .expect("Set OPENAI_API_KEY or ANTHROPIC_API_KEY environment variable");

    // Determine provider
    let provider = if std::env::var("OPENAI_API_KEY").is_ok() {
        LLMProvider::OpenAI
    } else {
        LLMProvider::Anthropic
    };

    println!("Using LLM provider: {:?}", provider);

    // Build LLM client
    let llm = LLMBuilder::new()
        .provider(provider)
        .model(match provider {
            LLMProvider::OpenAI => "gpt-4",
            LLMProvider::Anthropic => "claude-3-5-sonnet-20241022",
            _ => "gpt-4",
        })
        .api_key(api_key)
        .build()?;

    // Create the coding agent
    let agent = AgentBuilder::new("redox_ml_coder")
        .description("Autonomous coding agent for Redox OS ML development")
        .instruction(
            r#"You are an expert Rust developer specializing in ML and systems programming.
            
Your primary focus is generating code that will run on Redox OS, a microkernel OS written in Rust.

When generating ML code:
1. Prefer pure Rust libraries: burn, candle, linfa, ndarray
2. Avoid C/C++ dependencies when possible (Redox compatibility)
3. Use safe Rust patterns - Redox benefits from memory safety
4. Keep code modular and well-documented
5. Always test compilation with cargo check

Available tools:
- read_file: Read existing code
- write_file: Create new files
- list_directory: Browse project structure  
- run_cargo: Build, test, check code
- search_crates: Find relevant crates

Project structure:
- src/main.rs: Main entry point
- src/lib.rs: Library code
- Cargo.toml: Dependencies
- tests/: Integration tests

When asked to create ML code:
1. Analyze requirements
2. Choose appropriate libraries
3. Generate well-structured code
4. Write to files
5. Compile and test
6. Report results

Format responses as JSON when possible:
{
  "code": "generated code here",
  "explanation": "what was done",
  "files_modified": ["src/main.rs"],
  "compiled": true
}
"#
        )
        .model(Arc::new(llm))
        .tool(Arc::new(FileReadTool))
        .tool(Arc::new(FileWriteTool))
        .tool(Arc::new(ListDirTool))
        .tool(Arc::new(CargoTool))
        .tool(Arc::new(SearchCratesTool))
        .executor(ReActAgent::default())
        .memory(SlidingWindowMemory::new(100))
        .build()?;

    println!("\n✅ Agent initialized successfully!");
    println!("\n📝 Example tasks:");
    println!("  1. 'Create a simple neural network using burn'");
    println!("  2. 'Implement linear regression with ndarray'");
    println!("  3. 'Write a CNN for image classification using candle'");
    println!("  4. 'Create a clustering algorithm with linfa'");
    println!("\nEnter your task (or 'quit' to exit):\n");

    // Interactive loop
    use std::io::{self, Write};
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "quit" || input == "exit" {
            println!("Goodbye! 👋");
            break;
        }

        if input.is_empty() {
            continue;
        }

        println!("\n🤖 Agent working on: {}\n", input);

        // Create task
        let task = Task::new(input);

        // Execute
        match agent.execute(task).await {
            Ok(result) => {
                println!("\n✅ Task completed!");
                println!("\n📄 Response:");
                println!("{}", result.response);
                println!("\n{'='*60}\n");
            }
            Err(e) => {
                eprintln!("\n❌ Error: {}", e);
                println!("\n{'='*60}\n");
            }
        }
    }

    Ok(())
}

// ============================================================================
// EXAMPLE USAGE
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_operations() {
        let write_tool = FileWriteTool;
        let read_tool = FileReadTool;

        // Write test
        let write_args = json!({
            "path": "/tmp/test_agent.txt",
            "content": "Hello from agent!"
        });
        let result = write_tool.execute(write_args).await.unwrap();
        assert!(result["success"].as_bool().unwrap());

        // Read test
        let read_args = json!({
            "path": "/tmp/test_agent.txt"
        });
        let result = read_tool.execute(read_args).await.unwrap();
        assert!(result["success"].as_bool().unwrap());
        assert_eq!(result["content"].as_str().unwrap(), "Hello from agent!");
    }
}

/* 
EXAMPLE RUN:

$ export OPENAI_API_KEY="sk-..."
$ cargo run --example redox_ml_coding_agent

🦞 Redox ML Coding Agent Starting...

Using LLM provider: OpenAI

✅ Agent initialized successfully!

📝 Example tasks:
  1. 'Create a simple neural network using burn'
  2. 'Implement linear regression with ndarray'
  3. 'Write a CNN for image classification using candle'
  4. 'Create a clustering algorithm with linfa'

Enter your task (or 'quit' to exit):

> Create a simple 2-layer neural network for MNIST using burn

🤖 Agent working on: Create a simple 2-layer neural network for MNIST using burn

[Agent uses tools to create files, write code, and test compilation]

✅ Task completed!

📄 Response:
{
  "code": "...",
  "explanation": "Created a 2-layer neural network with 784 input neurons...",
  "files_modified": ["src/mnist_model.rs", "Cargo.toml"],
  "compiled": true
}

============================================================

> quit
Goodbye! 👋
*/
