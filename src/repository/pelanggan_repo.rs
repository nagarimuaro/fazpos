use crate::domain::pelanggan::DPelanggan;
use rusqlite::{params, Connection, OptionalExtension, Result, Row};

pub struct PelangganRepo<'a> {
    conn: &'a Connection,
}

impl<'a> PelangganRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    fn map_row(row: &Row) -> Result<DPelanggan> {
        let is_aktif_int: i32 = row.get(9)?;
        Ok(DPelanggan {
            id: row.get(0)?,
            cabang_id: row.get(1)?,
            kode: row.get(2)?,
            nama: row.get(3)?,
            alamat: row.get(4)?,
            telepon: row.get(5)?,
            plafonpiutang: row.get(6)?,
            poin_saldo: row.get(7)?,
            id_kartu: row.get(8)?,
            is_aktif: is_aktif_int == 1,
            sync_status: row.get(10)?,
            created_at: None,
            updated_at: None,
        })
    }

    /// Cari pelanggan dengan scan kartu (id_kartu), kode persis, atau partial nama / telepon
    /// Sesuai skema-database §12.2 langkah 3
    pub fn cari_member(&self, cabang_id: &str, query: &str) -> Result<Vec<DPelanggan>> {
        let query_trim = query.trim();
        if query_trim.is_empty() {
            return Ok(Vec::new());
        }

        // 1. Coba exact match id_kartu atau kode (hasil scan scanner barcode/kartu member fisik)
        let mut stmt_exact = self.conn.prepare(
            r#"
            SELECT id, cabang_id, kode, nama, alamat, telepon, plafonpiutang,
                   poin_saldo, id_kartu, is_aktif, sync_status
            FROM dpelanggan
            WHERE cabang_id = ?1 AND (id_kartu = ?2 OR kode = ?2) AND is_aktif = 1
            LIMIT 5;
            "#,
        )?;

        let exact_rows = stmt_exact.query_map(params![cabang_id, query_trim], Self::map_row)?;
        let mut list = Vec::new();
        for r in exact_rows {
            list.push(r?);
        }

        if !list.is_empty() {
            return Ok(list);
        }

        // 2. Partial match nama atau no telepon jika bukan exact scan
        let mut stmt_partial = self.conn.prepare(
            r#"
            SELECT id, cabang_id, kode, nama, alamat, telepon, plafonpiutang,
                   poin_saldo, id_kartu, is_aktif, sync_status
            FROM dpelanggan
            WHERE cabang_id = ?1 AND (nama LIKE ?2 OR telepon LIKE ?2) AND is_aktif = 1
            ORDER BY nama ASC
            LIMIT 10;
            "#,
        )?;

        let pola = format!("%{}%", query_trim);
        let partial_rows = stmt_partial.query_map(params![cabang_id, pola], Self::map_row)?;
        for r in partial_rows {
            list.push(r?);
        }

        Ok(list)
    }

    pub fn cari_by_id(&self, id: &str) -> Result<Option<DPelanggan>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cabang_id, kode, nama, alamat, telepon, plafonpiutang,
                   poin_saldo, id_kartu, is_aktif, sync_status
            FROM dpelanggan
            WHERE id = ?1
            LIMIT 1;
            "#,
        )?;
        stmt.query_row(params![id], Self::map_row).optional()
    }

    pub fn cari_by_kode(&self, cabang_id: &str, kode: &str) -> Result<Option<DPelanggan>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cabang_id, kode, nama, alamat, telepon, plafonpiutang,
                   poin_saldo, id_kartu, is_aktif, sync_status
            FROM dpelanggan
            WHERE cabang_id = ?1 AND kode = ?2
            LIMIT 1;
            "#,
        )?;
        stmt.query_row(params![cabang_id, kode], Self::map_row).optional()
    }

    pub fn semua_pelanggan(&self, cabang_id: &str) -> Result<Vec<DPelanggan>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cabang_id, kode, nama, alamat, telepon, plafonpiutang,
                   poin_saldo, id_kartu, is_aktif, sync_status
            FROM dpelanggan
            WHERE cabang_id = ?1
            ORDER BY kode ASC
            LIMIT 100;
            "#,
        )?;

        let rows = stmt.query_map(params![cabang_id], Self::map_row)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn simpan(&self, p: &DPelanggan) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO dpelanggan (
                id, cabang_id, kode, nama, alamat, telepon, plafonpiutang,
                poin_saldo, id_kartu, is_aktif, sync_status, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
            )
            ON CONFLICT(cabang_id, kode) DO UPDATE SET
                nama = excluded.nama,
                alamat = excluded.alamat,
                telepon = excluded.telepon,
                plafonpiutang = excluded.plafonpiutang,
                poin_saldo = excluded.poin_saldo,
                id_kartu = excluded.id_kartu,
                is_aktif = excluded.is_aktif,
                sync_status = 'pending',
                updated_at = CURRENT_TIMESTAMP;
            "#,
            params![
                p.id, p.cabang_id, p.kode, p.nama, p.alamat, p.telepon,
                p.plafonpiutang, p.poin_saldo, p.id_kartu,
                if p.is_aktif { 1 } else { 0 }, p.sync_status
            ],
        )?;
        Ok(())
    }
}
