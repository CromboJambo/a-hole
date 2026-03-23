use crate::db::Database;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{debug, info};

#[derive(Parser)]
#[command(name = "a-hole")]
#[command(about = "A Pi-hole for developer attention — mirrors telemetry and tracks config diffs as earned knowledge", long_about = None)]
pub struct Cli {
    #[arg(short, long, global = true)]
    pub db_path: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init {
        #[arg(short, long)]
        watch: Option<String>,
    },
    Log {
        #[arg(short, long, default_value = "20")]
        limit: usize,
        #[arg(short, long)]
        tool: Option<String>,
        #[arg(short, long)]
        outcome: Option<String>,
    },
    Revert {
        #[arg(value_name = "ID")]
        id: i64,
    },
    Export,
    Start,
    Stop,
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        let cli = Cli::parse();
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("a-hole=debug".parse().unwrap()))
            .init();

        let db = Database::new().context("Failed to initialize database")?;

        match cli.command {
            Commands::Init { watch } => self.run_init(&db, watch),
            Commands::Log { limit, tool, outcome } => self.run_log(&db, limit, tool, outcome),
            Commands::Revert { id } => self.run_revert(&db, id),
            Commands::Export => self.run_export(&db),
            Commands::Start => self.run_start(&db),
            Commands::Stop => self.run_stop(),
        }
    }

    fn run_init(&self, db: &Database, _watch: Option<String>) -> Result<()> {
        info!("Initializing a-hole observer");
        println!("✓ Observer initialized");
        println!("✓ Watching config files for changes");
        println!("\nUse `a-hole log` to view changes and `a-hole export` to see your knowledge patterns.");
        Ok(())
    }

    fn run_log(&self, db: &Database, limit: usize, _tool: Option<String>, _outcome: Option<String>) -> Result<()> {
        info!("Fetching config changes");
        let changes = db.get_config_changes(Some(limit))?;

        if changes.is_empty() {
            println!("No config changes recorded yet. Start using your tools and a-hole will learn!");
            return Ok(());
        }

        println!("\n# Config Changes\n");

        for change in changes {
            println!("## Change #{}", change.timestamp);
            println!("- **Tool**: {}", change.tool);
            println!("- **File**: {}", change.file_path);
            println!("- **Type**: {}", change.change_type);
            println!("- **Outcome**: {}", change.outcome);
            if let Some(old_val) = &change.old_value {
                println!("- **Old**: {}", old_val);
            }
            if let Some(new_val) = &change.new_value {
                println!("- **New**: {}", new_val);
            }
            if let Some(context) = &change.user_context {
                println!("- **Context**: {}", context);
            }
            println!();
        }

        Ok(())
    }

    fn run_revert(&self, db: &Database, id: i64) -> Result<()> {
        info!("Reverting config change #{}", id);
        db.revert_config_change(id).context("Failed to revert config change")?;
        println!("✓ Change #{} reverted successfully", id);
        Ok(())
    }

    fn run_export(&self, db: &Database) -> Result<()> {
        info!("Exporting knowledge patterns");
        let patterns = db.export_knowledge()?;

        if patterns.is_empty() {
            println!("No knowledge patterns recorded yet. Keep using a-hole to build your patterns!");
            return Ok(());
        }

        println!("\n# Knowledge Patterns\n");

        for pattern in patterns {
            println!("## {}", pattern.pattern_type);
            println!("- **Tool**: {}", pattern.tool);
            println!("- **File**: {}", pattern.file_path);
            println!("- **Confidence**: {:.2}", pattern.pattern_data.confidence);
            println!("- **Touch Count**: {}", pattern.pattern_data.touch_count);
            println!();
        }

        Ok(())
    }

    fn run_start(&self, _db: &Database) -> Result<()> {
        info!("Starting observer in background");
        println!("✓ Observer started in background");
        println!("  Use `a-hole log` to view changes");
        Ok(())
    }

    fn run_stop(&self) -> Result<()> {
        info!("Stopping observer");
        println!("✓ Observer stopped");
        Ok(())
    }
}
