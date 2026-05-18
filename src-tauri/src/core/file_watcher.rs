#![allow(missing_docs)]
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeEvent {
    pub event_type: String,
    pub file_path: String,
    pub timestamp: u64,
}

pub struct FileWatcherService {
    watcher: Arc<Mutex<Option<RecommendedWatcher>>>,
    watched_paths: Arc<Mutex<HashSet<PathBuf>>>,
    tx: broadcast::Sender<FileChangeEvent>,
}

impl FileWatcherService {
    pub fn new(tx: broadcast::Sender<FileChangeEvent>) -> Self {
        Self {
            watcher: Arc::new(Mutex::new(None)),
            watched_paths: Arc::new(Mutex::new(HashSet::new())),
            tx,
        }
    }

    pub fn watch_directory(&self, dir: &Path) -> Result<(), String> {
        let tx_clone = self.tx.clone();
        let _watched = self.watched_paths.clone();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    for path in &event.paths {
                        let change_event = FileChangeEvent {
                            event_type: format!("{:?}", event.kind),
                            file_path: path.to_string_lossy().to_string(),
                            timestamp: SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                        };
                        let _ = tx_clone.send(change_event);
                    }
                }
            },
            Config::default(),
        )
        .map_err(|e| format!("创建Watcher失败: {}", e))?;

        watcher
            .watch(dir, RecursiveMode::Recursive)
            .map_err(|e| format!("监控目录失败: {}", e))?;

        self.watcher
            .lock()
            .map_err(|e| format!("watcher mutex poisoned: {}", e))?
            .replace(watcher);
        self.watched_paths
            .lock()
            .map_err(|e| format!("paths mutex poisoned: {}", e))?
            .insert(dir.to_path_buf());
        Ok(())
    }

    pub fn unwatch(&self) {
        if let Some(mut w) = self
            .watcher
            .lock()
            .map_err(|e| format!("watcher mutex poisoned: {}", e))
            .ok()
            .and_then(|mut w| w.take())
        {
            let paths: Vec<PathBuf> = self
                .watched_paths
                .lock()
                .map_err(|e| format!("paths mutex poisoned: {}", e))
                .ok()
                .map(|mut set| set.drain().collect())
                .unwrap_or_default();
            for path in &paths {
                let _ = w.unwatch(path);
            }
        }
    }

    pub fn get_watched_count(&self) -> usize {
        self.watched_paths.lock().map(|set| set.len()).unwrap_or(0)
    }
}
