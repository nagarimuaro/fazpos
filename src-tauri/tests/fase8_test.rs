use fazpos::db::Database;
use fazpos::domain::barang::DBarang;
use fazpos::domain::cabang::{Cabang, Device};
use fazpos::domain::transaksi::KeranjangItem;
use fazpos::report::sales_report::SalesReporter;
use fazpos::repository::barang_repo::BarangRepo;
use fazpos::repository::cabang_repo::CabangRepo;
use fazpos::services::kasir_service::KasirService;
use fazpos::services::shift_service::ShiftService;

#[test]
fn test_laporan_omzet_laba_kotor_dan_produk_terlaris() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = Database::buka_in_memory()?;

    // 1. Setup Cabang, Device, Shift
    let cabang = Cabang::baru("CB01", "Cabang Laporan", true);
    CabangRepo::new(db.conn()).simpan_cabang(&cabang)?;

    let device = Device::baru(&cabang.id, "DEV01", "Kasir 1", "server", "M-LAPORAN");
    CabangRepo::new(db.conn()).simpan_device(&device)?;

    let shift_svc = ShiftService::new(&cabang.id, &device.id);
    let _shift = shift_svc.buka_shift(&db, "OP01", 100_000.0)?;

    // 2. Setup Produk
    // B1: HPP 2000, Jual 3000 (Margin 1000/pcs)
    // B2: HPP 5000, Jual 8000 (Margin 3000/pcs)
    let b1_id: String;
    let b2_id: String;
    {
        let repo = BarangRepo::new(db.conn());
        let mut b1 = DBarang::baru(&cabang.id, "B01", "Pensil 2B", 2000.0, 3000.0, 100.0);
        b1.barcode = Some("899001".to_string());
        repo.simpan(&b1)?;
        b1_id = b1.id;

        let mut b2 = DBarang::baru(&cabang.id, "B02", "Buku Tulis", 5000.0, 8000.0, 50.0);
        b2.barcode = Some("899002".to_string());
        repo.simpan(&b2)?;
        b2_id = b2.id;
    }

    // 3. Transaksi 1: Jual 10 pcs Pensil (Omzet 30.000, HPP 20.000, Laba 10.000)
    let mut kasir = KasirService::new(&cabang.id, &device.id);
    kasir.keranjang.push(KeranjangItem::baru(&b1_id, "B01", "Pensil 2B", "PCS", 3000.0, 2000.0, 10.0));
    kasir.checkout(&mut db, "OP01", 30_000.0, 0.0, 0.0, "TUNAI")?;

    // 4. Transaksi 2: Jual 2 pcs Buku (Omzet 16.000, HPP 10.000, Laba 6.000)
    kasir.keranjang.push(KeranjangItem::baru(&b2_id, "B02", "Buku Tulis", "PCS", 8000.0, 5000.0, 2.0));
    kasir.checkout(&mut db, "OP01", 20_000.0, 0.0, 0.0, "TUNAI")?;

    // 5. Uji SalesReporter
    let reporter = SalesReporter::new(db.conn());
    let ringkasan = reporter.ringkasan_penjualan(&cabang.id, "2000-01-01", "2099-12-31")?;

    assert_eq!(ringkasan.total_transaksi, 2);
    assert_eq!(ringkasan.total_omzet, 46_000.0); // 30.000 + 16.000
    assert_eq!(ringkasan.total_hpp, 30_000.0);   // 20.000 + 10.000
    assert_eq!(ringkasan.laba_kotor, 16_000.0);  // 46.000 - 30.000 = 16.000

    // 6. Uji Produk Terlaris
    let terlaris = reporter.produk_terlaris(&cabang.id, 5)?;
    assert_eq!(terlaris.len(), 2);
    // Pensil 2B terjual 10 pcs (terlaris #1)
    assert_eq!(terlaris[0].kode_barang, "B01");
    assert_eq!(terlaris[0].total_qty, 10.0);
    // Buku Tulis terjual 2 pcs (#2)
    assert_eq!(terlaris[1].kode_barang, "B02");
    assert_eq!(terlaris[1].total_qty, 2.0);

    Ok(())
}
