use crate::domain::log::KeluarMasuk;
use rusqlite::{params, Connection, Result};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ReturBeliItem {
    pub id: String,
    pub no_retur: String,
    pub faktur_beli: String,
    pub suplier_nama: String,
    pub tanggal: String,
    pub total: f64,
}

pub struct ReturPembelianRepo<'a> {
    conn: &'a mut Connection,
}

impl<'a> ReturPembelianRepo<'a> {
    pub fn new(conn: &'a mut Connection) -> Self {
        Self { conn }
    }

    /// Proses retur pembelian ke supplier secara atomik:
    /// 1. Insert header treturpembelian
    /// 2. Insert detail treturpembeliandetail
    /// 3. Kurangi stok barang di dbarang
    /// 4. Catat log keluarmasuk (jenis = 'RETUR_PEMBELIAN')
    /// 5. Potong sisa hutang di thutang jika faktur tersebut bertempo/belum lunas
    pub fn proses_retur_pembelian_atomic(
        &mut self,
        cabang_id: &str,
        device_id: &str,
        operator_id: &str,
        faktur_pembelian: &str,
        suplier_id: Option<&str>,
        nama_suplier: &str,
        barang_id: &str,
        kode_barang: &str,
        nama_barang: &str,
        jumlah_retur: f64,
        satuan: &str,
        hargabeli: f64,
        keterangan: Option<&str>,
    ) -> Result<String> {
        let tx = self.conn.transaction()?;

        let rand_suffix = &Uuid::new_v4().to_string()[..4].to_uppercase();
        let no_retur = format!("RB-{}-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"), rand_suffix);
        let retur_id = Uuid::new_v4().to_string();
        let total_subtotal = jumlah_retur * hargabeli;

        // 1. Insert Header treturpembelian
        tx.execute(
            r#"
            INSERT INTO treturpembelian (
                id, cabang_id, device_id, no_retur, faktur_pembelian,
                suplier_id, nama_suplier, tanggal, total_retur, keterangan,
                operator_id, sync_status, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5,
                ?6, ?7, CURRENT_TIMESTAMP, ?8, ?9,
                ?10, 'pending', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
            );
            "#,
            params![
                retur_id,
                cabang_id,
                device_id,
                no_retur,
                faktur_pembelian,
                suplier_id,
                nama_suplier,
                total_subtotal,
                keterangan,
                operator_id,
            ],
        )?;

        // 2. Insert Detail treturpembeliandetail
        let detail_id = Uuid::new_v4().to_string();
        tx.execute(
            r#"
            INSERT INTO treturpembeliandetail (
                id, retur_id, cabang_id, barang_id, kode_barang, nama_barang,
                jumlah, satuan, hargabeli, subtotal, sync_status, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6,
                ?7, ?8, ?9, ?10, 'pending', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
            );
            "#,
            params![
                detail_id,
                retur_id,
                cabang_id,
                barang_id,
                kode_barang,
                nama_barang,
                jumlah_retur,
                satuan,
                hargabeli,
                total_subtotal,
            ],
        )?;

        // 3. Kurangi stok barang karena dikembalikan ke supplier
        tx.execute(
            r#"
            UPDATE dbarang
            SET stok = stok - ?1,
                updated_at = CURRENT_TIMESTAMP,
                sync_status = 'pending'
            WHERE id = ?2;
            "#,
            params![jumlah_retur, barang_id],
        )?;

        let sisa_stok: f64 = tx.query_row(
            "SELECT stok FROM dbarang WHERE id = ?1",
            params![barang_id],
            |r| r.get(0),
        )?;

        // 4. Catat log otomatis keluarmasuk
        let log = KeluarMasuk::untuk_retur(
            cabang_id,
            device_id,
            barang_id,
            kode_barang,
            &no_retur,
            jumlah_retur,
            sisa_stok,
        );
        tx.execute(
            r#"
            INSERT INTO keluarmasuk (
                id, cabang_id, device_id, barang_id, kode_barang, tanggal,
                jenis, referensi, masuk, keluar, sisa, keterangan, sync_status, created_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6,
                'RETUR_PEMBELIAN', ?7, 0, ?8, ?9, 'Retur barang ke supplier', 'pending', CURRENT_TIMESTAMP
            );
            "#,
            params![
                log.id, log.cabang_id, log.device_id, log.barang_id, log.kode_barang,
                log.tanggal.to_rfc3339(), log.referensi, jumlah_retur, sisa_stok
            ],
        )?;

        // 5. Kurangi sisa hutang jika faktur beli ini ada di thutang
        let _ = tx.execute(
            r#"
            UPDATE thutang
            SET sisa = MAX(0.0, sisa - ?1),
                status = CASE WHEN (sisa - ?1) <= 0 THEN 'lunas' ELSE status END,
                updated_at = CURRENT_TIMESTAMP
            WHERE cabang_id = ?2 AND faktur_beli = ?3;
            "#,
            params![total_subtotal, cabang_id, faktur_pembelian],
        );

        tx.commit()?;
        Ok(no_retur)
    }

    pub fn semua_retur_pembelian(&self, cabang_id: &str) -> Result<Vec<ReturBeliItem>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, no_retur, faktur_pembelian, nama_suplier, tanggal, total_retur
            FROM treturpembelian
            WHERE cabang_id = ?1
            ORDER BY tanggal DESC
            LIMIT 100;
            "#
        )?;

        let rows = stmt.query_map(params![cabang_id], |row| {
            Ok(ReturBeliItem {
                id: row.get(0)?,
                no_retur: row.get(1)?,
                faktur_beli: row.get(2)?,
                suplier_nama: row.get(3)?,
                tanggal: row.get(4)?,
                total: row.get(5)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }
}
