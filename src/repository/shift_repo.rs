use crate::domain::shift::TShift;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Result};

pub struct ShiftRepo<'a> {
    conn: &'a Connection,
}

impl<'a> ShiftRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Simpan buka shift baru
    pub fn buka_shift(&self, s: &TShift) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO tshift (
                id, cabang_id, device_id, operator_id, waktu_buka, modal_awal,
                total_penjualan_tunai, total_penjualan_nontunai, total_retur, total_biaya,
                uang_seharusnya, status, sync_status, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6,
                ?7, ?8, ?9, ?10,
                ?11, ?12, ?13, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
            );
            "#,
            params![
                s.id, s.cabang_id, s.device_id, s.operator_id,
                s.waktu_buka.to_rfc3339(), s.modal_awal,
                s.total_penjualan_tunai, s.total_penjualan_nontunai, s.total_retur, s.total_biaya,
                s.uang_seharusnya, s.status, s.sync_status
            ],
        )?;
        Ok(())
    }

    /// Ambil shift yang sedang aktif (status = 'open') untuk kombinasi cabang dan device tertentu
    pub fn ambil_shift_aktif(&self, cabang_id: &str, device_id: &str) -> Result<Option<TShift>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cabang_id, device_id, operator_id, waktu_buka, waktu_tutup,
                   modal_awal, total_penjualan_tunai, total_penjualan_nontunai, total_retur,
                   total_biaya, uang_seharusnya, uang_aktual, selisih, status, catatan,
                   sync_status, created_at, updated_at
            FROM tshift
            WHERE cabang_id = ?1 AND device_id = ?2 AND status = 'open'
            ORDER BY waktu_buka DESC
            LIMIT 1;
            "#,
        )?;

        stmt.query_row(params![cabang_id, device_id], Self::map_row).optional()
    }

    /// Tambahkan total penjualan kasir ke shift aktif
    pub fn tambah_penjualan(&self, shift_id: &str, tunai: f64, nontunai: f64) -> Result<()> {
        self.conn.execute(
            r#"
            UPDATE tshift
            SET total_penjualan_tunai = total_penjualan_tunai + ?1,
                total_penjualan_nontunai = total_penjualan_nontunai + ?2,
                uang_seharusnya = modal_awal + (total_penjualan_tunai + ?1) - total_retur - total_biaya,
                updated_at = CURRENT_TIMESTAMP,
                sync_status = 'pending'
            WHERE id = ?3;
            "#,
            params![tunai, nontunai, shift_id],
        )?;
        Ok(())
    }

    /// Tutup shift dengan merekam uang fisik aktual dan catatan
    pub fn tutup_shift(
        &self,
        shift_id: &str,
        uang_aktual: f64,
        uang_seharusnya: f64,
        selisih: f64,
        catatan: Option<&str>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            r#"
            UPDATE tshift
            SET waktu_tutup = ?1,
                uang_seharusnya = ?2,
                uang_aktual = ?3,
                selisih = ?4,
                status = 'closed',
                catatan = ?5,
                updated_at = CURRENT_TIMESTAMP,
                sync_status = 'pending'
            WHERE id = ?6;
            "#,
            params![now, uang_seharusnya, uang_aktual, selisih, catatan, shift_id],
        )?;
        Ok(())
    }

    fn map_row(row: &rusqlite::Row) -> Result<TShift> {
        let waktu_buka_str: String = row.get(4)?;
        let waktu_buka = DateTime::parse_from_rfc3339(&waktu_buka_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let waktu_tutup = match row.get::<_, Option<String>>(5)? {
            Some(s) => DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok(),
            None => None,
        };

        Ok(TShift {
            id: row.get(0)?,
            cabang_id: row.get(1)?,
            device_id: row.get(2)?,
            operator_id: row.get(3)?,
            waktu_buka,
            waktu_tutup,
            modal_awal: row.get(6)?,
            total_penjualan_tunai: row.get(7)?,
            total_penjualan_nontunai: row.get(8)?,
            total_retur: row.get(9)?,
            total_biaya: row.get(10)?,
            uang_seharusnya: row.get(11)?,
            uang_aktual: row.get(12)?,
            selisih: row.get(13)?,
            status: row.get(14)?,
            catatan: row.get(15)?,
            sync_status: row.get(16)?,
            created_at: None,
            updated_at: None,
        })
    }
}
