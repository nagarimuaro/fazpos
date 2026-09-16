use sha2::{Digest, Sha256};
use std::process::Command;
use std::sync::OnceLock;

static CACHED_MACHINE_ID: OnceLock<String> = OnceLock::new();

/// Generator Machine ID permanen berbasis Serial Fisik SSD / HDD
/// Aturan:
/// - Jika hanya SSD -> bind ke nomor seri fisik SSD
/// - Jika hanya HDD -> bind ke nomor seri fisik HDD
/// - Jika ada keduanya -> bind ke nomor seri fisik SSD (SSD diprioritaskan)
pub struct MachineId;

impl MachineId {
    /// Hasilkan Machine ID dalam format terbaca: `FAZ-XXXX-XXXX-XXXX-XXXX`
    /// Menggunakan cache OnceLock agar pemindaian hardware hanya terjadi 1 kali saat startup
    pub fn dapatkan() -> String {
        CACHED_MACHINE_ID
            .get_or_init(|| {
                let raw_fingerprint = Self::kumpulkan_hardware_info();
                Self::format_hash(&raw_fingerprint)
            })
            .clone()
    }

    /// Kumpulkan nomor seri fisik SSD/HDD
    pub fn kumpulkan_hardware_info() -> String {
        if let Some(serial) = Self::deteksi_disk_serial() {
            return format!("DISK:{}", serial);
        }

        // Fallback jika permission sistem membatasi pembacaan controller disk:
        // Gunakan MachineGuid Windows Registry atau identifier OS
        if let Some(guid) = Self::baca_machine_guid() {
            return format!("GUID:{}", guid);
        }

        // Fallback Android / cross-platform persistent UUID di direktori data
        if let Some(id) = Self::baca_atau_buat_local_id() {
            return format!("DEVUUID:{}", id);
        }

        "FALLBACK_GENERIC_DRIVE".to_string()
    }

    /// Simpan dan baca device UUID unik lokal agar mesin/perangkat tetap konsisten lintas restart
    fn baca_atau_buat_local_id() -> Option<String> {
        let path = std::path::Path::new(".fazpos_machine_id");
        if let Ok(content) = std::fs::read_to_string(path) {
            let trimmed = content.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
        let new_id = uuid::Uuid::new_v4().to_string();
        let _ = std::fs::write(path, &new_id);
        Some(new_id)
    }

    /// Format hash SHA-256 menjadi 16 karakter heksadesimal dengan pemisah strip
    pub fn format_hash(raw_data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"FAZPOS_SALT_2026_");
        hasher.update(raw_data.as_bytes());
        let result = hasher.finalize();
        let hex: String = result.iter().map(|b| format!("{:02X}", b)).collect();

        // Format FAZ-XXXX-XXXX-XXXX-XXXX
        format!(
            "FAZ-{}-{}-{}-{}",
            &hex[0..4],
            &hex[4..8],
            &hex[8..12],
            &hex[12..16]
        )
    }

    /// Deteksi nomor seri fisik disk dengan prioritas SSD over HDD
    pub fn deteksi_disk_serial() -> Option<String> {
        #[cfg(target_os = "windows")]
        {
            if let Some(serial) = Self::deteksi_windows_disk() {
                return Some(serial);
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(serial) = Self::deteksi_macos_disk() {
                return Some(serial);
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Some(serial) = Self::deteksi_linux_disk() {
                return Some(serial);
            }
        }

        #[cfg(target_os = "android")]
        {
            if let Some(serial) = Self::deteksi_android_disk() {
                return Some(serial);
            }
        }

        None
    }

    #[cfg(target_os = "windows")]
    fn deteksi_windows_disk() -> Option<String> {
        // 1. Coba via PowerShell: Cek PhysicalDisk untuk MediaType (SSD/HDD) dan Model
        let ps_cmd = r#"
        try {
            $disks = Get-PhysicalDisk 2>$null
            if ($disks) {
                $ssd = $disks | Where-Object { $_.MediaType -eq 'SSD' -or $_.Model -match 'SSD|NVMe' } | Select-Object -First 1
                if ($ssd -and $ssd.SerialNumber) { Write-Output ('SSD:' + $ssd.SerialNumber.Trim()); exit 0 }
                $hdd = $disks | Where-Object { $_.MediaType -eq 'HDD' } | Select-Object -First 1
                if ($hdd -and $hdd.SerialNumber) { Write-Output ('HDD:' + $hdd.SerialNumber.Trim()); exit 0 }
                $any = $disks | Select-Object -First 1
                if ($any -and $any.SerialNumber) { Write-Output ('DISK:' + $any.SerialNumber.Trim()); exit 0 }
            }
        } catch {}
        try {
            $drives = Get-CimInstance Win32_DiskDrive 2>$null
            if ($drives) {
                $ssd = $drives | Where-Object { $_.Model -match 'SSD|NVMe' } | Select-Object -First 1
                if ($ssd -and $ssd.SerialNumber) { Write-Output ('SSD:' + $ssd.SerialNumber.Trim()); exit 0 }
                $any = $drives | Select-Object -First 1
                if ($any -and $any.SerialNumber) { Write-Output ('DISK:' + $any.SerialNumber.Trim()); exit 0 }
            }
        } catch {}
        "#;

        if let Ok(output) = Command::new("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", ps_cmd])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !text.is_empty() && (text.starts_with("SSD:") || text.starts_with("HDD:") || text.starts_with("DISK:")) {
                    return Some(text);
                }
            }
        }

