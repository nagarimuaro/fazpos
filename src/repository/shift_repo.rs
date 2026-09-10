use crate::domain::shift::TShift;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct RekapShiftLengkap {
    pub modal_awal: f64,
    pub total_penjualan_tunai: f64,
    pub total_penjualan_nontunai: f64,
    pub total_kas_masuk_lain: f64,
    pub total_kas_keluar: f64,
    pub total_retur_tunai: f64,
    pub uang_seharusnya: f64,
}

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

    /// Ambil shift yang sedang aktif untuk kombinasi cabang dan device tertentu
    pub fn ambil_shift_aktif(&self, cabang_id: &str, device_id: &str) -> Result<Option<TShift>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cabang_id, device_id, operator_id, waktu_buka, waktu_tutup,
                   modal_awal, total_penjualan_tunai, total_penjualan_nontunai, total_retur,
                   total_biaya, uang_seharusnya, uang_aktual, selisih, status, catatan,
                   sync_status, created_at, updated_at
            FROM tshift
            WHERE cabang_id = ?1 AND device_id = ?2 AND (status = 'open' OR status = 'berjalan')
            ORDER BY waktu_buka DESC
            LIMIT 1;
            "#,
        )?;

        stmt.query_row(params![cabang_id, device_id], Self::map_row).optional()
    }

    /// Hitung otomatis seluruh rekapitulasi transaksi selama shift berjalan (skema-database.md §12.4)
    pub fn hitung_rekapitulasi_lengkap(&self, shift_id: &str) -> Result<RekapShiftLengkap> {
        let modal_awal: f64 = self
            .conn
            .query_row("SELECT modal_awal FROM tshift WHERE id = ?1;", params![shift_id], |r| r.get(0))
            .unwrap_or(0.0);

        let total_tunai: f64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(bayar_tunai - kembalian), 0.0) FROM tpenjualan WHERE shift_id = ?1 AND status = 'selesai';",
                params![shift_id],
                |r| r.get(0),
            )
            .unwrap_or(0.0);

        let total_nontunai: f64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(bayar_nontunai), 0.0) FROM tpenjualan WHERE shift_id = ?1 AND status = 'selesai';",
                params![shift_id],
                |r| r.get(0),
            )
            .unwrap_or(0.0);

        let total_masuk_lain: f64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(nominal), 0.0) FROM tcashflow WHERE shift_id = ?1 AND jenis = 'MASUK';",
                params![shift_id],
                |r| r.get(0),
            )
            .unwrap_or(0.0);

        let total_keluar_kas: f64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(nominal), 0.0) FROM tcashflow WHERE shift_id = ?1 AND jenis = 'KELUAR';",
                params![shift_id],
                |r| r.get(0),
            )
            .unwrap_or(0.0);

        let total_retur: f64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(total_retur), 0.0) FROM treturpenjualan WHERE shift_id = ?1;",
                params![shift_id],
                |r| r.get(0),
            )
            .unwrap_or(0.0);

        let uang_seharusnya = (modal_awal + total_tunai + total_masuk_lain - total_keluar_kas - total_retur).max(0.0);

        Ok(RekapShiftLengkap {
            modal_awal,
            total_penjualan_tunai: total_tunai,
            total_penjualan_nontunai: total_nontunai,
            total_kas_masuk_lain: total_masuk_lain,
            total_kas_keluar: total_keluar_kas,
            total_retur_tunai: total_retur,
            uang_seharusnya,
        })
    }

    /// Tambahkan total penjualan kasir ke shift aktif
    pub fn tambah_penjualan(&self, shift_id: &str, tunai: f64, nontunai: f64) -> Result<()> {
        self.conn.execute(
            r#"
            UPDATE tshift
            SET total_penjualan_tunai = total_penjualan_tunai + ?1,
                total_penjualan_nontunai = total_penjualan_nontunai + ?2,
                uang_seharusnya = modal_awal + (total_penjualan_tunai + ?1) + total_kas_masuk_lain - total_kas_keluar - total_retur,
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
        let rekap = self.hitung_rekapitulasi_lengkap(shift_id).unwrap_or(RekapShiftLengkap {
            modal_awal: 0.0,
            total_penjualan_tunai: 0.0,
            total_penjualan_nontunai: 0.0,
            total_kas_masuk_lain: 0.0,
            total_kas_keluar: 0.0,
            total_retur_tunai: 0.0,
            uang_seharusnya,
        });

        self.conn.execute(
            r#"
            UPDATE tshift
            SET waktu_tutup = ?1,
                total_penjualan_tunai = ?2,
                total_penjualan_nontunai = ?3,
                total_kas_masuk_lain = ?4,
                total_kas_keluar = ?5,
                total_retur_tunai = ?6,
                total_biaya = ?5,
                total_retur = ?6,
                uang_seharusnya = ?7,
                uang_aktual = ?8,
                selisih = ?9,
                status = 'closed',
                catatan = ?10,
                updated_at = CURRENT_TIMESTAMP,
                sync_status = 'pending'
            WHERE id = ?11;
            "#,
            params![
                now,
                rekap.total_penjualan_tunai,
                rekap.total_penjualan_nontunai,
                rekap.total_kas_masuk_lain,
                rekap.total_kas_keluar,
                rekap.total_retur_tunai,
                uang_seharusnya,
                uang_aktual,
                selisih,
                catatan,
                shift_id
            ],
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
