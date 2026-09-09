use crate::domain::poin::PengaturanPoin;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Result};
use uuid::Uuid;

pub struct PoinRepo<'a> {
    conn: &'a Connection,
}

impl<'a> PoinRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Ambil pengaturan poin cabang, atau buat default jika belum ada
    pub fn ambil_pengaturan(&self, cabang_id: &str) -> Result<PengaturanPoin> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cabang_id, rupiah_per_poin, nilai_tukar_poin, minimal_tukar, is_aktif
            FROM pengaturan_poin
            WHERE cabang_id = ?1
            LIMIT 1;
            "#,
        )?;

        let opt = stmt.query_row(params![cabang_id], |row| {
            let is_aktif_int: i32 = row.get(5)?;
            Ok(PengaturanPoin {
                id: row.get(0)?,
                cabang_id: row.get(1)?,
                rupiah_per_poin: row.get(2)?,
                nilai_tukar_poin: row.get(3)?,
                minimal_tukar: row.get(4)?,
                is_aktif: is_aktif_int == 1,
            })
        }).optional()?;

        match opt {
            Some(p) => Ok(p),
            None => {
                let default = PengaturanPoin::default(cabang_id);
                self.simpan_pengaturan(&default)?;
                Ok(default)
            }
        }
    }

    /// Simpan atau update pengaturan poin
    pub fn simpan_pengaturan(&self, p: &PengaturanPoin) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO pengaturan_poin (
                id, cabang_id, rupiah_per_poin, nilai_tukar_poin, minimal_tukar, is_aktif,
                sync_status, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            ON CONFLICT(cabang_id) DO UPDATE SET
                rupiah_per_poin = excluded.rupiah_per_poin,
                nilai_tukar_poin = excluded.nilai_tukar_poin,
                minimal_tukar = excluded.minimal_tukar,
                is_aktif = excluded.is_aktif,
                updated_at = CURRENT_TIMESTAMP;
            "#,
            params![
                p.id, p.cabang_id, p.rupiah_per_poin, p.nilai_tukar_poin,
                p.minimal_tukar, if p.is_aktif { 1 } else { 0 }
            ],
        )?;
        Ok(())
    }

    /// Tambah poin ke pelanggan (saat belanja) dan catat riwayat
    pub fn tambah_poin(
        &self,
        cabang_id: &str,
        pelanggan_id: &str,
        faktur: &str,
        poin_tambah: i64,
        keterangan: &str,
    ) -> Result<i64> {
        if poin_tambah <= 0 {
            return self.ambil_saldo(pelanggan_id);
        }

        self.conn.execute(
            "UPDATE dpelanggan SET poin_saldo = poin_saldo + ?1 WHERE id = ?2;",
            params![poin_tambah, pelanggan_id],
        )?;

        let saldo_akhir = self.ambil_saldo(pelanggan_id)?;
        self.catat_riwayat(cabang_id, pelanggan_id, Some(faktur), "DAPAT", poin_tambah, saldo_akhir, keterangan)?;

        Ok(saldo_akhir)
    }

    /// Kurangi poin pelanggan (saat ditukar diskon) dan catat riwayat
    pub fn kurangi_poin(
        &self,
        cabang_id: &str,
        pelanggan_id: &str,
        faktur: &str,
        poin_kurang: i64,
        keterangan: &str,
    ) -> Result<i64> {
        if poin_kurang <= 0 {
            return self.ambil_saldo(pelanggan_id);
        }

        self.conn.execute(
            "UPDATE dpelanggan SET poin_saldo = poin_saldo - ?1 WHERE id = ?2;",
            params![poin_kurang, pelanggan_id],
        )?;

        let saldo_akhir = self.ambil_saldo(pelanggan_id)?;
        self.catat_riwayat(cabang_id, pelanggan_id, Some(faktur), "TUKAR", poin_kurang, saldo_akhir, keterangan)?;

        Ok(saldo_akhir)
    }

    /// Ambil saldo poin pelanggan saat ini
    pub fn ambil_saldo(&self, pelanggan_id: &str) -> Result<i64> {
        self.conn.query_row(
            "SELECT poin_saldo FROM dpelanggan WHERE id = ?1;",
            params![pelanggan_id],
            |r| r.get(0),
        )
    }

    fn catat_riwayat(
        &self,
        cabang_id: &str,
        pelanggan_id: &str,
        faktur: Option<&str>,
        jenis: &str,
        jumlah: i64,
        saldo_akhir: i64,
        keterangan: &str,
    ) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            r#"
            INSERT INTO riwayat_poin (
                id, cabang_id, pelanggan_id, faktur, tanggal, jenis, jumlah,
                saldo_akhir, keterangan, sync_status, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'pending', CURRENT_TIMESTAMP);
            "#,
            params![id, cabang_id, pelanggan_id, faktur, now, jenis, jumlah, saldo_akhir, keterangan],
        )?;
        Ok(())
    }
}
