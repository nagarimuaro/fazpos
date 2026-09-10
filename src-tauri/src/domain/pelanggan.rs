use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DPelanggan {
    pub id: String,
    pub cabang_id: String,
    pub kode: String,
    pub nama: String,
    pub alamat: Option<String>,
    pub telepon: Option<String>,
    pub plafonpiutang: f64,
    pub poin_saldo: i64,
    pub id_kartu: Option<String>,
    pub is_aktif: bool,
    pub sync_status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl DPelanggan {
    pub fn baru(cabang_id: impl Into<String>, kode: impl Into<String>, nama: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            cabang_id: cabang_id.into(),
            kode: kode.into(),
            nama: nama.into(),
            alamat: None,
            telepon: None,
            plafonpiutang: 0.0,
            poin_saldo: 0,
            id_kartu: None,
            is_aktif: true,
            sync_status: "pending".to_string(),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        }
    }
}
