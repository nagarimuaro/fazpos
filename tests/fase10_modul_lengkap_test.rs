use fazpos::db::Database;
use fazpos::domain::barang::DBarang;
use fazpos::repository::barang_repo::BarangRepo;
use fazpos::repository::buku_kas_repo::BukuKasRepo;
use fazpos::repository::hutang_piutang_repo::HutangPiutangRepo;
use fazpos::repository::opname_repo::OpnameRepo;
use fazpos::repository::pembelian_repo::{PembelianItem, PembelianRepo};
use fazpos::repository::supplier_repo::{DSupplier, SupplierRepo};

#[test]
fn test_supplier_crud() {
    let db = Database::buka_in_memory().unwrap();
    let repo = SupplierRepo::new(db.conn());

    let sup = DSupplier {
        id: "SUP01".into(),
        cabang_id: "CB01".into(),
        kode: "VND01".into(),
        nama: "PT Sumber Berkah".into(),
        alamat: Some("Jl. Merdeka 10".into()),
        telepon: Some("08123456789".into()),
        rekening: Some("BCA 1234567890".into()),
        keterangan: None,
    };
    repo.simpan(&sup).unwrap();

    let list = repo.semua("CB01").unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].nama, "PT Sumber Berkah");

    repo.hapus("SUP01").unwrap();
    let list_after = repo.semua("CB01").unwrap();
    assert_eq!(list_after.len(), 0);
}

#[test]
fn test_pembelian_dan_auto_stok_dan_hutang() {
    let mut db = Database::buka_in_memory().unwrap();

    // 1. Setup barang awal (stok = 10)
    let brg = DBarang::baru("CB01", "BRG01", "Minyak Goreng", 14000.0, 16000.0, 10.0);
    BarangRepo::new(db.conn()).simpan(&brg).unwrap();

    // 2. Transaksi pembelian (beli 20 qty tempo, bayar muka 100k, total 280k, sisa 180k)
    let item = PembelianItem {
        barang_id: brg.id.clone(),
        kode_barang: "BRG01".into(),
        nama_barang: "Minyak Goreng".into(),
        jumlah: 20.0,
        satuan: "Pouch".into(),
        harga_beli: 14000.0,
        subtotal: 280_000.0,
    };

    let faktur = PembelianRepo::new(db.conn_mut())
        .simpan_pembelian_atomic(
            "CB01",
            "SUP01",
            "PT Distributor Minyak",
            None,
            "OP01",
            &[item],
            0.0,
            100_000.0,
            "KREDIT",
        )
        .unwrap();

    assert!(faktur.starts_with("PB-"));

    // 3. Stok barang bertambah 10 + 20 = 30
    let brg_cek = BarangRepo::new(db.conn()).cari_by_id(&brg.id).unwrap().unwrap();
    assert_eq!(brg_cek.stok, 30.0);

    // 4. Hutang tercatat sisa 180.000
    let hp_repo = HutangPiutangRepo::new(db.conn());
    let list_h = hp_repo.semua_hutang("CB01").unwrap();
    assert_eq!(list_h.len(), 1);
    assert_eq!(list_h[0].sisa, 180_000.0);
    assert_eq!(list_h[0].status, "belum_lunas");

    // 5. Bayar cicilan hutang 100.000 -> sisa 80.000
    hp_repo.bayar_hutang(&list_h[0].id, 100_000.0).unwrap();
    let list_h2 = hp_repo.semua_hutang("CB01").unwrap();
    assert_eq!(list_h2[0].sisa, 80_000.0);

    // 6. Lunasi sisa 80.000 -> status lunas
    hp_repo.bayar_hutang(&list_h[0].id, 80_000.0).unwrap();
    let list_h3 = hp_repo.semua_hutang("CB01").unwrap();
    assert_eq!(list_h3[0].sisa, 0.0);
    assert_eq!(list_h3[0].status, "lunas");
}

#[test]
fn test_stok_opname_dan_penyesuaian_stok() {
    let mut db = Database::buka_in_memory().unwrap();

    // 1. Setup barang sistem stok = 50
    let brg = DBarang::baru("CB01", "BRG02", "Beras 5kg", 60000.0, 70000.0, 50.0);
    BarangRepo::new(db.conn()).simpan(&brg).unwrap();

    // 2. Opname fisik ternyata ada 48 (selisih -2 karena rusak)
    {
        let mut op_repo = OpnameRepo::new(db.conn_mut());
        op_repo
            .simpan_opname(
                "CB01",
                &brg.id,
                &brg.kode,
                &brg.nama,
                48.0,
                "Kemasan Rusak Digigit Hama",
                "OP01",
            )
            .unwrap();
    }

    // 3. Stok barang di dbarang harus ter-update menjadi 48
    let brg_cek = BarangRepo::new(db.conn()).cari_by_id(&brg.id).unwrap().unwrap();
    assert_eq!(brg_cek.stok, 48.0);

    // 4. Riwayat opname tercatat
    let riwayat = OpnameRepo::new(db.conn_mut()).riwayat_opname("CB01").unwrap();
    assert_eq!(riwayat.len(), 1);
    assert_eq!(riwayat[0].selisih, -2.0);
}

#[test]
fn test_buku_kas_dan_beban_operasional() {
    let db = Database::buka_in_memory().unwrap();
    let repo = BukuKasRepo::new(db.conn());

    // 1. Catat kas masuk modal tambahan Rp 500.000
    repo.catat_arus_kas("CB01", "MASUK", "Modal Tambahan", 500_000.0, "Setoran Kas", "OP01").unwrap();

    // 2. Catat biaya listrik Rp 150.000 (KELUAR)
    repo.catat_arus_kas("CB01", "KELUAR", "Listrik & Token", 150_000.0, "Token PLN", "OP01").unwrap();

    // 3. Rekap kas
    let rekap = repo.rekap_kas("CB01").unwrap();
    assert_eq!(rekap.total_masuk, 500_000.0);
    assert_eq!(rekap.total_keluar, 150_000.0);
    assert_eq!(rekap.saldo_kas, 350_000.0);

    // 4. Riwayat kas ada 2 item
    let riwayat = repo.riwayat_kas("CB01").unwrap();
    assert_eq!(riwayat.len(), 2);
}
