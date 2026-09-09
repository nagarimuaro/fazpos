use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncQueueItem {
    pub tabel: String,
    pub record_id: String,
    pub operasi: String, // "INSERT" | "UPDATE"
    pub payload_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CloudSyncBatch {
    pub cabang_id: String,
    pub device_id: String,
    pub batch_id: String,
    pub items: Vec<SyncQueueItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CloudSyncAck {
    pub batch_id: String,
    pub synced_ids: Vec<String>,
    pub status: String, // "OK" | "PARTIAL" | "ERROR"
    pub pesan: Option<String>,
}
