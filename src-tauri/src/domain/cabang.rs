use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cabang {
    pub id: String,
    pub kode: String,
    pub nama: String,
    pub alamat: Option<String>,
    pub telepon: Option<String>,
    pub is_pusat: bool,
    pub sync_status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Cabang {
    pub fn baru(kode: impl Into<String>, nama: impl Into<String>, is_pusat: bool) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            kode: kode.into(),
            nama: nama.into(),
            alamat: None,
            telepon: None,
            is_pusat,
            sync_status: "pending".to_string(),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Device {
    pub id: String,
    pub cabang_id: String,
    pub kode: String,
    pub nama: String,
    pub role: String, // "server" | "client"
    pub machine_id: String,
    pub ip_address: Option<String>,
    pub is_active: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Device {
    pub fn baru(
        cabang_id: impl Into<String>,
        kode: impl Into<String>,
        nama: impl Into<String>,
        role: impl Into<String>,
        machine_id: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            cabang_id: cabang_id.into(),
            kode: kode.into(),
            nama: nama.into(),
            role: role.into(),
            machine_id: machine_id.into(),
            ip_address: None,
            is_active: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        }
    }
}
