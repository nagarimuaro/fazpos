use crate::domain::transaksi::{TPenjualanPending, TPenjualanPendingDetail};
use chrono::Utc;
use rusqlite::{params, Connection, Result};

pub struct PendingRepo<'a> {
    conn: &'a Connection,
}

impl<'a> PendingRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Simpan transaksi tertahan (Hold) ke database
    pub fn simpan_pending(
        &self,
        header: &TPenjualanPending,
        details: &[TPenjualanPendingDetail],
    ) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO tpenjualanpending (
                id, cabang_id, device_id, shift_id, faktur, tanggal,
                kode_pelanggan, operator_id, subtotal, diskon_rp, total_akhir,
                keterangan, created_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6,
                ?7, ?8, ?9, ?10, ?11,
                ?12, CURRENT_TIMESTAMP
            );
            "#,
            params![
                header.id,
                header.cabang_id,
                header.device_id,
                header.shift_id,
                header.faktur,
                header.tanggal.to_rfc3339(),
                header.kode_pelanggan,
                header.operator_id,
                header.subtotal,
                header.diskon_rp,
                header.total_akhir,
                header.keterangan,
            ],
        )?;

        for d in details {
            self.conn.execute(
                r#"
                INSERT INTO tpenjualanpendingdetail (
                    id, pending_id, cabang_id, barang_id, kode_barang, nama_barang,
                    jumlah, satuan, hargajual, hargapokok, diskon_persen, diskon_rp,
                    subtotal, created_at
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6,
                    ?7, ?8, ?9, ?10, ?11, ?12,
                    ?13, CURRENT_TIMESTAMP
                );
                "#,
                params![
                    d.id,
                    d.pending_id,
                    d.cabang_id,
                    d.barang_id,
                    d.kode_barang,
                    d.nama_barang,
                    d.jumlah,
                    d.satuan,
                    d.hargajual,
                    d.hargapokok,
                    d.diskon_persen,
                    d.diskon_rp,
                    d.subtotal,
                ],
            )?;
        }

        Ok(())
    }

    /// Ambil daftar semua transaksi pending di cabang dan device tertentu
    pub fn semua_pending(&self, cabang_id: &str, device_id: &str) -> Result<Vec<TPenjualanPending>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cabang_id, device_id, shift_id, faktur, tanggal,
                   kode_pelanggan, operator_id, subtotal, diskon_rp, total_akhir,
                   keterangan, created_at
            FROM tpenjualanpending
            WHERE cabang_id = ?1 AND device_id = ?2
            ORDER BY created_at DESC;
            "#,
        )?;

        let rows = stmt.query_map(params![cabang_id, device_id], |row| {
            let tgl_str: String = row.get(5)?;
            let tanggal = chrono::DateTime::parse_from_rfc3339(&tgl_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(TPenjualanPending {
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
                keterangan: row.get(11)?,
                created_at: None,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Ambil detail barang dari suatu transaksi pending (untuk di-Recall)
    pub fn ambil_detail(&self, pending_id: &str) -> Result<Vec<TPenjualanPendingDetail>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, pending_id, cabang_id, barang_id, kode_barang, nama_barang,
                   jumlah, satuan, hargajual, hargapokok, diskon_persen, diskon_rp,
                   subtotal, created_at
            FROM tpenjualanpendingdetail
            WHERE pending_id = ?1
            ORDER BY created_at ASC;
            "#,
        )?;

        let rows = stmt.query_map(params![pending_id], |row| {
            Ok(TPenjualanPendingDetail {
                id: row.get(0)?,
                pending_id: row.get(1)?,
                cabang_id: row.get(2)?,
                barang_id: row.get(3)?,
                kode_barang: row.get(4)?,
                nama_barang: row.get(5)?,
                jumlah: row.get(6)?,
                satuan: row.get(7)?,
                hargajual: row.get(8)?,
                hargapokok: row.get(9)?,
                diskon_persen: row.get(10)?,
                diskon_rp: row.get(11)?,
                subtotal: row.get(12)?,
                created_at: None,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Hapus transaksi pending setelah berhasil di-Recall atau dibatalkan
    pub fn hapus_pending(&self, pending_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM tpenjualanpendingdetail WHERE pending_id = ?1;",
            params![pending_id],
        )?;
        self.conn.execute(
            "DELETE FROM tpenjualanpending WHERE id = ?1;",
            params![pending_id],
        )?;
        Ok(())
    }
}
