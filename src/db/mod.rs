pub mod migrations;

use rusqlite::{Connection, Result};
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Buka koneksi database SQLite di path tertentu dan jalankan migrasi
    pub fn buka<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        Self::konfigurasi_koneksi(&conn)?;
        migrations::jalankan_migrasi(&conn)?;
        Ok(Self { conn })
    }

    /// Buka koneksi database in-memory (sangat berguna untuk unit & integration test)
    pub fn buka_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::konfigurasi_koneksi(&conn)?;
        migrations::jalankan_migrasi(&conn)?;
        Ok(Self { conn })
    }

    /// Set pragmas: WAL mode, foreign keys, synchronous NORMAL, busy timeout
    fn konfigurasi_koneksi(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;
            PRAGMA synchronous = NORMAL;
            PRAGMA busy_timeout = 5000;
            "#,
        )?;
        Ok(())
    }

    /// Akses referensi ke rusqlite Connection
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Akses mutabel ke rusqlite Connection
    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}
