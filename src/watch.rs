use crate::source::SourceEvent;
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::time::Duration;
use std::{
    path::{Path, PathBuf},
    sync::mpsc::{self, Sender},
};

fn find_source_paths(root_path: &Path) -> Option<Vec<PathBuf>> {
    let source_pattern = root_path.join("**/*.vamp");
    let source_glob = glob::glob(source_pattern.to_str().unwrap_or("**/*.vamp")).ok()?;
    let source_paths: Vec<_> = source_glob.filter_map(|result| result.ok()).collect();
    return Some(source_paths);
}

pub fn watch(root_path: &Path, source_events: Sender<SourceEvent>) -> notify::Result<()> {
    let source_paths = find_source_paths(root_path).unwrap_or(vec![]);
    for path in source_paths {
        source_events.send(SourceEvent::File(path.clone())).unwrap();
    }
    let (sender, receiver) = mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(500), None, sender)?;
    debouncer
        .watcher()
        .watch(root_path.as_ref(), RecursiveMode::Recursive)?;
    for result in receiver {
        match result {
            Ok(events) => {
                for event in events {
                    if event.path.extension().and_then(|e| e.to_str()) == Some("vamp") {
                        source_events
                            .send(SourceEvent::File(event.path.clone()))
                            .unwrap();
                    }
                }
            }
            Err(error) => {
                eprintln!("error: {error:?}");
            }
        }
    }
    Ok(())
}
