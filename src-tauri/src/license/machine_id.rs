use sha2::{Digest, Sha256};
use std::process::Command;

/// Generator Machine ID unik berbasis hardware Windows
pub struct MachineId;

impl MachineId {
    /// Hasilkan Machine ID dalam format terbaca: `RTG-XXXX-XXXX-XXXX-XXXX`
    pub fn dapatkan() -> String {
        let raw_fingerprint = Self::kumpulkan_hardware_info();
        Self::format_hash(&raw_fingerprint)
    }

    /// Kumpulkan kombinasi identifier hardware
    pub fn kumpulkan_hardware_info() -> String {
        let machine_guid = Self::baca_machine_guid().unwrap_or_else(|| "UNKNOWN_GUID".to_string());
        let processor = std::env::var("PROCESSOR_IDENTIFIER").unwrap_or_else(|_| "UNKNOWN_CPU".to_string());
        let computer_name = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "UNKNOWN_PC".to_string());

        format!("{}|{}|{}", machine_guid, processor, computer_name)
    }

    /// Format hash SHA-256 menjadi 16 karakter heksadesimal dengan pemisah strip
    pub fn format_hash(raw_data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"FAZPOS_SALT_2026_");
        hasher.update(raw_data.as_bytes());
        let result = hasher.finalize();
        let hex: String = result.iter().map(|b| format!("{:02X}", b)).collect();

        // Ambil 16 karakter pertama dan buat format RTG-XXXX-XXXX-XXXX-XXXX
        format!(
            "RTG-{}-{}-{}-{}",
            &hex[0..4],
            &hex[4..8],
            &hex[8..12],
            &hex[12..16]
        )
    }

    /// Baca MachineGuid dari Windows Registry via perintah reg
    fn baca_machine_guid() -> Option<String> {
        let output = Command::new("reg")
            .args([
                "query",
                r"HKLM\SOFTWARE\Microsoft\Cryptography",
                "/v",
                "MachineGuid",
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.contains("MachineGuid") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(guid) = parts.last() {
                    return Some(guid.to_string());
                }
            }
        }
        None
    }
}
