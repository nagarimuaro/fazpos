use fazpos::db::Database;
use fazpos::domain::barang::DBarang;
use fazpos::domain::cabang::{Cabang, Device};
use fazpos::domain::transaksi::KeranjangItem;
use fazpos::printer::escpos::EscPosBuilder;
use fazpos::printer::struk::StrukKasir;
use fazpos::repository::barang_repo::BarangRepo;
use fazpos::repository::cabang_repo::CabangRepo;
use fazpos::repository::retur_repo::{ReturItem, ReturRepo};
use fazpos::repository::shift_repo::ShiftRepo;
use fazpos::services::kasir_service::KasirService;
use fazpos::services::shift_service::ShiftService;

#[test]
fn test_escpos_thermal_receipt_generation() {
    let mut builder = EscPosBuilder::new_58mm();
    builder
        .tebal(true)
        .baris("TOKO SENTOSA")
        .tebal(false)
        .garis_pemisah()
        .dua_kolom("Aqua 600ml", "Rp 3.500")
        .buka_laci_kasir()
        .potong_kertas();

    let bytes = builder.ambil_bytes();

    // Pastikan bytes mengandung komando init ESC @ (0x1B, 0x40)
    assert!(bytes.windows(2).any(|w| w == [0x1B, 0x40]));
    // Pastikan bytes mengandung drawer kick ESC p (0x1B, 0x70)
    assert!(bytes.windows(2).any(|w| w == [0x1B, 0x70]));
    // Pastikan bytes mengandung teks toko
    let text_repr = String::from_utf8_lossy(bytes);
    assert!(text_repr.contains("TOKO SENTOSA"));
    assert!(text_repr.contains("Aqua 600ml"));
    assert!(text_repr.contains("Rp 3.500"));
}

#[test]
fn test_alur_retur_penjualan_atomic_dan_update_stok_shift() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = Database::buka_in_memory()?;

    // 1. Setup Cabang & Device
    let cabang = Cabang::baru("CB01", "Toko Cabang", true);
    CabangRepo::new(db.conn()).simpan_cabang(&cabang)?;

    let device = Device::baru(&cabang.id, "DEV01", "Kasir 1", "server", "MACH-01");
    CabangRepo::new(db.conn()).simpan_device(&device)?;

    // 2. Setup Produk (Stok awal = 50)
    let b1_id: String;
    {
        let barang_repo = BarangRepo::new(db.conn());
        let mut b1 = DBarang::baru(&cabang.id, "BRG-AQUA", "Aqua 600ml", 2500.0, 3500.0, 50.0);
        b1.barcode = Some("899123".to_string());
        barang_repo.simpan(&b1)?;
        b1_id = b1.id;
    }

    // 3. Buka Shift Kasir (Modal awal 100.000)
    let shift_svc = ShiftService::new(&cabang.id, &device.id);
    let shift = shift_svc.buka_shift(&db, "OP01", 100_000.0)?;

    // 4. Kasir jual 5 pcs Aqua (5 x 3.500 = 17.500)
    let mut kasir = KasirService::new(&cabang.id, &device.id);
    kasir.keranjang.push(KeranjangItem::baru(
        &b1_id,
        "BRG-AQUA",
        "Aqua 600ml",
        "PCS",
        3500.0,
        2500.0,
        5.0,
    ));
    let penjualan = kasir.checkout(&mut db, "OP01", 20_000.0, 0.0, 0.0, "TUNAI")?;

    // Verifikasi stok berkurang jadi 45
    {
        let barang_repo = BarangRepo::new(db.conn());
        let aqua = barang_repo.cari_by_id(&b1_id)?.unwrap();
        assert_eq!(aqua.stok, 45.0);
    }

    // 5. Test Cetak Struk ESC/POS
    let details = vec![fazpos::domain::transaksi::TPenjualanDetail {
        id: "det-1".to_string(),
        penjualan_id: penjualan.id.clone(),
        cabang_id: cabang.id.clone(),
        barang_id: b1_id.clone(),
        kode_barang: "BRG-AQUA".to_string(),
        nama_barang: "Aqua 600ml".to_string(),
        jumlah: 5.0,
        satuan: "PCS".to_string(),
        hargajual: 3500.0,
        hargapokok: 2500.0,
        diskon_persen: 0.0,
        diskon_rp: 0.0,
        subtotal: 17500.0,
        sync_status: "pending".to_string(),
        created_at: None,
        updated_at: None,
    }];
    let struk_bytes = StrukKasir::buat_struk(&cabang, &penjualan, &details, true);
    assert!(!struk_bytes.is_empty());

    // 6. Pelanggan Retur 2 pcs Aqua (2 x 3.500 = 7.000)
    let retur_items = vec![ReturItem {
        barang_id: b1_id.clone(),
        kode_barang: "BRG-AQUA".to_string(),
        nama_barang: "Aqua 600ml".to_string(),
        jumlah: 2.0,
        satuan: "PCS".to_string(),
        hargajual: 3500.0,
        subtotal: 7000.0,
    }];

    let mut retur_repo = ReturRepo::new(db.conn_mut());
    let faktur_retur = retur_repo.simpan_retur_atomic(
        &cabang.id,
        &device.id,
        &shift.id,
        &penjualan.faktur,
        "OP01",
        &retur_items,
        0.0,
    )?;
    assert!(faktur_retur.starts_with("RJ-"));

    // 7. Verifikasi Stok Bertambah Kembali (45 + 2 = 47)
    {
        let barang_repo = BarangRepo::new(db.conn());
        let aqua = barang_repo.cari_by_id(&b1_id)?.unwrap();
        assert_eq!(aqua.stok, 47.0);
    }

    // 8. Verifikasi Log Otomatis keluarmasuk Tercatat untuk RETUR_PENJUALAN
    let retur_log_count: i64 = db.conn().query_row(
        "SELECT COUNT(*) FROM keluarmasuk WHERE jenis = 'RETUR_PENJUALAN'",
        [],
        |r| r.get(0),
    )?;
    assert_eq!(retur_log_count, 1);

    // 9. Verifikasi Shift Kasir Terupdate
    // Modal awal 100.000 + Penjualan tunai 17.500 - Retur 7.000 = Uang seharusnya 110.500
    let shift_repo = ShiftRepo::new(db.conn());
    let shift_sekarang = shift_repo.ambil_shift_aktif(&cabang.id, &device.id)?.unwrap();
    assert_eq!(shift_sekarang.total_retur, 7000.0);
    assert_eq!(shift_sekarang.uang_seharusnya, 110_500.0);

    Ok(())
}
