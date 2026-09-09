use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DOperator {
    pub id: String,
    pub cabang_id: String,
    pub kode: String,
    pub nama: String,
    pub password_hash: String,
    pub role: String, // "admin" | "kasir" | "gudang"
    pub is_aktif: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl DOperator {
    pub fn baru(
        cabang_id: impl Into<String>,
        kode: impl Into<String>,
        nama: impl Into<String>,
        password_hash: impl Into<String>,
        role: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            cabang_id: cabang_id.into(),
            kode: kode.into(),
            nama: nama.into(),
            password_hash: password_hash.into(),
            role: role.into(),
            is_aktif: true,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        }
    }
}
