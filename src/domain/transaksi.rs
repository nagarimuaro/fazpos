use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeranjangItem {
    pub barang_id: String,
    pub kode_barang: String,
    pub nama_barang: String,
    pub satuan: String,
    pub hargajual: f64,
    pub hargapokok: f64,
    pub jumlah: f64,
    pub diskon_persen: f64,
    pub diskon_rp: f64,
    pub subtotal: f64,
}

impl KeranjangItem {
    pub fn baru(
        barang_id: impl Into<String>,
        kode_barang: impl Into<String>,
        nama_barang: impl Into<String>,
        satuan: impl Into<String>,
        hargajual: f64,
        hargapokok: f64,
        jumlah: f64,
    ) -> Self {
        let mut item = Self {
            barang_id: barang_id.into(),
            kode_barang: kode_barang.into(),
            nama_barang: nama_barang.into(),
            satuan: satuan.into(),
            hargajual,
            hargapokok,
            jumlah,
            diskon_persen: 0.0,
            diskon_rp: 0.0,
            subtotal: 0.0,
        };
        item.hitung_ulang_subtotal();
        item
    }

    pub fn hitung_ulang_subtotal(&mut self) {
        let gross = self.jumlah * self.hargajual;
        let pot_persen = gross * (self.diskon_persen / 100.0);
        let total_diskon = pot_persen + self.diskon_rp;
        self.subtotal = (gross - total_diskon).max(0.0);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TPenjualan {
    pub id: String,
    pub cabang_id: String,
    pub device_id: String,
    pub shift_id: String,
    pub faktur: String,
    pub tanggal: DateTime<Utc>,
    pub kode_pelanggan: String,
    pub operator_id: String,
    pub subtotal: f64,
    pub diskon_rp: f64,
    pub total_akhir: f64,
    pub bayar_tunai: f64,
    pub bayar_nontunai: f64,
    pub kembalian: f64,
    pub metode_bayar: String, // "TUNAI" | "NONTUNAI" | "SPLIT"
    pub status: String,       // "selesai" | "batal"
    pub poin_didapat: i64,
    pub poin_ditukar: i64,
    pub nilai_tukar_poin: f64,
    pub sync_status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TPenjualanDetail {
    pub id: String,
    pub penjualan_id: String,
    pub cabang_id: String,
    pub barang_id: String,
    pub kode_barang: String,
    pub nama_barang: String,
    pub jumlah: f64,
    pub satuan: String,
    pub hargajual: f64,
    pub hargapokok: f64,
    pub diskon_persen: f64,
    pub diskon_rp: f64,
    pub subtotal: f64,
    pub sync_status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl TPenjualan {
    /// Helper hitung total & kembalian
    pub fn hitung_ringkasan(
        subtotal: f64,
        diskon_rp: f64,
        nilai_tukar_poin: f64,
        bayar_tunai: f64,
        bayar_nontunai: f64,
    ) -> (f64, f64) {
        let total_akhir = (subtotal - diskon_rp - nilai_tukar_poin).max(0.0);
        let total_bayar = bayar_tunai + bayar_nontunai;
        let kembalian = (total_bayar - total_akhir).max(0.0);
        (total_akhir, kembalian)
    }

    /// Buat record header penjualan dan detailnya dari keranjang belanja
    pub fn buat_transaksi(
        cabang_id: impl Into<String>,
        device_id: impl Into<String>,
        shift_id: impl Into<String>,
        faktur: impl Into<String>,
        operator_id: impl Into<String>,
        kode_pelanggan: impl Into<String>,
        items: &[KeranjangItem],
        diskon_rp: f64,
        nilai_tukar_poin: f64,
        bayar_tunai: f64,
        bayar_nontunai: f64,
        metode_bayar: impl Into<String>,
    ) -> (Self, Vec<TPenjualanDetail>) {
        let cabang_id = cabang_id.into();
        let penjualan_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let subtotal: f64 = items.iter().map(|i| i.subtotal).sum();
        let (total_akhir, kembalian) = Self::hitung_ringkasan(
            subtotal,
            diskon_rp,
            nilai_tukar_poin,
            bayar_tunai,
            bayar_nontunai,
        );

        let header = Self {
            id: penjualan_id.clone(),
            cabang_id: cabang_id.clone(),
            device_id: device_id.into(),
            shift_id: shift_id.into(),
            faktur: faktur.into(),
            tanggal: now,
            kode_pelanggan: kode_pelanggan.into(),
            operator_id: operator_id.into(),
            subtotal,
            diskon_rp,
            total_akhir,
            bayar_tunai,
            bayar_nontunai,
            kembalian,
            metode_bayar: metode_bayar.into(),
            status: "selesai".to_string(),
            poin_didapat: 0,
            poin_ditukar: 0,
            nilai_tukar_poin,
            sync_status: "pending".to_string(),
            created_at: Some(now),
            updated_at: Some(now),
        };

        let details = items
            .iter()
            .map(|item| TPenjualanDetail {
                id: Uuid::new_v4().to_string(),
                penjualan_id: penjualan_id.clone(),
                cabang_id: cabang_id.clone(),
                barang_id: item.barang_id.clone(),
                kode_barang: item.kode_barang.clone(),
                nama_barang: item.nama_barang.clone(),
                jumlah: item.jumlah,
                satuan: item.satuan.clone(),
                hargajual: item.hargajual,
                hargapokok: item.hargapokok,
                diskon_persen: item.diskon_persen,
                diskon_rp: item.diskon_rp,
                subtotal: item.subtotal,
                sync_status: "pending".to_string(),
                created_at: Some(now),
                updated_at: Some(now),
            })
            .collect();

        (header, details)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TPenjualanPending {
    pub id: String,
    pub cabang_id: String,
    pub device_id: String,
    pub shift_id: String,
    pub faktur: String,
    pub tanggal: DateTime<Utc>,
    pub kode_pelanggan: Option<String>,
    pub operator_id: String,
    pub subtotal: f64,
    pub diskon_rp: f64,
    pub total_akhir: f64,
    pub keterangan: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TPenjualanPendingDetail {
    pub id: String,
    pub pending_id: String,
    pub cabang_id: String,
    pub barang_id: String,
    pub kode_barang: String,
    pub nama_barang: String,
    pub jumlah: f64,
    pub satuan: String,
    pub hargajual: f64,
    pub hargapokok: f64,
    pub diskon_persen: f64,
    pub diskon_rp: f64,
    pub subtotal: f64,
    pub created_at: Option<DateTime<Utc>>,
}

impl TPenjualanPending {
    pub fn buat_pending(
        cabang_id: impl Into<String>,
        device_id: impl Into<String>,
        shift_id: impl Into<String>,
        faktur: impl Into<String>,
        operator_id: impl Into<String>,
        kode_pelanggan: Option<String>,
        items: &[KeranjangItem],
        keterangan: Option<String>,
    ) -> (Self, Vec<TPenjualanPendingDetail>) {
        let cabang_id = cabang_id.into();
        let pending_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let subtotal: f64 = items.iter().map(|i| i.subtotal).sum();
        let header = Self {
            id: pending_id.clone(),
            cabang_id: cabang_id.clone(),
            device_id: device_id.into(),
            shift_id: shift_id.into(),
            faktur: faktur.into(),
            tanggal: now,
            kode_pelanggan,
            operator_id: operator_id.into(),
            subtotal,
            diskon_rp: 0.0,
            total_akhir: subtotal,
            keterangan,
            created_at: Some(now),
        };

        let details = items
            .iter()
            .map(|item| TPenjualanPendingDetail {
                id: Uuid::new_v4().to_string(),
                pending_id: pending_id.clone(),
                cabang_id: cabang_id.clone(),
                barang_id: item.barang_id.clone(),
                kode_barang: item.kode_barang.clone(),
                nama_barang: item.nama_barang.clone(),
                jumlah: item.jumlah,
                satuan: item.satuan.clone(),
                hargajual: item.hargajual,
                hargapokok: item.hargapokok,
                diskon_persen: item.diskon_persen,
                diskon_rp: item.diskon_rp,
                subtotal: item.subtotal,
                created_at: Some(now),
            })
            .collect();

        (header, details)
    }
}

impl From<&TPenjualanPendingDetail> for KeranjangItem {
    fn from(d: &TPenjualanPendingDetail) -> Self {
        Self {
            barang_id: d.barang_id.clone(),
            kode_barang: d.kode_barang.clone(),
            nama_barang: d.nama_barang.clone(),
            satuan: d.satuan.clone(),
            hargajual: d.hargajual,
            hargapokok: d.hargapokok,
            jumlah: d.jumlah,
            diskon_persen: d.diskon_persen,
            diskon_rp: d.diskon_rp,
            subtotal: d.subtotal,
        }
    }
}
