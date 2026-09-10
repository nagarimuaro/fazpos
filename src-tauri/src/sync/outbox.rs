use crate::sync::payload::{CloudSyncAck, CloudSyncBatch, SyncQueueItem};
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use uuid::Uuid;

pub struct OutboxManager<'a> {
    conn: &'a mut Connection,
}

impl<'a> OutboxManager<'a> {
    pub fn new(conn: &'a mut Connection) -> Self {
        Self { conn }
    }

    /// Kumpulkan seluruh record yang masih berstatus `pending` untuk dikirim ke Cloud
    pub fn kumpulkan_batch_pending(
        &self,
        cabang_id: &str,
        device_id: &str,
        limit: usize,
    ) -> Result<CloudSyncBatch> {
        let mut items = Vec::new();

        // 1. Kumpulkan tpenjualan pending
        let mut stmt_jual = self.conn.prepare(
            r#"
            SELECT id, faktur, total_akhir, metode_bayar, tanggal
            FROM tpenjualan
            WHERE cabang_id = ?1 AND sync_status = 'pending'
            LIMIT ?2;
            "#,
        )?;

        let rows_jual = stmt_jual.query_map(params![cabang_id, limit as i64], |row| {
            let id: String = row.get(0)?;
            let faktur: String = row.get(1)?;
            let total_akhir: f64 = row.get(2)?;
            let metode_bayar: String = row.get(3)?;
            let tanggal: String = row.get(4)?;
            let json = format!(
                r#"{{"faktur":"{}","total_akhir":{},"metode_bayar":"{}","tanggal":"{}"}}"#,
                faktur, total_akhir, metode_bayar, tanggal
            );
            Ok(SyncQueueItem {
                tabel: "tpenjualan".to_string(),
                record_id: id,
                operasi: "INSERT".to_string(),
                payload_json: json,
            })
        })?;

        for r in rows_jual {
            items.push(r?);
        }

        // 2. Kumpulkan dbarang pending (update stok/harga)
        let sisa_limit = limit.saturating_sub(items.len());
        if sisa_limit > 0 {
            let mut stmt_brg = self.conn.prepare(
                r#"
                SELECT id, kode, nama, hargajual1, stok
                FROM dbarang
                WHERE cabang_id = ?1 AND sync_status = 'pending'
                LIMIT ?2;
                "#,
            )?;

            let rows_brg = stmt_brg.query_map(params![cabang_id, sisa_limit as i64], |row| {
                let id: String = row.get(0)?;
                let kode: String = row.get(1)?;
                let nama: String = row.get(2)?;
                let hargajual1: f64 = row.get(3)?;
                let stok: f64 = row.get(4)?;
                let json = format!(
                    r#"{{"kode":"{}","nama":"{}","hargajual1":{},"stok":{}}}"#,
                    kode, nama, hargajual1, stok
                );
                Ok(SyncQueueItem {
                    tabel: "dbarang".to_string(),
                    record_id: id,
                    operasi: "UPDATE".to_string(),
                    payload_json: json,
                })
            })?;

            for r in rows_brg {
                items.push(r?);
            }
        }

        Ok(CloudSyncBatch {
            cabang_id: cabang_id.to_string(),
            device_id: device_id.to_string(),
            batch_id: Uuid::new_v4().to_string(),
            items,
        })
    }

    /// Setelah cloud mengonfirmasi penerimaan (ACK), update status lokal jadi `synced`
    pub fn tandai_ter_sync(&mut self, ack: &CloudSyncAck) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let now = Utc::now().to_rfc3339();
        let mut total = 0;

        for id in &ack.synced_ids {
            let n1 = tx.execute(
                "UPDATE tpenjualan SET sync_status = 'synced', sync_at = ?1 WHERE id = ?2;",
                params![now, id],
            )?;
            let n2 = tx.execute(
                "UPDATE dbarang SET sync_status = 'synced', sync_at = ?1 WHERE id = ?2;",
                params![now, id],
            )?;
            let n3 = tx.execute(
                "UPDATE tshift SET sync_status = 'synced' WHERE id = ?1;",
                params![id],
            )?;
            total += n1 + n2 + n3;
        }

        tx.commit()?;
        Ok(total)
    }
}
