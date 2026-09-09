use chrono::Utc;
use rusqlite::{params, Connection, Result};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ReturItem {
    pub barang_id: String,
    pub kode_barang: String,
    pub nama_barang: String,
    pub jumlah: f64,
    pub satuan: String,
    pub hargajual: f64,
    pub subtotal: f64,
}

pub struct ReturRepo<'a> {
    conn: &'a mut Connection,
}

impl<'a> ReturRepo<'a> {
    pub fn new(conn: &'a mut Connection) -> Self {
        Self { conn }
    }

    /// Simpan transaksi retur penjualan secara ATOMIK:
    /// 1. Insert header treturpenjualan
    /// 2. Insert tiap detail treturpenjualandetail
    /// 3. Kembalikan stok ke dbarang (stok bertambah)
    /// 4. Generate log otomatis ke keluarmasuk (jenis = 'RETUR_PENJUALAN')
    /// 5. Update shift (tambah total_retur)
    pub fn simpan_retur_atomic(
        &mut self,
        cabang_id: &str,
        device_id: &str,
        shift_id: &str,
        faktur_penjualan: &str,
        operator_id: &str,
        items: &[ReturItem],
        diskon_rp: f64,
    ) -> Result<String> {
        let tx = self.conn.transaction()?;

        let retur_id = Uuid::new_v4().to_string();
        let rand_suffix = &uuid::Uuid::new_v4().to_string()[..4].to_uppercase();
        let faktur_retur = format!("RJ-{}-{}", Utc::now().format("%Y%m%d%H%M%S"), rand_suffix);
        let now = Utc::now().to_rfc3339();

        let subtotal: f64 = items.iter().map(|i| i.subtotal).sum();
        let total_akhir = (subtotal - diskon_rp).max(0.0);

        // 1. Insert Header Retur
        tx.execute(
            r#"
            INSERT INTO treturpenjualan (
                id, cabang_id, device_id, shift_id, faktur, tanggal,
                faktur_penjualan, operator_id, subtotal, diskon_rp, total_akhir,
                sync_status, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'pending', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);
            "#,
            params![
                retur_id, cabang_id, device_id, shift_id, faktur_retur, now,
                faktur_penjualan, operator_id, subtotal, diskon_rp, total_akhir
            ],
        )?;

        // 2. Insert Detail & Kembalikan Stok & Log
        for item in items {
            let detail_id = Uuid::new_v4().to_string();
            tx.execute(
                r#"
                INSERT INTO treturpenjualandetail (
                    id, retur_id, cabang_id, barang_id, kode_barang, nama_barang,
                    jumlah, satuan, hargajual, subtotal, sync_status, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'pending', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);
                "#,
                params![
                    detail_id, retur_id, cabang_id, item.barang_id, item.kode_barang,
                    item.nama_barang, item.jumlah, item.satuan, item.hargajual, item.subtotal
                ],
            )?;

            // Kembalikan stok barang (stok bertambah)
            tx.execute(
                r#"
                UPDATE dbarang
                SET stok = stok + ?1,
                    updated_at = CURRENT_TIMESTAMP,
                    sync_status = 'pending'
                WHERE id = ?2;
                "#,
                params![item.jumlah, item.barang_id],
            )?;

            // Ambil sisa stok terbaru
            let sisa_stok: f64 = tx.query_row(
                "SELECT stok FROM dbarang WHERE id = ?1",
                params![item.barang_id],
                |r| r.get(0),
            )?;

            // Catat log otomatis keluarmasuk
            let log_id = Uuid::new_v4().to_string();
            tx.execute(
                r#"
                INSERT INTO keluarmasuk (
                    id, cabang_id, device_id, barang_id, kode_barang, tanggal,
                    jenis, referensi, masuk, keluar, sisa, keterangan, sync_status, created_at
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6,
                    'RETUR_PENJUALAN', ?7, ?8, 0, ?9, 'Retur Penjualan Kasir', 'pending', CURRENT_TIMESTAMP
                );
                "#,
                params![
                    log_id, cabang_id, device_id, item.barang_id, item.kode_barang,
                    now, faktur_retur, item.jumlah, sisa_stok
                ],
            )?;
        }

        // 3. Update Shift Kasir (tambah total_retur dan kurangi uang kas seharusnya)
        tx.execute(
            r#"
            UPDATE tshift
            SET total_retur = total_retur + ?1,
                uang_seharusnya = modal_awal + total_penjualan_tunai - (total_retur + ?1) - total_biaya,
                updated_at = CURRENT_TIMESTAMP,
                sync_status = 'pending'
            WHERE id = ?2;
            "#,
            params![total_akhir, shift_id],
        )?;

        tx.commit()?;
        Ok(faktur_retur)
    }
}
