use chrono::Utc;
use rusqlite::{params, Connection, Result};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PembelianItem {
    pub barang_id: String,
    pub kode_barang: String,
    pub nama_barang: String,
    pub jumlah: f64,
    pub satuan: String,
    pub harga_beli: f64,
    pub subtotal: f64,
}

pub struct PembelianRepo<'a> {
    conn: &'a mut Connection,
}

impl<'a> PembelianRepo<'a> {
    pub fn new(conn: &'a mut Connection) -> Self {
        Self { conn }
    }

    pub fn simpan_pembelian_atomic(
        &mut self,
        cabang_id: &str,
        suplier_id: &str,
        nama_suplier: &str,
        faktur_supplier: Option<&str>,
        operator_id: &str,
        items: &[PembelianItem],
        diskon_rp: f64,
        bayar: f64,
        tunai_kredit: &str,
    ) -> Result<String> {
        let tx = self.conn.transaction()?;

        let pembelian_id = Uuid::new_v4().to_string();
        let rand_suffix = &Uuid::new_v4().to_string()[..4].to_uppercase();
        let faktur_beli = format!("PB-{}-{}", Utc::now().format("%Y%m%d%H%M%S"), rand_suffix);
        let now = Utc::now().to_rfc3339();

        let subtotal: f64 = items.iter().map(|i| i.subtotal).sum();
        let total_akhir = (subtotal - diskon_rp).max(0.0);
        let sisa = (total_akhir - bayar).max(0.0);

        // 1. Insert Header Pembelian
        tx.execute(
            r#"
            INSERT INTO tpembelian (
                id, cabang_id, faktur, faktur_supplier, tanggal,
                suplier_id, nama_suplier, operator_id, total, diskon_rp,
                total_akhir, bayar, sisa, tunai_kredit, status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, 'selesai');
            "#,
            params![
                pembelian_id, cabang_id, faktur_beli, faktur_supplier, now,
                suplier_id, nama_suplier, operator_id, subtotal, diskon_rp,
                total_akhir, bayar, sisa, tunai_kredit
            ],
        )?;

        // 2. Insert Detail & Update Stok & Mutasi
        for item in items {
            let detail_id = Uuid::new_v4().to_string();
            tx.execute(
                r#"
                INSERT INTO tpembeliandetail (
                    id, pembelian_id, cabang_id, barang_id, kode_barang,
                    nama_barang, jumlah, satuan, harga_beli, subtotal
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10);
                "#,
                params![
                    detail_id, pembelian_id, cabang_id, item.barang_id, item.kode_barang,
                    item.nama_barang, item.jumlah, item.satuan, item.harga_beli, item.subtotal
                ],
            )?;

            // Update stok dan hargapokok
            tx.execute(
                r#"
                UPDATE dbarang
                SET stok = stok + ?1,
                    hargapokok = ?2,
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = ?3;
                "#,
                params![item.jumlah, item.harga_beli, item.barang_id],
            )?;

            let sisa_stok: f64 = tx.query_row(
                "SELECT stok FROM dbarang WHERE id = ?1",
                params![item.barang_id],
                |r| r.get(0),
            )?;

            // Catat log mutasi
            let log_id = Uuid::new_v4().to_string();
            tx.execute(
                r#"
                INSERT INTO keluarmasuk (
                    id, cabang_id, device_id, barang_id, kode_barang, tanggal,
                    jenis, referensi, masuk, keluar, sisa, keterangan
                ) VALUES (
                    ?1, ?2, 'SERVER', ?3, ?4, ?5,
                    'PEMBELIAN', ?6, ?7, 0, ?8, 'Pengadaan Stok Supplier'
                );
                "#,
                params![
                    log_id, cabang_id, item.barang_id, item.kode_barang, now,
                    faktur_beli, item.jumlah, sisa_stok
                ],
            )?;
        }

        // 3. Catat Hutang jika ada sisa
        if sisa > 0.0 || tunai_kredit == "KREDIT" {
            let hutang_id = Uuid::new_v4().to_string();
            tx.execute(
                r#"
                INSERT INTO thutang (
                    id, cabang_id, faktur_beli, suplier_id, nama_suplier,
                    tanggal, tagihan_awal, telah_dibayar, sisa, status, keterangan
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'belum_lunas', 'Hutang Faktur Pembelian');
                "#,
                params![
                    hutang_id, cabang_id, faktur_beli, suplier_id, nama_suplier,
                    now, total_akhir, bayar, sisa
                ],
            )?;
        }

        tx.commit()?;
        Ok(faktur_beli)
    }
}
