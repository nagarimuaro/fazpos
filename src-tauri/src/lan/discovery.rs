use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::UdpSocket;

pub const DISCOVERY_PORT: u16 = 7891;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoveryPacket {
    pub app: String,            // "fazpos"
    pub role: String,           // "server" | "client"
    pub cabang_id: String,
    pub cabang_nama: String,
    pub device_id: String,
    pub device_nama: String,
    pub machine_id: String,
    pub license_status: String, // "AKTIF" | "BELUM"
    pub port: u16,              // HTTP Axum port (misal 7890)
    pub ip: String,             // IP pengirim jika diketahui
    pub versi: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoveredDevice {
    pub packet: DiscoveryPacket,
    pub ip_address: String,
    pub last_seen: String,       // ISO 8601 string
    pub latency_ms: Option<u64>,
    pub is_online: bool,
}

/// Jalankan loop broadcast UDP di background setiap interval tertentu
pub async fn mulai_broadcast(packet: DiscoveryPacket, interval_ms: u64) {
    let socket = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[LAN Discovery] Gagal bind sender socket: {}", e);
            return;
        }
    };

    if let Err(e) = socket.set_broadcast(true) {
        eprintln!("[LAN Discovery] Gagal set broadcast: {}", e);
        return;
    }

    let target_addr: SocketAddr = format!("255.255.255.255:{}", DISCOVERY_PORT)
        .parse()
        .unwrap();

    let mut interval = tokio::time::interval(Duration::from_millis(interval_ms));

    loop {
        interval.tick().await;

        let json_data = match serde_json::to_vec(&packet) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };

        let _ = socket.send_to(&json_data, target_addr).await;
    }
}

/// Jalankan listener UDP di port 7891 dan kelola daftar perangkat LAN
pub fn mulai_listener(my_machine_id: String) -> Arc<Mutex<Vec<DiscoveredDevice>>> {
    let devices = Arc::new(Mutex::new(Vec::<DiscoveredDevice>::new()));
    let devices_clone = Arc::clone(&devices);

    // 1. Task listener UDP
    tauri::async_runtime::spawn(async move {
        let socket = match UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_PORT)).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[LAN Discovery] Gagal bind listener port {}: {}", DISCOVERY_PORT, e);
                return;
            }
        };

        let mut buf = [0u8; 2048];

        loop {
            match socket.recv_from(&mut buf).await {
                Ok((len, src_addr)) => {
                    if let Ok(packet) = serde_json::from_slice::<DiscoveryPacket>(&buf[..len]) {
                        // Filter hanya paket fazpos dan abaikan paket dari diri sendiri
                        if packet.app == "fazpos" && packet.machine_id != my_machine_id {
                            let now = Utc::now().to_rfc3339();
                            let ip = src_addr.ip().to_string();

                            if let Ok(mut list) = devices_clone.lock() {
                                if let Some(existing) = list.iter_mut().find(|d| d.packet.machine_id == packet.machine_id) {
                                    existing.packet = packet;
                                    existing.ip_address = ip;
                                    existing.last_seen = now;
                                    existing.is_online = true;
                                } else {
                                    list.push(DiscoveredDevice {
                                        packet,
                                        ip_address: ip,
                                        last_seen: now,
                                        latency_ms: None,
                                        is_online: true,
                                    });
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    });

    // 2. Task pembersih liveness (set offline jika > 15 detik tidak ada broadcast)
    let cleaner_devices = Arc::clone(&devices);
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            if let Ok(mut list) = cleaner_devices.lock() {
                let now = Utc::now();
                for dev in list.iter_mut() {
                    if let Ok(seen) = chrono::DateTime::parse_from_rfc3339(&dev.last_seen) {
                        let dur = now.signed_duration_since(seen.with_timezone(&Utc));
                        if dur.num_seconds() > 15 {
                            dev.is_online = false;
                        }
                    }
                }
            }
        }
    });

    devices
}

/// Ping HTTP ke LAN device untuk tes konektivitas & ukur latency
pub async fn ping_lan_http(ip: &str, port: u16) -> Result<(bool, u64, String), String> {
    let url = format!("http://{}:{}/api/v1/ping", ip, port);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|e| e.to_string())?;

    let start = std::time::Instant::now();
    match client.get(&url).send().await {
        Ok(resp) => {
            let latency = start.elapsed().as_millis() as u64;
            if resp.status().is_success() {
                let text = resp.text().await.unwrap_or_default();
                Ok((true, latency, text))
            } else {
                Err(format!("HTTP status {}", resp.status()))
            }
        }
        Err(e) => Err(format!("Gagal koneksi ke {}: {}", url, e)),
    }
}
