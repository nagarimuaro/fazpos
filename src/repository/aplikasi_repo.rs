use rusqlite::{params, Connection, OptionalExtension, Result, Row};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DAplikasi {
    pub id: String,
    pub cabang_id: String,
    pub nama_toko: String,
    pub alamat: Option<String>,
    pub telepon: Option<String>,
    pub header_struk: Option<String>,
    pub footer_struk: Option<String>,
    pub printer_default: Option<String>,
    pub lebar_struk_mm: i32,
}

impl DAplikasi {
    pub fn default_cabang(cabang_id: impl Into<String>, nama_toko: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            cabang_id: cabang_id.into(),
            nama_toko: nama_toko.into(),
            alamat: None,
            telepon: None,
            header_struk: Some("Selamat Berbelanja".to_string()),
            footer_struk: Some("Barang yang sudah dibeli tidak dapat ditukar".to_string()),
            printer_default: None,
            lebar_struk_mm: 58,
        }
    }
}

pub struct AplikasiRepo<'a> {
    conn: &'a Connection,
}

impl<'a> AplikasiRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    fn map_row(row: &Row) -> Result<DAplikasi> {
        Ok(DAplikasi {
            id: row.get(0)?,
            cabang_id: row.get(1)?,
            nama_toko: row.get(2)?,
            alamat: row.get(3)?,
            telepon: row.get(4)?,
            header_struk: row.get(5)?,
            footer_struk: row.get(6)?,
            printer_default: row.get(7)?,
            lebar_struk_mm: row.get(8)?,
        })
    }

    /// Ambil konfigurasi aplikasi cabang, atau buat default jika belum ada
    pub fn ambil_konfigurasi(&self, cabang_id: &str) -> Result<DAplikasi> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cabang_id, nama_toko, alamat, telepon,
                   header_struk, footer_struk, printer_default, lebar_struk_mm
            FROM daplikasi
            WHERE cabang_id = ?1
            LIMIT 1;
            "#,
        )?;

        let opt = stmt.query_row(params![cabang_id], Self::map_row).optional()?;

        match opt {
            Some(app) => Ok(app),
            None => {
                let def = DAplikasi::default_cabang(cabang_id, "FAZPOS Store");
                self.simpan_konfigurasi(&def)?;
                Ok(def)
            }
        }
    }

    /// Simpan atau update konfigurasi aplikasi cabang
    pub fn simpan_konfigurasi(&self, app: &DAplikasi) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO daplikasi (
                id, cabang_id, nama_toko, alamat, telepon,
                header_struk, footer_struk, printer_default, lebar_struk_mm,
                sync_status, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5,
                ?6, ?7, ?8, ?9,
                'pending', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
            )
            ON CONFLICT(cabang_id) DO UPDATE SET
                nama_toko = excluded.nama_toko,
                alamat = excluded.alamat,
                telepon = excluded.telepon,
                header_struk = excluded.header_struk,
                footer_struk = excluded.footer_struk,
                printer_default = excluded.printer_default,
                lebar_struk_mm = excluded.lebar_struk_mm,
                sync_status = 'pending',
                updated_at = CURRENT_TIMESTAMP;
            "#,
            params![
                app.id,
                app.cabang_id,
                app.nama_toko,
                app.alamat,
                app.telepon,
                app.header_struk,
                app.footer_struk,
                app.printer_default,
                app.lebar_struk_mm,
            ],
        )?;
        Ok(())
    }
}
