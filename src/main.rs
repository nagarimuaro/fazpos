slint::include_modules!();

use fazpos::backup::sqlite_backup::BackupManager;
use fazpos::db::Database;
use fazpos::domain::barang::DBarang;
use fazpos::domain::cabang::{Cabang, Device};
use fazpos::domain::transaksi::{TPenjualan, TPenjualanDetail};
use fazpos::importer::import_service::ImportService;
use fazpos::license::machine_id::MachineId;
use fazpos::license::verification::{LicenseStatus, LicenseVerifier};
use fazpos::printer::struk::StrukKasir;
use fazpos::report::sales_report::SalesReporter;
use fazpos::repository::barang_repo::BarangRepo;
use fazpos::repository::buku_kas_repo::BukuKasRepo;
use fazpos::repository::cabang_repo::CabangRepo;
use fazpos::repository::hutang_piutang_repo::HutangPiutangRepo;
use fazpos::repository::opname_repo::OpnameRepo;
use fazpos::repository::pelanggan_repo::PelangganRepo;
use fazpos::repository::pembelian_repo::{PembelianItem, PembelianRepo};
use fazpos::repository::pending_repo::PendingRepo;
use fazpos::repository::retur_repo::{ReturItem, ReturRepo};
use fazpos::repository::shift_repo::ShiftRepo;
use fazpos::repository::supplier_repo::{DSupplier, SupplierRepo};
use fazpos::services::kasir_service::KasirService;
use fazpos::services::shift_service::ShiftService;
use rusqlite::OptionalExtension;
use slint::{ModelRc, VecModel};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Inisialisasi Database SQLite Lokal
    let db_path = "pos_data.db";
    let db = Database::buka(db_path)?;
    let db_ref = Rc::new(RefCell::new(db));

    // 2. Setup Default Cabang & Device jika belum ada
    let (cabang_id, device_id) = inisialisasi_data_dasar(&db_ref.borrow())?;

    // 3. Setup Default Shift Kasir jika belum ada yang open
    let shift_svc = ShiftService::new(&cabang_id, &device_id);
    let shift_aktif = match shift_svc.ambil_shift_aktif(&db_ref.borrow())? {
        Some(s) => s,
        None => shift_svc.buka_shift(&db_ref.borrow(), "OP01", 100_000.0)?,
    };

    // 4. Inisialisasi Kasir Service
    let kasir_svc = Rc::new(RefCell::new(KasirService::new(cabang_id.clone(), device_id.clone())));

    // 5. Inisialisasi UI Slint (Ukuran Default Layar Penuh Ter-maksimalkan / Maximized)
    let main_window = MainWindow::new()?;
    main_window.window().set_maximized(true);
    main_window.set_is_maximized(true);
    main_window.set_toko_nama("Nama Ritel Grosir".into());
    main_window.set_operator_nama("Operator : Master".into());
    main_window.set_shift_status(format!("SHIFT: {}", &shift_aktif.id[..8]).into());

    // Inisialisasi Lisensi
    let cur_machine_id = MachineId::dapatkan();
    main_window.set_machine_id(cur_machine_id.clone().into());
    let lic_status = LicenseVerifier::baca_dari_file(LicenseVerifier::path_lisensi_default(), &cur_machine_id);
    match lic_status {
        LicenseStatus::Aktif(payload) => {
            main_window.set_license_status_text("TERAKTIVASI (LIFETIME)".into());
            main_window.set_license_toko(payload.nama_toko.into());
        }
        _ => {
            main_window.set_license_status_text("BELUM AKTIVASI".into());
        }
    }

    // Inisialisasi data antrean tertahan (pending) & katalog barang di kasir
    muat_tabel_pending(&main_window, &db_ref.borrow(), &cabang_id, &device_id);
    muat_tabel_barang(&main_window, &db_ref.borrow(), &cabang_id, "");

    let window_handle = main_window.as_weak();

    // ==========================================
    // CALLBACK: Navigasi Menu Ribbon (Instant 0ms Switching via Cache)
    // ==========================================
    let loaded_views = Rc::new(RefCell::new([false; 13]));
    loaded_views.borrow_mut()[0] = true;
    let loaded_views_nav = Rc::clone(&loaded_views);
    let db_nav = Rc::clone(&db_ref);
    let win_handle_nav = window_handle.clone();
    let cid_nav = cabang_id.clone();
    let did_nav = device_id.clone();
    main_window.on_navigasi(move |view_idx| {
        let idx = view_idx as usize;
        let mut loaded = loaded_views_nav.borrow_mut();
        if idx < loaded.len() && loaded[idx] {
            return;
        }
        if idx < loaded.len() {
            loaded[idx] = true;
        }
        let mut database = db_nav.borrow_mut();
        if let Some(win) = win_handle_nav.upgrade() {
            match view_idx {
                0 => muat_tabel_pending(&win, &database, &cid_nav, &did_nav),
                1 => muat_tabel_riwayat(&win, &database, &cid_nav),
                2 => muat_tabel_retur(&win, &database, &cid_nav),
                3 => muat_tabel_pembelian(&win, &database, &cid_nav),
                4 => muat_tabel_barang(&win, &database, &cid_nav, ""),
                5 => muat_tabel_pelanggan(&win, &database, &cid_nav),
                6 => muat_tabel_supplier(&win, &database, &cid_nav),
                7 => muat_tabel_opname(&win, &mut database, &cid_nav),
                8 => muat_laporan_omzet(&win, &database, &cid_nav),
                9 => muat_tabel_hutang_piutang(&win, &database, &cid_nav),
                10 => muat_tabel_buku_kas(&win, &database, &cid_nav),
                11 => muat_data_shift(&win, &database, &cid_nav, &did_nav),
                _ => {}
            }
        }
    });

    // ==========================================
    // CALLBACK: KASIR (Scan Barcode, Bayar, Batal)
    // ==========================================
    let kasir_clone = Rc::clone(&kasir_svc);
    let db_clone = Rc::clone(&db_ref);
    let win_handle_barcode = window_handle.clone();
    main_window.on_cari_barcode(move |kode| {
        let mut svc = kasir_clone.borrow_mut();
        let database = db_clone.borrow();
        match svc.scan_barcode(&database, kode.as_str()) {
            Ok(Some(_)) => {
                if let Some(win) = win_handle_barcode.upgrade() {
                    sinkronkan_tabel_kasir(&win, &svc);
                    win.set_status_pesan("Barang berhasil ditambahkan.".into());
                }
            }
            Ok(None) => {
                if let Some(win) = win_handle_barcode.upgrade() {
                    win.set_status_pesan(format!("Barang [{}] tidak ditemukan!", kode).into());
                }
            }
            Err(err) => {
                if let Some(win) = win_handle_barcode.upgrade() {
                    win.set_status_pesan(format!("Error: {}", err).into());
                }
            }
        }
    });

    let kasir_checkout = Rc::clone(&kasir_svc);
    let db_checkout = Rc::clone(&db_ref);
    let win_handle_checkout = window_handle.clone();
    let cid_checkout = cabang_id.clone();
    let did_checkout = device_id.clone();
    main_window.on_proses_bayar(move |tunai| {
        let mut svc = kasir_checkout.borrow_mut();
        let mut database = db_checkout.borrow_mut();
        let total = svc.total_belanja();

        if total <= 0.0 {
            if let Some(win) = win_handle_checkout.upgrade() {
                win.set_status_pesan("Keranjang masih kosong!".into());
            }
            return;
        }

        let metode = if let Some(win) = win_handle_checkout.upgrade() {
            win.get_kasir_metode_bayar().to_string()
        } else {
            "TUNAI".to_string()
        };

        let tunai_f64 = tunai as f64;
        let total_setelah_potongan = (total - svc.nilai_tukar_poin).max(0.0);
        let kembalian = if tunai_f64 >= total_setelah_potongan { tunai_f64 - total_setelah_potongan } else { 0.0 };

        match svc.checkout(&mut database, "OP01", tunai_f64, 0.0, 0.0, &metode) {
            Ok(penjualan) => {
                if let Some(win) = win_handle_checkout.upgrade() {
                    sinkronkan_tabel_kasir(&win, &svc);
                    win.set_kembalian(kembalian as f32);
                    win.set_status_pesan(format!("Transaksi [{}] ({}) BERHASIL disimpan!", penjualan.faktur, metode).into());
                    muat_data_shift(&win, &database, &cid_checkout, &did_checkout);

                    // Tampilkan Popup Modal Interaktif Sukses Transaksi (Kembalian & Cetak Struk)
                    win.set_sukses_faktur(penjualan.faktur.clone().into());
                    win.set_sukses_total(penjualan.total_akhir as f32);
                    win.set_sukses_bayar(penjualan.bayar_tunai as f32);
                    win.set_sukses_kembalian(kembalian as f32);
                    win.set_show_modal_sukses(true);
                }
            }
            Err(err) => {
                if let Some(win) = win_handle_checkout.upgrade() {
                    win.set_status_pesan(format!("Gagal checkout: {}", err).into());
                }
            }
        }
    });

    // ==========================================
    // CALLBACK: WINDOW CONTROLS (MINIMIZE, MAXIMIZE, CLOSE)
    // ==========================================
    let win_handle_min = window_handle.clone();
    main_window.on_minimize_window(move || {
        if let Some(win) = win_handle_min.upgrade() {
            let _ = win.window().set_minimized(true);
        }
    });

    let win_handle_max = window_handle.clone();
    main_window.on_toggle_maximize_window(move || {
        if let Some(win) = win_handle_max.upgrade() {
            let is_max = win.window().is_maximized();
            win.window().set_maximized(!is_max);
            win.set_is_maximized(!is_max);
        }
    });

    let win_handle_close = window_handle.clone();
    main_window.on_close_window(move || {
        if let Some(win) = win_handle_close.upgrade() {
            let _ = win.hide();
        }
    });

    let win_handle_fs = window_handle.clone();
    main_window.on_toggle_fullscreen(move || {
        if let Some(win) = win_handle_fs.upgrade() {
            let is_max = win.window().is_maximized();
            win.window().set_maximized(!is_max);
            win.set_is_maximized(!is_max);
        }
    });

    let db_struk_last = Rc::clone(&db_ref);
    let win_handle_struk_last = window_handle.clone();
    let cid_struk_last = cabang_id.clone();
    main_window.on_cetak_struk_terakhir(move || {
        if let Some(win) = win_handle_struk_last.upgrade() {
            let faktur = win.get_sukses_faktur().to_string();
            if !faktur.is_empty() {
                cetak_struk_ke_escpos(&db_struk_last.borrow(), &cid_struk_last, &faktur);
                win.set_status_pesan(format!("Struk [{}] dicetak ke thermal.", faktur).into());
            }
        }
    });

    let kasir_reset = Rc::clone(&kasir_svc);
    let win_handle_reset = window_handle.clone();
    main_window.on_reset_transaksi(move || {
        let mut svc = kasir_reset.borrow_mut();
        svc.bersihkan_keranjang();
        if let Some(win) = win_handle_reset.upgrade() {
            sinkronkan_tabel_kasir(&win, &svc);
            win.set_status_pesan("Keranjang dibersihkan.".into());
        }
    });

    // ==========================================
    // CALLBACK: MEMBER & LOYALTY POIN KASIR
    // ==========================================
    let kasir_member = Rc::clone(&kasir_svc);
    let db_member = Rc::clone(&db_ref);
    let win_handle_member = window_handle.clone();
    let cid_member = cabang_id.clone();
    main_window.on_cari_member_kasir(move |q| {
        let mut svc = kasir_member.borrow_mut();
        let database = db_member.borrow();
        let repo = PelangganRepo::new(database.conn());
        if let Some(win) = win_handle_member.upgrade() {
            match repo.cari_member(&cid_member, q.as_str()) {
                Ok(list) if !list.is_empty() => {
                    let member = list[0].clone();
                    win.set_status_pesan(format!("Member [{}] terpasang.", member.nama).into());
                    svc.attach_member(member);
                    sinkronkan_tabel_kasir(&win, &svc);
                }
                _ => {
                    win.set_status_pesan("Member tidak ditemukan.".into());
                }
            }
        }
    });

    let kasir_lepas = Rc::clone(&kasir_svc);
    let win_handle_lepas = window_handle.clone();
    main_window.on_lepas_member_kasir(move || {
        let mut svc = kasir_lepas.borrow_mut();
        svc.detach_member();
        if let Some(win) = win_handle_lepas.upgrade() {
            sinkronkan_tabel_kasir(&win, &svc);
            win.set_status_pesan("Member dilepas dari transaksi.".into());
        }
    });

    let kasir_tukar = Rc::clone(&kasir_svc);
    let db_tukar = Rc::clone(&db_ref);
    let win_handle_tukar = window_handle.clone();
    main_window.on_tukar_poin_kasir(move |poin| {
        let mut svc = kasir_tukar.borrow_mut();
        let database = db_tukar.borrow();
        if let Some(win) = win_handle_tukar.upgrade() {
            match svc.tukar_poin(&database, poin as i64) {
                Ok(potongan) => {
                    sinkronkan_tabel_kasir(&win, &svc);
                    win.set_status_pesan(format!("Tukar poin berhasil! Potongan: Rp {}", potongan).into());
                }
                Err(e) => {
                    win.set_status_pesan(format!("Gagal tukar poin: {}", e).into());
                }
            }
        }
    });

    let kasir_batal_tukar = Rc::clone(&kasir_svc);
    let db_batal_tukar = Rc::clone(&db_ref);
    let win_handle_batal_tukar = window_handle.clone();
    main_window.on_batal_tukar_poin_kasir(move || {
        let mut svc = kasir_batal_tukar.borrow_mut();
        let database = db_batal_tukar.borrow();
        let _ = svc.tukar_poin(&database, 0);
        if let Some(win) = win_handle_batal_tukar.upgrade() {
            sinkronkan_tabel_kasir(&win, &svc);
            win.set_status_pesan("Penukaran poin dibatalkan.".into());
        }
    });

    // ==========================================
    // CALLBACK: PENDING ANTREAN (HOLD & RECALL)
    // ==========================================
    let kasir_hold = Rc::clone(&kasir_svc);
    let db_hold = Rc::clone(&db_ref);
    let win_handle_hold = window_handle.clone();
    let cid_hold = cabang_id.clone();
    let did_hold = device_id.clone();
    main_window.on_hold_transaksi_kasir(move |ket| {
        let mut svc = kasir_hold.borrow_mut();
        let database = db_hold.borrow();
        if let Some(win) = win_handle_hold.upgrade() {
            let ket_s = ket.to_string();
            let ket_opt = if ket_s.trim().is_empty() { None } else { Some(ket_s) };
            match svc.hold_transaksi(&database, "OP01", ket_opt) {
                Ok(faktur) => {
                    sinkronkan_tabel_kasir(&win, &svc);
                    muat_tabel_pending(&win, &database, &cid_hold, &did_hold);
                    win.set_status_pesan(format!("Antrean berhasil ditahan [{}]", faktur).into());
                }
                Err(e) => {
                    win.set_status_pesan(format!("Gagal tahan antrean: {}", e).into());
                }
            }
        }
    });

    let kasir_recall = Rc::clone(&kasir_svc);
    let db_recall = Rc::clone(&db_ref);
    let win_handle_recall = window_handle.clone();
    let cid_recall = cabang_id.clone();
    let did_recall = device_id.clone();
    main_window.on_recall_transaksi_kasir(move |pid| {
        let mut svc = kasir_recall.borrow_mut();
        let database = db_recall.borrow();
        if let Some(win) = win_handle_recall.upgrade() {
            match svc.recall_transaksi(&database, pid.as_str()) {
                Ok(_) => {
                    sinkronkan_tabel_kasir(&win, &svc);
                    muat_tabel_pending(&win, &database, &cid_recall, &did_recall);
                    win.set_status_pesan("Antrean berhasil dipanggil kembali ke keranjang!".into());
                }
                Err(e) => {
                    win.set_status_pesan(format!("Gagal panggil antrean: {}", e).into());
                }
            }
        }
    });

    let db_del_pending = Rc::clone(&db_ref);
    let win_handle_del_pending = window_handle.clone();
    let cid_del_pending = cabang_id.clone();
    let did_del_pending = device_id.clone();
    main_window.on_hapus_pending_kasir(move |pid| {
        let database = db_del_pending.borrow();
        let repo = PendingRepo::new(database.conn());
        if let Some(win) = win_handle_del_pending.upgrade() {
            let _ = repo.hapus_pending(pid.as_str());
            muat_tabel_pending(&win, &database, &cid_del_pending, &did_del_pending);
            win.set_status_pesan("Antrean tertahan berhasil dihapus.".into());
        }
    });

    // ==========================================
    // CALLBACK: MASTER BARANG
    // ==========================================
    let db_brg_cari = Rc::clone(&db_ref);
    let win_handle_brg_cari = window_handle.clone();
    let cid_brg_cari = cabang_id.clone();
    main_window.on_cari_barang(move |kw| {
        let database = db_brg_cari.borrow();
        if let Some(win) = win_handle_brg_cari.upgrade() {
            muat_tabel_barang(&win, &database, &cid_brg_cari, kw.as_str());
        }
    });

    let db_brg_simpan = Rc::clone(&db_ref);
    let win_handle_brg_simpan = window_handle.clone();
    let cid_brg_simpan = cabang_id.clone();
    main_window.on_simpan_barang_baru(move |kode, nama, barcode, kategori, satuan, hpp, jual, stok| {
        let kode = kode.to_string();
        let nama = nama.to_string();
        let barcode = barcode.to_string();
        let kategori = kategori.to_string();
        let satuan_s = satuan.to_string();

        if kode.trim().is_empty() || nama.trim().is_empty() {
            if let Some(win) = win_handle_brg_simpan.upgrade() {
                win.set_status_pesan("Kode dan Nama barang wajib diisi!".into());
            }
            return;
        }

        let database = db_brg_simpan.borrow();
        let repo = BarangRepo::new(database.conn());
        let mut baru = DBarang::baru(&cid_brg_simpan, &kode, &nama, hpp as f64, jual as f64, stok as f64);
        if !barcode.trim().is_empty() {
            baru.barcode = Some(barcode);
        }
        if !kategori.trim().is_empty() {
            baru.kategori = Some(kategori);
        }
        if !satuan_s.trim().is_empty() {
            baru.satuan = satuan_s;
        }

        if let Some(win) = win_handle_brg_simpan.upgrade() {
            match repo.simpan(&baru) {
                Ok(_) => {
                    win.set_status_pesan(format!("Barang [{}] berhasil disimpan!", nama).into());
                    muat_tabel_barang(&win, &database, &cid_brg_simpan, "");
                }
                Err(e) => {
                    win.set_status_pesan(format!("Gagal simpan barang: {}", e).into());
                }
            }
        }
    });

    // ==========================================
    // CALLBACK: SHIFT KASIR
    // ==========================================
    let db_shift = Rc::clone(&db_ref);
    let win_handle_shift = window_handle.clone();
    let cid_shift = cabang_id.clone();
    let did_shift = device_id.clone();
    main_window.on_tutup_shift_kasir(move || {
        if let Some(win) = win_handle_shift.upgrade() {
            let database = db_shift.borrow();
            let svc = ShiftService::new(&cid_shift, &did_shift);
            if let Ok(Some(s)) = svc.ambil_shift_aktif(&database) {
                let cat = win.get_shift_catatan().to_string();
                let cat_opt = if cat.trim().is_empty() { None } else { Some(cat.as_str()) };
                match svc.tutup_shift(&database, &s.id, s.uang_seharusnya, cat_opt) {
                    Ok(_) => {
                        win.set_shift_status("SHIFT CLOSED".into());
                        win.set_status_pesan("Shift kasir berhasil ditutup!".into());
                        muat_data_shift(&win, &database, &cid_shift, &did_shift);
                    }
                    Err(e) => {
                        win.set_status_pesan(format!("Gagal tutup shift: {}", e).into());
                    }
                }
            }
        }
    });

    // ==========================================
    // CALLBACK: IMPORT SQL DUMP LEGACY
    // ==========================================
    let db_import = Rc::clone(&db_ref);
    let win_handle_import = window_handle.clone();
    let cid_import = cabang_id.clone();
    main_window.on_jalankan_import_sql(move |sql_input| {
        let sql = sql_input.to_string();
        if sql.trim().is_empty() {
            if let Some(win) = win_handle_import.upgrade() {
                win.set_import_summary("Silakan tempel teks query INSERT SQL!".into());
            }
            return;
        }

        let mut database = db_import.borrow_mut();
        match ImportService::import_dari_teks(&mut database, &cid_import, &sql) {
            Ok(report) => {
                let summary = format!(
                    "Hasil Import:\n• Produk Sukses   : {}\n• Pelanggan Sukses: {}\n• Operator Sukses : {}\n• Baris Gagal     : {}",
                    report.total_barang_sukses,
                    report.total_pelanggan_sukses,
                    report.total_operator_sukses,
                    report.total_gagal
                );
                if let Some(win) = win_handle_import.upgrade() {
                    win.set_import_summary(summary.into());
                    win.set_status_pesan("Import SQL iB Retago selesai!".into());
                }
            }
            Err(e) => {
                if let Some(win) = win_handle_import.upgrade() {
                    win.set_import_summary(format!("Import gagal: {}", e).into());
                }
            }
        }
    });

    // ==========================================
    // CALLBACK: AKTIVASI LISENSI
    // ==========================================
    let win_handle_lic = window_handle.clone();
    main_window.on_aktivasi_lisensi(move |token_input| {
        let token = token_input.to_string();
        let cur_id = MachineId::dapatkan();

        match LicenseVerifier::verifikasi(&token, &cur_id) {
            Ok(payload) => {
                let _ = LicenseVerifier::simpan_ke_file(LicenseVerifier::path_lisensi_default(), &token, &cur_id);
                if let Some(win) = win_handle_lic.upgrade() {
                    win.set_license_status_text("TERAKTIVASI (LIFETIME)".into());
                    win.set_license_toko(payload.nama_toko.into());
                    win.set_status_pesan("FAZPOS berhasil teraktivasi permanen!".into());
                }
            }
            Err(e) => {
                if let Some(win) = win_handle_lic.upgrade() {
                    win.set_status_pesan(format!("Aktivasi gagal: {}", e).into());
                }
            }
        }
    });

    // ==========================================
    // CALLBACK: BACKUP DATABASE
    // ==========================================
    let db_backup = Rc::clone(&db_ref);
    let win_handle_backup = window_handle.clone();
    main_window.on_buat_backup_db(move || {
        if let Some(win) = win_handle_backup.upgrade() {
            let database = db_backup.borrow();
            let folder = Path::new("backups");
            match BackupManager::buat_backup(&database, folder) {
                Ok(path) => {
                    win.set_status_pesan(format!("Backup berhasil dibuat: {:?}", path).into());
                }
                Err(e) => {
                    win.set_status_pesan(format!("Gagal backup: {}", e).into());
                }
            }
        }
    });

    // ==========================================
    // CALLBACK: UBAH QTY KASIR
    // ==========================================
    let kasir_ubah_qty = Rc::clone(&kasir_svc);
    let win_handle_ubah_qty = window_handle.clone();
    main_window.on_ubah_qty_kasir(move |idx, new_qty| {
        let mut svc = kasir_ubah_qty.borrow_mut();
        let i = idx as usize;
        if i < svc.keranjang.len() {
            svc.keranjang[i].jumlah = new_qty as f64;
            svc.keranjang[i].subtotal = svc.keranjang[i].jumlah * svc.keranjang[i].hargajual;
            if let Some(win) = win_handle_ubah_qty.upgrade() {
                sinkronkan_tabel_kasir(&win, &svc);
            }
        }
    });

    // ==========================================
    // CALLBACK: HAPUS ITEM KASIR
    // ==========================================
    let kasir_hapus = Rc::clone(&kasir_svc);
    let win_handle_hapus_item = window_handle.clone();
    main_window.on_hapus_item_kasir(move |idx| {
        let mut svc = kasir_hapus.borrow_mut();
        let i = idx as usize;
        if i < svc.keranjang.len() {
            svc.keranjang.remove(i);
            if let Some(win) = win_handle_hapus_item.upgrade() {
                sinkronkan_tabel_kasir(&win, &svc);
            }
        }
    });

    // ==========================================
    // CALLBACK: HAPUS BARANG
    // ==========================================
    let db_hapus_brg = Rc::clone(&db_ref);
    let win_handle_hapus_brg = window_handle.clone();
    let cid_hapus_brg = cabang_id.clone();
    main_window.on_hapus_barang(move |id| {
        let database = db_hapus_brg.borrow();
        let id_str = id.to_string();
        match database.conn().execute("DELETE FROM dbarang WHERE id = ?1", [&id_str]) {
            Ok(_) => {
                if let Some(win) = win_handle_hapus_brg.upgrade() {
                    win.set_status_pesan(format!("Barang [{}] dihapus.", id_str).into());
                    muat_tabel_barang(&win, &database, &cid_hapus_brg, "");
                }
            }
            Err(e) => {
                if let Some(win) = win_handle_hapus_brg.upgrade() {
                    win.set_status_pesan(format!("Gagal hapus: {}", e).into());
                }
            }
        }
    });

    // ==========================================
    // CALLBACK: CETAK STRUK RIWAYAT
    // ==========================================
    let db_cetak = Rc::clone(&db_ref);
    let win_handle_cetak = window_handle.clone();
    let cid_cetak = cabang_id.clone();
    main_window.on_cetak_struk_riwayat(move |faktur| {
        let database = db_cetak.borrow();
        if cetak_struk_ke_escpos(&database, &cid_cetak, faktur.as_str()) {
            if let Some(win) = win_handle_cetak.upgrade() {
                win.set_status_pesan(format!("Struk [{}] berhasil dicetak ulang.", faktur).into());
            }
        }
    });

    // ==========================================
    // CALLBACK: REFRESH RIWAYAT
    // ==========================================
    let db_refresh_riw = Rc::clone(&db_ref);
    let win_handle_refresh_riw = window_handle.clone();
    let cid_refresh_riw = cabang_id.clone();
    main_window.on_refresh_riwayat(move || {
        let database = db_refresh_riw.borrow();
        if let Some(win) = win_handle_refresh_riw.upgrade() {
            muat_tabel_riwayat(&win, &database, &cid_refresh_riw);
        }
    });

    // ==========================================
    // CALLBACK: RETUR PENJUALAN
    // ==========================================
    let db_retur = Rc::clone(&db_ref);
    let win_handle_retur = window_handle.clone();
    let cid_retur = cabang_id.clone();
    let did_retur = device_id.clone();
    main_window.on_proses_retur_penjualan(move |faktur_jual, kode_brg, qty| {
        let mut database = db_retur.borrow_mut();
        let brg_repo = BarangRepo::new(database.conn());
        let opt_brg = brg_repo.cari_by_barcode_atau_kode(&cid_retur, kode_brg.as_str()).ok().flatten();

        if let Some(brg) = opt_brg {
            let shift_svc = ShiftService::new(&cid_retur, &did_retur);
            let shift_id = shift_svc.ambil_shift_aktif(&database).ok().flatten()
                .map(|s| s.id)
                .unwrap_or_else(|| "SHIFT-MANUAL".to_string());

            let item = ReturItem {
                barang_id: brg.id.clone(),
                kode_barang: brg.kode.clone(),
                nama_barang: brg.nama.clone(),
                jumlah: qty as f64,
                satuan: brg.satuan.clone(),
                hargajual: brg.hargajual1,
                subtotal: (qty as f64) * brg.hargajual1,
            };

            let mut retur_repo = ReturRepo::new(database.conn_mut());
            match retur_repo.simpan_retur_atomic(
                &cid_retur,
                &did_retur,
                &shift_id,
                faktur_jual.as_str(),
                "OP01",
                &[item],
                0.0,
            ) {
                Ok(no_retur) => {
                    if let Some(win) = win_handle_retur.upgrade() {
                        win.set_status_pesan(format!("Retur [{}] berhasil diproses!", no_retur).into());
                        muat_tabel_retur(&win, &database, &cid_retur);
                        muat_tabel_barang(&win, &database, &cid_retur, "");
                    }
                }
                Err(e) => {
                    if let Some(win) = win_handle_retur.upgrade() {
                        win.set_status_pesan(format!("Gagal retur: {}", e).into());
                    }
                }
            }
        } else {
            if let Some(win) = win_handle_retur.upgrade() {
                win.set_status_pesan(format!("Barang [{}] tidak ditemukan!", kode_brg).into());
            }
        }
    });

    let db_ref_retur = Rc::clone(&db_ref);
    let win_handle_ref_retur = window_handle.clone();
    let cid_ref_retur = cabang_id.clone();
    main_window.on_refresh_retur_penjualan(move || {
        let database = db_ref_retur.borrow();
        if let Some(win) = win_handle_ref_retur.upgrade() {
            muat_tabel_retur(&win, &database, &cid_ref_retur);
        }
    });

    // ==========================================
    // CALLBACK: PEMBELIAN BARANG
    // ==========================================
    let db_pembelian = Rc::clone(&db_ref);
    let win_handle_pembelian = window_handle.clone();
    let cid_pembelian = cabang_id.clone();
    main_window.on_simpan_transaksi_pembelian(move |kode, suplier, qty, harga, bayar, metode| {
        let mut database = db_pembelian.borrow_mut();
        let brg_repo = BarangRepo::new(database.conn());
        let opt_brg = brg_repo.cari_by_barcode_atau_kode(&cid_pembelian, kode.as_str()).ok().flatten();

        let brg = match opt_brg {
            Some(b) => b,
            None => {
                let baru = DBarang::baru(&cid_pembelian, kode.as_str(), &format!("Produk {}", kode), harga as f64, (harga as f64) * 1.25, 0.0);
                let _ = brg_repo.simpan(&baru);
                baru
            }
        };

        let item = PembelianItem {
            barang_id: brg.id.clone(),
            kode_barang: brg.kode.clone(),
            nama_barang: brg.nama.clone(),
            jumlah: qty as f64,
            satuan: brg.satuan.clone(),
            harga_beli: harga as f64,
            subtotal: (qty as f64) * (harga as f64),
        };

        let mut beli_repo = PembelianRepo::new(database.conn_mut());
        match beli_repo.simpan_pembelian_atomic(
            &cid_pembelian,
            "SUP-01",
            suplier.as_str(),
            None,
            "OP01",
            &[item],
            0.0,
            bayar as f64,
            metode.as_str(),
        ) {
            Ok(faktur) => {
                if let Some(win) = win_handle_pembelian.upgrade() {
                    win.set_status_pesan(format!("Faktur Pembelian [{}] berhasil disimpan!", faktur).into());
                    muat_tabel_pembelian(&win, &database, &cid_pembelian);
                    muat_tabel_barang(&win, &database, &cid_pembelian, "");
                }
            }
            Err(e) => {
                if let Some(win) = win_handle_pembelian.upgrade() {
                    win.set_status_pesan(format!("Gagal simpan pembelian: {}", e).into());
                }
            }
        }
    });

    let db_ref_beli = Rc::clone(&db_ref);
    let win_handle_ref_beli = window_handle.clone();
    let cid_ref_beli = cabang_id.clone();
    main_window.on_refresh_transaksi_pembelian(move || {
        let database = db_ref_beli.borrow();
        if let Some(win) = win_handle_ref_beli.upgrade() {
            muat_tabel_pembelian(&win, &database, &cid_ref_beli);
        }
    });

    // ==========================================
    // CALLBACK: MASTER SUPPLIER
    // ==========================================
    let db_sup = Rc::clone(&db_ref);
    let win_handle_sup = window_handle.clone();
    let cid_sup = cabang_id.clone();
    main_window.on_tambah_supplier_baru(move |kode, nama, telp, alm, rek| {
        let database = db_sup.borrow();
        let repo = SupplierRepo::new(database.conn());
        let s = DSupplier {
            id: uuid::Uuid::new_v4().to_string(),
            cabang_id: cid_sup.clone(),
            kode: kode.to_string(),
            nama: nama.to_string(),
            alamat: if alm.trim().is_empty() { None } else { Some(alm.to_string()) },
            telepon: if telp.trim().is_empty() { None } else { Some(telp.to_string()) },
            rekening: if rek.trim().is_empty() { None } else { Some(rek.to_string()) },
            keterangan: None,
        };
        let _ = repo.simpan(&s);
        if let Some(win) = win_handle_sup.upgrade() {
            win.set_status_pesan(format!("Supplier [{}] berhasil disimpan!", s.nama).into());
            muat_tabel_supplier(&win, &database, &cid_sup);
        }
    });

    let db_del_sup = Rc::clone(&db_ref);
    let win_handle_del_sup = window_handle.clone();
    let cid_del_sup = cabang_id.clone();
    main_window.on_hapus_supplier_id(move |id| {
        let database = db_del_sup.borrow();
        let repo = SupplierRepo::new(database.conn());
        let _ = repo.hapus(id.as_str());
        if let Some(win) = win_handle_del_sup.upgrade() {
            win.set_status_pesan("Supplier berhasil dihapus.".into());
            muat_tabel_supplier(&win, &database, &cid_del_sup);
        }
    });

    let db_ref_sup = Rc::clone(&db_ref);
    let win_handle_ref_sup = window_handle.clone();
    let cid_ref_sup = cabang_id.clone();
    main_window.on_refresh_data_supplier(move || {
        let database = db_ref_sup.borrow();
        if let Some(win) = win_handle_ref_sup.upgrade() {
            muat_tabel_supplier(&win, &database, &cid_ref_sup);
        }
    });

    // ==========================================
    // CALLBACK: STOK OPNAME
    // ==========================================
    let db_opname = Rc::clone(&db_ref);
    let win_handle_opname = window_handle.clone();
    let cid_opname = cabang_id.clone();
    main_window.on_proses_stok_opname(move |kode, fisik, alasan| {
        let mut database = db_opname.borrow_mut();
        let brg_repo = BarangRepo::new(database.conn());
        let opt_brg = brg_repo.cari_by_barcode_atau_kode(&cid_opname, kode.as_str()).ok().flatten();

        if let Some(brg) = opt_brg {
            let mut op_repo = OpnameRepo::new(database.conn_mut());
            match op_repo.simpan_opname(
                &cid_opname,
                &brg.id,
                &brg.kode,
                &brg.nama,
                fisik as f64,
                alasan.as_str(),
                "OP01",
            ) {
                Ok(_) => {
                    if let Some(win) = win_handle_opname.upgrade() {
                        win.set_status_pesan(format!("Stok Opname [{}] berhasil disesuaikan!", brg.nama).into());
                        muat_tabel_opname(&win, &mut database, &cid_opname);
                        muat_tabel_barang(&win, &database, &cid_opname, "");
                    }
                }
                Err(e) => {
                    if let Some(win) = win_handle_opname.upgrade() {
                        win.set_status_pesan(format!("Gagal opname: {}", e).into());
                    }
                }
            }
        } else {
            if let Some(win) = win_handle_opname.upgrade() {
                win.set_status_pesan(format!("Barang [{}] tidak ditemukan!", kode).into());
            }
        }
    });

    let db_ref_op = Rc::clone(&db_ref);
    let win_handle_ref_op = window_handle.clone();
    let cid_ref_op = cabang_id.clone();
    main_window.on_refresh_stok_opname(move || {
        let mut database = db_ref_op.borrow_mut();
        if let Some(win) = win_handle_ref_op.upgrade() {
            muat_tabel_opname(&win, &mut database, &cid_ref_op);
        }
    });

    // ==========================================
    // CALLBACK: HUTANG & PIUTANG
    // ==========================================
    let db_hp = Rc::clone(&db_ref);
    let win_handle_hp = window_handle.clone();
    let cid_hp = cabang_id.clone();
    main_window.on_bayar_cicilan_hutang(move |id, jml| {
        let database = db_hp.borrow();
        let repo = HutangPiutangRepo::new(database.conn());
        let _ = repo.bayar_hutang(id.as_str(), jml as f64);
        if let Some(win) = win_handle_hp.upgrade() {
            win.set_status_pesan("Pembayaran hutang berhasil dicatat.".into());
            muat_tabel_hutang_piutang(&win, &database, &cid_hp);
        }
    });

    let db_pi = Rc::clone(&db_ref);
    let win_handle_pi = window_handle.clone();
    let cid_pi = cabang_id.clone();
    main_window.on_bayar_cicilan_piutang(move |id, jml| {
        let database = db_pi.borrow();
        let repo = HutangPiutangRepo::new(database.conn());
        let _ = repo.bayar_piutang(id.as_str(), jml as f64);
        if let Some(win) = win_handle_pi.upgrade() {
            win.set_status_pesan("Penerimaan piutang berhasil dicatat.".into());
            muat_tabel_hutang_piutang(&win, &database, &cid_pi);
        }
    });

    let db_ref_hp = Rc::clone(&db_ref);
    let win_handle_ref_hp = window_handle.clone();
    let cid_ref_hp = cabang_id.clone();
    main_window.on_refresh_hutang_piutang(move || {
        let database = db_ref_hp.borrow();
        if let Some(win) = win_handle_ref_hp.upgrade() {
            muat_tabel_hutang_piutang(&win, &database, &cid_ref_hp);
        }
    });

    // ==========================================
    // CALLBACK: BUKU KAS & BIAYA
    // ==========================================
    let db_kas = Rc::clone(&db_ref);
    let win_handle_kas = window_handle.clone();
    let cid_kas = cabang_id.clone();
    main_window.on_catat_arus_kas_biaya(move |jenis, kat, nom, ket| {
        let database = db_kas.borrow();
        let repo = BukuKasRepo::new(database.conn());
        let _ = repo.catat_arus_kas(&cid_kas, jenis.as_str(), kat.as_str(), nom as f64, ket.as_str(), "OP01");
        if let Some(win) = win_handle_kas.upgrade() {
            win.set_status_pesan("Transaksi kas berhasil dicatat.".into());
            muat_tabel_buku_kas(&win, &database, &cid_kas);
        }
    });

    let db_ref_kas = Rc::clone(&db_ref);
    let win_handle_ref_kas = window_handle.clone();
    let cid_ref_kas = cabang_id.clone();
    main_window.on_refresh_buku_kas(move || {
        let database = db_ref_kas.borrow();
        if let Some(win) = win_handle_ref_kas.upgrade() {
            muat_tabel_buku_kas(&win, &database, &cid_ref_kas);
        }
    });

    // ==========================================
    // CALLBACK: BUKA SHIFT BARU
    // ==========================================
    let db_buka_shift = Rc::clone(&db_ref);
    let win_handle_buka_shift = window_handle.clone();
    let cid_buka_shift = cabang_id.clone();
    let did_buka_shift = device_id.clone();
    main_window.on_buka_shift_baru(move |modal| {
        let database = db_buka_shift.borrow();
        let svc = ShiftService::new(&cid_buka_shift, &did_buka_shift);
        match svc.buka_shift(&database, "OP01", modal as f64) {
            Ok(s) => {
                if let Some(win) = win_handle_buka_shift.upgrade() {
                    win.set_shift_status(format!("SHIFT: {}", &s.id[..8]).into());
                    win.set_status_pesan("Shift baru berhasil dibuka!".into());
                    muat_data_shift(&win, &database, &cid_buka_shift, &did_buka_shift);
                }
            }
            Err(e) => {
                if let Some(win) = win_handle_buka_shift.upgrade() {
                    win.set_status_pesan(format!("Gagal buka shift: {}", e).into());
                }
            }
        }
    });

    // ==========================================
    // CALLBACK: REFRESH LAPORAN
    // ==========================================
    let db_refresh_lap = Rc::clone(&db_ref);
    let win_handle_refresh_lap = window_handle.clone();
    let cid_refresh_lap = cabang_id.clone();
    main_window.on_refresh_laporan(move || {
        let database = db_refresh_lap.borrow();
        if let Some(win) = win_handle_refresh_lap.upgrade() {
            muat_laporan_omzet(&win, &database, &cid_refresh_lap);
        }
    });

    // ==========================================
    // CALLBACK: CARI PELANGGAN
    // ==========================================
    let db_cari_plg = Rc::clone(&db_ref);
    let win_handle_cari_plg = window_handle.clone();
    let cid_cari_plg = cabang_id.clone();
    main_window.on_cari_pelanggan(move |_kw| {
        let database = db_cari_plg.borrow();
        if let Some(win) = win_handle_cari_plg.upgrade() {
            muat_tabel_pelanggan(&win, &database, &cid_cari_plg);
        }
    });

    main_window.run()?;
    Ok(())
}

