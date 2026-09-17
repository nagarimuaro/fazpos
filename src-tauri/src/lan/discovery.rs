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

/// Dapatkan IP LAN aktif dari sistem (misal 192.168.100.75)
pub fn ambil_ip_lan_lokal() -> String {
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                let ip = addr.ip();
                if !ip.is_loopback() {
                    return ip.to_string();
                }
            }
        }
    }
    "127.0.0.1".to_string()
}

/// Hitung target broadcast berdasarkan IP lokal (global + directed subnet broadcast)
pub fn hitung_broadcast_targets(local_ip: &str) -> Vec<SocketAddr> {
    let mut targets = Vec::new();

    // 1. Global broadcast (255.255.255.255)
    if let Ok(addr) = format!("255.255.255.255:{}", DISCOVERY_PORT).parse() {
        targets.push(addr);
    }

    // 2. Subnet /24 broadcast (misal 192.168.100.75 -> 192.168.100.255)
    let parts: Vec<&str> = local_ip.split('.').collect();
    if parts.len() == 4 {
        if let Ok(subnet24) = format!("{}.{}.{}.255:{}", parts[0], parts[1], parts[2], DISCOVERY_PORT).parse() {
            targets.push(subnet24);
        }
        // Subnet /16 broadcast (misal 192.168.255.255)
        if let Ok(subnet16) = format!("{}.{}.255.255:{}", parts[0], parts[1], DISCOVERY_PORT).parse() {
            targets.push(subnet16);
        }
    }

    // 3. Loopback untuk pengujian mesin yang sama
    if let Ok(loopback) = format!("127.0.0.1:{}", DISCOVERY_PORT).parse() {
        targets.push(loopback);
    }

    targets
}

/// Buat socket UDP dengan opsi SO_REUSEADDR dan SO_REUSEPORT
fn create_reuse_udp_socket(port: u16) -> Result<UdpSocket, Box<dyn std::error::Error + Send + Sync>> {
    let socket = socket2::Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )?;

    let _ = socket.set_reuse_address(true);
    #[cfg(not(windows))]
    let _ = socket.set_reuse_port(true);
    let _ = socket.set_broadcast(true);
    let _ = socket.set_nonblocking(true);

    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    socket.bind(&addr.into())?;

    let std_sock: std::net::UdpSocket = socket.into();
    let tokio_sock = UdpSocket::from_std(std_sock)?;
    Ok(tokio_sock)
}

/// Jalankan loop broadcast UDP di background setiap interval tertentu
pub async fn mulai_broadcast(mut packet: DiscoveryPacket, interval_ms: u64) {
    let local_ip = ambil_ip_lan_lokal();
    if packet.ip == "0.0.0.0" || packet.ip.is_empty() {
        packet.ip = local_ip.clone();
    }

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

    let targets = hitung_broadcast_targets(&local_ip);
    println!(
        "[LAN Discovery] Mulai broadcast dari IP {} ke {} target (port {})",
        local_ip,
        targets.len(),
        DISCOVERY_PORT
    );

    let mut interval = tokio::time::interval(Duration::from_millis(interval_ms));

    loop {
        interval.tick().await;

        let json_data = match serde_json::to_vec(&packet) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };

        for &target in &targets {
            let _ = socket.send_to(&json_data, target).await;
        }
    }
}

/// Jalankan listener UDP di port 7891 dan kelola daftar perangkat LAN
pub fn mulai_listener(my_machine_id: String) -> Arc<Mutex<Vec<DiscoveredDevice>>> {
    let devices = Arc::new(Mutex::new(Vec::<DiscoveredDevice>::new()));
    let devices_clone = Arc::clone(&devices);

    // 1. Task listener UDP
    tauri::async_runtime::spawn(async move {
        let socket = match create_reuse_udp_socket(DISCOVERY_PORT) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[LAN Discovery] Gagal bind listener port {}: {}", DISCOVERY_PORT, e);
                return;
            }
        };

        println!("[LAN Discovery] Listener aktif di port {}", DISCOVERY_PORT);
        let mut buf = [0u8; 2048];

        loop {
            match socket.recv_from(&mut buf).await {
                Ok((len, src_addr)) => {
                    if let Ok(packet) = serde_json::from_slice::<DiscoveryPacket>(&buf[..len]) {
                        // Filter hanya paket fazpos
                        if packet.app == "fazpos" {
                            if packet.machine_id != my_machine_id {
                                println!(
                                    "[LAN Discovery] Ditemukan perangkat: {} ({}:{}) [{}]",
                                    packet.device_nama, src_addr.ip(), packet.port, packet.role
                                );
                                let now = Utc::now().to_rfc3339();
                                let ip = if packet.ip != "0.0.0.0" && !packet.ip.is_empty() {
                                    packet.ip.clone()
                                } else {
                                    src_addr.ip().to_string()
                                };

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
