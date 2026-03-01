use notify::{Watcher, RecursiveMode, Config, RecommendedWatcher};
use std::path::Path;
use std::sync::mpsc::channel;

pub fn start_sentinel(path: &str, log_tx: tokio::sync::mpsc::UnboundedSender<String>) {
    let (tx, rx) = channel();

    // Usiamo match per gestire l'errore ed evitare il panic
    let mut watcher = match RecommendedWatcher::new(tx, Config::default()) {
        Ok(w) => w,
        Err(_) => return,
    };

    if let Err(_e) = watcher.watch(Path::new(path), RecursiveMode::Recursive) {
        return;
    }

    for res in rx {
        match res {
            Ok(event) => {
                if event.kind.is_create() {
                    let _ = log_tx.send(format!("✨ IA Real-time: Nuovo file rilevato {:?}", event.paths[0]));
                }
            }
            Err(_) => continue,
        }
    }
}
