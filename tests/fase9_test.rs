use fazpos::backup::sqlite_backup::BackupManager;
use fazpos::db::Database;
use fazpos::domain::cabang::Cabang;
use fazpos::repository::cabang_repo::CabangRepo;
use std::fs;

#[test]
fn test_sqlite_backup_and_integrity_check() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join("pos_backup_test");
    if temp_dir.exists() {
        let _ = fs::remove_dir_all(&temp_dir);
    }
    fs::create_dir_all(&temp_dir)?;

    let db_path = temp_dir.join("pos_original.db");
    let db = Database::buka(&db_path)?;

    // 1. Verifikasi PRAGMA integrity check
    let is_sehat = BackupManager::cek_integritas(&db)?;
    assert!(is_sehat, "Database baru harus lolos integrity check");

    // 2. Isi data master
    let cabang = Cabang::baru("CB01", "Toko Backup", true);
    CabangRepo::new(db.conn()).simpan_cabang(&cabang)?;

    // 3. Buat online backup via VACUUM INTO
    let backup_dir = temp_dir.join("backups");
    let backup_file = BackupManager::buat_backup(&db, &backup_dir)?;
    assert!(backup_file.exists());

    // 4. Buka file backup dan pastikan data di dalamnya utuh
    let db_backup = Database::buka(&backup_file)?;
    let cabang_backup = CabangRepo::new(db_backup.conn()).ambil_cabang_pertama()?.unwrap();
    assert_eq!(cabang_backup.nama, "Toko Backup");

    // 5. Test rotasi backup
    let dihapus = BackupManager::rotasi_backup(&backup_dir, 1)?;
    assert_eq!(dihapus, 0); // baru ada 1 file, batas simpan 1 -> tidak ada yang dihapus

    // Cleanup
    drop(db);
    drop(db_backup);
    let _ = fs::remove_dir_all(&temp_dir);

    Ok(())
}
