use fazpos::db::Database;
use fazpos::domain::barang::DBarang;
use fazpos::domain::cabang::{Cabang, Device};
use fazpos::domain::operator::DOperator;
use fazpos::repository::aplikasi_repo::AplikasiRepo;
use fazpos::repository::barang_repo::BarangRepo;
use fazpos::repository::buku_kas_repo::BukuKasRepo;
use fazpos::repository::cabang_repo::CabangRepo;
use fazpos::repository::operator_repo::OperatorRepo;
use fazpos::repository::retur_pembelian_repo::ReturPembelianRepo;
use fazpos::repository::shift_repo::ShiftRepo;
use fazpos::services::shift_service::ShiftService;
use rusqlite::params;

fn setup_test_db() -> (Database, String, String) {
    let db = Database::buka_in_memory().expect("Gagal buka in-memory db");
    let cabang = Cabang::baru("CAB-TEST-12", "Toko Sempurna", true);
    let device = Device::baru(&cabang.id, "DEV-TEST-12", "Terminal 1", "server", "MACH-12");

    {
        let repo = CabangRepo::new(db.conn());
        repo.simpan_cabang(&cabang).unwrap();
        repo.simpan_device(&device).unwrap();
    }

    let cid = cabang.id;
    let did = device.id;
    (db, cid, did)
}

#[test]
fn test_verifikasi_seluruh_32_tabel_skema_database() {
    let (db, _, _) = setup_test_db();
    let tabel_wajib = vec![
        "cabang", "device", "dbarang", "dpelanggan", "doperator",
        "tshift", "tpenjualan", "tpenjualandetail", "keluarmasuk", "perubahanharga",
        "treturpenjualan", "treturpenjualandetail", "pengaturan_poin", "riwayat_poin",
        "dsuplier", "tpembelian", "tpembeliandetail", "thutang", "tpiutang",
        "tstokopname", "tcashflow", "tbiaya", "tpenjualanpending", "tpenjualanpendingdetail",
        "dsales", "dkaryawan", "treturpembelian", "treturpembeliandetail",
        "tpembelianpending", "tpembelianpendingdetail", "daplikasi", "autokode",
    ];

    for t in tabel_wajib {
        let count: i64 = db.conn().query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1;",
            params![t],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 1, "Tabel wajib [{}] harus ada di database SQLite!", t);
    }
}

