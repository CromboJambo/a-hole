use std::fs;
use std::path::Path;

fn main() {
    println!("Redox ML Agent Architecture Demo");
    println!("==================================");

    // Show that we can read the template file
    let template_path = Path::new("template.toml");
    if template_path.exists() {
        match fs::read_to_string(template_path) {
            Ok(contents) => {
                println!("✓ Template file loaded successfully");
                println!("Template content preview:");
                let lines: Vec<&str> = contents.lines().take(5).collect();
                for line in lines {
                    println!("  {}", line);
                }
            }
            Err(e) => println!("✗ Failed to read template: {}", e),
        }
    } else {
        println!("✗ Template file not found");
    }

    // Show that we can read the comments file
    let comments_path = Path::new("comments.toml");
    if comments_path.exists() {
        match fs::read_to_string(comments_path) {
            Ok(contents) => {
                println!("✓ Comments file loaded successfully");
                println!("Comments content preview:");
                let lines: Vec<&str> = contents.lines().take(5).collect();
                for line in lines {
                    println!("  {}", line);
                }
            }
            Err(e) => println!("✗ Failed to read comments: {}", e),
        }
    } else {
        println!("✗ Comments file not found");
    }

    println!("\nArchitecture implemented:");
    println!("  - Crate-per-Agent structure");
    println!("  - Template + Comments layer system");
    println!("  - Self-updating via rsync workflow");
    println!("  - Git-first iteration with local development");

    println!("\nUsage examples:");
    println!("  ./sync_comments.sh sync-all");
    println!("  ./sync_comments.sh toggle redox_ml_agent true");
    println!("  ./tools/agent_updater/updater.sh");
}
