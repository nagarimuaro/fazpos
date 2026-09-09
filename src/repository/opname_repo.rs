use chrono::Utc;
use rusqlite::{params, Connection, Result};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct OpnameItem {
    pub id: String,
    pub kode_barang: String,
    pub nama_barang: String,
    pub tanggal: String,
    pub stok_komputer: f64,
    pub stok_nyata: f64,
    pub selisih: f64,
    pub alasan: String,
}

pub struct OpnameRepo<'a> {
    conn: &'a mut Connection,
}

impl<'a> OpnameRepo<'a> {
    pub fn new(conn: &'a mut Connection) -> Self {
        Self { conn }
    }

    pub fn simpan_opname(
        &mut self,
        cabang_id: &str,
        barang_id: &str,
        kode_barang: &str,
        nama_barang: &str,
        stok_nyata: f64,
        alasan: &str,
        operator_id: &str,
    ) -> Result<()> {
        let tx = self.conn.transaction()?;
        let now = Utc::now().to_rfc3339();
        let opname_id = Uuid::new_v4().to_string();

        let (stok_komputer, hargapokok): (f64, f64) = tx.query_row(
            "SELECT stok, hargapokok FROM dbarang WHERE id = ?1",
            params![barang_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;

        let selisih = stok_nyata - stok_komputer;
        let total_selisih = selisih * hargapokok;

        tx.execute(
            r#"
            INSERT INTO tstokopname (
                id, cabang_id, barang_id, kode_barang, nama_barang, tanggal,
                stok_komputer, stok_nyata, selisih, total_selisih, alasan, operator_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12);
            "#,
            params![
                opname_id, cabang_id, barang_id, kode_barang, nama_barang, now,
                stok_komputer, stok_nyata, selisih, total_selisih, alasan, operator_id
            ],
        )?;

        tx.execute(
            "UPDATE dbarang SET stok = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2;",
            params![stok_nyata, barang_id],
        )?;

        let masuk = if selisih > 0.0 { selisih } else { 0.0 };
        let keluar = if selisih < 0.0 { -selisih } else { 0.0 };
        let log_id = Uuid::new_v4().to_string();

        tx.execute(
            r#"
            INSERT INTO keluarmasuk (
                id, cabang_id, device_id, barang_id, kode_barang, tanggal,
                jenis, referensi, masuk, keluar, sisa, keterangan
            ) VALUES (
                ?1, ?2, 'SERVER', ?3, ?4, ?5,
                'STOK_OPNAME', ?6, ?7, ?8, ?9, ?10
            );
            "#,
            params![
                log_id, cabang_id, barang_id, kode_barang, now,
                opname_id, masuk, keluar, stok_nyata, alasan
            ],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn riwayat_opname(&self, cabang_id: &str) -> Result<Vec<OpnameItem>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, kode_barang, nama_barang, tanggal, stok_komputer, stok_nyata, selisih, alasan
            FROM tstokopname
            WHERE cabang_id = ?1
            ORDER BY tanggal DESC
            LIMIT 100;
            "#
        )?;
        let rows = stmt.query_map(params![cabang_id], |row| {
            Ok(OpnameItem {
                id: row.get(0)?,
                kode_barang: row.get(1)?,
                nama_barang: row.get(2)?,
                tanggal: row.get(3)?,
                stok_komputer: row.get(4)?,
                stok_nyata: row.get(5)?,
                selisih: row.get(6)?,
                alasan: row.get(7)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }
}
