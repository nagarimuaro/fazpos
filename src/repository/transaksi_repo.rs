use crate::domain::log::KeluarMasuk;
use crate::domain::transaksi::{TPenjualan, TPenjualanDetail};
use rusqlite::{params, Connection, OptionalExtension, Result};

pub struct TransaksiRepo<'a> {
    conn: &'a mut Connection,
}

impl<'a> TransaksiRepo<'a> {
    pub fn new(conn: &'a mut Connection) -> Self {
        Self { conn }
    }

    /// Simpan transaksi lengkap secara ATOMIK:
    /// 1. Insert header tpenjualan
    /// 2. Insert tiap detail tpenjualandetail
    /// 3. Kurangi stok di dbarang
    /// 4. Generate log otomatis ke keluarmasuk
    /// 5. Update shift (tambah total tunai / nontunai)
    pub fn simpan_transaksi_atomic(
        &mut self,
        header: &TPenjualan,
        details: &[TPenjualanDetail],
    ) -> Result<()> {
        let tx = self.conn.transaction()?;

        // 1. Insert Header Penjualan
        tx.execute(
            r#"
            INSERT INTO tpenjualan (
                id, cabang_id, device_id, shift_id, faktur, tanggal,
                kode_pelanggan, operator_id, subtotal, diskon_rp, total_akhir,
                bayar_tunai, bayar_nontunai, kembalian, metode_bayar, status,
                poin_didapat, poin_ditukar, nilai_tukar_poin, sync_status,
                created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6,
                ?7, ?8, ?9, ?10, ?11,
                ?12, ?13, ?14, ?15, ?16,
                ?17, ?18, ?19, ?20,
                CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
            );
            "#,
            params![
                header.id, header.cabang_id, header.device_id, header.shift_id,
                header.faktur, header.tanggal.to_rfc3339(), header.kode_pelanggan,
                header.operator_id, header.subtotal, header.diskon_rp, header.total_akhir,
                header.bayar_tunai, header.bayar_nontunai, header.kembalian, header.metode_bayar,
                header.status, header.poin_didapat, header.poin_ditukar, header.nilai_tukar_poin,
                header.sync_status
            ],
        )?;

        // 2. Insert Detail & Update Stok & Catat Log
        for item in details {
            tx.execute(
                r#"
                INSERT INTO tpenjualandetail (
                    id, penjualan_id, cabang_id, barang_id, kode_barang, nama_barang,
                    jumlah, satuan, hargajual, hargapokok, diskon_persen, diskon_rp,
                    subtotal, sync_status, created_at, updated_at
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6,
                    ?7, ?8, ?9, ?10, ?11, ?12,
                    ?13, ?14, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
                );
                "#,
                params![
                    item.id, item.penjualan_id, item.cabang_id, item.barang_id,
                    item.kode_barang, item.nama_barang, item.jumlah, item.satuan,
                    item.hargajual, item.hargapokok, item.diskon_persen, item.diskon_rp,
                    item.subtotal, item.sync_status
                ],
            )?;

            // Kurangi stok barang
            tx.execute(
                r#"
                UPDATE dbarang
                SET stok = stok - ?1,
                    updated_at = CURRENT_TIMESTAMP,
                    sync_status = 'pending'
                WHERE id = ?2;
                "#,
                params![item.jumlah, item.barang_id],
            )?;

            // Ambil sisa stok terbaru untuk dimasukkan ke log keluarmasuk
            let sisa_stok: f64 = tx.query_row(
                "SELECT stok FROM dbarang WHERE id = ?1",
                params![item.barang_id],
                |r| r.get(0),
            )?;

            // Generate log otomatis keluarmasuk (aturan 5 SKILL.MD)
            let log = KeluarMasuk::untuk_penjualan(
                &header.cabang_id,
                &header.device_id,
                &item.barang_id,
                &item.kode_barang,
                &header.faktur,
                item.jumlah,
                sisa_stok,
            );

            tx.execute(
                r#"
                INSERT INTO keluarmasuk (
                    id, cabang_id, device_id, barang_id, kode_barang, tanggal,
                    jenis, referensi, masuk, keluar, sisa, keterangan, sync_status, created_at
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6,
                    ?7, ?8, ?9, ?10, ?11, ?12, ?13, CURRENT_TIMESTAMP
                );
                "#,
                params![
                    log.id, log.cabang_id, log.device_id, log.barang_id, log.kode_barang,
                    log.tanggal.to_rfc3339(), log.jenis, log.referensi, log.masuk, log.keluar,
                    log.sisa, log.keterangan, log.sync_status
                ],
            )?;
        }

        // 3. Update Shift Kasir (hanya uang kas bersih yang masuk laci kasir setelah dipotong kembalian)
        let kas_tunai_masuk = (header.bayar_tunai - header.kembalian).max(0.0);
        tx.execute(
            r#"
            UPDATE tshift
            SET total_penjualan_tunai = total_penjualan_tunai + ?1,
                total_penjualan_nontunai = total_penjualan_nontunai + ?2,
                uang_seharusnya = modal_awal + (total_penjualan_tunai + ?1) - total_retur - total_biaya,
                updated_at = CURRENT_TIMESTAMP,
                sync_status = 'pending'
            WHERE id = ?3;
            "#,
            params![kas_tunai_masuk, header.bayar_nontunai, header.shift_id],
        )?;

        tx.commit()?;
        Ok(())
    }

    /// Ambil header penjualan berdasarkan faktur dan cabang
    pub fn cari_by_faktur(&self, cabang_id: &str, faktur: &str) -> Result<Option<TPenjualan>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cabang_id, device_id, shift_id, faktur, tanggal,
                   kode_pelanggan, operator_id, subtotal, diskon_rp, total_akhir,
                   bayar_tunai, bayar_nontunai, kembalian, metode_bayar, status,
                   poin_didapat, poin_ditukar, nilai_tukar_poin, sync_status,
                   created_at, updated_at
            FROM tpenjualan
            WHERE cabang_id = ?1 AND faktur = ?2
            LIMIT 1;
            "#,
        )?;

        stmt.query_row(params![cabang_id, faktur], |row| {
            let tgl_str: String = row.get(5)?;
            let tanggal = chrono::DateTime::parse_from_rfc3339(&tgl_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            Ok(TPenjualan {
                id: row.get(0)?,
                cabang_id: row.get(1)?,
                device_id: row.get(2)?,
                shift_id: row.get(3)?,
                faktur: row.get(4)?,
                tanggal,
                kode_pelanggan: row.get(6)?,
                operator_id: row.get(7)?,
                subtotal: row.get(8)?,
                diskon_rp: row.get(9)?,
                total_akhir: row.get(10)?,
                bayar_tunai: row.get(11)?,
                bayar_nontunai: row.get(12)?,
                kembalian: row.get(13)?,
                metode_bayar: row.get(14)?,
                status: row.get(15)?,
                poin_didapat: row.get(16)?,
                poin_ditukar: row.get(17)?,
                nilai_tukar_poin: row.get(18)?,
                sync_status: row.get(19)?,
                created_at: None,
                updated_at: None,
            })
        })
        .optional()
    }
}