        // 2. Fallback wmic jika PowerShell restricted / dinonaktifkan
        if let Ok(output) = Command::new("wmic")
            .args(["diskdrive", "get", "model,serialnumber"])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                let mut ssd_serial = None;
                let mut hdd_serial = None;

                for line in text.lines().skip(1) {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let serial = parts.last().unwrap().to_string();
                        let upper = trimmed.to_uppercase();
                        if upper.contains("SSD") || upper.contains("NVME") {
                            if ssd_serial.is_none() {
                                ssd_serial = Some(format!("SSD:{}", serial));
                            }
                        } else if hdd_serial.is_none() {
                            hdd_serial = Some(format!("HDD:{}", serial));
                        }
                    }
                }

                if let Some(ssd) = ssd_serial {
                    return Some(ssd);
                }
                if let Some(hdd) = hdd_serial {
                    return Some(hdd);
                }
            }
        }

        None
    }

    #[cfg(target_os = "macos")]
    fn deteksi_macos_disk() -> Option<String> {
        if let Ok(output) = Command::new("system_profiler")
            .args(["SPNVMeDataType", "SPSerialATADataType"])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                let mut current_model = String::new();
                let mut ssd_serial = None;
                let mut hdd_serial = None;

                for line in text.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("Model:") {
                        current_model = trimmed.trim_start_matches("Model:").trim().to_uppercase();
                    } else if trimmed.starts_with("Serial Number:") {
                        let serial = trimmed.trim_start_matches("Serial Number:").trim().to_string();
                        if !serial.is_empty() {
                            if current_model.contains("SSD") || current_model.contains("APPLE SSD") || current_model.contains("NVME") {
                                if ssd_serial.is_none() {
                                    ssd_serial = Some(format!("SSD:{}", serial));
                                }
                            } else if hdd_serial.is_none() {
                                hdd_serial = Some(format!("HDD:{}", serial));
                            }
                        }
                    }
                }

                if let Some(ssd) = ssd_serial {
                    return Some(ssd);
                }
                if let Some(hdd) = hdd_serial {
                    return Some(hdd);
                }
            }
        }

        // Fallback Mac hardware IOPlatformUUID jika disk driver dibatasi
        if let Ok(output) = Command::new("ioreg")
            .args(["-rd1", "-c", "IOPlatformExpertDevice"])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    if line.contains("IOPlatformUUID") {
                        if let Some(pos) = line.find('"') {
                            let parts: Vec<&str> = line[pos..].split('"').collect();
                            if parts.len() >= 4 {
                                return Some(format!("MAC-HW:{}", parts[3]));
                            }
                        }
                    }
                }
            }
        }

        None
    }

    #[cfg(target_os = "linux")]
    fn deteksi_linux_disk() -> Option<String> {
        if let Ok(entries) = std::fs::read_dir("/sys/block") {
            let mut ssd_serial = None;
            let mut hdd_serial = None;

            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("loop") || name.starts_with("ram") || name.starts_with("sr") {
                    continue;
                }

                let path = entry.path();
                let rotational_path = path.join("queue/rotational");
                let is_ssd = if let Ok(rot) = std::fs::read_to_string(&rotational_path) {
                    rot.trim() == "0"
                } else {
                    false
                };

                let serial_path = path.join("device/serial");
                if let Ok(serial) = std::fs::read_to_string(&serial_path) {
                    let s = serial.trim().to_string();
                    if !s.is_empty() {
                        if is_ssd && ssd_serial.is_none() {
                            ssd_serial = Some(format!("SSD:{}", s));
                        } else if hdd_serial.is_none() {
                            hdd_serial = Some(format!("HDD:{}", s));
                        }
                    }
                }
            }

            if let Some(ssd) = ssd_serial {
                return Some(ssd);
            }
            if let Some(hdd) = hdd_serial {
                return Some(hdd);
            }
        }

        None
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

    #[cfg(target_os = "android")]
    fn deteksi_android_disk() -> Option<String> {
        let candidates = [
            "/sys/class/android_usb/android0/iSerial",
            "/proc/sys/kernel/random/boot_id",
        ];
        for path in candidates {
            if let Ok(content) = std::fs::read_to_string(path) {
                let trimmed = content.trim().to_string();
                if !trimmed.is_empty() {
                    return Some(format!("ANDROID:{}", trimmed));
                }
            }
        }
        None
    }
}
