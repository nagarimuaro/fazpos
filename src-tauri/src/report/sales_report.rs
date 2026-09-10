use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RingkasanOmzet {
    pub total_transaksi: i64,
    pub total_omzet: f64,
    pub total_diskon: f64,
    pub total_hpp: f64,
    pub laba_kotor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProdukTerlaris {
    pub barang_id: String,
    pub kode_barang: String,
    pub nama_barang: String,
    pub total_qty: f64,
    pub total_penjualan: f64,
}

pub struct SalesReporter<'a> {
    conn: &'a Connection,
}

impl<'a> SalesReporter<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Hitung ringkasan omzet dan laba kotor untuk cabang tertentu dalam rentang tanggal
    pub fn ringkasan_penjualan(
        &self,
        cabang_id: &str,
        tanggal_mulai: &str,
        tanggal_selesai: &str,
    ) -> Result<RingkasanOmzet> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT 
                COUNT(p.id) as total_trx,
                COALESCE(SUM(p.total_akhir), 0.0) as omzet,
                COALESCE(SUM(p.diskon_rp), 0.0) as diskon,
                COALESCE(SUM(d.hargapokok * d.jumlah), 0.0) as total_hpp
            FROM tpenjualan p
            LEFT JOIN tpenjualandetail d ON p.id = d.penjualan_id
            WHERE p.cabang_id = ?1 
              AND p.tanggal >= ?2 
              AND p.tanggal <= ?3
              AND p.status = 'selesai';
            "#,
        )?;

        stmt.query_row(params![cabang_id, tanggal_mulai, tanggal_selesai], |row| {
            let total_trx: i64 = row.get(0)?;
            let total_omzet: f64 = row.get(1)?;
            let total_diskon: f64 = row.get(2)?;
            let total_hpp: f64 = row.get(3)?;
            let laba_kotor = total_omzet - total_hpp;

            Ok(RingkasanOmzet {
                total_transaksi: total_trx,
                total_omzet,
                total_diskon,
                total_hpp,
                laba_kotor,
            })
        })
    }

    /// Dapatkan produk terlaris berdasarkan kuantitas penjualan
    pub fn produk_terlaris(
        &self,
        cabang_id: &str,
        limit: usize,
    ) -> Result<Vec<ProdukTerlaris>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT barang_id, kode_barang, nama_barang,
                   SUM(jumlah) as total_qty,
                   SUM(subtotal) as total_rev
            FROM tpenjualandetail
            WHERE cabang_id = ?1
            GROUP BY barang_id, kode_barang, nama_barang
            ORDER BY total_qty DESC
            LIMIT ?2;
            "#,
        )?;

        let rows = stmt.query_map(params![cabang_id, limit as i64], |row| {
            Ok(ProdukTerlaris {
                barang_id: row.get(0)?,
                kode_barang: row.get(1)?,
                nama_barang: row.get(2)?,
                total_qty: row.get(3)?,
                total_penjualan: row.get(4)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }
}