/// Helper sinkronisasi keranjang belanja kasir
fn sinkronkan_tabel_kasir(win: &MainWindow, svc: &KasirService) {
    let slint_items: Vec<CartItemData> = svc
        .keranjang
        .iter()
        .map(|item| CartItemData {
            id: item.barang_id.clone().into(),
            kode: item.kode_barang.clone().into(),
            nama: item.nama_barang.clone().into(),
            satuan: item.satuan.clone().into(),
            harga: item.hargajual as f32,
            jumlah: item.jumlah as f32,
            subtotal: item.subtotal as f32,
        })
        .collect();

    win.set_cart_items(ModelRc::from(Rc::new(VecModel::from(slint_items))));
    win.set_total_belanja(svc.total_belanja() as f32);
    win.set_nilai_tukar_poin(svc.nilai_tukar_poin as f32);

    if let Some(m) = &svc.member_terpilih {
        win.set_member_nama(m.nama.clone().into());
        win.set_member_kode(m.kode.clone().into());
        win.set_member_poin_saldo(m.poin_saldo as i32);
        win.set_is_member_attached(true);
    } else {
        win.set_member_nama("UMUM (Non-Member)".into());
        win.set_member_kode("UMUM".into());
        win.set_member_poin_saldo(0);
        win.set_is_member_attached(false);
    }
}

