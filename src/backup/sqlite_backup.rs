use crate::db::Database;
use chrono::Utc;
use rusqlite::Result;
use std::fs;
use std::path::{Path, PathBuf};

pub struct BackupManager;

impl BackupManager {
    /// Buat backup database online SQLite secara atomik (menggunakan VACUUM INTO)
    /// Aman dijalankan bahkan saat transaksi kasir sedang berlangsung (mode WAL)
    pub fn buat_backup(db: &Database, folder_tujuan: &Path) -> Result<PathBuf, String> {
        if !folder_tujuan.exists() {
            fs::create_dir_all(folder_tujuan)
                .map_err(|e| format!("Gagal membuat direktori backup: {}", e))?;
        }

        let nama_file = format!(
            "pos_backup_{}.db",
            Utc::now().format("%Y%m%d_%H%M%S")
        );
        let path_file = folder_tujuan.join(&nama_file);
        let path_str = path_file.to_string_lossy().replace('\\', "/");

        let sql = format!("VACUUM INTO '{}';", path_str);
        db.conn()
            .execute(&sql, [])
            .map_err(|e| format!("VACUUM INTO gagal: {}", e))?;

        Ok(path_file)
    }

    /// Verifikasi integritas fisik database SQLite (PRAGMA integrity_check)
    pub fn cek_integritas(db: &Database) -> Result<bool, String> {
        let hasil: String = db
            .conn()
            .query_row("PRAGMA integrity_check;", [], |r| r.get(0))
            .map_err(|e| format!("Gagal menjalankan integrity check: {}", e))?;

        Ok(hasil.eq_ignore_ascii_case("ok"))
    }

    /// Rotasi file backup: pertahankan N backup terbaru, hapus sisanya agar hemat disk
    pub fn rotasi_backup(folder_backup: &Path, maksimal_simpan: usize) -> Result<usize, String> {
        if !folder_backup.exists() {
            return Ok(0);
        }

        let mut files = Vec::new();
        let entries = fs::read_dir(folder_backup)
            .map_err(|e| format!("Gagal membaca folder backup: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "db") {
                if let Ok(meta) = entry.metadata() {
                    if let Ok(modified) = meta.modified() {
                        files.push((path, modified));
                    }
                }
            }
        }

        // Urutkan dari yang paling baru ke paling lama
        files.sort_by(|a, b| b.1.cmp(&a.1));

        let mut dihapus = 0;
        if files.len() > maksimal_simpan {
            for (path_lama, _) in &files[maksimal_simpan..] {
                if fs::remove_file(path_lama).is_ok() {
                    dihapus += 1;
                }
            }
        }

        Ok(dihapus)
    }
}
