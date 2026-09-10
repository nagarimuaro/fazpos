use rusqlite::{params, Connection, Result};

#[derive(Debug, Clone)]
pub struct HutangItem {
    pub id: String,
    pub faktur: String,
    pub suplier_nama: String,
    pub tanggal: String,
    pub tagihan_awal: f64,
    pub telah_dibayar: f64,
    pub sisa: f64,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct PiutangItem {
    pub id: String,
    pub faktur: String,
    pub pelanggan_nama: String,
    pub tanggal: String,
    pub tagihan_awal: f64,
    pub telah_dibayar: f64,
    pub sisa: f64,
    pub status: String,
}

pub struct HutangPiutangRepo<'a> {
    conn: &'a Connection,
}

impl<'a> HutangPiutangRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn semua_hutang(&self, cabang_id: &str) -> Result<Vec<HutangItem>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, faktur_beli, nama_suplier, tanggal, tagihan_awal, telah_dibayar, sisa, status
            FROM thutang
            WHERE cabang_id = ?1
            ORDER BY tanggal DESC
            LIMIT 100;
            "#
        )?;
        let rows = stmt.query_map(params![cabang_id], |row| {
            Ok(HutangItem {
                id: row.get(0)?,
                faktur: row.get(1)?,
                suplier_nama: row.get(2)?,
                tanggal: row.get(3)?,
                tagihan_awal: row.get(4)?,
                telah_dibayar: row.get(5)?,
                sisa: row.get(6)?,
                status: row.get(7)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn bayar_hutang(&self, id: &str, jumlah: f64) -> Result<()> {
        let (sisa_awal, telah_dibayar): (f64, f64) = self.conn.query_row(
            "SELECT sisa, telah_dibayar FROM thutang WHERE id = ?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;

        let bayar = jumlah.min(sisa_awal);
        let sisa_baru = (sisa_awal - bayar).max(0.0);
        let telah_dibayar_baru = telah_dibayar + bayar;
        let status = if sisa_baru <= 0.0 { "lunas" } else { "belum_lunas" };

        self.conn.execute(
            r#"
            UPDATE thutang
            SET sisa = ?1, telah_dibayar = ?2, status = ?3, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?4;
            "#,
            params![sisa_baru, telah_dibayar_baru, status, id],
        )?;
        Ok(())
    }

    pub fn semua_piutang(&self, cabang_id: &str) -> Result<Vec<PiutangItem>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, faktur_jual, nama_pelanggan, tanggal, tagihan_awal, telah_dibayar, sisa, status
            FROM tpiutang
            WHERE cabang_id = ?1
            ORDER BY tanggal DESC
            LIMIT 100;
            "#
        )?;
        let rows = stmt.query_map(params![cabang_id], |row| {
            Ok(PiutangItem {
                id: row.get(0)?,
                faktur: row.get(1)?,
                pelanggan_nama: row.get(2)?,
                tanggal: row.get(3)?,
                tagihan_awal: row.get(4)?,
                telah_dibayar: row.get(5)?,
                sisa: row.get(6)?,
                status: row.get(7)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn bayar_piutang(&self, id: &str, jumlah: f64) -> Result<()> {
        let (sisa_awal, telah_dibayar): (f64, f64) = self.conn.query_row(
            "SELECT sisa, telah_dibayar FROM tpiutang WHERE id = ?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;

        let bayar = jumlah.min(sisa_awal);
        let sisa_baru = (sisa_awal - bayar).max(0.0);
        let telah_dibayar_baru = telah_dibayar + bayar;
        let status = if sisa_baru <= 0.0 { "lunas" } else { "belum_lunas" };

        self.conn.execute(
            r#"
            UPDATE tpiutang
            SET sisa = ?1, telah_dibayar = ?2, status = ?3, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?4;
            "#,
            params![sisa_baru, telah_dibayar_baru, status, id],
        )?;
        Ok(())
    }

    pub fn catat_piutang_baru(
        &self,
        cabang_id: &str,
        faktur_jual: &str,
        pelanggan_id: &str,
        nama_pelanggan: &str,
        tagihan_awal: f64,
        telah_dibayar: f64,
        jatuh_tempo: &str,
        keterangan: &str,
    ) -> Result<()> {
        let sisa = (tagihan_awal - telah_dibayar).max(0.0);
        let status = if sisa <= 0.0 { "lunas" } else { "belum_lunas" };
        let id = uuid::Uuid::new_v4().to_string();

        self.conn.execute(
            r#"
            INSERT INTO tpiutang (
                id, cabang_id, faktur_jual, pelanggan_id, nama_pelanggan,
                tanggal, jatuh_tempo, tagihan_awal, telah_dibayar, sisa,
                status, keterangan, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5,
                CURRENT_TIMESTAMP, ?6, ?7, ?8, ?9,
                ?10, ?11, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
            );
            "#,
            params![
                id, cabang_id, faktur_jual, pelanggan_id, nama_pelanggan,
                jatuh_tempo, tagihan_awal, telah_dibayar, sisa, status, keterangan
            ],
        )?;
        Ok(())
    }
}
