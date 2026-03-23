use crate::db::Database;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Represents a semantic diff of config changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticDiff {
    pub tool: String,
    pub file_path: String,
    pub changes: Vec<Change>,
    pub summary: DiffSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub key: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub change_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub total_changes: usize,
    pub keys_added: usize,
    pub keys_removed: usize,
    pub keys_updated: usize,
}

pub struct DiffParser;

impl DiffParser {
    pub fn parse(_tool: &str, _content: &str) -> Result<Vec<Change>, anyhow::Error> {
        Ok(vec![])
    }

    pub fn compare(old_content: &str, new_content: &str, tool: &str) -> Result<SemanticDiff, anyhow::Error> {
        let old_changes = Self::parse(tool, old_content)?;
        let new_changes = Self::parse(tool, new_content)?;
        let mut changes = Vec::new();

        for new in &new_changes {
            if !old_changes.iter().any(|old| old.key == new.key) {
                changes.push(Change {
                    key: new.key.clone(),
                    old_value: None,
                    new_value: Some(new.new_value.clone().unwrap_or_default()),
                    change_type: "set".to_string(),
                });
            }
        }

        for old in &old_changes {
            if !new_changes.iter().any(|new| new.key == old.key) {
                changes.push(Change {
                    key: old.key.clone(),
                    old_value: Some(old.old_value.clone().unwrap_or_default()),
                    new_value: None,
                    change_type: "remove".to_string(),
                });
            }
        }

        for new in &new_changes {
            for old in &old_changes {
                if new.key == old.key {
                    if new.new_value != old.old_value {
                        changes.push(Change {
                            key: new.key.clone(),
                            old_value: Some(old.old_value.clone()),
                            new_value: Some(new.new_value.clone()),
                            change_type: "update".to_string(),
                        });
                    }
                    break;
                }
            }
        }

        let summary = DiffSummary {
            total_changes: changes.len(),
            keys_added: changes.iter().filter(|c| c.change_type == "set").count(),
            keys_removed: changes.iter().filter(|c| c.change_type == "remove").count(),
            keys_updated: changes.iter().filter(|c| c.change_type == "update").count(),
        };

        Ok(SemanticDiff {
            tool: tool.to_string(),
            file_path: "unknown".to_string(),
            changes,
            summary,
        })
    }
}
