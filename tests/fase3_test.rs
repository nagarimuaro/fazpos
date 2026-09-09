use fazpos::db::Database;
use fazpos::domain::cabang::Cabang;
use fazpos::importer::import_service::ImportService;
use fazpos::importer::sql_parser::SqlDumpParser;
use fazpos::repository::barang_repo::BarangRepo;
use fazpos::repository::cabang_repo::CabangRepo;

#[test]
fn test_sql_lexer_insert_parser() {
    let sql = r#"
        -- Komentar SQL
        /* Blok komentar */
        INSERT INTO `dbarang` (`kode`, `nama`, `hargapokok`, `hargajual1`, `stok`) VALUES
        ('B001', 'Kopi (Mix, Spesial)', '1500.00', '2000.00', '50'),
        ('B002', 'Teh Celup \'Melati\'', '3000.00', '4500.00', '100');
    "#;

    let statements = SqlDumpParser::parse_dump(sql);
    assert_eq!(statements.len(), 1);

    let stmt = &statements[0];
    assert_eq!(stmt.tabel, "dbarang");
    assert_eq!(
        stmt.kolom.as_ref().unwrap(),
        &vec!["kode", "nama", "hargapokok", "hargajual1", "stok"]
    );
    assert_eq!(stmt.rows.len(), 2);

    // Verifikasi penanganan koma di dalam tanda petik
    assert_eq!(stmt.rows[0][0], "B001");
    assert_eq!(stmt.rows[0][1], "Kopi (Mix, Spesial)");
    assert_eq!(stmt.rows[0][2], "1500.00");

    // Verifikasi penanganan escape quote \'
    assert_eq!(stmt.rows[1][0], "B002");
    assert_eq!(stmt.rows[1][1], "Teh Celup 'Melati'");
}

#[test]
fn test_import_dbarang_legacy_ke_sqlite() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = Database::buka_in_memory()?;
    let cabang = Cabang::baru("CB01", "Cabang Import", true);
    CabangRepo::new(db.conn()).simpan_cabang(&cabang)?;

    // Contoh SQL dump iB Retago 5 aktual
    let dump_sql = r#"
        INSERT INTO `dbarang` (
            `kode`, `barcode`, `nama`, `satuan`, `kategori`, `rak`,
            `hargapokok`, `hargajual1`, `hargajual2`, `hargajual3`, `hargapartai`,
            `stok`, `stokminimum`, `is_aktif`
        ) VALUES
        ('BRG-01', '8991234567890', 'Biskuit Roma Kelapa 300g', 'BKS', 'MAKANAN', 'RAK-A1', '8500', '10000', '9500', '9200', '9000', '48', '10', 'true'),
        ('BRG-02', '8999999000111', 'Susu UHT Cokelat 1L', 'KOTAK', 'MINUMAN', 'RAK-B2', '16000', '19500', '19000', '18500', '18000', '24', '5', '1');
    "#;

    let report = ImportService::import_dari_teks(&mut db, &cabang.id, dump_sql)?;

    assert_eq!(report.total_barang_sukses, 2);
    assert_eq!(report.total_gagal, 0);

    // Verifikasi data masuk ke SQLite dan dapat dicari kasir
    let repo = BarangRepo::new(db.conn());
    let roma = repo.cari_by_barcode_atau_kode(&cabang.id, "8991234567890")?.unwrap();
    assert_eq!(roma.nama, "Biskuit Roma Kelapa 300g");
    assert_eq!(roma.hargapokok, 8500.0);
    assert_eq!(roma.hargajual1, 10000.0);
    assert_eq!(roma.hargajual2, 9500.0);
    assert_eq!(roma.hargapartai, 9000.0);
    assert_eq!(roma.stok, 48.0);
    assert_eq!(roma.satuan, "BKS");
    assert_eq!(roma.is_aktif, true);

    let susu = repo.cari_by_barcode_atau_kode(&cabang.id, "BRG-02")?.unwrap();
    assert_eq!(susu.nama, "Susu UHT Cokelat 1L");
    assert_eq!(susu.stok, 24.0);

    Ok(())
}

#[test]
fn test_import_pelanggan_dan_operator_legacy() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = Database::buka_in_memory()?;
    let cabang = Cabang::baru("CB01", "Cabang Toko", true);
    CabangRepo::new(db.conn()).simpan_cabang(&cabang)?;

    let dump_sql = r#"
        INSERT INTO `dpelanggan` (`kode`, `nama`, `alamat`, `telepon`, `poin_saldo`, `id_kartu`) VALUES
        ('PLG01', 'Budi Santoso', 'Jl. Merdeka No. 10', '08123456789', '150', 'MBR-BUDI-01'),
        ('PLG02', 'Siti Aminah', 'Jl. Sudirman No. 5', '08198765432', '50', 'MBR-SITI-02');

        INSERT INTO `doperator` (`kode`, `nama`, `password_hash`, `role`) VALUES
        ('ADMIN', 'Administrator Sistem', 'hash_rahasia', 'admin'),
        ('KASIR1', 'Kasir Depan', 'hash_kasir', 'kasir');
    "#;

    let report = ImportService::import_dari_teks(&mut db, &cabang.id, dump_sql)?;

    assert_eq!(report.total_pelanggan_sukses, 2);
    assert_eq!(report.total_operator_sukses, 2);
    assert_eq!(report.total_gagal, 0);

    // Cek query langsung ke SQLite
    let count_pelanggan: i64 = db.conn().query_row(
        "SELECT COUNT(*) FROM dpelanggan WHERE cabang_id = ?1",
        rusqlite::params![cabang.id],
        |r| r.get(0),
    )?;
    assert_eq!(count_pelanggan, 2);

    let count_operator: i64 = db.conn().query_row(
        "SELECT COUNT(*) FROM doperator WHERE cabang_id = ?1",
        rusqlite::params![cabang.id],
        |r| r.get(0),
    )?;
    assert_eq!(count_operator, 2);

    Ok(())
}

#[test]
fn test_import_resilient_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = Database::buka_in_memory()?;
    let cabang = Cabang::baru("CB01", "Cabang Toko", true);
    CabangRepo::new(db.conn()).simpan_cabang(&cabang)?;

    // SQL dump berisi 1 baris valid, 1 baris tidak valid (kode kosong)
    let dump_sql = r#"
        INSERT INTO `dbarang` (`kode`, `nama`, `hargapokok`, `hargajual1`, `stok`) VALUES
        ('BRG-OK', 'Barang Bagus', '5000', '7000', '10'),
        ('', 'Barang Tanpa Kode', '1000', '2000', '5');
    "#;

    let report = ImportService::import_dari_teks(&mut db, &cabang.id, dump_sql)?;

    // 1 baris berhasil, 1 baris gagal dicatat ke log error
    assert_eq!(report.total_barang_sukses, 1);
    assert_eq!(report.total_gagal, 1);
    assert_eq!(report.errors.len(), 1);
    assert!(report.errors[0].contains("Kode barang tidak boleh kosong"));

    // Pastikan barang yang valid tetap tersimpan
    let repo = BarangRepo::new(db.conn());
    assert!(repo.cari_by_barcode_atau_kode(&cabang.id, "BRG-OK")?.is_some());

    Ok(())
}
