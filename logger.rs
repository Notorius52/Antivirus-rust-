use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use chrono::Local;

pub fn save_scan_log(content: &str) -> std::io::Result<()> {
    // Crea la cartella logs se non esiste
    create_dir_all("logs")?;

    // Genera un nome file basato sulla data e ora attuale
    let now = Local::now();
    let filename = format!("logs/scan_{}.txt", now.format("%Y-%m-%d_%H-%M-%S"));

    // Crea il file e scrive il contenuto
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(filename)?;

    writeln!(file, "=== FIBER GUARD IA - REPORT DI SCANSIONE ===")?;
    writeln!(file, "Data: {}", now.format("%d/%m/%Y %H:%M:%S"))?;
    writeln!(file, "------------------------------------------")?;
    writeln!(file, "{}", content)?;
    writeln!(file, "------------------------------------------")?;
    writeln!(file, "Fine Report.")?;

    Ok(())
}
