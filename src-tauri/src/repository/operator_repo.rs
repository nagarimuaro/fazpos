use crate::domain::operator::DOperator;
use rusqlite::{params, Connection, OptionalExtension, Result, Row};
use sha2::{Digest, Sha256};

pub struct OperatorRepo<'a> {
    conn: &'a Connection,
}

impl<'a> OperatorRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Hash password atau PIN operator menggunakan SHA-256 (Aturan §4 skema-database.md)
    pub fn hash_password(raw_password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"FAZPOS_SALT_");
        hasher.update(raw_password.as_bytes());
        let res = hasher.finalize();
        res.iter().map(|b| format!("{:02x}", b)).collect::<String>()
    }

    fn map_row(row: &Row) -> Result<DOperator> {
        let is_aktif_int: i32 = row.get(6)?;
        Ok(DOperator {
            id: row.get(0)?,
            cabang_id: row.get(1)?,
            kode: row.get(2)?,
            nama: row.get(3)?,
            password_hash: row.get(4)?,
            role: row.get(5)?,
            is_aktif: is_aktif_int == 1,
            created_at: None,
            updated_at: None,
        })
    }

    /// Verifikasi kredensial login/scan operator kasir
    pub fn verifikasi(
        &self,
        cabang_id: &str,
        kode: &str,
        raw_password: &str,
    ) -> Result<Option<DOperator>> {
        let expected_hash = Self::hash_password(raw_password);
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cabang_id, kode, nama, password_hash, role, is_aktif
            FROM doperator
            WHERE cabang_id = ?1 AND kode = ?2 AND is_aktif = 1
            LIMIT 1;
            "#,
        )?;

        let opt_op = stmt.query_row(params![cabang_id, kode], Self::map_row).optional()?;

        match opt_op {
            Some(op) if op.password_hash == expected_hash || op.password_hash == raw_password => Ok(Some(op)),
            _ => Ok(None),
        }
    }

    pub fn cari_by_id(&self, id: &str) -> Result<Option<DOperator>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cabang_id, kode, nama, password_hash, role, is_aktif
            FROM doperator
            WHERE id = ?1
            LIMIT 1;
            "#,
        )?;
        stmt.query_row(params![id], Self::map_row).optional()
    }

    pub fn cari_by_kode(&self, cabang_id: &str, kode: &str) -> Result<Option<DOperator>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cabang_id, kode, nama, password_hash, role, is_aktif
            FROM doperator
            WHERE cabang_id = ?1 AND kode = ?2
            LIMIT 1;
            "#,
        )?;
        stmt.query_row(params![cabang_id, kode], Self::map_row).optional()
    }

    pub fn semua_operator(&self, cabang_id: &str) -> Result<Vec<DOperator>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cabang_id, kode, nama, password_hash, role, is_aktif
            FROM doperator
            WHERE cabang_id = ?1
            ORDER BY kode ASC;
            "#,
        )?;

        let rows = stmt.query_map(params![cabang_id], Self::map_row)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn simpan(&self, op: &DOperator) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO doperator (
                id, cabang_id, kode, nama, password_hash, role, is_aktif,
                created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
            )
            ON CONFLICT(cabang_id, kode) DO UPDATE SET
                nama = excluded.nama,
                password_hash = excluded.password_hash,
                role = excluded.role,
                is_aktif = excluded.is_aktif,
                updated_at = CURRENT_TIMESTAMP;
            "#,
            params![
                op.id,
                op.cabang_id,
                op.kode,
                op.nama,
                op.password_hash,
                op.role,
                if op.is_aktif { 1 } else { 0 },
            ],
        )?;
        Ok(())
    }

    pub fn hapus(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM doperator WHERE id = ?1;", params![id])?;
        Ok(())
    }
}