/// Helper cetak struk ke printer thermal ESC/POS (file struk_terakhir.bin)
fn cetak_struk_ke_escpos(database: &Database, cabang_id: &str, faktur: &str) -> bool {
    let cabang_repo = CabangRepo::new(database.conn());
    let cabang = cabang_repo
        .ambil_cabang_pertama()
        .ok()
        .flatten()
        .unwrap_or_else(|| Cabang::baru(cabang_id, "Toko Pusat", true));

    let mut stmt = match database.conn().prepare(
        "SELECT id, cabang_id, device_id, shift_id, faktur, tanggal, kode_pelanggan, operator_id, subtotal, diskon_rp, total_akhir, bayar_tunai, bayar_nontunai, kembalian, metode_bayar, status, poin_didapat, poin_ditukar, nilai_tukar_poin, sync_status FROM tpenjualan WHERE cabang_id = ?1 AND faktur = ?2 LIMIT 1;"
    ) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let opt_penjualan = stmt
        .query_row(rusqlite::params![cabang_id, faktur], |row| {
            let tgl_str: String = row.get(5)?;
            let tgl = chrono::DateTime::parse_from_rfc3339(&tgl_str)
                .map(|d| d.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            Ok(TPenjualan {
                id: row.get(0)?,
                cabang_id: row.get(1)?,
                device_id: row.get(2)?,
                shift_id: row.get(3)?,
                faktur: row.get(4)?,
                tanggal: tgl,
                kode_pelanggan: row.get(6)?,
                operator_id: row.get(7)?,
                subtotal: row.get(8)?,
                diskon_rp: row.get(9)?,
                total_akhir: row.get(10)?,
                bayar_tunai: row.get(11)?,
                bayar_nontunai: row.get(12)?,
                kembalian: row.get(13)?,
                metode_bayar: row.get(14)?,
                status: row.get(15)?,
                poin_didapat: row.get(16)?,
                poin_ditukar: row.get(17)?,
                nilai_tukar_poin: row.get(18)?,
                sync_status: row.get(19)?,
                created_at: None,
                updated_at: None,
            })
        })
        .optional()
        .ok()
        .flatten();

    if let Some(penjualan) = opt_penjualan {
        let mut det_stmt = match database.conn().prepare(
            "SELECT id, penjualan_id, cabang_id, barang_id, kode_barang, nama_barang, jumlah, satuan, hargajual, hargapokok, diskon_persen, diskon_rp, subtotal, sync_status FROM tpenjualandetail WHERE penjualan_id = ?1;"
        ) {
            Ok(s) => s,
            Err(_) => return false,
        };
        let det_rows = det_stmt.query_map(rusqlite::params![penjualan.id], |r| {
            Ok(TPenjualanDetail {
                id: r.get(0)?,
                penjualan_id: r.get(1)?,
                cabang_id: r.get(2)?,
                barang_id: r.get(3)?,
                kode_barang: r.get(4)?,
                nama_barang: r.get(5)?,
                jumlah: r.get(6)?,
                satuan: r.get(7)?,
                hargajual: r.get(8)?,
                hargapokok: r.get(9)?,
                diskon_persen: r.get(10)?,
                diskon_rp: r.get(11)?,
                subtotal: r.get(12)?,
                sync_status: r.get(13)?,
                created_at: None,
                updated_at: None,
            })
        });
        let details: Vec<TPenjualanDetail> = det_rows
            .map(|m| m.filter_map(|x| x.ok()).collect())
            .unwrap_or_default();
        let struk_bytes = StrukKasir::buat_struk(&cabang, &penjualan, &details, true);
        let _ = std::fs::write("struk_terakhir.bin", struk_bytes);
        true
    } else {
        false
    }
}

/// Helper muat daftar transaksi yang di-hold (antrean pending)
fn muat_tabel_pending(win: &MainWindow, db: &Database, cabang_id: &str, device_id: &str) {
    let repo = PendingRepo::new(db.conn());
    if let Ok(list) = repo.semua_pending(cabang_id, device_id) {
        let items: Vec<PendingGridData> = list
            .into_iter()
            .map(|p| PendingGridData {
                id: p.id.into(),
                faktur: p.faktur.into(),
                tanggal: p.tanggal.format("%Y-%m-%d %H:%M").to_string().into(),
                pelanggan: p.kode_pelanggan.unwrap_or_else(|| "-".to_string()).into(),
                total: p.total_akhir as f32,
                keterangan: p.keterangan.unwrap_or_else(|| "-".to_string()).into(),
            })
            .collect();

        win.set_pending_items(ModelRc::from(Rc::new(VecModel::from(items))));
    }
}

/// Helper muat data master barang ke tabel (sesuai Screenshot 1)
fn muat_tabel_barang(win: &MainWindow, db: &Database, cabang_id: &str, keyword: &str) {
    let repo = BarangRepo::new(db.conn());
    if let Ok(list) = repo.cari_by_nama(cabang_id, keyword, 200) {
        let total_item = list.len();
        let mut tot_beli = 0.0;
        let mut tot_jual = 0.0;

        let items: Vec<ProductGridData> = list
            .into_iter()
            .map(|b| {
                tot_beli += b.hargapokok * b.stok;
                tot_jual += b.hargajual1 * b.stok;
                let is_kritis = b.stok <= b.stokminimum;

                ProductGridData {
                    id: b.id.into(),
                    kode: b.kode.into(),
                    barcode: b.barcode.unwrap_or_default().into(),
                    nama: b.nama.into(),
                    kategori: b.kategori.unwrap_or_else(|| "Umum".to_string()).into(),
                    satuan: b.satuan.into(),
                    hargapokok: b.hargapokok as f32,
                    hargajual1: b.hargajual1 as f32,
                    hargajual2: b.hargajual2 as f32,
                    hargajual3: b.hargajual3 as f32,
                    stok: b.stok as f32,
                    stokminimum: b.stokminimum as f32,
                    is_kritis,
                }
            })
            .collect();

        win.set_products(ModelRc::from(Rc::new(VecModel::from(items))));
        win.set_total_item_barang(total_item as i32);
        win.set_total_nilai_beli(tot_beli as f32);
        win.set_total_nilai_jual(tot_jual as f32);
    }
}

/// Helper muat data master pelanggan
fn muat_tabel_pelanggan(win: &MainWindow, db: &Database, cabang_id: &str) {
    let mut stmt = match db.conn().prepare(
        "SELECT id, kode, nama, alamat, telepon, poin_saldo FROM dpelanggan WHERE cabang_id = ?1 ORDER BY kode ASC LIMIT 100;"
    ) {
        Ok(s) => s,
        Err(_) => return,
    };

    let rows = stmt.query_map([cabang_id], |row| {
        let id: String = row.get(0)?;
        let kode: String = row.get(1)?;
        let nama: String = row.get(2)?;
        let alamat: Option<String> = row.get(3)?;
        let telepon: Option<String> = row.get(4)?;
        let poin: i64 = row.get(5)?;

        Ok(PelangganGridData {
            id: id.into(),
            kode: kode.into(),
            nama: nama.into(),
            alamat: alamat.unwrap_or_default().into(),
            telepon: telepon.unwrap_or_default().into(),
            poin_saldo: poin as i32,
        })
    });

    if let Ok(mapped) = rows {
        let list: Vec<PelangganGridData> = mapped.filter_map(|r| r.ok()).collect();
        win.set_pelanggan_items(ModelRc::from(Rc::new(VecModel::from(list))));
    }
}

/// Helper muat histori riwayat penjualan
fn muat_tabel_riwayat(win: &MainWindow, db: &Database, cabang_id: &str) {
    let mut stmt = match db.conn().prepare(
        "SELECT id, faktur, tanggal, total_akhir, bayar_tunai + bayar_nontunai, metode_bayar, operator_id FROM tpenjualan WHERE cabang_id = ?1 ORDER BY tanggal DESC LIMIT 50;"
    ) {
        Ok(s) => s,
        Err(_) => return,
    };

    let mut omzet_total = 0.0;
    let rows = stmt.query_map([cabang_id], |row| {
        let id: String = row.get(0)?;
        let faktur: String = row.get(1)?;
        let tanggal: String = row.get(2)?;
        let total: f64 = row.get(3)?;
        let bayar: f64 = row.get(4)?;
        let metode: String = row.get(5)?;
        let kasir: String = row.get(6)?;

        Ok((total, TransactionGridData {
            id: id.into(),
            faktur: faktur.into(),
            tanggal: tanggal.into(),
            total: total as f32,
            bayar: bayar as f32,
            metode: metode.into(),
            kasir: kasir.into(),
        }))
    });

    if let Ok(mapped) = rows {
        let mut list = Vec::new();
        for r in mapped.flatten() {
            omzet_total += r.0;
            list.push(r.1);
        }
        win.set_history_items(ModelRc::from(Rc::new(VecModel::from(list))));
        win.set_total_omzet_riwayat(omzet_total as f32);
    }
}

/// Helper muat data shift
fn muat_data_shift(win: &MainWindow, db: &Database, cabang_id: &str, device_id: &str) {
    let repo = ShiftRepo::new(db.conn());
    match repo.ambil_shift_aktif(cabang_id, device_id) {
        Ok(Some(s)) => {
            win.set_shift_modal_awal(s.modal_awal as f32);
            win.set_shift_tunai(s.total_penjualan_tunai as f32);
            win.set_shift_nontunai(s.total_penjualan_nontunai as f32);
            win.set_shift_retur(s.total_retur as f32);
            win.set_shift_uang_seharusnya(s.uang_seharusnya as f32);
            win.set_shift_status(format!("SHIFT: {}", &s.id[..8]).into());
        }
        _ => {
            win.set_shift_modal_awal(0.0);
            win.set_shift_tunai(0.0);
            win.set_shift_nontunai(0.0);
            win.set_shift_retur(0.0);
            win.set_shift_uang_seharusnya(0.0);
            win.set_shift_status("CLOSED".into());
        }
    }
}

/// Helper muat data laporan keuangan
fn muat_laporan_omzet(win: &MainWindow, db: &Database, cabang_id: &str) {
    let reporter = SalesReporter::new(db.conn());
    if let Ok(ringkasan) = reporter.ringkasan_penjualan(cabang_id, "2000-01-01", "2099-12-31") {
        let metrics = vec![
            LaporanMetricData {
                metrik: "Total Transaksi".into(),
                nilai: format!("{}", ringkasan.total_transaksi).into(),
                keterangan: "Jumlah nota penjualan".into(),
            },
            LaporanMetricData {
                metrik: "Total Omzet".into(),
                nilai: format!("Rp {:.0}", ringkasan.total_omzet).into(),
                keterangan: "Penjualan bruto".into(),
            },
            LaporanMetricData {
                metrik: "Total HPP".into(),
                nilai: format!("Rp {:.0}", ringkasan.total_hpp).into(),
                keterangan: "Harga pokok penjualan".into(),
            },
            LaporanMetricData {
                metrik: "Laba Kotor".into(),
                nilai: format!("Rp {:.0}", ringkasan.laba_kotor).into(),
                keterangan: "Omzet - HPP".into(),
            },
        ];
        win.set_laporan_metrics(ModelRc::from(Rc::new(VecModel::from(metrics))));
    }

    if let Ok(top) = reporter.produk_terlaris(cabang_id, 5) {
        let items: Vec<TopProductData> = top
            .into_iter()
            .map(|t| TopProductData {
                kode: t.kode_barang.into(),
                nama: t.nama_barang.into(),
                qty: t.total_qty as f32,
                omzet: t.total_penjualan as f32,
            })
            .collect();
        win.set_top_products(ModelRc::from(Rc::new(VecModel::from(items))));
    }
}

/// Inisialisasi data dasar jika tabel kosong
fn inisialisasi_data_dasar(db: &Database) -> Result<(String, String), Box<dyn std::error::Error>> {
    let cabang_repo = CabangRepo::new(db.conn());
    let cabang = match cabang_repo.ambil_cabang_pertama()? {
        Some(c) => c,
        None => {
            let baru = Cabang::baru("CB01", "Toko Pusat", true);
            cabang_repo.simpan_cabang(&baru)?;
            baru
        }
    };

    let device = match cabang_repo.ambil_device_pertama(&cabang.id)? {
        Some(d) => d,
        None => {
            let baru = Device::baru(&cabang.id, "DEV01", "Kasir 1", "server", "MACHINE-LOCAL-01");
            cabang_repo.simpan_device(&baru)?;
            baru
        }
    };

    // Tambah sampel produk sesuai screenshot iB Retago (Aqua Galon, dll)
    let barang_repo = BarangRepo::new(db.conn());
    if barang_repo.cari_by_barcode_atau_kode(&cabang.id, "BRG001")?.is_none() {
        let mut b1 = DBarang::baru(&cabang.id, "BRG001", "Aqua Galon 1", 8000.0, 11000.0, 377.0);
        b1.stokminimum = 5.0;
        b1.satuan = "Botol".to_string();
        b1.kategori = Some("Minuman".to_string());
        b1.hargajual2 = 10500.0;
        b1.hargajual3 = 10000.0;
        barang_repo.simpan(&b1)?;

        let mut b2 = DBarang::baru(&cabang.id, "BRG002", "Aqua Galon 2", 8000.0, 11000.0, 96.0);
        b2.stokminimum = 5.0;
        b2.satuan = "Dus".to_string();
        b2.kategori = Some("Minuman".to_string());
        b2.hargajual2 = 10550.0;
        b2.hargajual3 = 10500.0;
        barang_repo.simpan(&b2)?;

        // Contoh barang stok kritis (stok 3 <= min 5)
        let mut b3 = DBarang::baru(&cabang.id, "BRG003", "Aqua Galon 3", 8000.0, 11000.0, 3.0);
        b3.stokminimum = 5.0;
        b3.satuan = "Botol".to_string();
        b3.kategori = Some("Minuman".to_string());
        b3.hargajual2 = 10600.0;
        b3.hargajual3 = 11000.0;
        barang_repo.simpan(&b3)?;
    }

    Ok((cabang.id, device.id))
}

/// Helper muat data retur penjualan
fn muat_tabel_retur(win: &MainWindow, db: &Database, cabang_id: &str) {
    let mut stmt = match db.conn().prepare(
        "SELECT id, faktur, faktur_penjualan, tanggal, total_akhir FROM treturpenjualan WHERE cabang_id = ?1 ORDER BY tanggal DESC LIMIT 50;"
    ) {
        Ok(s) => s,
        Err(_) => return,
    };
    let rows = stmt.query_map([cabang_id], |row| {
        let total: f64 = row.get(4)?;
        Ok(ReturGridData {
            id: row.get::<_, String>(0)?.into(),
            faktur: row.get::<_, String>(1)?.into(),
            faktur_penjualan: row.get::<_, String>(2)?.into(),
            tanggal: row.get::<_, String>(3)?.into(),
            total: total as f32,
        })
    });
    if let Ok(mapped) = rows {
        let list: Vec<ReturGridData> = mapped.filter_map(|r| r.ok()).collect();
        win.set_retur_items(ModelRc::from(Rc::new(VecModel::from(list))));
    }
}

/// Helper muat data supplier
fn muat_tabel_supplier(win: &MainWindow, db: &Database, cabang_id: &str) {
    let repo = SupplierRepo::new(db.conn());
    if let Ok(list) = repo.semua(cabang_id) {
        let slint_items: Vec<SupplierGridData> = list
            .into_iter()
            .map(|s| SupplierGridData {
                id: s.id.into(),
                kode: s.kode.into(),
                nama: s.nama.into(),
                alamat: s.alamat.unwrap_or_default().into(),
                telepon: s.telepon.unwrap_or_default().into(),
                rekening: s.rekening.unwrap_or_default().into(),
            })
            .collect();
        win.set_supplier_items(ModelRc::from(Rc::new(VecModel::from(slint_items))));
    }
}

/// Helper muat data pembelian
fn muat_tabel_pembelian(win: &MainWindow, db: &Database, cabang_id: &str) {
    let mut stmt = match db.conn().prepare(
        "SELECT id, faktur, nama_suplier, tanggal, total_akhir, bayar, sisa, status FROM tpembelian WHERE cabang_id = ?1 ORDER BY tanggal DESC LIMIT 50;"
    ) {
        Ok(s) => s,
        Err(_) => return,
    };
    let rows = stmt.query_map([cabang_id], |row| {
        let tot: f64 = row.get(4)?;
        let byr: f64 = row.get(5)?;
        let sisa: f64 = row.get(6)?;
        Ok(PembelianGridData {
            id: row.get::<_, String>(0)?.into(),
            faktur: row.get::<_, String>(1)?.into(),
            supplier: row.get::<_, String>(2)?.into(),
            tanggal: row.get::<_, String>(3)?.into(),
            total: tot as f32,
            bayar: byr as f32,
            sisa: sisa as f32,
            status: row.get::<_, String>(7)?.into(),
        })
    });
    if let Ok(mapped) = rows {
        let list: Vec<PembelianGridData> = mapped.filter_map(|r| r.ok()).collect();
        win.set_pembelian_items(ModelRc::from(Rc::new(VecModel::from(list))));
    }
}

/// Helper muat data stok opname
fn muat_tabel_opname(win: &MainWindow, db: &mut Database, cabang_id: &str) {
    let repo = OpnameRepo::new(db.conn_mut());
    if let Ok(list) = repo.riwayat_opname(cabang_id) {
        let slint_items: Vec<OpnameGridData> = list
            .into_iter()
            .map(|o| OpnameGridData {
                id: o.id.into(),
                kode: o.kode_barang.into(),
                nama: o.nama_barang.into(),
                tanggal: o.tanggal.into(),
                stok_sistem: o.stok_komputer as f32,
                stok_fisik: o.stok_nyata as f32,
                selisih: o.selisih as f32,
                alasan: o.alasan.into(),
            })
            .collect();
        win.set_opname_items(ModelRc::from(Rc::new(VecModel::from(slint_items))));
    }
}

/// Helper muat data hutang & piutang
fn muat_tabel_hutang_piutang(win: &MainWindow, db: &Database, cabang_id: &str) {
    let repo = HutangPiutangRepo::new(db.conn());
    if let Ok(list_h) = repo.semua_hutang(cabang_id) {
        let h_items: Vec<HutangGridData> = list_h
            .into_iter()
            .map(|h| HutangGridData {
                id: h.id.into(),
                faktur: h.faktur.into(),
                pihak: h.suplier_nama.into(),
                tanggal: h.tanggal.into(),
                tagihan: h.tagihan_awal as f32,
                dibayar: h.telah_dibayar as f32,
                sisa: h.sisa as f32,
                status: h.status.into(),
            })
            .collect();
        win.set_hutang_items(ModelRc::from(Rc::new(VecModel::from(h_items))));
    }
    if let Ok(list_p) = repo.semua_piutang(cabang_id) {
        let p_items: Vec<HutangGridData> = list_p
            .into_iter()
            .map(|p| HutangGridData {
                id: p.id.into(),
                faktur: p.faktur.into(),
                pihak: p.pelanggan_nama.into(),
                tanggal: p.tanggal.into(),
                tagihan: p.tagihan_awal as f32,
                dibayar: p.telah_dibayar as f32,
                sisa: p.sisa as f32,
                status: p.status.into(),
            })
            .collect();
        win.set_piutang_items(ModelRc::from(Rc::new(VecModel::from(p_items))));
    }
}

/// Helper muat data buku kas dan biaya operasional
fn muat_tabel_buku_kas(win: &MainWindow, db: &Database, cabang_id: &str) {
    let repo = BukuKasRepo::new(db.conn());
    if let Ok(rekap) = repo.rekap_kas(cabang_id) {
        win.set_kas_total_masuk(rekap.total_masuk as f32);
        win.set_kas_total_keluar(rekap.total_keluar as f32);
        win.set_kas_saldo(rekap.saldo_kas as f32);
    }
    if let Ok(list) = repo.riwayat_kas(cabang_id) {
        let items: Vec<CashflowGridData> = list
            .into_iter()
            .map(|k| CashflowGridData {
                id: k.id.into(),
                tanggal: k.tanggal.into(),
                jenis: k.jenis.into(),
                kategori: k.kategori.into(),
                nominal: k.nominal as f32,
                keterangan: k.keterangan.into(),
            })
            .collect();
        win.set_cashflow_items(ModelRc::from(Rc::new(VecModel::from(items))));
    }
}




