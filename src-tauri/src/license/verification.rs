use crate::license::payload::{LicensePayload, SignedLicenseToken};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

// Public Key default Ed25519 vendor (32 bytes) yang tertanam di binary klien
// Hanya bisa digunakan untuk memverifikasi token, tidak dapat digunakan untuk memalsukan/membuat token baru.
pub const EMBEDDED_VENDOR_PUBLIC_KEY: [u8; 32] = [
    0xe9, 0xc8, 0x5d, 0x53, 0x2c, 0xde, 0xb8, 0xb8,
    0x73, 0x31, 0x11, 0x17, 0x68, 0x52, 0xbb, 0x78,
    0x5e, 0x01, 0xa6, 0xe3, 0xa8, 0x65, 0x7b, 0xb7,
    0xf1, 0xb2, 0x74, 0xc9, 0xae, 0x68, 0xd9, 0x4a,
];

#[derive(Debug, Clone, PartialEq)]
pub enum LicenseStatus {
    Aktif(LicensePayload),
    BelumAktivasi,
    TidakValid(String),
}

pub struct LicenseVerifier;

impl LicenseVerifier {
    /// Verifikasi token menggunakan public key vendor tertanam dan Machine ID saat ini
    pub fn verifikasi(
        token_str: &str,
        current_machine_id: &str,
    ) -> Result<LicensePayload, String> {
        Self::verifikasi_dengan_public_key(token_str, current_machine_id, &EMBEDDED_VENDOR_PUBLIC_KEY)
    }

    /// Verifikasi token dengan public key kustom (berguna untuk testing & fleksibilitas rotasi)
    pub fn verifikasi_dengan_public_key(
        token_str: &str,
        current_machine_id: &str,
        public_key_bytes: &[u8; 32],
    ) -> Result<LicensePayload, String> {
        let signed_token = SignedLicenseToken::dari_string_token(token_str)?;

        // 1. Verifikasi Ed25519 Signature
        let verifying_key = VerifyingKey::from_bytes(public_key_bytes)
            .map_err(|e| format!("Public key vendor rusak: {}", e))?;

        let sig_bytes = B64
            .decode(&signed_token.signature_b64)
            .map_err(|e| format!("Format signature base64 tidak valid: {}", e))?;

        if sig_bytes.len() != 64 {
            return Err("Ukuran signature digital tidak valid (harus 64 bytes)".to_string());
        }

        let mut sig_arr = [0u8; 64];
        sig_arr.copy_from_slice(&sig_bytes);
        let signature = Signature::from_bytes(&sig_arr);

        verifying_key
            .verify(signed_token.payload_json.as_bytes(), &signature)
            .map_err(|_| "Tanda tangan digital lisensi PALSU atau tidak valid!".to_string())?;

        // 2. Parse Payload JSON
        let payload: LicensePayload = serde_json::from_str(&signed_token.payload_json)
            .map_err(|e| format!("Struktur payload lisensi rusak: {}", e))?;

        // 3. Hardware Lock Check — Cocokkan Machine ID
        if payload.machine_id != current_machine_id {
            return Err(format!(
                "Lisensi terikat pada mesin [{}] tetapi dijalankan pada [{}]!",
                payload.machine_id, current_machine_id
            ));
        }

        Ok(payload)
    }

    /// Simpan token lisensi ke file lokal dengan checksum anti-tamper
    pub fn simpan_ke_file<P: AsRef<Path>>(
        file_path: P,
        token_str: &str,
        current_machine_id: &str,
    ) -> Result<(), String> {
        let checksum = Self::hitung_anti_tamper(token_str, current_machine_id);
        let content = format!("{}\n{}", token_str.trim(), checksum);
        fs::write(file_path, content).map_err(|e| format!("Gagal menulis file lisensi: {}", e))?;
        Ok(())
    }

    /// Baca dan verifikasi file lisensi lokal
    pub fn baca_dari_file<P: AsRef<Path>>(
        file_path: P,
        current_machine_id: &str,
    ) -> LicenseStatus {
        Self::baca_dari_file_dengan_key(file_path, current_machine_id, &EMBEDDED_VENDOR_PUBLIC_KEY)
    }

    /// Baca dan verifikasi file lisensi lokal dengan public key tertentu
    pub fn baca_dari_file_dengan_key<P: AsRef<Path>>(
        file_path: P,
        current_machine_id: &str,
        public_key_bytes: &[u8; 32],
    ) -> LicenseStatus {
        let path = file_path.as_ref();
        if !path.exists() {
            return LicenseStatus::BelumAktivasi;
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => return LicenseStatus::TidakValid(format!("Gagal membaca file lisensi: {}", e)),
        };

        let mut lines = content.lines();
        let token_str = match lines.next() {
            Some(t) if !t.trim().is_empty() => t.trim(),
            _ => return LicenseStatus::TidakValid("File lisensi kosong".to_string()),
        };

        let checksum_tersimpan = lines.next().unwrap_or("");
        let checksum_harapan = Self::hitung_anti_tamper(token_str, current_machine_id);

        if checksum_tersimpan != checksum_harapan {
            return LicenseStatus::TidakValid(
                "Integritas file lisensi rusak atau disalin dari mesin lain!".to_string(),
            );
        }

        match Self::verifikasi_dengan_public_key(token_str, current_machine_id, public_key_bytes) {
            Ok(payload) => LicenseStatus::Aktif(payload),
            Err(err) => LicenseStatus::TidakValid(err),
        }
    }

    /// Path default penyimpanan lisensi lokal
    pub fn path_lisensi_default() -> PathBuf {
        PathBuf::from("pos_license.lic")
    }

    /// Hitung hash anti-tamper untuk mengikat file ke mesin
    fn hitung_anti_tamper(token_str: &str, machine_id: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"FAZPOS_LICENSE_GUARD_2026_");
        hasher.update(token_str.as_bytes());
        hasher.update(machine_id.as_bytes());
        let result = hasher.finalize();
        result.iter().map(|b| format!("{:02X}", b)).collect()
    }
}
