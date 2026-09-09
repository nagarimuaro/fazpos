use rusqlite::{params, Connection, Result};

#[derive(Debug, Clone)]
pub struct DSupplier {
    pub id: String,
    pub cabang_id: String,
    pub kode: String,
    pub nama: String,
    pub alamat: Option<String>,
    pub telepon: Option<String>,
    pub rekening: Option<String>,
    pub keterangan: Option<String>,
}

pub struct SupplierRepo<'a> {
    conn: &'a Connection,
}

impl<'a> SupplierRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn simpan(&self, s: &DSupplier) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO dsuplier (id, cabang_id, kode, nama, alamat, telepon, rekening, keterangan)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(cabang_id, kode) DO UPDATE SET
                nama = excluded.nama,
                alamat = excluded.alamat,
                telepon = excluded.telepon,
                rekening = excluded.rekening,
                keterangan = excluded.keterangan,
                updated_at = CURRENT_TIMESTAMP;
            "#,
            params![s.id, s.cabang_id, s.kode, s.nama, s.alamat, s.telepon, s.rekening, s.keterangan],
        )?;
        Ok(())
    }

    pub fn semua(&self, cabang_id: &str) -> Result<Vec<DSupplier>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, cabang_id, kode, nama, alamat, telepon, rekening, keterangan FROM dsuplier WHERE cabang_id = ?1 ORDER BY kode ASC LIMIT 200;"
        )?;
        let rows = stmt.query_map(params![cabang_id], |row| {
            Ok(DSupplier {
                id: row.get(0)?,
                cabang_id: row.get(1)?,
                kode: row.get(2)?,
                nama: row.get(3)?,
                alamat: row.get(4)?,
                telepon: row.get(5)?,
                rekening: row.get(6)?,
                keterangan: row.get(7)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn hapus(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM dsuplier WHERE id = ?1", params![id])?;
        Ok(())
    }
}
