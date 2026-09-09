use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LicensePayload {
    pub machine_id: String,
    pub nama_toko: String,
    pub tipe: String, // "LIFETIME" | "ANNUAL" | "TRIAL"
    pub tanggal_terbit: String,
    pub berlaku_sampai: Option<String>,
}

impl LicensePayload {
    pub fn baru_lifetime(
        machine_id: impl Into<String>,
        nama_toko: impl Into<String>,
        tanggal_terbit: impl Into<String>,
    ) -> Self {
        Self {
            machine_id: machine_id.into(),
            nama_toko: nama_toko.into(),
            tipe: "LIFETIME".to_string(),
            tanggal_terbit: tanggal_terbit.into(),
            berlaku_sampai: None,
        }
    }

    /// Serialisasi payload ke representasi bytes kanonikal untuk ditandatangani
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
}

/// Struktur token lisensi lengkap yang berisi payload dan tanda tangan digital Ed25519
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignedLicenseToken {
    pub payload_json: String,
    pub signature_b64: String,
}

impl SignedLicenseToken {
    /// Encode paket lisensi menjadi satu string token ringkas Base64
    pub fn ke_string_token(&self) -> Result<String, String> {
        let json = serde_json::to_string(self).map_err(|e| e.to_string())?;
        Ok(B64.encode(json.as_bytes()))
    }

    /// Decode string token Base64 kembali ke SignedLicenseToken
    pub fn dari_string_token(token_str: &str) -> Result<Self, String> {
        let cleaned = token_str.trim();
        let bytes = B64
            .decode(cleaned)
            .map_err(|e| format!("Format token base64 tidak valid: {}", e))?;
        let token: Self = serde_json::from_slice(&bytes)
            .map_err(|e| format!("Format JSON token tidak valid: {}", e))?;
        Ok(token)
    }
}
