use crate::db::Database;
use crate::domain::transaksi::TPenjualan;
use crate::lan::protocol::{LanCatalogItem, LanCheckoutRequest, LanCheckoutResponse, LanPingResponse};
use crate::repository::barang_repo::BarangRepo;
use crate::repository::transaksi_repo::TransaksiRepo;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::Utc;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct LanServerState {
    pub cabang_id: String,
    pub device_id: String,
    pub db: Arc<Mutex<Database>>,
}

/// Buat Axum Router untuk melayani komunikasi LAN antar kasir
pub fn buat_lan_router(state: LanServerState) -> Router {
    Router::new()
        .route("/api/v1/ping", get(handle_ping))
        .route("/api/v1/catalog", get(handle_catalog))
        .route("/api/v1/checkout", post(handle_checkout))
        .with_state(state)
}

async fn handle_ping(State(state): State<LanServerState>) -> Json<LanPingResponse> {
    Json(LanPingResponse {
        status: "ok".to_string(),
        cabang_id: state.cabang_id,
        device_id: state.device_id,
        role: "server".to_string(),
        versi: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn handle_catalog(State(state): State<LanServerState>) -> (StatusCode, Json<Vec<LanCatalogItem>>) {
    let db = match state.db.lock() {
        Ok(guard) => guard,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new())),
    };

    let repo = BarangRepo::new(db.conn());
    match repo.cari_by_nama(&state.cabang_id, "", 1000) {
        Ok(items) => {
            let list = items
                .into_iter()
                .map(|b| LanCatalogItem {
                    id: b.id,
                    kode: b.kode,
                    barcode: b.barcode,
                    nama: b.nama,
                    satuan: b.satuan,
                    hargajual1: b.hargajual1,
                    stok: b.stok,
                })
                .collect();
            (StatusCode::OK, Json(list))
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new())),
    }
}

async fn handle_checkout(
    State(state): State<LanServerState>,
    Json(payload): Json<LanCheckoutRequest>,
) -> (StatusCode, Json<LanCheckoutResponse>) {
    let mut db = match state.db.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LanCheckoutResponse {
                    sukses: false,
                    faktur: None,
                    total_akhir: 0.0,
                    kembalian: 0.0,
                    pesan: Some("Database mutex lock error".to_string()),
                }),
            )
        }
    };

    if payload.items.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(LanCheckoutResponse {
                sukses: false,
                faktur: None,
                total_akhir: 0.0,
                kembalian: 0.0,
                pesan: Some("Keranjang belanja kosong".to_string()),
            }),
        );
    }

    let subtotal: f64 = payload.items.iter().map(|i| i.subtotal).sum();
    let (total_akhir, kembalian) = TPenjualan::hitung_ringkasan(
        subtotal,
        payload.diskon_rp,
        0.0,
        payload.bayar_tunai,
        payload.bayar_nontunai,
    );

    let rand_suffix = &uuid::Uuid::new_v4().to_string()[..4].to_uppercase();
    let faktur = format!("PJ-{}-{}", Utc::now().format("%Y%m%d%H%M%S"), rand_suffix);

    let (header, details) = TPenjualan::buat_transaksi(
        &state.cabang_id,
        &payload.device_id,
        &payload.shift_id,
        &faktur,
        &payload.operator_id,
        "UMUM",
        &payload.items,
        payload.diskon_rp,
        0.0,
        payload.bayar_tunai,
        payload.bayar_nontunai,
        &payload.metode_bayar,
    );

    let mut tx_repo = TransaksiRepo::new(db.conn_mut());
    match tx_repo.simpan_transaksi_atomic(&header, &details) {
        Ok(_) => (
            StatusCode::OK,
            Json(LanCheckoutResponse {
                sukses: true,
                faktur: Some(faktur),
                total_akhir,
                kembalian,
                pesan: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(LanCheckoutResponse {
                sukses: false,
                faktur: None,
                total_akhir,
                kembalian,
                pesan: Some(format!("Gagal simpan transaksi: {}", e)),
            }),
        ),
    }
}
