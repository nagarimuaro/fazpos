use fazpos::db::Database;
use fazpos::domain::barang::DBarang;
use fazpos::domain::cabang::{Cabang, Device};
use fazpos::domain::transaksi::KeranjangItem;
use fazpos::repository::barang_repo::BarangRepo;
use fazpos::repository::cabang_repo::CabangRepo;
use fazpos::services::kasir_service::KasirService;
use fazpos::services::shift_service::ShiftService;
use fazpos::sync::outbox::OutboxManager;
use fazpos::sync::payload::CloudSyncAck;

#[test]
fn test_cloud_sync_outbox_pattern_dan_ack() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = Database::buka_in_memory()?;

    // 1. Setup Cabang, Device, Shift
    let cabang = Cabang::baru("CB01", "Cabang Sync", true);
    CabangRepo::new(db.conn()).simpan_cabang(&cabang)?;

    let device = Device::baru(&cabang.id, "DEV01", "Kasir Utama", "server", "M-SYNC");
    CabangRepo::new(db.conn()).simpan_device(&device)?;

    let shift_svc = ShiftService::new(&cabang.id, &device.id);
    let _shift = shift_svc.buka_shift(&db, "OP01", 100_000.0)?;

    // 2. Tambah Barang (status pending)
    let b1_id: String;
    {
        let repo = BarangRepo::new(db.conn());
        let mut b = DBarang::baru(&cabang.id, "BRG-SYNC", "Kecap Manis", 5000.0, 7500.0, 30.0);
        b.barcode = Some("899777".to_string());
        repo.simpan(&b)?;
        b1_id = b.id;
    }

    // 3. Checkout Kasir Transaksi Baru (status pending)
    let mut kasir = KasirService::new(&cabang.id, &device.id);
    kasir.keranjang.push(KeranjangItem::baru(
        &b1_id,
        "BRG-SYNC",
        "Kecap Manis",
        "BTL",
        7500.0,
        5000.0,
        2.0,
    ));
    let penjualan = kasir.checkout(&mut db, "OP01", 15_000.0, 0.0, 0.0, "TUNAI")?;

    // 4. Kumpulkan batch pending dari Outbox
    let batch = {
        let outbox = OutboxManager::new(db.conn_mut());
        outbox.kumpulkan_batch_pending(&cabang.id, &device.id, 50)?
    };

    assert_eq!(batch.cabang_id, cabang.id);
    // Harus ada minimal penjualan dan update barang
    assert!(batch.items.len() >= 2);
    assert!(batch.items.iter().any(|i| i.tabel == "tpenjualan" && i.record_id == penjualan.id));

    // 5. Simulasikan Cloud ACK balasan sukses terima data
    let ack = CloudSyncAck {
        batch_id: batch.batch_id.clone(),
        synced_ids: vec![penjualan.id.clone(), b1_id.clone()],
        status: "OK".to_string(),
        pesan: None,
    };

    // 6. Tandai sudah ter-sync secara lokal di SQLite
    let updated_count = {
        let mut outbox = OutboxManager::new(db.conn_mut());
        outbox.tandai_ter_sync(&ack)?
    };
    assert_eq!(updated_count, 2);

    // 7. Verifikasi outbox berikutnya sudah bersih (tidak mengulang kirim data lama)
    let batch_kedua = {
        let outbox = OutboxManager::new(db.conn_mut());
        outbox.kumpulkan_batch_pending(&cabang.id, &device.id, 50)?
    };
    assert!(!batch_kedua.items.iter().any(|i| i.record_id == penjualan.id));

    Ok(())
}
