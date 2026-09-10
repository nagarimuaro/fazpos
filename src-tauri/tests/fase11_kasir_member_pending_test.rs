use fazpos::db::Database;
use fazpos::domain::barang::DBarang;
use fazpos::domain::cabang::{Cabang, Device};
use fazpos::domain::pelanggan::DPelanggan;
use fazpos::domain::poin::PengaturanPoin;
use fazpos::repository::barang_repo::BarangRepo;
use fazpos::repository::cabang_repo::CabangRepo;
use fazpos::repository::hutang_piutang_repo::HutangPiutangRepo;
use fazpos::repository::pelanggan_repo::PelangganRepo;
use fazpos::repository::pending_repo::PendingRepo;
use fazpos::repository::poin_repo::PoinRepo;
use fazpos::services::kasir_service::KasirService;
use fazpos::services::shift_service::ShiftService;

fn setup_test_db() -> (Database, String, String) {
    let db = Database::buka_in_memory().expect("Gagal buka in-memory db");
    let cabang = Cabang::baru("CAB-TEST-01", "Toko Uji Coba", true);
    let device = Device::baru(&cabang.id, "DEV-TEST-01", "Kasir 1", "server", "MACH-TEST-01");

    {
        let repo = CabangRepo::new(db.conn());
        repo.simpan_cabang(&cabang).unwrap();
        repo.simpan_device(&device).unwrap();
    }

    // Buka shift kasir aktif
    let shift_svc = ShiftService::new(&cabang.id, &device.id);
    shift_svc.buka_shift(&db, "OP01", 100_000.0).unwrap();

    let cid = cabang.id;
    let did = device.id;
    (db, cid, did)
}

#[test]
fn test_kasir_hold_dan_recall_antrean() {
    let (db, cabang_id, device_id) = setup_test_db();

    // Setup 2 barang
    {
        let b_repo = BarangRepo::new(db.conn());
        let b1 = DBarang::baru(&cabang_id, "BRG-01", "Gula Pasir 1kg", 12000.0, 15000.0, 50.0);
        let b2 = DBarang::baru(&cabang_id, "BRG-02", "Minyak Goreng 2L", 28000.0, 32000.0, 30.0);
        b_repo.simpan(&b1).unwrap();
        b_repo.simpan(&b2).unwrap();
    }

    let mut kasir = KasirService::new(&cabang_id, &device_id);
    kasir.scan_barcode(&db, "BRG-01").unwrap();
    kasir.scan_barcode(&db, "BRG-02").unwrap();

    assert_eq!(kasir.keranjang.len(), 2);
    assert_eq!(kasir.total_belanja(), 47000.0);

    // 1. TAHAN ANTREAN (HOLD)
    let faktur_hold = kasir
        .hold_transaksi(&db, "OP01", Some("Pelanggan lupa dompet".to_string()))
        .expect("Hold transaksi gagal");

    assert!(faktur_hold.starts_with("HLD-"));
    // Keranjang harus kembali bersih
    assert_eq!(kasir.keranjang.len(), 0);
    assert_eq!(kasir.total_belanja(), 0.0);

    // Cek di tabel tpenjualanpending
    let pending_id: String;
    {
        let pending_repo = PendingRepo::new(db.conn());
        let pending_list = pending_repo.semua_pending(&cabang_id, &device_id).unwrap();
        assert_eq!(pending_list.len(), 1);
        assert_eq!(pending_list[0].faktur, faktur_hold);
        assert_eq!(pending_list[0].total_akhir, 47000.0);

        pending_id = pending_list[0].id.clone();
        let detail_list = pending_repo.ambil_detail(&pending_id).unwrap();
        assert_eq!(detail_list.len(), 2);
    }

    // 2. PANGGIL KEMBALI ANTREAN (RECALL)
    kasir
        .recall_transaksi(&db, &pending_id)
        .expect("Recall transaksi gagal");

    // Keranjang harus kembali berisi 2 item dengan total yang sama
    assert_eq!(kasir.keranjang.len(), 2);
    assert_eq!(kasir.total_belanja(), 47000.0);

    // Tabel tpenjualanpending harus sudah bersih (terhapus setelah di-recall)
    {
        let pending_repo = PendingRepo::new(db.conn());
        let sisa_pending = pending_repo.semua_pending(&cabang_id, &device_id).unwrap();
        assert_eq!(sisa_pending.len(), 0);
    }
}

