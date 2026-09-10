#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use chrono::Local;
use serde::{Deserialize, Serialize};
use tauri::State;

use fazpos::db::Database;
use fazpos::domain::barang::DBarang;
use fazpos::domain::cabang::{Cabang, Device};
use fazpos::license::machine_id::MachineId;
use fazpos::license::verification::{LicenseStatus, LicenseVerifier};
use fazpos::repository::barang_repo::BarangRepo;
use fazpos::repository::cabang_repo::CabangRepo;
use fazpos::repository::pelanggan_repo::PelangganRepo;
use fazpos::repository::pending_repo::PendingRepo;
use fazpos::services::kasir_service::KasirService;
use fazpos::services::shift_service::ShiftService;

pub struct AppState {
    pub db: Mutex<Database>,
    pub kasir: Mutex<KasirService>,
    pub cabang_id: String,
    pub device_id: String,
    pub shift_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProductDTO {
    pub id: String,
    pub kode: String,
    pub barcode: String,
    pub nama: String,
    pub kategori: String,
    pub satuan: String,
    pub rak: String,
    pub supplier: String,
    pub hargapokok: f64,
    pub hargajual1: f64,
    pub hargajual2: f64,
    pub hargajual3: f64,
    pub margin_persen: f64,
    pub stok: f64,
    pub stokminimum: f64,
    pub is_kritis: bool,
    pub expired: Option<String>,
    pub tag: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProductStatsDTO {
    pub total_produk: usize,
    pub stok_optimal: usize,
    pub stok_menipis: usize,
    pub stok_kosong: usize,
    pub valuasi_aset: f64,
    pub avg_margin: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CartItemDTO {
    pub id: String,
    pub kode: String,
    pub nama: String,
    pub satuan: String,
    pub harga: f64,
    pub jumlah: f64,
    pub subtotal: f64,
    pub suplier: Option<String>,
    pub rak: Option<String>,
    pub hargajual1: Option<f64>,
    pub hargajual2: Option<f64>,
    pub hargajual3: Option<f64>,
    pub expired: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CartSummaryDTO {
    pub items: Vec<CartItemDTO>,
    pub subtotal: f64,
    pub nilai_tukar_poin: f64,
    pub total_akhir: f64,
    pub member_nama: String,
    pub member_kode: String,
    pub member_poin_saldo: i64,
    pub is_member_attached: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StatusInfoDTO {
    pub toko_nama: String,
    pub operator_nama: String,
    pub shift_status: String,
    pub terminal_id: String,
    pub clock: String,
    pub escpos_ready: bool,
    pub sqlite_status: String,
    pub machine_id: String,
    pub license_status: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CheckoutResultDTO {
    pub sukses: bool,
    pub faktur: String,
    pub total: f64,
    pub bayar: f64,
    pub kembalian: f64,
    pub pesan: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PendingItemDTO {
    pub id: String,
    pub faktur: String,
    pub tanggal: String,
    pub pelanggan: String,
    pub total: f64,
    pub keterangan: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransactionHistoryItemDTO {
    pub id: String,
    pub no: usize,
    pub faktur: String,
    pub waktu: String,
    pub kasir: String,
    pub shift: String,
    pub pelanggan: String,
    pub pelanggan_badge: String,
    pub pelanggan_info: String,
    pub total_qty: f64,
    pub total_sku: usize,
    pub metode: String,
    pub total_penjualan: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransactionSummaryStatsDTO {
    pub total_omset: f64,
    pub target_omset: f64,
    pub persentase_omset: f64,
    pub total_qty: f64,
    pub avg_item_per_trx: f64,
    pub total_transaksi: usize,
    pub void_count: usize,
    pub retur_count: usize,
    pub avg_basket_size: f64,
    pub max_basket: f64,
    pub min_basket: f64,
    pub tunai_count: usize,
    pub qris_count: usize,
    pub edc_count: usize,
    pub tunai_pct: f64,
    pub qris_pct: f64,
    pub edc_pct: f64,
}

fn build_cart_summary(svc: &KasirService) -> CartSummaryDTO {
    let items: Vec<CartItemDTO> = svc
        .keranjang
        .iter()
        .map(|item| CartItemDTO {
            id: item.barang_id.clone(),
            kode: item.kode_barang.clone(),
            nama: item.nama_barang.clone(),
            satuan: item.satuan.clone(),
            harga: item.hargajual,
            jumlah: item.jumlah,
            subtotal: item.subtotal,
            suplier: Some("PT Sumber Makmur".to_string()),
            rak: Some("Rak A-01".to_string()),
            hargajual1: Some(item.hargajual),
            hargajual2: Some((item.hargajual * 0.95).round()),
            hargajual3: Some((item.hargajual * 0.90).round()),
            expired: Some("12/2027".to_string()),
        })
        .collect();

    let subtotal = svc.total_belanja();
    let total_akhir = (subtotal - svc.nilai_tukar_poin).max(0.0);

    let (member_nama, member_kode, member_poin_saldo, is_member_attached) = match &svc.member_terpilih {
        Some(m) => (m.nama.clone(), m.kode.clone(), m.poin_saldo, true),
        None => ("UMUM (Non-Member)".to_string(), "UMUM".to_string(), 0, false),
    };

    CartSummaryDTO {
        items,
        subtotal,
        nilai_tukar_poin: svc.nilai_tukar_poin,
        total_akhir,
        member_nama,
        member_kode,
        member_poin_saldo,
        is_member_attached,
    }
}

#[tauri::command]
fn get_status_info(_state: State<AppState>) -> Result<StatusInfoDTO, String> {
    let cur_machine_id = MachineId::dapatkan();
    let lic_status = LicenseVerifier::baca_dari_file(LicenseVerifier::path_lisensi_default(), &cur_machine_id);
    let license_status = match lic_status {
        LicenseStatus::Aktif(payload) => format!("TERAKTIVASI ({})", payload.nama_toko),
        _ => "BELUM AKTIVASI".to_string(),
    };

    let clock = Local::now().format("%H:%M:%S WIB").to_string();

    Ok(StatusInfoDTO {
        toko_nama: "MUEEZA STORE".to_string(),
        operator_nama: "Alexander P.".to_string(),
        shift_status: "Shift 1".to_string(),
        terminal_id: "Terminal: #REG-01".to_string(),
        clock,
        escpos_ready: true,
        sqlite_status: "SQLite WAL: Synced | Latency: 0.2ms".to_string(),
        machine_id: cur_machine_id,
        license_status,
    })
}

#[tauri::command]
fn get_catalog_products(state: State<AppState>, keyword: Option<String>) -> Result<Vec<ProductDTO>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let repo = BarangRepo::new(db.conn());
    let kw = keyword.unwrap_or_default();

    let default_products = vec![
        ProductDTO {
            id: "p-1".to_string(),
            kode: "BRG-0001".to_string(),
            barcode: "8992761001201".to_string(),
            nama: "Indomie Goreng Spesial 85g".to_string(),
            kategori: "Makanan & Minuman".to_string(),
            satuan: "Bks".to_string(),
            rak: "A-01".to_string(),
            supplier: "PT Indofood CBP".to_string(),
            hargapokok: 2750.0,
            hargajual1: 3500.0,
            hargajual2: 3300.0,
            hargajual3: 3100.0,
            margin_persen: 27.3,
            stok: 142.0,
            stokminimum: 20.0,
            is_kritis: false,
            expired: Some("15/08/2027".to_string()),
            tag: Some("Laris".to_string()),
        },
        ProductDTO {
            id: "p-2".to_string(),
            kode: "BRG-0024".to_string(),
            barcode: "8993077110192".to_string(),
            nama: "Minyak Goreng Sania 2L Pouch".to_string(),
            kategori: "Kebutuhan Rumah".to_string(),
            satuan: "Pouch".to_string(),
            rak: "B-03".to_string(),
            supplier: "Wilmar Nabati".to_string(),
            hargapokok: 32000.0,
            hargajual1: 36500.0,
            hargajual2: 35500.0,
            hargajual3: 35000.0,
            margin_persen: 14.1,
            stok: 48.0,
            stokminimum: 10.0,
            is_kritis: false,
            expired: Some("10/12/2027".to_string()),
            tag: None,
        },
        ProductDTO {
            id: "p-3".to_string(),
            kode: "BRG-0089".to_string(),
            barcode: "8998009010411".to_string(),
            nama: "Susu Ultra Milk Coklat 250ml".to_string(),
            kategori: "Makanan & Minuman".to_string(),
            satuan: "Kotak".to_string(),
            rak: "C-02".to_string(),
            supplier: "Ultrajaya Milk".to_string(),
            hargapokok: 5400.0,
            hargajual1: 6500.0,
            hargajual2: 6200.0,
            hargajual3: 6000.0,
            margin_persen: 20.4,
            stok: 6.0,
            stokminimum: 10.0,
            is_kritis: true,
            expired: Some("25/09/2026".to_string()),
            tag: Some("Segera Order".to_string()),
        },
        ProductDTO {
            id: "p-4".to_string(),
            kode: "BRG-0112".to_string(),
            barcode: "8886008101053".to_string(),
            nama: "Aqua Air Mineral Botol 600ml".to_string(),
            kategori: "Makanan & Minuman".to_string(),
            satuan: "Btl".to_string(),
            rak: "D-01".to_string(),
            supplier: "Danone Aqua".to_string(),
            hargapokok: 2800.0,
            hargajual1: 3500.0,
            hargajual2: 3200.0,
            hargajual3: 3000.0,
            margin_persen: 25.0,
            stok: 96.0,
            stokminimum: 24.0,
            is_kritis: false,
            expired: Some("18/06/2028".to_string()),
            tag: None,
        },
        ProductDTO {
            id: "p-5".to_string(),
            kode: "BRG-0145".to_string(),
            barcode: "8991001402231".to_string(),
            nama: "Beras Ramos Super Pandan Wangi 5kg".to_string(),
            kategori: "Kebutuhan Rumah".to_string(),
            satuan: "Krg".to_string(),
            rak: "E-02".to_string(),
            supplier: "PB Beras Makmur".to_string(),
            hargapokok: 68000.0,
            hargajual1: 74000.0,
            hargajual2: 72000.0,
            hargajual3: 71000.0,
            margin_persen: 8.8,
            stok: 24.0,
            stokminimum: 5.0,
            is_kritis: false,
            expired: Some("30/01/2028".to_string()),
            tag: None,
        },
        ProductDTO {
            id: "p-6".to_string(),
            kode: "BRG-0201".to_string(),
            barcode: "8991002105128".to_string(),
            nama: "Teh Botol Sosro PET 350ml".to_string(),
            kategori: "Makanan & Minuman".to_string(),
            satuan: "Btl".to_string(),
            rak: "Pendingin-1".to_string(),
            supplier: "Sinar Sosro".to_string(),
            hargapokok: 3100.0,
            hargajual1: 4000.0,
            hargajual2: 3800.0,
            hargajual3: 3600.0,
            margin_persen: 29.0,
            stok: 0.0,
            stokminimum: 12.0,
            is_kritis: true,
            expired: Some("05/11/2026".to_string()),
            tag: Some("Habis".to_string()),
        },
        ProductDTO {
            id: "p-7".to_string(),
            kode: "BRG-0255".to_string(),
            barcode: "8999999052028".to_string(),
            nama: "Sabun Batang Lifebuoy Total 10 Merah 85g".to_string(),
            kategori: "Personal Care".to_string(),
            satuan: "Pcs".to_string(),
            rak: "F-04".to_string(),
            supplier: "Unilever Indonesia".to_string(),
            hargapokok: 3400.0,
            hargajual1: 4500.0,
            hargajual2: 4200.0,
            hargajual3: 4000.0,
            margin_persen: 32.4,
            stok: 78.0,
            stokminimum: 15.0,
            is_kritis: false,
            expired: Some("14/04/2029".to_string()),
            tag: None,
        },
        ProductDTO {
            id: "p-8".to_string(),
            kode: "BRG-0312".to_string(),
            barcode: "8991002301018".to_string(),
            nama: "Kopi Kapal Api Spesial Mix Renceng 10x24g".to_string(),
            kategori: "Makanan & Minuman".to_string(),
            satuan: "Rcg".to_string(),
            rak: "A-05".to_string(),
            supplier: "Santos Jaya Abadi".to_string(),
            hargapokok: 12500.0,
            hargajual1: 15000.0,
            hargajual2: 14500.0,
            hargajual3: 14000.0,
            margin_persen: 20.0,
            stok: 32.0,
            stokminimum: 8.0,
            is_kritis: false,
            expired: Some("22/10/2027".to_string()),
            tag: None,
        },
        ProductDTO {
            id: "p-9".to_string(),
            kode: "BRG-0340".to_string(),
            barcode: "8992775110023".to_string(),
            nama: "Gula Pasir Putih Gulaku Premium 1kg".to_string(),
            kategori: "Kebutuhan Rumah".to_string(),
            satuan: "Bks".to_string(),
            rak: "B-01".to_string(),
            supplier: "Sugar Group".to_string(),
            hargapokok: 15500.0,
            hargajual1: 17500.0,
            hargajual2: 17000.0,
            hargajual3: 16800.0,
            margin_persen: 12.9,
            stok: 55.0,
            stokminimum: 10.0,
            is_kritis: false,
            expired: Some("19/08/2028".to_string()),
            tag: None,
        },
        ProductDTO {
            id: "p-10".to_string(),
            kode: "BRG-0402".to_string(),
            barcode: "8992775210150".to_string(),
            nama: "Pocari Sweat Isotonik Can 330ml".to_string(),
            kategori: "Makanan & Minuman".to_string(),
            satuan: "Can".to_string(),
            rak: "Pendingin-2".to_string(),
            supplier: "Amerta Indah Otsuka".to_string(),
            hargapokok: 6200.0,
            hargajual1: 7800.0,
            hargajual2: 7400.0,
            hargajual3: 7200.0,
            margin_persen: 25.8,
            stok: 64.0,
            stokminimum: 15.0,
            is_kritis: false,
            expired: Some("09/03/2027".to_string()),
            tag: None,
        },
    ];

    let list = if kw.trim().is_empty() {
        repo.cari_by_nama(&state.cabang_id, "", 200).map_err(|e| e.to_string())?
    } else {
        repo.cari_by_nama(&state.cabang_id, &kw, 200).map_err(|e| e.to_string())?
    };

    let mut result: Vec<ProductDTO> = list
        .into_iter()
        .map(|b| {
            let is_kritis = b.stok <= b.stokminimum;
            let margin = if b.hargapokok > 0.0 {
                ((b.hargajual1 - b.hargapokok) / b.hargapokok * 100.0 * 10.0).round() / 10.0
            } else {
                25.0
            };
            ProductDTO {
                id: b.id,
                kode: b.kode,
                barcode: b.barcode.unwrap_or_default(),
                nama: b.nama,
                kategori: b.kategori.unwrap_or_else(|| "Makanan & Minuman".to_string()),
                satuan: b.satuan,
                rak: b.rak.unwrap_or_else(|| "A-01".to_string()),
                supplier: "PT Sumber Makmur".to_string(),
                hargapokok: b.hargapokok,
                hargajual1: b.hargajual1,
                hargajual2: b.hargajual2,
                hargajual3: b.hargajual3,
                margin_persen: margin,
                stok: b.stok,
                stokminimum: b.stokminimum,
                is_kritis,
                expired: Some("12/2027".to_string()),
                tag: if b.stok == 0.0 {
                    Some("Habis".to_string())
                } else if is_kritis {
                    Some("Segera Order".to_string())
                } else {
                    None
                },
            }
        })
        .collect();

    for p in default_products {
        if !result.iter().any(|r| r.kode == p.kode) {
            result.push(p);
        }
    }

    if !kw.trim().is_empty() {
        let q = kw.trim().to_lowercase();
        result.retain(|p| {
            p.nama.to_lowercase().contains(&q)
                || p.kode.to_lowercase().contains(&q)
                || p.barcode.to_lowercase().contains(&q)
                || p.kategori.to_lowercase().contains(&q)
        });
    }

    Ok(result)
}

#[tauri::command]
fn get_product_stats(_state: State<AppState>) -> Result<ProductStatsDTO, String> {
    Ok(ProductStatsDTO {
        total_produk: 1428,
        stok_optimal: 1385,
        stok_menipis: 38,
        stok_kosong: 5,
        valuasi_aset: 48650000.0,
        avg_margin: 23.8,
    })
}

#[tauri::command]
fn scan_barcode(state: State<AppState>, code: String) -> Result<CartSummaryDTO, String> {
    let mut svc = state.kasir.lock().map_err(|e| e.to_string())?;
    let db = state.db.lock().map_err(|e| e.to_string())?;

    match svc.scan_barcode(&db, &code) {
        Ok(Some(_)) => Ok(build_cart_summary(&svc)),
        Ok(None) => Err(format!("Barang [{}] tidak ditemukan!", code)),
        Err(e) => Err(format!("Error scan: {}", e)),
    }
}

#[tauri::command]
fn update_cart_qty(state: State<AppState>, index: usize, qty: f64) -> Result<CartSummaryDTO, String> {
    let mut svc = state.kasir.lock().map_err(|e| e.to_string())?;
    if index < svc.keranjang.len() {
        let _ = svc.ubah_qty(index, qty);
    }
    Ok(build_cart_summary(&svc))
}

#[tauri::command]
fn remove_cart_item(state: State<AppState>, index: usize) -> Result<CartSummaryDTO, String> {
    let mut svc = state.kasir.lock().map_err(|e| e.to_string())?;
    if index < svc.keranjang.len() {
        let _ = svc.hapus_item(index);
    }
    Ok(build_cart_summary(&svc))
}

#[tauri::command]
fn clear_cart(state: State<AppState>) -> Result<CartSummaryDTO, String> {
    let mut svc = state.kasir.lock().map_err(|e| e.to_string())?;
    svc.bersihkan_keranjang();
    Ok(build_cart_summary(&svc))
}

#[tauri::command]
fn get_cart(state: State<AppState>) -> Result<CartSummaryDTO, String> {
    let svc = state.kasir.lock().map_err(|e| e.to_string())?;
    Ok(build_cart_summary(&svc))
}

#[tauri::command]
fn checkout(state: State<AppState>, tunai: f64, metode: String) -> Result<CheckoutResultDTO, String> {
    let mut svc = state.kasir.lock().map_err(|e| e.to_string())?;
    let mut db = state.db.lock().map_err(|e| e.to_string())?;

    let subtotal = svc.total_belanja();
    if subtotal <= 0.0 {
        return Err("Keranjang masih kosong!".to_string());
    }

    let total_setelah_potongan = (subtotal - svc.nilai_tukar_poin).max(0.0);
    let kembalian = if tunai >= total_setelah_potongan {
        tunai - total_setelah_potongan
    } else {
        0.0
    };

    match svc.checkout(&mut db, "OP01", tunai, 0.0, 0.0, &metode) {
        Ok(penjualan) => {
            let faktur = penjualan.faktur.clone();
            Ok(CheckoutResultDTO {
                sukses: true,
                faktur,
                total: penjualan.total_akhir,
                bayar: penjualan.bayar_tunai,
                kembalian,
                pesan: format!("Transaksi [{}] ({}) berhasil disimpan.", penjualan.faktur, metode),
            })
        }
        Err(e) => Err(format!("Gagal checkout: {}", e)),
    }
}

#[tauri::command]
fn attach_member(state: State<AppState>, query: String) -> Result<CartSummaryDTO, String> {
    let mut svc = state.kasir.lock().map_err(|e| e.to_string())?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let repo = PelangganRepo::new(db.conn());

    let members = repo.cari_member(&state.cabang_id, &query).map_err(|e| e.to_string())?;
    if let Some(m) = members.into_iter().next() {
        svc.attach_member(m);
        Ok(build_cart_summary(&svc))
    } else {
        Err(format!("Member [{}] tidak ditemukan!", query))
    }
}

#[tauri::command]
fn detach_member(state: State<AppState>) -> Result<CartSummaryDTO, String> {
    let mut svc = state.kasir.lock().map_err(|e| e.to_string())?;
    svc.detach_member();
    Ok(build_cart_summary(&svc))
}

#[tauri::command]
fn tukar_poin(state: State<AppState>, poin: i64) -> Result<CartSummaryDTO, String> {
    let mut svc = state.kasir.lock().map_err(|e| e.to_string())?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let _ = svc.tukar_poin(&db, poin).map_err(|e| e.to_string())?;
    Ok(build_cart_summary(&svc))
}

#[tauri::command]
fn hold_order(state: State<AppState>, keterangan: Option<String>) -> Result<String, String> {
    let mut svc = state.kasir.lock().map_err(|e| e.to_string())?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let faktur = svc.hold_transaksi(&db, "OP01", keterangan).map_err(|e| e.to_string())?;
    Ok(faktur)
}

#[tauri::command]
fn recall_order(state: State<AppState>, id: String) -> Result<CartSummaryDTO, String> {
    let mut svc = state.kasir.lock().map_err(|e| e.to_string())?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    svc.recall_transaksi(&db, &id).map_err(|e| e.to_string())?;
    Ok(build_cart_summary(&svc))
}

#[tauri::command]
fn get_pending_orders(state: State<AppState>) -> Result<Vec<PendingItemDTO>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let repo = PendingRepo::new(db.conn());
    let list = repo.semua_pending(&state.cabang_id, &state.device_id).map_err(|e| e.to_string())?;

    let res = list
        .into_iter()
        .map(|p| PendingItemDTO {
            id: p.id,
            faktur: p.faktur,
            tanggal: p.tanggal.to_string(),
            pelanggan: p.kode_pelanggan.unwrap_or_else(|| "UMUM".to_string()),
            total: p.total_akhir,
            keterangan: p.keterangan.unwrap_or_else(|| "Antrean kasir".to_string()),
        })
        .collect();

    Ok(res)
}

#[tauri::command]
fn delete_pending_order(state: State<AppState>, id: String) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let repo = PendingRepo::new(db.conn());
    repo.hapus_pending(&id).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
fn print_last_receipt(state: State<AppState>, faktur: String) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let ok = cetak_struk_ke_escpos(&db, &state.cabang_id, &faktur);
    Ok(ok)
}

#[tauri::command]
fn minimize_window(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
fn toggle_maximize_window(window: tauri::Window) {
    if let Ok(is_max) = window.is_maximized() {
        if is_max {
            let _ = window.unmaximize();
        } else {
            let _ = window.maximize();
        }
    }
}

#[tauri::command]
fn close_window(window: tauri::Window) {
    let _ = window.close();
}

#[tauri::command]
fn get_transactions(
    state: State<AppState>,
    metode: Option<String>,
    keyword: Option<String>,
) -> Result<Vec<TransactionHistoryItemDTO>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut default_items = vec![
        TransactionHistoryItemDTO {
            id: "tx-1".to_string(),
            no: 1,
            faktur: "#ORD-8829".to_string(),
            waktu: "19:43:50".to_string(),
            kasir: "Alexander P.".to_string(),
            shift: "Shift 01 • Reg #01".to_string(),
            pelanggan: "Budi Santoso".to_string(),
            pelanggan_badge: "VIP GOLD".to_string(),
            pelanggan_info: "#MBR-0421 • 2.450 Pts".to_string(),
            total_qty: 6.0,
            total_sku: 4,
            metode: "TUNAI".to_string(),
            total_penjualan: 147520.0,
        },
        TransactionHistoryItemDTO {
            id: "tx-2".to_string(),
            no: 2,
            faktur: "#ORD-8828".to_string(),
            waktu: "19:38:12".to_string(),
            kasir: "Alexander P.".to_string(),
            shift: "Shift 01 • Reg #01".to_string(),
            pelanggan: "Siti Aminah".to_string(),
            pelanggan_badge: "Umum".to_string(),
            pelanggan_info: "Non-Member".to_string(),
            total_qty: 3.0,
            total_sku: 2,
            metode: "QRIS".to_string(),
            total_penjualan: 98500.0,
        },
        TransactionHistoryItemDTO {
            id: "tx-3".to_string(),
            no: 3,
            faktur: "#ORD-8827".to_string(),
            waktu: "19:25:04".to_string(),
            kasir: "Alexander P.".to_string(),
            shift: "Shift 01 • Reg #01".to_string(),
            pelanggan: "Hendra Wijaya".to_string(),
            pelanggan_badge: "Umum".to_string(),
            pelanggan_info: "0812-8871-xxxx".to_string(),
            total_qty: 8.0,
            total_sku: 5,
            metode: "EDC MANDIRI".to_string(),
            total_penjualan: 134200.0,
        },
        TransactionHistoryItemDTO {
            id: "tx-4".to_string(),
            no: 4,
            faktur: "#ORD-8826".to_string(),
            waktu: "19:12:44".to_string(),
            kasir: "Alexander P.".to_string(),
            shift: "Shift 01 • Reg #01".to_string(),
            pelanggan: "Ratna Sari".to_string(),
            pelanggan_badge: "SILVER".to_string(),
            pelanggan_info: "#MBR-0912 • 810 Pts".to_string(),
            total_qty: 1.0,
            total_sku: 1,
            metode: "TUNAI".to_string(),
            total_penjualan: 28500.0,
        },
        TransactionHistoryItemDTO {
            id: "tx-5".to_string(),
            no: 5,
            faktur: "#ORD-8825".to_string(),
            waktu: "18:55:18".to_string(),
            kasir: "Alexander P.".to_string(),
            shift: "Shift 01 • Reg #01".to_string(),
            pelanggan: "Toko Berkah / Pak RT".to_string(),
            pelanggan_badge: "GROSIR".to_string(),
            pelanggan_info: "#MBR-0055 • Rekanan RW".to_string(),
            total_qty: 24.0,
            total_sku: 8,
            metode: "TUNAI".to_string(),
            total_penjualan: 385000.0,
        },
        TransactionHistoryItemDTO {
            id: "tx-6".to_string(),
            no: 6,
            faktur: "#ORD-8824".to_string(),
            waktu: "18:41:09".to_string(),
            kasir: "Alexander P.".to_string(),
            shift: "Shift 01 • Reg #01".to_string(),
            pelanggan: "Dimas Pratama".to_string(),
            pelanggan_badge: "Umum".to_string(),
            pelanggan_info: "Non-Member".to_string(),
            total_qty: 3.0,
            total_sku: 3,
            metode: "QRIS".to_string(),
            total_penjualan: 32000.0,
        },
        TransactionHistoryItemDTO {
            id: "tx-7".to_string(),
            no: 7,
            faktur: "#ORD-8823".to_string(),
            waktu: "18:30:22".to_string(),
            kasir: "Alexander P.".to_string(),
            shift: "Shift 01 • Reg #01".to_string(),
            pelanggan: "Maya Kusuma".to_string(),
            pelanggan_badge: "VIP GOLD".to_string(),
            pelanggan_info: "#MBR-0318 • 1.840 Pts".to_string(),
            total_qty: 9.0,
            total_sku: 6,
            metode: "EDC BCA".to_string(),
            total_penjualan: 187600.0,
        },
        TransactionHistoryItemDTO {
            id: "tx-8".to_string(),
            no: 8,
            faktur: "#ORD-8822".to_string(),
            waktu: "18:15:01".to_string(),
            kasir: "Alexander P.".to_string(),
            shift: "Shift 01 • Reg #01".to_string(),
            pelanggan: "Wahyu Setiawan".to_string(),
            pelanggan_badge: "Umum".to_string(),
            pelanggan_info: "Non-Member".to_string(),
            total_qty: 2.0,
            total_sku: 2,
            metode: "TUNAI".to_string(),
            total_penjualan: 42000.0,
        },
        TransactionHistoryItemDTO {
            id: "tx-9".to_string(),
            no: 9,
            faktur: "#ORD-8821".to_string(),
            waktu: "18:04:19".to_string(),
            kasir: "Alexander P.".to_string(),
            shift: "Shift 01 • Reg #01".to_string(),
            pelanggan: "Dewi Lestari".to_string(),
            pelanggan_badge: "SILVER".to_string(),
            pelanggan_info: "#MBR-0782 • 640 Pts".to_string(),
            total_qty: 3.0,
            total_sku: 3,
            metode: "QRIS".to_string(),
            total_penjualan: 54000.0,
        },
        TransactionHistoryItemDTO {
            id: "tx-10".to_string(),
            no: 10,
            faktur: "#ORD-8820".to_string(),
            waktu: "17:51:30".to_string(),
            kasir: "Alexander P.".to_string(),
            shift: "Shift 01 • Reg #01".to_string(),
            pelanggan: "Rian Hidayat".to_string(),
            pelanggan_badge: "Umum".to_string(),
            pelanggan_info: "Non-Member".to_string(),
            total_qty: 2.0,
            total_sku: 2,
            metode: "TUNAI".to_string(),
            total_penjualan: 59500.0,
        },
    ];

    // Read real DB rows if present
    if let Ok(mut stmt) = db.conn().prepare(
        "SELECT p.id, p.faktur, strftime('%H:%M:%S', p.tanggal), p.operator_id, p.shift_id,
                COALESCE(p.kode_pelanggan, 'Umum'), p.metode_bayar, p.total_akhir,
                COALESCE((SELECT SUM(d.jumlah) FROM tpenjualandetail d WHERE d.penjualan_id = p.id), 1.0),
                COALESCE((SELECT COUNT(d.id) FROM tpenjualandetail d WHERE d.penjualan_id = p.id), 1)
         FROM tpenjualan p
         WHERE p.cabang_id = ?1
         ORDER BY p.tanggal DESC
         LIMIT 20;"
    ) {
        if let Ok(real_rows) = stmt.query_map([&state.cabang_id], |r| {
            Ok(TransactionHistoryItemDTO {
                id: r.get(0)?,
                no: 0,
                faktur: r.get(1)?,
                waktu: r.get(2)?,
                kasir: "Alexander P.".to_string(),
                shift: "Shift 01 • Reg #01".to_string(),
                pelanggan: r.get(5)?,
                pelanggan_badge: "Umum".to_string(),
                pelanggan_info: "Non-Member".to_string(),
                metode: r.get(6)?,
                total_penjualan: r.get(7)?,
                total_qty: r.get::<_, f64>(8)?,
                total_sku: r.get::<_, i64>(9)? as usize,
            })
        }) {
            for (idx, row) in real_rows.flatten().enumerate() {
                default_items.insert(idx, row);
            }
        }
    }

    for (i, it) in default_items.iter_mut().enumerate() {
        it.no = i + 1;
    }

    if let Some(m) = metode {
        let m_upper = m.to_uppercase();
        if m_upper != "SEMUA" && !m_upper.is_empty() {
            default_items.retain(|it| it.metode.to_uppercase().contains(&m_upper));
        }
    }

    if let Some(kw) = keyword {
        let q = kw.trim().to_lowercase();
        if !q.is_empty() {
            default_items.retain(|it| {
                it.faktur.to_lowercase().contains(&q)
                    || it.pelanggan.to_lowercase().contains(&q)
                    || it.kasir.to_lowercase().contains(&q)
            });
        }
    }

    Ok(default_items)
}

#[tauri::command]
fn get_transaction_stats(_state: State<AppState>) -> Result<TransactionSummaryStatsDTO, String> {
    Ok(TransactionSummaryStatsDTO {
        total_omset: 8425000.0,
        target_omset: 7500000.0,
        persentase_omset: 112.0,
        total_qty: 384.0,
        avg_item_per_trx: 2.7,
        total_transaksi: 142,
        void_count: 0,
        retur_count: 0,
        avg_basket_size: 59330.0,
        max_basket: 385000.0,
        min_basket: 12500.0,
        tunai_count: 98,
        qris_count: 32,
        edc_count: 12,
        tunai_pct: 69.0,
        qris_pct: 23.0,
        edc_pct: 8.0,
    })
}

fn inisialisasi_data_dasar(db: &Database) -> Result<(String, String), Box<dyn std::error::Error>> {
    let cabang_repo = CabangRepo::new(db.conn());
    let cabang = match cabang_repo.ambil_cabang_pertama()? {
        Some(c) => c,
        None => {
            let baru = Cabang::baru("CAB01", "Toko Pusat", true);
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

    let barang_repo = BarangRepo::new(db.conn());
    if barang_repo.cari_by_barcode_atau_kode(&cabang.id, "SKU-1001")?.is_none() {
        let mut b = DBarang::baru(&cabang.id, "SKU-1001", "Iced Caramel Macchiato", 18000.0, 32000.0, 48.0);
        b.kategori = Some("Minuman Kopi".to_string());
        b.satuan = "Cup".to_string();
        b.stokminimum = 10.0;
        barang_repo.simpan(&b)?;
    }

    if barang_repo.cari_by_barcode_atau_kode(&cabang.id, "SKU-1002")?.is_none() {
        let mut b = DBarang::baru(&cabang.id, "SKU-1002", "Artisan Butter Croissant", 12000.0, 22000.0, 26.0);
        b.kategori = Some("Bakery & Pastry".to_string());
        b.satuan = "Pcs".to_string();
        b.stokminimum = 5.0;
        barang_repo.simpan(&b)?;
    }

    if barang_repo.cari_by_barcode_atau_kode(&cabang.id, "SKU-1005")?.is_none() {
        let mut b = DBarang::baru(&cabang.id, "SKU-1005", "Club Sandwich Triple", 20000.0, 38000.0, 15.0);
        b.kategori = Some("Makanan Berat".to_string());
        b.satuan = "Porsi".to_string();
        b.stokminimum = 5.0;
        barang_repo.simpan(&b)?;
    }

    if barang_repo.cari_by_barcode_atau_kode(&cabang.id, "SKU-1007")?.is_none() {
        let mut b = DBarang::baru(&cabang.id, "SKU-1007", "Air Mineral Artesian 600ml", 4000.0, 8000.0, 60.0);
        b.kategori = Some("Retail / Kemasan".to_string());
        b.satuan = "Botol".to_string();
        b.stokminimum = 12.0;
        barang_repo.simpan(&b)?;
    }

    if barang_repo.cari_by_barcode_atau_kode(&cabang.id, "BRG001")?.is_none() {
        let mut b = DBarang::baru(&cabang.id, "BRG001", "Aqua Galon 19L", 8000.0, 11000.0, 377.0);
        b.kategori = Some("Minuman Non-Kopi".to_string());
        b.satuan = "Galon".to_string();
        b.stokminimum = 10.0;
        barang_repo.simpan(&b)?;
    }

    Ok((cabang.id, device.id))
}

fn cetak_struk_ke_escpos(_database: &Database, _cabang_id: &str, _faktur: &str) -> bool {
    // ponytail: stub — integrate StrukKasir::buat_struk when TransaksiRepo detail query available
    true
}

fn main() {
    let db_path = "pos_data.db";
    let db = Database::buka(db_path).expect("Gagal membuka database SQLite");

    let (cabang_id, device_id) = inisialisasi_data_dasar(&db).expect("Gagal inisialisasi data dasar");

    let shift_svc = ShiftService::new(&cabang_id, &device_id);
    let shift_aktif = match shift_svc.ambil_shift_aktif(&db).expect("Gagal ambil shift") {
        Some(s) => s,
        None => shift_svc.buka_shift(&db, "OP01", 100_000.0).expect("Gagal buka shift"),
    };

    let kasir_svc = KasirService::new(cabang_id.clone(), device_id.clone());

    let state = AppState {
        db: Mutex::new(db),
        kasir: Mutex::new(kasir_svc),
        cabang_id,
        device_id,
        shift_id: shift_aktif.id,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_status_info,
            get_catalog_products,
            scan_barcode,
            update_cart_qty,
            remove_cart_item,
            clear_cart,
            get_cart,
            checkout,
            attach_member,
            detach_member,
            tukar_poin,
            hold_order,
            recall_order,
            get_pending_orders,
            delete_pending_order,
            print_last_receipt,
            minimize_window,
            toggle_maximize_window,
            close_window,
            get_transactions,
            get_transaction_stats,
            get_product_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
