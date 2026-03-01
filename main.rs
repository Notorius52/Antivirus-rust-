use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use slint::ComponentHandle; 
use std::time::{Instant, Duration};

mod scanner;
mod guardian;
mod realtime; 
mod updater;  
mod logger;

use scanner::FileScanner;

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = ort::init().with_name("FiberGuard_IA").commit();

    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();

    let is_scanning = Arc::new(AtomicBool::new(false));
    // Gestione Cooldown: 10 minuti di blocco se l'utente sospende
    let last_manual_stop = Arc::new(std::sync::Mutex::new(Instant::now() - Duration::from_secs(601)));
    
    let (log_tx, mut log_rx) = mpsc::unbounded_channel::<String>();

    // --- 1. PATH UTENTE ---
    let os = std::env::consts::OS;
    let linux_user = std::env::var("SUDO_USER")
        .unwrap_or_else(|_| std::env::var("USER").unwrap_or_else(|_| "biacca".into()));
    
    let base_home = if os == "windows" {
        std::env::var("USERPROFILE").unwrap_or_else(|_| ".".into())
    } else {
        format!("/home/{}", linux_user)
    };

    // --- 2. SENTINELLA REAL-TIME ---
    let sentinel_log_tx = log_tx.clone();
    let home_sentinel = base_home.clone();
    tokio::spawn(async move {
        let mut path_watch = std::path::PathBuf::from(&home_sentinel);
        let scaricati = path_watch.join("Scaricati");
        if scaricati.exists() { path_watch = scaricati; } 
        else { path_watch.push("Downloads"); }
        if let Some(p_str) = path_watch.to_str() {
            realtime::start_sentinel(p_str, sentinel_log_tx);
        }
    });

    // --- 3. CALLBACK UI: TERMINA CON COOLDOWN ---
    let stop_status = Arc::clone(&is_scanning);
    let stop_ui_h = ui_handle.clone();
    let stop_time = Arc::clone(&last_manual_stop);
    ui.on_termina_scansione(move || {
        stop_status.store(false, Ordering::Relaxed);
        if let Ok(mut t) = stop_time.lock() {
            *t = Instant::now(); // Attiva il cooldown da questo istante
        }
        if let Some(ui) = stop_ui_h.upgrade() {
            ui.set_scan_progress(0.0);
            ui.set_scan_results("🛑 IA Sospesa. Cooldown 10 min attivato.".into());
        }
    });

    // --- 4. CALLBACK UI: AGGIORNAMENTO FIRME ---
    let update_ui_h = ui_handle.clone();
    ui.on_aggiorna_firme(move || {
        let ui_u = update_ui_h.clone();
        tokio::spawn(async move {
            if let Ok(_) = updater::update_definitions().await {
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_u.upgrade() { ui.set_last_update_date("Oggi (Aggiornato)".into()); }
                });
            }
        });
    });

    // --- 5. GESTIONE LOG E UI ---
    let log_ui_h = ui_handle.clone();
    tokio::spawn(async move {
        while let Some(msg) = log_rx.recv().await {
            let m = msg.clone();
            let ui_u = log_ui_h.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_u.upgrade() {
                    if m.starts_with("NET_LOG:") {
                        ui.set_network_logs(m[8..].into());
                    } else if m.starts_with("PROGRESS:") {
                        if let Ok(p) = m[9..].parse::<f32>() { ui.set_scan_progress(p / 100.0); }
                    } else {
                        ui.set_scan_results(m.into());
                    }
                }
            });
        }
    });

    // --- 6. GUARDIANO DI RETE CON SOGLIE ALTE ---
    let net_ui_h = ui_handle.clone();
    let net_st = Arc::clone(&is_scanning);
    let net_tx_main = log_tx.clone();
    let net_stop_time = Arc::clone(&last_manual_stop);
    let target_auto = base_home.clone();

    tokio::task::spawn_blocking(move || {
        let device = match pcap::Device::lookup() {
            Ok(Some(dev)) => dev,
            _ => return,
        };
        let mut cap = pcap::Capture::from_device(device).unwrap().promisc(true).open().unwrap();
        let mut suspicious = 0;
        let mut total = 0;

        while let Ok(packet) = cap.next_packet() {
            total += 1;
            // Solo pacchetti grandi (dati reali)
            if packet.header.len > 1200 { suspicious += 1; }
            
            if total % 100 == 0 {
                let ui_n = net_ui_h.clone();
                let count = suspicious;
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_n.upgrade() { ui.set_network_suspicious_count(count as i32); }
                });
            }

            // Soglia 100.000 pacchetti
            if suspicious > 100_000 && !net_st.load(Ordering::Relaxed) {
                let can_start = if let Ok(t) = net_stop_time.lock() {
                    t.elapsed() > Duration::from_secs(600) 
                } else { true };

                if can_start {
                    net_st.store(true, Ordering::Relaxed);
                    let tx_for_thread = net_tx_main.clone();
                    let st_for_thread = Arc::clone(&net_st);
                    let path_for_thread = target_auto.clone();

                    tokio::spawn(async move {
                        let _ = tx_for_thread.send("NET_LOG:🛡️ IA: Flusso dati elevato. Analisi in corso...".into());
                        
                        let tx_scan = tx_for_thread.clone();
                        let st_scan = Arc::clone(&st_for_thread);
                        
                        let _ = tokio::task::spawn_blocking(move || {
                            let scanner = FileScanner::new(st_scan);
                            scanner.scan_directory(&path_for_thread, tx_scan) 
                        }).await;

                        let _ = tx_for_thread.send("NET_LOG:✅ Analisi completata.".into());
                        st_for_thread.store(false, Ordering::Relaxed);
                    });
                }
                suspicious = 0; 
            }
        }
    });

    ui.run()?;
    Ok(())
}
