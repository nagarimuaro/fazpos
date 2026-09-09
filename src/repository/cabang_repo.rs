use crate::domain::cabang::{Cabang, Device};
use rusqlite::{params, Connection, OptionalExtension, Result};

pub struct CabangRepo<'a> {
    conn: &'a Connection,
}

impl<'a> CabangRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn simpan_cabang(&self, c: &Cabang) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO cabang (
                id, kode, nama, alamat, telepon, is_pusat, sync_status, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            ON CONFLICT(kode) DO UPDATE SET
                nama = excluded.nama,
                alamat = excluded.alamat,
                telepon = excluded.telepon,
                is_pusat = excluded.is_pusat,
                updated_at = CURRENT_TIMESTAMP;
            "#,
            params![
                c.id, c.kode, c.nama, c.alamat, c.telepon,
                if c.is_pusat { 1 } else { 0 }, c.sync_status
            ],
        )?;
        Ok(())
    }

    pub fn ambil_cabang_pertama(&self) -> Result<Option<Cabang>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, kode, nama, alamat, telepon, is_pusat, sync_status, created_at, updated_at
            FROM cabang
            LIMIT 1;
            "#,
        )?;

        stmt.query_row([], |row| {
            let is_pusat_int: i32 = row.get(5)?;
            Ok(Cabang {
                id: row.get(0)?,
                kode: row.get(1)?,
                nama: row.get(2)?,
                alamat: row.get(3)?,
                telepon: row.get(4)?,
                is_pusat: is_pusat_int == 1,
                sync_status: row.get(6)?,
                created_at: None,
                updated_at: None,
            })
        })
        .optional()
    }

    pub fn simpan_device(&self, d: &Device) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO device (
                id, cabang_id, kode, nama, role, machine_id, ip_address, is_active, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            ON CONFLICT(cabang_id, kode) DO UPDATE SET
                nama = excluded.nama,
                role = excluded.role,
                machine_id = excluded.machine_id,
                ip_address = excluded.ip_address,
                is_active = excluded.is_active,
                updated_at = CURRENT_TIMESTAMP;
            "#,
            params![
                d.id, d.cabang_id, d.kode, d.nama, d.role, d.machine_id,
                d.ip_address, if d.is_active { 1 } else { 0 }
            ],
        )?;
        Ok(())
    }

    pub fn ambil_device_pertama(&self, cabang_id: &str) -> Result<Option<Device>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cabang_id, kode, nama, role, machine_id, ip_address, is_active, created_at, updated_at
            FROM device
            WHERE cabang_id = ?1
            LIMIT 1;
            "#,
        )?;

        stmt.query_row(params![cabang_id], |row| {
            let is_active_int: i32 = row.get(7)?;
            Ok(Device {
                id: row.get(0)?,
                cabang_id: row.get(1)?,
                kode: row.get(2)?,
                nama: row.get(3)?,
                role: row.get(4)?,
                machine_id: row.get(5)?,
                ip_address: row.get(6)?,
                is_active: is_active_int == 1,
                created_at: None,
                updated_at: None,
            })
        })
        .optional()
    }
}
