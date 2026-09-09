use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TShift {
    pub id: String,
    pub cabang_id: String,
    pub device_id: String,
    pub operator_id: String,
    pub waktu_buka: DateTime<Utc>,
    pub waktu_tutup: Option<DateTime<Utc>>,
    pub modal_awal: f64,
    pub total_penjualan_tunai: f64,
    pub total_penjualan_nontunai: f64,
    pub total_retur: f64,
    pub total_biaya: f64,
    pub uang_seharusnya: f64,
    pub uang_aktual: Option<f64>,
    pub selisih: Option<f64>,
    pub status: String, // "open" | "closed"
    pub catatan: Option<String>,
    pub sync_status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl TShift {
    /// Buka shift baru untuk operator di device tertentu
    pub fn buka(
        cabang_id: impl Into<String>,
        device_id: impl Into<String>,
        operator_id: impl Into<String>,
        modal_awal: f64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            cabang_id: cabang_id.into(),
            device_id: device_id.into(),
            operator_id: operator_id.into(),
            waktu_buka: now,
            waktu_tutup: None,
            modal_awal,
            total_penjualan_tunai: 0.0,
            total_penjualan_nontunai: 0.0,
            total_retur: 0.0,
            total_biaya: 0.0,
            uang_seharusnya: modal_awal,
            uang_aktual: None,
            selisih: None,
            status: "open".to_string(),
            catatan: None,
            sync_status: "pending".to_string(),
            created_at: Some(now),
            updated_at: Some(now),
        }
    }

    /// Hitung uang kas yang seharusnya ada di laci kasir
    pub fn hitung_uang_seharusnya(
        modal_awal: f64,
        total_tunai: f64,
        total_retur: f64,
        total_biaya: f64,
    ) -> f64 {
        modal_awal + total_tunai - total_retur - total_biaya
    }

    /// Hitung selisih antara uang fisik yang dihitung kasir dengan hitungan sistem
    pub fn hitung_selisih(uang_aktual: f64, uang_seharusnya: f64) -> f64 {
        uang_aktual - uang_seharusnya
    }

    /// Tutup shift dengan memasukkan uang fisik aktual
    pub fn tutup(&mut self, uang_aktual: f64, catatan: Option<String>) {
        let seharusnya = Self::hitung_uang_seharusnya(
            self.modal_awal,
            self.total_penjualan_tunai,
            self.total_retur,
            self.total_biaya,
        );
        let selisih = Self::hitung_selisih(uang_aktual, seharusnya);

        self.waktu_tutup = Some(Utc::now());
        self.uang_seharusnya = seharusnya;
        self.uang_aktual = Some(uang_aktual);
        self.selisih = Some(selisih);
        self.status = "closed".to_string();
        self.catatan = catatan;
        self.updated_at = Some(Utc::now());
    }
}
