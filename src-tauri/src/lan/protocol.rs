use crate::domain::transaksi::KeranjangItem;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LanPingResponse {
    pub status: String,
    pub cabang_id: String,
    pub device_id: String,
    pub role: String,
    pub versi: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LanCheckoutRequest {
    pub device_id: String,
    pub operator_id: String,
    pub shift_id: String,
    pub items: Vec<KeranjangItem>,
    pub bayar_tunai: f64,
    pub bayar_nontunai: f64,
    pub diskon_rp: f64,
    pub metode_bayar: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LanCheckoutResponse {
    pub sukses: bool,
    pub faktur: Option<String>,
    pub total_akhir: f64,
    pub kembalian: f64,
    pub pesan: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LanCatalogItem {
    pub id: String,
    pub kode: String,
    pub barcode: Option<String>,
    pub nama: String,
    pub satuan: String,
    pub hargajual1: f64,
    pub stok: f64,
}
