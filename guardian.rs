use std::time::Instant;

#[derive(Clone, Copy, PartialEq)]
pub enum DefenseStatus {
    Silent,     // 🟢 Verde
    Fortifying, // 🟡 Giallo
    Hardened,   // 🚨 Rosso
}

pub struct Guardian {
    pub status: DefenseStatus,
    pub trusted_sizes: Vec<f32>,
    last_threat_time: Instant, 
}

impl Guardian {
    pub fn new() -> Self {
        Guardian { 
            status: DefenseStatus::Silent,
            // Pattern predefiniti (MTU standard e pacchetti piccoli)
            trusted_sizes: vec![64.0, 512.0, 1292.0, 1514.0],
            last_threat_time: Instant::now(),
        }
    }

    pub fn evaluate_risk(&mut self, suspicious_count: u64, total: u64, last_size: f32, delta: f32) -> DefenseStatus {
        // Evitiamo analisi su campioni troppo piccoli per evitare falsi positivi iniziali
        if total < 50 { return self.status; }

        // Calcolo velocità (bitrate)
        let _mbps = if delta > 0.001 { (last_size * 8.0) / (delta * 1000.0) } else { 0.0 };
        
        // Calcolo della percentuale di minacce (con protezione divisione per zero)
        let ratio = if total > 0 {
            (suspicious_count as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        // --- LOGICA DI STATO ---
        if ratio > 8.0 {
            // Se superiamo l'8%, aggiorniamo il timer dell'ultima minaccia
            self.last_threat_time = Instant::now(); 
            
            if ratio > 20.0 {
                self.status = DefenseStatus::Hardened;
            } else {
                // Se siamo già in Hardened, non declassiamo subito a Fortifying 
                // a meno che non passi il tempo di cooldown
                if self.status != DefenseStatus::Hardened {
                    self.status = DefenseStatus::Fortifying;
                }
            }
        } else {
            // --- LOGICA DI AUTO-RESET (COOLDOWN) ---
            // Se la situazione è tornata sotto la soglia critica, aspettiamo 30 secondi
            // prima di tornare allo stato SILENT (Verde)
            if self.status != DefenseStatus::Silent && self.last_threat_time.elapsed().as_secs() > 30 {
                self.status = DefenseStatus::Silent;
            }
        }

        self.status
    }
}
