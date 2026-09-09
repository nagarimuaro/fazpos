use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DBarang {
    pub id: String,
    pub cabang_id: String,
    pub kode: String,
    pub barcode: Option<String>,
    pub nama: String,
    pub satuan: String,
    pub kategori: Option<String>,
    pub rak: Option<String>,
    pub hargapokok: f64,
    pub hargajual1: f64,
    pub hargajual2: f64,
    pub hargajual3: f64,
    pub hargajual4: f64,
    pub hargapartai: f64,
    pub stok: f64,
    pub stokminimum: f64,
    pub is_aktif: bool,
    pub sync_status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl DBarang {
    pub fn baru(
        cabang_id: impl Into<String>,
        kode: impl Into<String>,
        nama: impl Into<String>,
        hargapokok: f64,
        hargajual1: f64,
        stok: f64,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            cabang_id: cabang_id.into(),
            kode: kode.into(),
            barcode: None,
            nama: nama.into(),
            satuan: "PCS".to_string(),
            kategori: None,
            rak: None,
            hargapokok,
            hargajual1,
            hargajual2: hargajual1,
            hargajual3: hargajual1,
            hargajual4: hargajual1,
            hargapartai: hargajual1,
            stok,
            stokminimum: 0.0,
            is_aktif: true,
            sync_status: "pending".to_string(),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        }
    }

    /// Ambil harga jual berdasarkan tier (1..=4) atau default ke tier 1
    pub fn ambil_harga_tier(&self, tier: u8) -> f64 {
        match tier {
            1 => self.hargajual1,
            2 => if self.hargajual2 > 0.0 { self.hargajual2 } else { self.hargajual1 },
            3 => if self.hargajual3 > 0.0 { self.hargajual3 } else { self.hargajual1 },
            4 => if self.hargajual4 > 0.0 { self.hargajual4 } else { self.hargajual1 },
            5 => if self.hargapartai > 0.0 { self.hargapartai } else { self.hargajual1 },
            _ => self.hargajual1,
        }
    }

    /// Cek apakah stok di bawah batas minimum
    pub fn is_stok_kritis(&self) -> bool {
        self.stok <= self.stokminimum
    }
}
