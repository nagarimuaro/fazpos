use fazpos::db::Database;
use fazpos::domain::cabang::Cabang;
use fazpos::domain::pelanggan::DPelanggan;
use fazpos::domain::poin::PengaturanPoin;
use fazpos::repository::cabang_repo::CabangRepo;
use fazpos::repository::poin_repo::PoinRepo;

#[test]
fn test_perhitungan_poin_logic() {
    let mut config = PengaturanPoin::default("CB01");
    config.rupiah_per_poin = 10_000.0;
    config.nilai_tukar_poin = 100.0;
    config.minimal_tukar = 10;

    // Perolehan poin
    assert_eq!(config.hitung_perolehan(45_000.0), 4);
    assert_eq!(config.hitung_perolehan(9_999.0), 0);
    assert_eq!(config.hitung_perolehan(100_000.0), 10);

    // Penukaran poin sah
    let rupiah = config.hitung_nilai_tukar(50, 20).unwrap();
    assert_eq!(rupiah, 2_000.0); // 20 poin * Rp 100 = Rp 2.000

    // Gagal karena di bawah minimal tukar
    let err_min = config.hitung_nilai_tukar(50, 5);
    assert!(err_min.is_err());
    assert!(err_min.unwrap_err().contains("Minimal penukaran"));

    // Gagal karena saldo kurang
    let err_saldo = config.hitung_nilai_tukar(15, 20);
    assert!(err_saldo.is_err());
    assert!(err_saldo.unwrap_err().contains("Saldo poin tidak cukup"));
}

#[test]
fn test_alur_poin_member_sqlite() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::buka_in_memory()?;

    let cabang = Cabang::baru("CB01", "Cabang Toko", true);
    CabangRepo::new(db.conn()).simpan_cabang(&cabang)?;

    // Buat pelanggan member dengan saldo awal 50 poin
    let mut member = DPelanggan::baru(&cabang.id, "MBR001", "Pelanggan Setia");
    member.poin_saldo = 50;
    db.conn().execute(
        r#"
        INSERT INTO dpelanggan (id, cabang_id, kode, nama, poin_saldo)
        VALUES (?1, ?2, ?3, ?4, ?5);
        "#,
        rusqlite::params![member.id, member.cabang_id, member.kode, member.nama, member.poin_saldo],
    )?;

    let poin_repo = PoinRepo::new(db.conn());

    // 1. Ambil & cek pengaturan poin default
    let setting = poin_repo.ambil_pengaturan(&cabang.id)?;
    assert_eq!(setting.rupiah_per_poin, 10_000.0);

    // 2. Member menukarkan 20 poin saat transaksi PJ-001
    let saldo_setelah_tukar = poin_repo.kurangi_poin(
        &cabang.id,
        &member.id,
        "PJ-001",
        20,
        "Tukar poin transaksi PJ-001",
    )?;
    assert_eq!(saldo_setelah_tukar, 30);

    // 3. Member mendapatkan 5 poin baru dari transaksi tersebut
    let saldo_setelah_dapat = poin_repo.tambah_poin(
        &cabang.id,
        &member.id,
        "PJ-001",
        5,
        "Poin reward transaksi PJ-001",
    )?;
    assert_eq!(saldo_setelah_dapat, 35);

    // 4. Verifikasi saldo akhir di database
    let saldo_db = poin_repo.ambil_saldo(&member.id)?;
    assert_eq!(saldo_db, 35);

    // 5. Verifikasi riwayat poin tercatat 2 kali ('TUKAR' dan 'DAPAT')
    let count_riwayat: i64 = db.conn().query_row(
        "SELECT COUNT(*) FROM riwayat_poin WHERE pelanggan_id = ?1",
        rusqlite::params![member.id],
        |r| r.get(0),
    )?;
    assert_eq!(count_riwayat, 2);

    Ok(())
}
