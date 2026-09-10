use fazpos::db::Database;
use fazpos::domain::barang::DBarang;
use fazpos::domain::cabang::{Cabang, Device};
use fazpos::domain::shift::TShift;
use fazpos::domain::transaksi::TPenjualan;
use fazpos::repository::barang_repo::BarangRepo;
use fazpos::repository::cabang_repo::CabangRepo;
use fazpos::repository::shift_repo::ShiftRepo;
use fazpos::services::kasir_service::KasirService;
use fazpos::services::shift_service::ShiftService;

#[test]
fn test_perhitungan_tier_harga_barang() {
    let mut b = DBarang::baru("CB01", "BRG01", "Barang Tes", 1000.0, 1500.0, 50.0);
    b.hargajual2 = 1400.0;
    b.hargajual3 = 1300.0;
    b.hargapartai = 1200.0;

    assert_eq!(b.ambil_harga_tier(1), 1500.0);
    assert_eq!(b.ambil_harga_tier(2), 1400.0);
    assert_eq!(b.ambil_harga_tier(3), 1300.0);
    assert_eq!(b.ambil_harga_tier(4), 1500.0); // fallback ke tier 1 jika tier 4 nol
    assert_eq!(b.ambil_harga_tier(5), 1200.0); // harga partai
}

#[test]
fn test_perhitungan_shift_kasir() {
    let modal_awal = 100_000.0;
    let total_tunai = 450_000.0;
    let total_retur = 20_000.0;
    let total_biaya = 30_000.0;

    // Uang seharusnya = 100.000 + 450.000 - 20.000 - 30.000 = 500.000
    let uang_seharusnya = TShift::hitung_uang_seharusnya(
        modal_awal,
        total_tunai,
        total_retur,
        total_biaya,
    );
    assert_eq!(uang_seharusnya, 500_000.0);

    // Kasir hitung fisik ada 495.000 -> tekor / selisih -5.000
    let selisih_tekor = TShift::hitung_selisih(495_000.0, uang_seharusnya);
    assert_eq!(selisih_tekor, -5_000.0);

    // Kasir hitung fisik ada 505.000 -> lebih / selisih +5.000
    let selisih_lebih = TShift::hitung_selisih(505_000.0, uang_seharusnya);
    assert_eq!(selisih_lebih, 5_000.0);
}

#[test]
fn test_perhitungan_ringkasan_transaksi() {
    let subtotal = 100_000.0;
    let diskon = 10_000.0;
    let nilai_poin = 5_000.0;
    let bayar_tunai = 100_000.0;

    let (total_akhir, kembalian) = TPenjualan::hitung_ringkasan(
        subtotal,
        diskon,
        nilai_poin,
        bayar_tunai,
        0.0,
    );

    assert_eq!(total_akhir, 85_000.0);
    assert_eq!(kembalian, 15_000.0);
}

#[test]
fn test_alur_lengkap_transaksi_kasir_dan_stok_sqlite() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Inisialisasi Database In-Memory
    let mut db = Database::buka_in_memory()?;

    // 2. Setup Cabang & Device
    let cabang_repo = CabangRepo::new(db.conn());
    let cabang = Cabang::baru("CB01", "Toko Utama", true);
    cabang_repo.simpan_cabang(&cabang)?;

    let device = Device::baru(&cabang.id, "DEV01", "Kasir 1", "server", "MACH-001");
    cabang_repo.simpan_device(&device)?;

    // 3. Tambah Master Barang (Aqua stok 50, Indomie stok 100)
    let b1_id: String;
    let b2_id: String;
    {
        let barang_repo = BarangRepo::new(db.conn());
        let mut b1 = DBarang::baru(&cabang.id, "BRG01", "Aqua 600ml", 2500.0, 3500.0, 50.0);
        b1.barcode = Some("899111".to_string());
        barang_repo.simpan(&b1)?;
        b1_id = b1.id;

        let mut b2 = DBarang::baru(&cabang.id, "BRG02", "Indomie", 2500.0, 3000.0, 100.0);
        b2.barcode = Some("899222".to_string());
        barang_repo.simpan(&b2)?;
        b2_id = b2.id;
    }

    // 4. Buka Shift Kasir
    let shift_svc = ShiftService::new(&cabang.id, &device.id);
    let shift = shift_svc.buka_shift(&db, "OP01", 100_000.0)?;
    assert_eq!(shift.modal_awal, 100_000.0);
    assert_eq!(shift.status, "open");

    // 5. Kasir scan barang
    let mut kasir = KasirService::new(&cabang.id, &device.id);
    kasir.scan_barcode(&db, "899111")?; // Aqua 1 pcs
    kasir.scan_barcode(&db, "899111")?; // Aqua jadi 2 pcs
    kasir.scan_barcode(&db, "899222")?; // Indomie 1 pcs

    // Total belanja: (2 * 3500) + (1 * 3000) = 10.000
    assert_eq!(kasir.total_belanja(), 10_000.0);

    // 6. Checkout Bayar Tunai Rp 20.000 -> Kembalian Rp 10.000
    let penjualan = kasir.checkout(&mut db, "OP01", 20_000.0, 0.0, 0.0, "TUNAI")?;
    assert_eq!(penjualan.total_akhir, 10_000.0);
    assert_eq!(penjualan.kembalian, 10_000.0);

    // Keranjang harus sudah bersih
    assert_eq!(kasir.keranjang.len(), 0);

    // 7. Verifikasi Stok Berkurang di Database
    {
        let barang_repo = BarangRepo::new(db.conn());
        let aqua_terbaru = barang_repo.cari_by_id(&b1_id)?.unwrap();
        assert_eq!(aqua_terbaru.stok, 48.0); // 50 - 2 = 48

        let indomie_terbaru = barang_repo.cari_by_id(&b2_id)?.unwrap();
        assert_eq!(indomie_terbaru.stok, 99.0); // 100 - 1 = 99
    }

    // 8. Verifikasi Log Otomatis keluarmasuk Tercatat (aturan 5 SKILL.MD)
    let count_log: i64 = db.conn().query_row(
        "SELECT COUNT(*) FROM keluarmasuk WHERE cabang_id = ?1",
        rusqlite::params![cabang.id],
        |r| r.get(0),
    )?;
    assert_eq!(count_log, 2); // 2 item dicatat keluar

    // 9. Verifikasi Shift Terupdate
    let shift_repo = ShiftRepo::new(db.conn());
    let shift_sekarang = shift_repo.ambil_shift_aktif(&cabang.id, &device.id)?.unwrap();
    assert_eq!(shift_sekarang.total_penjualan_tunai, 10_000.0);
    assert_eq!(shift_sekarang.uang_seharusnya, 110_000.0); // modal 100k + tunai 10k

    // 10. Tutup Shift Kasir
    let shift_tutup = shift_svc.tutup_shift(&db, &shift.id, 110_000.0, Some("Shift pagi lancar"))?;
    assert_eq!(shift_tutup.status, "closed");
    assert_eq!(shift_tutup.uang_aktual, Some(110_000.0));
    assert_eq!(shift_tutup.selisih, Some(0.0));

    Ok(())
}
