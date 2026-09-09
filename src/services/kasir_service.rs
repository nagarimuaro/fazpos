use crate::db::Database;
use crate::domain::transaksi::{KeranjangItem, TPenjualan};
use crate::repository::barang_repo::BarangRepo;
use crate::repository::shift_repo::ShiftRepo;
use crate::repository::transaksi_repo::TransaksiRepo;
use chrono::Utc;

pub struct KasirService {
    pub cabang_id: String,
    pub device_id: String,
    pub keranjang: Vec<KeranjangItem>,
}

impl KasirService {
    pub fn new(cabang_id: impl Into<String>, device_id: impl Into<String>) -> Self {
        Self {
            cabang_id: cabang_id.into(),
            device_id: device_id.into(),
            keranjang: Vec::new(),
        }
    }

    /// Scan barcode atau ketik kode barang, tambahkan ke keranjang
    pub fn scan_barcode(&mut self, db: &Database, kode_atau_barcode: &str) -> Result<Option<KeranjangItem>, String> {
        let repo = BarangRepo::new(db.conn());
        let barang_opt = repo
            .cari_by_barcode_atau_kode(&self.cabang_id, kode_atau_barcode)
            .map_err(|e| format!("Database error: {}", e))?;

        let barang = match barang_opt {
            Some(b) => b,
            None => return Ok(None),
        };

        // Jika sudah ada di keranjang, tambah qty +1
        if let Some(existing) = self.keranjang.iter_mut().find(|i| i.barang_id == barang.id) {
            existing.jumlah += 1.0;
            existing.hitung_ulang_subtotal();
            return Ok(Some(existing.clone()));
        }

        // Jika belum ada, buat item baru
        let item = KeranjangItem::baru(
            barang.id,
            barang.kode,
            barang.nama,
            barang.satuan,
            barang.hargajual1,
            barang.hargapokok,
            1.0,
        );
        self.keranjang.push(item.clone());
        Ok(Some(item))
    }

    /// Ubah jumlah item di keranjang
    pub fn ubah_qty(&mut self, index: usize, jumlah_baru: f64) -> Result<(), String> {
        if index >= self.keranjang.len() {
            return Err("Item tidak ditemukan di keranjang".to_string());
        }

        if jumlah_baru <= 0.0 {
            self.keranjang.remove(index);
        } else {
            self.keranjang[index].jumlah = jumlah_baru;
            self.keranjang[index].hitung_ulang_subtotal();
        }
        Ok(())
    }

    /// Hapus item dari keranjang
    pub fn hapus_item(&mut self, index: usize) -> Result<(), String> {
        if index >= self.keranjang.len() {
            return Err("Index di luar jangkauan".to_string());
        }
        self.keranjang.remove(index);
        Ok(())
    }

    /// Hitung subtotal seluruh keranjang
    pub fn total_belanja(&self) -> f64 {
        self.keranjang.iter().map(|i| i.subtotal).sum()
    }

    /// Kosongkan keranjang
    pub fn bersihkan_keranjang(&mut self) {
        self.keranjang.clear();
    }

    /// Eksekusi checkout transaksi kasir
    pub fn checkout(
        &mut self,
        db: &mut Database,
        operator_id: &str,
        bayar_tunai: f64,
        bayar_nontunai: f64,
        diskon_rp: f64,
        metode_bayar: &str,
    ) -> Result<TPenjualan, String> {
        if self.keranjang.is_empty() {
            return Err("Keranjang belanja masih kosong".to_string());
        }

        // Cek shift kasir aktif (aturan 6 SKILL.MD)
        let shift_repo = ShiftRepo::new(db.conn());
        let shift_aktif = shift_repo
            .ambil_shift_aktif(&self.cabang_id, &self.device_id)
            .map_err(|e| format!("Gagal cek shift aktif: {}", e))?
            .ok_or_else(|| "Tidak ada shift kasir yang aktif. Buka shift terlebih dahulu.".to_string())?;

        let total_subtotal = self.total_belanja();
        let total_akhir = (total_subtotal - diskon_rp).max(0.0);
        let total_bayar = bayar_tunai + bayar_nontunai;

        if total_bayar < total_akhir {
            return Err(format!(
                "Pembayaran kurang! Total: Rp {}, Dibayar: Rp {}",
                total_akhir, total_bayar
            ));
        }

        // Generate nomor faktur: format PJ-YYYYMMDDHHMMSS-XXXX (unik per milidetik)
        let rand_suffix = &uuid::Uuid::new_v4().to_string()[..4].to_uppercase();
        let faktur = format!("PJ-{}-{}", Utc::now().format("%Y%m%d%H%M%S"), rand_suffix);

        // Buat objek header dan detail
        let (header, details) = TPenjualan::buat_transaksi(
            &self.cabang_id,
            &self.device_id,
            &shift_aktif.id,
            &faktur,
            operator_id,
            "UMUM",
            &self.keranjang,
            diskon_rp,
            0.0,
            bayar_tunai,
            bayar_nontunai,
            metode_bayar,
        );

        // Simpan atomik (tpenjualan, tpenjualandetail, update stok dbarang, log keluarmasuk, update tshift)
        let mut tx_repo = TransaksiRepo::new(db.conn_mut());
        tx_repo
            .simpan_transaksi_atomic(&header, &details)
            .map_err(|e| format!("Gagal menyimpan transaksi kasir: {}", e))?;

        // Kosongkan keranjang belanja setelah berhasil checkout
        self.bersihkan_keranjang();

        Ok(header)
    }
}
