use crate::config::ConfigObserver;
use crate::db::Database;
use anyhow::Result;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc;
use std::time::Duration;
use tracing::{debug, info};

pub struct FileWatcher {
    watcher: RecommendedWatcher,
    tx: mpsc::Sender<notify::Event>,
    config_observer: ConfigObserver,
}

impl FileWatcher {
    pub fn new(config_observer: ConfigObserver) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let mut watcher = Watcher::new_debounced(Duration::from_secs(1), rx, RecommendedWatcher::default())?;

        for file in &config_observer.watched_files {
            watcher.watch(file.path.as_str(), RecursiveMode::NonRecursive)?;
            debug!("Watching: {}", file.path);
        }

        Ok(Self { watcher, tx, config_observer })
    }

    pub fn start(&mut self) -> Result<()> {
        info!("Starting file watching for a-hole observer");

        loop {
            match self.watcher.recv() {
                Ok(event) => {
                    debug!("File event: {:?}", event);
                    self.handle_event(event)?;
                }
                Err(e) => {
                    debug!("Watcher error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: notify::Event) -> Result<()> {
        for path in &event.paths {
            debug!("File changed: {}", path);
        }
        Ok(())
    }
}
