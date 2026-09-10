use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeluarMasuk {
    pub id: String,
    pub cabang_id: String,
    pub device_id: String,
    pub barang_id: String,
    pub kode_barang: String,
    pub tanggal: DateTime<Utc>,
    pub jenis: String, // "PENJUALAN" | "RETUR_PENJUALAN" | "PEMBELIAN" | dst
    pub referensi: String,
    pub masuk: f64,
    pub keluar: f64,
    pub sisa: f64,
    pub keterangan: Option<String>,
    pub sync_status: String,
    pub created_at: Option<DateTime<Utc>>,
}

impl KeluarMasuk {
    pub fn untuk_penjualan(
        cabang_id: impl Into<String>,
        device_id: impl Into<String>,
        barang_id: impl Into<String>,
        kode_barang: impl Into<String>,
        faktur: impl Into<String>,
        qty_keluar: f64,
        sisa_stok_baru: f64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            cabang_id: cabang_id.into(),
            device_id: device_id.into(),
            barang_id: barang_id.into(),
            kode_barang: kode_barang.into(),
            tanggal: now,
            jenis: "PENJUALAN".to_string(),
            referensi: faktur.into(),
            masuk: 0.0,
            keluar: qty_keluar,
            sisa: sisa_stok_baru,
            keterangan: Some("Transaksi Kasir".to_string()),
            sync_status: "pending".to_string(),
            created_at: Some(now),
        }
    }

    pub fn untuk_retur(
        cabang_id: impl Into<String>,
        device_id: impl Into<String>,
        barang_id: impl Into<String>,
        kode_barang: impl Into<String>,
        referensi: impl Into<String>,
        qty: f64,
        sisa_stok_baru: f64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            cabang_id: cabang_id.into(),
            device_id: device_id.into(),
            barang_id: barang_id.into(),
            kode_barang: kode_barang.into(),
            tanggal: now,
            jenis: "RETUR_PEMBELIAN".to_string(),
            referensi: referensi.into(),
            masuk: 0.0,
            keluar: qty,
            sisa: sisa_stok_baru,
            keterangan: Some("Retur Barang ke Supplier".to_string()),
            sync_status: "pending".to_string(),
            created_at: Some(now),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerubahanHarga {
    pub id: String,
    pub cabang_id: String,
    pub barang_id: String,
    pub tanggal: DateTime<Utc>,
    pub operator_id: Option<String>,
    pub hargapokok_lama: Option<f64>,
    pub hargapokok_baru: Option<f64>,
    pub hargajual1_lama: Option<f64>,
    pub hargajual1_baru: Option<f64>,
    pub keterangan: Option<String>,
    pub sync_status: String,
    pub created_at: Option<DateTime<Utc>>,
}
