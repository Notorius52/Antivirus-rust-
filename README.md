# Antivirus-rust-
"Un potente Antivirus e Network Monitor per Linux scritto in Rust. Utilizza l'IA per l'analisi dei file e pcap per il monitoraggio del traffico in tempo reale, con un'interfaccia grafica ultra-leggera in Slint e un'architettura disaccoppiata (Demone + System Tray)."
# Antivirus-rust-
"Un potente Antivirus e Network Monitor per Linux scritto in Rust. Utilizza l'IA per l'analisi dei file e pcap per il monitoraggio del traffico in tempo reale, con un'interfaccia grafica ultra-leggera in Slint e un'architettura disaccoppiata (Demone + System Tray)."

# 🛡️ Fiber Guard IA v2.0 - Advanced Linux Protection

Fiber Guard IA è un antivirus e sistema di monitoraggio di rete per ambienti Linux (ottimizzato per Linux Mint e Ubuntu). È scritto interamente in **Rust** per garantire prestazioni estreme, sicurezza della memoria e un impatto quasi nullo sulle risorse di sistema.



## 🧠 Come Opera (Architettura del Sistema)

A differenza dei tradizionali applicativi monolitici che spesso causano conflitti con i server grafici di Linux (es. GTK/Wayland), Fiber Guard IA utilizza un'**architettura disaccoppiata a due livelli**:

1. **Il Motore Principale (Core):**
   * Risiede in background ed è il vero "cervello" dell'antivirus.
   * **Monitoraggio di Rete (`libpcap`):** Intercetta il traffico dati direttamente dalla scheda di rete (fibra/ethernet) in modalità promiscua, analizzando i pacchetti superiori a 1200 byte per individuare anomalie o picchi di traffico sospetti.
   * **Scansione IA (`ort`):** Utilizza ONNX Runtime per caricare modelli di intelligenza artificiale pre-addestrati, in grado di valutare file sospetti e classificare le minacce senza dipendere esclusivamente dai classici database di firme.
   * **Interfaccia On-Demand (Slint):** La dashboard grafica viene generata dal core solo quando richiesta, garantendo che il sistema grafico non pesi sulla RAM quando il programma è in ascolto.

2. **Il Telecomando (System Tray):**
   * Un programma leggerissimo (`fiber_tray`) che si avvia in automatico all'accensione del PC insieme all'utente.
   * Vive silenziosamente vicino all'orologio di sistema (Tray Icon).
   * **Funzione di Ponte:** Quando l'utente clicca su "Apri Fiber Guard", questo modulo invia un comando al terminale per "svegliare" il Motore Principale richiedendo i privilegi di amministratore (`sudo`), necessari per il monitoraggio hardware della rete.

## 🛠️ Requisiti di Sistema
Prima di compilare il progetto, assicurati di avere le librerie di sistema necessarie per la cattura dei pacchetti:
```bash
sudo apt update
sudo apt install libpcap-dev build-essential
