use crate::domain::barang::DBarang;
use rusqlite::{params, Connection, OptionalExtension, Result};

pub struct BarangRepo<'a> {
    conn: &'a Connection,
}

impl<'a> BarangRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Simpan atau update barang (UPSERT berbasis cabang_id & kode)
    /// Otomatis mencatat histori ke `perubahanharga` jika harga pokok atau jual berubah (Aturan 5 SKILL.MD)
    pub fn simpan(&self, b: &DBarang) -> Result<()> {
        let existing: Option<(String, f64, f64)> = self
            .conn
            .query_row(
                "SELECT id, hargapokok, hargajual1 FROM dbarang WHERE cabang_id = ?1 AND kode = ?2 LIMIT 1;",
                params![b.cabang_id, b.kode],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;

        if let Some((barang_id, hpp_lama, jual_lama)) = existing {
            if (b.hargapokok - hpp_lama).abs() > 0.001 || (b.hargajual1 - jual_lama).abs() > 0.001 {
                let log_id = uuid::Uuid::new_v4().to_string();
                let now = chrono::Utc::now().to_rfc3339();
                let _ = self.conn.execute(
                    r#"
                    INSERT INTO perubahanharga (
                        id, cabang_id, barang_id, tanggal, operator_id,
                        hargapokok_lama, hargapokok_baru, hargajual1_lama, hargajual1_baru,
                        keterangan, sync_status, created_at
                    ) VALUES (
                        ?1, ?2, ?3, ?4, 'SISTEM',
                        ?5, ?6, ?7, ?8,
                        'Update harga produk', 'pending', CURRENT_TIMESTAMP
                    );
                    "#,
                    params![log_id, b.cabang_id, barang_id, now, hpp_lama, b.hargapokok, jual_lama, b.hargajual1],
                );
            }
        }

        self.conn.execute(
            r#"
            INSERT INTO dbarang (
                id, cabang_id, kode, barcode, nama, satuan, kategori, rak,
                hargapokok, hargajual1, hargajual2, hargajual3, hargajual4, hargapartai,
                stok, stokminimum, is_aktif, sync_status, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                ?9, ?10, ?11, ?12, ?13, ?14,
                ?15, ?16, ?17, ?18, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
            )
            ON CONFLICT(cabang_id, kode) DO UPDATE SET
                barcode = excluded.barcode,
                nama = excluded.nama,
                satuan = excluded.satuan,
                kategori = excluded.kategori,
                rak = excluded.rak,
                hargapokok = excluded.hargapokok,
                hargajual1 = excluded.hargajual1,
                hargajual2 = excluded.hargajual2,
                hargajual3 = excluded.hargajual3,
                hargajual4 = excluded.hargajual4,
                hargapartai = excluded.hargapartai,
                stok = excluded.stok,
                stokminimum = excluded.stokminimum,
                is_aktif = excluded.is_aktif,
                sync_status = 'pending',
                updated_at = CURRENT_TIMESTAMP;
            "#,
            params![
                b.id, b.cabang_id, b.kode, b.barcode, b.nama, b.satuan, b.kategori, b.rak,
                b.hargapokok, b.hargajual1, b.hargajual2, b.hargajual3, b.hargajual4, b.hargapartai,
                b.stok, b.stokminimum, if b.is_aktif { 1 } else { 0 }, b.sync_status
            ],
        )?;
        Ok(())
    }

    /// Cari barang berdasarkan barcode persis atau kode barang dalam cabang tertentu
    pub fn cari_by_barcode_atau_kode(&self, cabang_id: &str, query: &str) -> Result<Option<DBarang>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cabang_id, kode, barcode, nama, satuan, kategori, rak,
                   hargapokok, hargajual1, hargajual2, hargajual3, hargajual4, hargapartai,
                   stok, stokminimum, is_aktif, sync_status, created_at, updated_at
            FROM dbarang
            WHERE cabang_id = ?1 AND (barcode = ?2 OR kode = ?2) AND is_aktif = 1
            LIMIT 1;
            "#,
        )?;

        stmt.query_row(params![cabang_id, query], Self::map_row).optional()
    }

    /// Cari barang berdasarkan ID
    pub fn cari_by_id(&self, id: &str) -> Result<Option<DBarang>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cabang_id, kode, barcode, nama, satuan, kategori, rak,
                   hargapokok, hargajual1, hargajual2, hargajual3, hargajual4, hargapartai,
                   stok, stokminimum, is_aktif, sync_status, created_at, updated_at
            FROM dbarang
            WHERE id = ?1
            LIMIT 1;
            "#,
        )?;

        stmt.query_row(params![id], Self::map_row).optional()
    }

    /// Cari barang berdasarkan nama (LIKE %keyword%) untuk pencarian kasir
    pub fn cari_by_nama(&self, cabang_id: &str, keyword: &str, limit: usize) -> Result<Vec<DBarang>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cabang_id, kode, barcode, nama, satuan, kategori, rak,
                   hargapokok, hargajual1, hargajual2, hargajual3, hargajual4, hargapartai,
                   stok, stokminimum, is_aktif, sync_status, created_at, updated_at
            FROM dbarang
            WHERE cabang_id = ?1 AND nama LIKE ?2 AND is_aktif = 1
            ORDER BY nama ASC
            LIMIT ?3;
            "#,
        )?;

        let pola = format!("%{}%", keyword);
        let rows = stmt.query_map(params![cabang_id, pola, limit as i64], Self::map_row)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Update stok (misalnya berkurang saat checkout atau bertambah saat retur)
    pub fn update_stok(&self, id: &str, delta: f64) -> Result<f64> {
        self.conn.execute(
            r#"
            UPDATE dbarang
            SET stok = stok + ?1,
                updated_at = CURRENT_TIMESTAMP,
                sync_status = 'pending'
            WHERE id = ?2;
            "#,
            params![delta, id],
        )?;

        let stok_baru: f64 = self.conn.query_row(
            "SELECT stok FROM dbarang WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        Ok(stok_baru)
    }

    fn map_row(row: &rusqlite::Row) -> Result<DBarang> {
        let is_aktif_int: i32 = row.get(16)?;
        Ok(DBarang {
            id: row.get(0)?,
            cabang_id: row.get(1)?,
            kode: row.get(2)?,
            barcode: row.get(3)?,
            nama: row.get(4)?,
            satuan: row.get(5)?,
            kategori: row.get(6)?,
            rak: row.get(7)?,
            hargapokok: row.get(8)?,
            hargajual1: row.get(9)?,
            hargajual2: row.get(10)?,
            hargajual3: row.get(11)?,
            hargajual4: row.get(12)?,
            hargapartai: row.get(13)?,
            stok: row.get(14)?,
            stokminimum: row.get(15)?,
            is_aktif: is_aktif_int == 1,
            sync_status: row.get(17)?,
            created_at: None,
            updated_at: None,
        })
    }
}