#[test]
fn test_kasir_loyalty_member_tukar_poin_dan_dapat_poin() {
    let (mut db, cabang_id, device_id) = setup_test_db();

    // 1. Setup Master Barang
    {
        let b_repo = BarangRepo::new(db.conn());
        let b1 = DBarang::baru(&cabang_id, "BRG-BERAS", "Beras Premium 5kg", 60000.0, 75000.0, 20.0);
        b_repo.simpan(&b1).unwrap();
    }

    // 2. Setup Member dengan Saldo Poin Awal 300
    let member_id: String;
    {
        let pel_repo = PelangganRepo::new(db.conn());
        let mut member = DPelanggan::baru(&cabang_id, "MBR-001", "Budi Santoso");
        member.poin_saldo = 300;
        member.id_kartu = Some("KARTU-BUDI-123".to_string());
        pel_repo.simpan(&member).unwrap();
        member_id = member.id.clone();
    }

    // 3. Setup Pengaturan Poin: 1 poin didapat per Rp 10.000 belanja, 1 poin bernilai Rp 100 potongan, minimal tukar 50
    {
        let poin_repo = PoinRepo::new(db.conn());
        let setting = PengaturanPoin {
            id: uuid::Uuid::new_v4().to_string(),
            cabang_id: cabang_id.clone(),
            rupiah_per_poin: 10000.0,
            nilai_tukar_poin: 100.0,
            minimal_tukar: 50,
            is_aktif: true,
        };
        poin_repo.simpan_pengaturan(&setting).unwrap();
    }

    // 4. Kasir Scan Barang & Pasang Member
    let mut kasir = KasirService::new(&cabang_id, &device_id);
    kasir.scan_barcode(&db, "BRG-BERAS").unwrap(); // Belanja 75.000

    {
        let pel_repo = PelangganRepo::new(db.conn());
        let cari_member = pel_repo.cari_member(&cabang_id, "KARTU-BUDI-123").unwrap();
        assert_eq!(cari_member.len(), 1);
        kasir.attach_member(cari_member[0].clone());
    }

    assert!(kasir.member_terpilih.is_some());

    // 5. Member Tukar 100 Poin (Nilai tukar = 100 x Rp 100 = Rp 10.000)
    let nilai_tukar = kasir.tukar_poin(&db, 100).expect("Tukar poin gagal");
    assert_eq!(nilai_tukar, 10000.0);
    assert_eq!(kasir.nilai_tukar_poin, 10000.0);

    // Total belanja 75.000 - 10.000 (poin) = 65.000
    // Kasir bayar tunai 70.000 (kembalian 5.000)
    let pj = kasir
        .checkout(&mut db, "OP01", 70000.0, 0.0, 0.0, "TUNAI")
        .expect("Checkout kasir gagal");

    assert_eq!(pj.subtotal, 75000.0);
    assert_eq!(pj.nilai_tukar_poin, 10000.0);
    assert_eq!(pj.total_akhir, 65000.0);
    assert_eq!(pj.kembalian, 5000.0);
    assert_eq!(pj.poin_ditukar, 100);

    // Poin didapat dari belanja 65.000 / 10.000 = 6 poin
    assert_eq!(pj.poin_didapat, 6);

    // Verifikasi saldo member akhir di database:
    // Saldo awal: 300 - 100 (tukar) + 6 (dapat) = 206 poin
    {
        let poin_repo = PoinRepo::new(db.conn());
        let saldo_terbaru = poin_repo.ambil_saldo(&member_id).unwrap();
        assert_eq!(saldo_terbaru, 206);
    }
}

#[test]
fn test_kasir_kredit_tercatat_di_piutang() {
    let (mut db, cabang_id, device_id) = setup_test_db();

    {
        let b_repo = BarangRepo::new(db.conn());
        let b1 = DBarang::baru(&cabang_id, "BRG-SEMEN", "Semen Tiga Roda", 55000.0, 65000.0, 100.0);
        b_repo.simpan(&b1).unwrap();
    }

    let member: DPelanggan;
    {
        let pel_repo = PelangganRepo::new(db.conn());
        member = DPelanggan::baru(&cabang_id, "MBR-KREDIT", "Toko Bangunan Sejahtera");
        pel_repo.simpan(&member).unwrap();
    }

    let mut kasir = KasirService::new(&cabang_id, &device_id);
    kasir.scan_barcode(&db, "BRG-SEMEN").unwrap();
    kasir.attach_member(member);

    // Checkout KREDIT dengan DP tunai 15.000 (total belanja 65.000, sisa piutang 50.000)
    let pj = kasir
        .checkout(&mut db, "OP01", 15000.0, 0.0, 0.0, "KREDIT")
        .expect("Checkout kredit gagal");

    assert_eq!(pj.total_akhir, 65000.0);
    assert_eq!(pj.metode_bayar, "KREDIT");

    // Verifikasi otomatis tercatat di tpiutang
    {
        let hp_repo = HutangPiutangRepo::new(db.conn());
        let piutang_list = hp_repo.semua_piutang(&cabang_id).unwrap();
        assert_eq!(piutang_list.len(), 1);
        assert_eq!(piutang_list[0].faktur, pj.faktur);
        assert_eq!(piutang_list[0].tagihan_awal, 65000.0);
        assert_eq!(piutang_list[0].telah_dibayar, 15000.0);
        assert_eq!(piutang_list[0].sisa, 50000.0);
        assert_eq!(piutang_list[0].status, "belum_lunas");
    }
}
