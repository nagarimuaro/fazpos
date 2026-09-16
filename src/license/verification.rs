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
    0x6d, 0x47, 0x3f, 0xde, 0x1c, 0x40, 0x60, 0x7e, 
    0xc7, 0xa9, 0x8b, 0x1b, 0xfb, 0xc7, 0x05, 0xdd, 
    0x2a, 0xeb, 0x0f, 0xd8, 0xde, 0x14, 0xf7, 0x66, 
    0xc9, 0xe0, 0x9b, 0xc5, 0x70, 0x1a, 0x5e, 0xc6, 
];

#[derive(Debug, Clone, PartialEq)]
pub enum LicenseStatus {
    Aktif(LicensePayload),
    BelumAktivasi,
    TidakValid(String),
}

pub struct LicenseVerifier;

impl LicenseVerifier {
    /// Hasilkan serial token 16-digit (format: XXXX-XXXX-XXXX-XXXX) berbasis Machine ID
    pub fn hitung_serial_16(machine_id: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"FAZPOS_SERIAL_SECRET_2026_");
        hasher.update(machine_id.trim().as_bytes());
        let result = hasher.finalize();
        let hex: String = result.iter().map(|b| format!("{:02X}", b)).collect();
        format!("{}-{}-{}-{}", &hex[0..4], &hex[4..8], &hex[8..12], &hex[12..16])
    }

    /// Verifikasi serial token 16-digit terhadap Machine ID
    pub fn verifikasi_serial_16(serial_token: &str, current_machine_id: &str) -> bool {
        let clean_input = serial_token.replace("-", "").trim().to_uppercase();
        let expected = Self::hitung_serial_16(current_machine_id).replace("-", "");
        clean_input == expected
    }

    /// Verifikasi token menggunakan serial 16-digit atau public key vendor tertanam
    pub fn verifikasi(
        token_str: &str,
        current_machine_id: &str,
    ) -> Result<LicensePayload, String> {
        let token_trim = token_str.trim();
        let clean_token = token_trim.replace("-", "");
        if clean_token.len() == 16 {
            if Self::verifikasi_serial_16(token_trim, current_machine_id) {
                return Ok(LicensePayload::baru_lifetime(
                    current_machine_id,
                    "MUEEZA STORE",
                    "2026-09-14",
                ));
            } else {
                return Err("Serial token 16 digit tidak cocok dengan Machine ID perangkat ini!".to_string());
            }
        }

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

        let clean_token = token_str.replace("-", "");
        let verify_result = if clean_token.len() == 16 {
            if Self::verifikasi_serial_16(token_str, current_machine_id) {
                Ok(LicensePayload::baru_lifetime(
                    current_machine_id,
                    "MUEEZA STORE",
                    "2026-09-14",
                ))
            } else {
                Err("Serial token tidak cocok dengan mesin ini".to_string())
            }
        } else {
            Self::verifikasi_dengan_public_key(token_str, current_machine_id, public_key_bytes)
        };

        match verify_result {
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
