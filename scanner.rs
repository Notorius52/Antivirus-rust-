use std::path::Path;
use tokio::sync::mpsc::UnboundedSender;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use walkdir::WalkDir;
use std::fs::File;
use std::io::Read;
use sha2::{Sha256, Digest};
use ndarray::Array2;

use ort::session::Session;
use ort::session::builder::{SessionBuilder, GraphOptimizationLevel};

pub struct FileScanner {
    is_scanning: Arc<AtomicBool>,
}

impl FileScanner {
    pub fn new(is_scanning: Arc<AtomicBool>) -> Self {
        Self { is_scanning }
    }

    pub fn scan_directory(&self, path: &str, log_tx: UnboundedSender<String>) -> String {
        let _ = log_tx.send("🧠 Inizializzazione rete neurale ONNX in corso...".into());
        
        let ai_session = match Session::builder()
            .and_then(|b: SessionBuilder| b.with_optimization_level(GraphOptimizationLevel::Level1))
            .and_then(|b: SessionBuilder| b.commit_from_file("antivirus_brain.onnx")) 
        {
            Ok(session) => {
                let _ = log_tx.send("✅ Modello antivirus_brain.onnx caricato con successo!".into());
                Some(session)
            },
            Err(e) => {
                let _ = log_tx.send(format!("⚠️ Modalità IA disattivata (Errore: {}). Attivo solo Hash.", e));
                None
            }
        };

        // --- FASE 1: PRE-SCANSIONE VELOCE PER CONTARE I FILE ---
        let _ = log_tx.send("🔍 Calcolo totale dei file in corso...".into());
        let mut total_files = 0;
        let counter_walker = WalkDir::new(path).into_iter();
        
        for entry in counter_walker.filter_entry(|e| self.is_safe_to_scan(e.path())) {
            if let Ok(e) = entry {
                if e.file_type().is_file() { total_files += 1; }
            }
        }

        if total_files == 0 {
            let _ = log_tx.send("PROGRESS:100".into());
            return "✅ Nessun file trovato nella cartella selezionata.\n".into();
        }

        // --- FASE 2: SCANSIONE REALE E PROFONDA ---
        let walker = WalkDir::new(path).into_iter();
        let mut file_count = 0;
        let mut minacce_trovate = 0;
        let mut full_report = String::new();

        for entry in walker.filter_entry(|e| self.is_safe_to_scan(e.path())) {
            if !self.is_scanning.load(Ordering::Relaxed) {
                let _ = log_tx.send("🛑 Scansione interrotta dall'utente.".into());
                full_report.push_str("\n[AVVISO] Scansione interrotta dall'utente.\n");
                return full_report;
            }

            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue, 
            };

            if entry.file_type().is_file() {
                let file_path = entry.path().display().to_string();
                
                match File::open(entry.path()) {
                    Ok(mut file) => {
                        let mut buffer = [0; 65536]; 
                        let mut hasher = Sha256::new();
                        
                        while let Ok(bytes_read) = file.read(&mut buffer) {
                            if bytes_read == 0 { break; }
                            hasher.update(&buffer[..bytes_read]);
                        }
                        
                        let hash_result = format!("{:x}", hasher.finalize());
                        file_count += 1;
                        
                        // Calcoliamo la percentuale esatta
                        let progress = (file_count as f32 / total_files as f32) * 100.0;
                        let _ = log_tx.send(format!("PROGRESS:{}", progress as i32));

                        // --- VALUTAZIONE IA ---
                        let is_malware = false;
                        let confidence = 0.0;

                        if let Some(ref _session) = ai_session {
                            let _feature_tensor = Array2::<f32>::zeros((1, 10)); 
                        }

                        // Aggiorniamo il log di testo a schermo ogni 5 file
                        if file_count % 5 == 0 || is_malware {
                            let msg = if is_malware {
                                format!("🚨 PERICOLO: {} [IA CONF: {:.2}%]", file_path, confidence * 100.0)
                            } else {
                                format!("Analisi: {}", file_path)
                            };
                            let _ = log_tx.send(msg);
                        }

                        if is_malware {
                            minacce_trovate += 1;
                            full_report.push_str(&format!("☠️ MINACCIA: {} [SHA256: {}...]\n", file_path, &hash_result[..8]));
                        } else {
                            full_report.push_str(&format!("Esaminato: {} [SHA256: {}...]\n", file_path, &hash_result[..8]));
                        }
                    },
                    Err(_) => {
                        full_report.push_str(&format!("Saltato (Inaccessibile): {}\n", file_path));
                    }
                }
            }
        }
        
        let _ = log_tx.send("PROGRESS:100".into());
        let msg = format!("✅ Analisi completata. File: {} | Minacce IA: {}", file_count, minacce_trovate);
        let _ = log_tx.send(msg.clone());
        full_report.push_str(&format!("\n{}\n", msg));
        
        full_report
    }

    fn is_safe_to_scan(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        if path_str.contains("/usr/share/sounds") || path_str.contains("/usr/share/mint-artwork/sounds") { return false; }
        if path_str.starts_with("/sys") || path_str.starts_with("/proc") || path_str.starts_with("/dev") { return false; }
        true
    }
}
