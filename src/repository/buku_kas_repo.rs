use chrono::Utc;
use rusqlite::{params, Connection, Result};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CashflowItem {
    pub id: String,
    pub tanggal: String,
    pub jenis: String,
    pub kategori: String,
    pub nominal: f64,
    pub keterangan: String,
}

#[derive(Debug, Clone)]
pub struct RekapKas {
    pub total_masuk: f64,
    pub total_keluar: f64,
    pub saldo_kas: f64,
}

pub struct BukuKasRepo<'a> {
    conn: &'a Connection,
}

impl<'a> BukuKasRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn catat_arus_kas(
        &self,
        cabang_id: &str,
        jenis: &str,
        kategori: &str,
        nominal: f64,
        keterangan: &str,
        operator_id: &str,
    ) -> Result<()> {
        self.catat_arus_kas_dengan_shift(cabang_id, None, jenis, kategori, nominal, keterangan, operator_id)
    }

    pub fn catat_arus_kas_dengan_shift(
        &self,
        cabang_id: &str,
        shift_id: Option<&str>,
        jenis: &str,
        kategori: &str,
        nominal: f64,
        keterangan: &str,
        operator_id: &str,
    ) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            r#"
            INSERT INTO tcashflow (id, cabang_id, shift_id, tanggal, jenis, kategori, nominal, keterangan, operator_id)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);
            "#,
            params![id, cabang_id, shift_id, now, jenis, kategori, nominal, keterangan, operator_id],
        )?;

        // Jika KELUAR, masukkan juga ke tbiaya (dengan shift_id)
        if jenis == "KELUAR" {
            let biaya_id = Uuid::new_v4().to_string();
            self.conn.execute(
                r#"
                INSERT INTO tbiaya (id, cabang_id, shift_id, tanggal, kategori, nominal, keterangan, operator_id)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);
                "#,
                params![biaya_id, cabang_id, shift_id, now, kategori, nominal, keterangan, operator_id],
            )?;
        }

        Ok(())
    }

    pub fn riwayat_kas(&self, cabang_id: &str) -> Result<Vec<CashflowItem>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, tanggal, jenis, kategori, nominal, keterangan
            FROM tcashflow
            WHERE cabang_id = ?1
            ORDER BY tanggal DESC
            LIMIT 100;
            "#
        )?;
        let rows = stmt.query_map(params![cabang_id], |row| {
            let ket: Option<String> = row.get(5)?;
            Ok(CashflowItem {
                id: row.get(0)?,
                tanggal: row.get(1)?,
                jenis: row.get(2)?,
                kategori: row.get(3)?,
                nominal: row.get(4)?,
                keterangan: ket.unwrap_or_default(),
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn rekap_kas(&self, cabang_id: &str) -> Result<RekapKas> {
        let total_masuk: f64 = self.conn.query_row(
            "SELECT COALESCE(SUM(nominal), 0) FROM tcashflow WHERE cabang_id = ?1 AND jenis = 'MASUK'",
            params![cabang_id],
            |r| r.get(0),
        ).unwrap_or(0.0);

        let total_keluar: f64 = self.conn.query_row(
            "SELECT COALESCE(SUM(nominal), 0) FROM tcashflow WHERE cabang_id = ?1 AND jenis = 'KELUAR'",
            params![cabang_id],
            |r| r.get(0),
        ).unwrap_or(0.0);

        Ok(RekapKas {
            total_masuk,
            total_keluar,
            saldo_kas: total_masuk - total_keluar,
        })
    }
}