#[test]
fn test_auto_log_perubahan_harga() {
    let (db, cabang_id, _) = setup_test_db();
    let repo = BarangRepo::new(db.conn());

    // 1. Simpan barang harga awal (HPP: 10.000, Jual1: 15.000)
    let mut b = DBarang::baru(&cabang_id, "BRG-HARGA", "Kopi Bubuk 250g", 10000.0, 15000.0, 50.0);
    repo.simpan(&b).unwrap();

    // Pastikan belum ada log perubahan harga untuk barang ini
    let count_awal: i64 = db.conn().query_row(
        "SELECT COUNT(*) FROM perubahanharga WHERE cabang_id = ?1;",
        params![cabang_id],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(count_awal, 0);

    // 2. Ubah harga pokok dan harga jual (HPP: 12.000, Jual1: 18.000)
    b.hargapokok = 12000.0;
    b.hargajual1 = 18000.0;
    repo.simpan(&b).unwrap();

    // 3. Verifikasi otomatis tercatat di perubahanharga (Aturan 5 SKILL.MD)
    let (hpp_lama, hpp_baru, jual_lama, jual_baru): (f64, f64, f64, f64) = db.conn().query_row(
        "SELECT hargapokok_lama, hargapokok_baru, hargajual1_lama, hargajual1_baru FROM perubahanharga WHERE cabang_id = ?1 LIMIT 1;",
        params![cabang_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    ).unwrap();

    assert_eq!(hpp_lama, 10000.0);
    assert_eq!(hpp_baru, 12000.0);
    assert_eq!(jual_lama, 15000.0);
    assert_eq!(jual_baru, 18000.0);
}

#[test]
fn test_operator_auth_dan_password_hashing() {
    let (db, cabang_id, _) = setup_test_db();
    let repo = OperatorRepo::new(db.conn());

    // Buat operator dengan password hash aman (SHA-256)
    let hash = OperatorRepo::hash_password("pin1234");
    let op = DOperator::baru(&cabang_id, "KASIR01", "Siti Aminah", hash, "kasir");
    repo.simpan(&op).unwrap();

    // Verifikasi login dengan password benar
    let login_sukses = repo.verifikasi(&cabang_id, "KASIR01", "pin1234").unwrap();
    assert!(login_sukses.is_some());
    let op_login = login_sukses.unwrap();
    assert_eq!(op_login.nama, "Siti Aminah");
    assert_eq!(op_login.role, "kasir");

    // Verifikasi login dengan password salah
    let login_gagal = repo.verifikasi(&cabang_id, "KASIR01", "salah_pin").unwrap();
    assert!(login_gagal.is_none());
}

#[test]
fn test_retur_pembelian_atomic_dan_potong_hutang() {
    let (mut db, cabang_id, device_id) = setup_test_db();

    // Setup barang stok awal 30
    let b_repo = BarangRepo::new(db.conn());
    let b = DBarang::baru(&cabang_id, "BRG-RETUR-BELI", "Kecap Manis 600ml", 15000.0, 18000.0, 30.0);
    b_repo.simpan(&b).unwrap();
    let barang_id = b.id.clone();

    // Setup hutang awal ke supplier Rp 300.000
    db.conn().execute(
        r#"
        INSERT INTO thutang (
            id, cabang_id, faktur_beli, suplier_id, nama_suplier,
            tanggal, tagihan_awal, telah_dibayar, sisa, status, keterangan
        ) VALUES (
            'HTG-01', ?1, 'FB-TEST-001', 'SUP-01', 'PT Pangan Makmur',
            CURRENT_TIMESTAMP, 300000.0, 0.0, 300000.0, 'belum_lunas', 'Tempo 30 hari'
        );
        "#,
        params![cabang_id],
    ).unwrap();

    // Retur 10 botol kecap rusak ke supplier @ 15.000 = Rp 150.000
    let mut retur_repo = ReturPembelianRepo::new(db.conn_mut());
    let no_retur = retur_repo.proses_retur_pembelian_atomic(
        &cabang_id,
        &device_id,
        "OP01",
        "FB-TEST-001",
        Some("SUP-01"),
        "PT Pangan Makmur",
        &barang_id,
        "BRG-RETUR-BELI",
        "Kecap Manis 600ml",
        10.0,
        "BTL",
        15000.0,
        Some("Kemasan pecah saat pengiriman"),
    ).expect("Retur pembelian gagal");

    assert!(no_retur.starts_with("RB-"));

    // 1. Stok barang berkurang dari 30 menjadi 20
    let stok_akhir: f64 = db.conn().query_row(
        "SELECT stok FROM dbarang WHERE id = ?1;",
        params![barang_id],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(stok_akhir, 20.0);

    // 2. Log keluarmasuk tercatat dengan jenis RETUR_PEMBELIAN
    let (keluar, sisa_log): (f64, f64) = db.conn().query_row(
        "SELECT keluar, sisa FROM keluarmasuk WHERE referensi = ?1 AND jenis = 'RETUR_PEMBELIAN';",
        params![no_retur],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ).unwrap();
    assert_eq!(keluar, 10.0);
    assert_eq!(sisa_log, 20.0);

    // 3. Sisa hutang berkurang dari 300.000 menjadi 150.000
    let sisa_hutang: f64 = db.conn().query_row(
        "SELECT sisa FROM thutang WHERE cabang_id = ?1 AND faktur_beli = 'FB-TEST-001';",
        params![cabang_id],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(sisa_hutang, 150000.0);
}

#[test]
fn test_rekapitulasi_otomatis_tutup_shift_skema_12_4() {
    let (db, cabang_id, device_id) = setup_test_db();
    let shift_svc = ShiftService::new(&cabang_id, &device_id);
    let shift = shift_svc.buka_shift(&db, "OP01", 100_000.0).unwrap();

    // 1. Penjualan Tunai Bersih Rp 250.000
    db.conn().execute(
        r#"
        INSERT INTO tpenjualan (
            id, cabang_id, device_id, shift_id, faktur, tanggal,
            operator_id, subtotal, total_akhir, bayar_tunai, kembalian, metode_bayar, status
        ) VALUES (
            'PJ-01', ?1, ?2, ?3, 'PJ-001', CURRENT_TIMESTAMP,
            'OP01', 250000.0, 250000.0, 300000.0, 50000.0, 'TUNAI', 'selesai'
        );
        "#,
        params![cabang_id, device_id, shift.id],
    ).unwrap();

    // 2. Pemasukan Kas Masuk Lain Rp 50.000
    let kas_repo = BukuKasRepo::new(db.conn());
    kas_repo.catat_arus_kas_dengan_shift(&cabang_id, Some(&shift.id), "MASUK", "Pemasukan Lain", 50000.0, "Uang tips toko", "OP01").unwrap();

    // 3. Pengeluaran Kas Biaya Rp 30.000
    kas_repo.catat_arus_kas_dengan_shift(&cabang_id, Some(&shift.id), "KELUAR", "Listrik & Air", 30000.0, "Token listrik kasir", "OP01").unwrap();

    // 4. Hitung Rekapitulasi Otomatis sesuai §12.4
    // uang_seharusnya = modal_awal (100k) + penjualan_tunai (250k) + kas_masuk_lain (50k) - kas_keluar (30k) - retur (0) = 370.000
    let shift_repo = ShiftRepo::new(db.conn());
    let rekap = shift_repo.hitung_rekapitulasi_lengkap(&shift.id).unwrap();

    assert_eq!(rekap.modal_awal, 100000.0);
    assert_eq!(rekap.total_penjualan_tunai, 250000.0);
    assert_eq!(rekap.total_kas_masuk_lain, 50000.0);
    assert_eq!(rekap.total_kas_keluar, 30000.0);
    assert_eq!(rekap.uang_seharusnya, 370000.0);

    // 5. Kasir input fisik 370.000 -> pas (selisih 0)
    shift_repo.tutup_shift(&shift.id, 370000.0, rekap.uang_seharusnya, 0.0, Some("Shift pagi lancar")).unwrap();

    let (status, uang_seharusnya_db, selisih_db): (String, f64, f64) = db.conn().query_row(
        "SELECT status, uang_seharusnya, selisih FROM tshift WHERE id = ?1;",
        params![shift.id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    ).unwrap();

    assert_eq!(status, "closed");
    assert_eq!(uang_seharusnya_db, 370000.0);
    assert_eq!(selisih_db, 0.0);
}

#[test]
fn test_konfigurasi_daplikasi_cabang() {
    let (db, cabang_id, _) = setup_test_db();
    let repo = AplikasiRepo::new(db.conn());

    // 1. Ambil default
    let cfg = repo.ambil_konfigurasi(&cabang_id).unwrap();
    assert_eq!(cfg.nama_toko, "FAZPOS Store");
    assert_eq!(cfg.lebar_struk_mm, 58);

    // 2. Update konfigurasi toko
    let mut updated = cfg.clone();
    updated.nama_toko = "Toko Grosir Berkah Bersama".to_string();
    updated.alamat = Some("Jl. Sudirman No. 88, Bukittinggi".to_string());
    updated.header_struk = Some("Pusat Belanja Grosir & Eceran".to_string());
    updated.footer_struk = Some("Terima kasih atas kunjungan Anda".to_string());
    updated.lebar_struk_mm = 80;
    repo.simpan_konfigurasi(&updated).unwrap();

    // 3. Verifikasi persistence
    let cek = repo.ambil_konfigurasi(&cabang_id).unwrap();
    assert_eq!(cek.nama_toko, "Toko Grosir Berkah Bersama");
    assert_eq!(cek.alamat.as_deref(), Some("Jl. Sudirman No. 88, Bukittinggi"));
    assert_eq!(cek.lebar_struk_mm, 80);
}
