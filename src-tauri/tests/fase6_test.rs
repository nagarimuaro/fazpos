use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use fazpos::db::Database;
use fazpos::domain::barang::DBarang;
use fazpos::domain::cabang::{Cabang, Device};
use fazpos::domain::transaksi::KeranjangItem;
use fazpos::lan::protocol::{LanCheckoutRequest, LanCheckoutResponse, LanPingResponse};
use fazpos::lan::server::{buat_lan_router, LanServerState};
use fazpos::repository::barang_repo::BarangRepo;
use fazpos::repository::cabang_repo::CabangRepo;
use fazpos::services::shift_service::ShiftService;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

#[tokio::test]
async fn test_lan_server_ping() {
    let db = Database::buka_in_memory().unwrap();
    let state = LanServerState {
        cabang_id: "CB01".to_string(),
        device_id: "SRV01".to_string(),
        db: Arc::new(Mutex::new(db)),
    };

    let app = buat_lan_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/ping")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let ping: LanPingResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(ping.status, "ok");
    assert_eq!(ping.role, "server");
    assert_eq!(ping.cabang_id, "CB01");
}

#[tokio::test]
async fn test_lan_server_remote_checkout_dari_client() {
    let db = Database::buka_in_memory().unwrap();

    // Setup Cabang, Server Device, Client Device
    let cabang = Cabang::baru("CB01", "Toko Pusat", true);
    CabangRepo::new(db.conn()).simpan_cabang(&cabang).unwrap();

    let srv_dev = Device::baru(&cabang.id, "SRV01", "PC Server", "server", "M-SRV");
    let cli_dev = Device::baru(&cabang.id, "CLI01", "Kasir 2", "client", "M-CLI");
    CabangRepo::new(db.conn()).simpan_device(&srv_dev).unwrap();
    CabangRepo::new(db.conn()).simpan_device(&cli_dev).unwrap();

    // Buka shift untuk Client Device CLI01
    let shift_svc = ShiftService::new(&cabang.id, &cli_dev.id);
    let shift = shift_svc.buka_shift(&db, "KASIR2", 100_000.0).unwrap();

    // Tambah barang di server (Stok awal = 20)
    let b1_id: String;
    {
        let repo = BarangRepo::new(db.conn());
        let mut b = DBarang::baru(&cabang.id, "BRG-SNACK", "Chiki Snack", 1000.0, 2000.0, 20.0);
        b.barcode = Some("899888".to_string());
        repo.simpan(&b).unwrap();
        b1_id = b.id;
    }

    let state = LanServerState {
        cabang_id: cabang.id.clone(),
        device_id: srv_dev.id.clone(),
        db: Arc::new(Mutex::new(db)),
    };

    let app = buat_lan_router(state.clone());

    // Client PC mengirim request checkout via LAN (Beli 3 pcs Chiki = 6.000)
    let checkout_req = LanCheckoutRequest {
        device_id: cli_dev.id.clone(),
        operator_id: "KASIR2".to_string(),
        shift_id: shift.id.clone(),
        items: vec![KeranjangItem::baru(
            &b1_id,
            "BRG-SNACK",
            "Chiki Snack",
            "PCS",
            2000.0,
            1000.0,
            3.0,
        )],
        bayar_tunai: 10_000.0,
        bayar_nontunai: 0.0,
        diskon_rp: 0.0,
        metode_bayar: "TUNAI".to_string(),
    };

    let req_body = serde_json::to_vec(&checkout_req).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/checkout")
                .header("Content-Type", "application/json")
                .body(Body::from(req_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let resp_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let resp: LanCheckoutResponse = serde_json::from_slice(&resp_bytes).unwrap();

    assert!(resp.sukses);
    assert_eq!(resp.total_akhir, 6000.0);
    assert_eq!(resp.kembalian, 4000.0);
    assert!(resp.faktur.unwrap().starts_with("PJ-"));

    // Verifikasi stok di Server berkurang (20 - 3 = 17)
    {
        let db_lock = state.db.lock().unwrap();
        let repo = BarangRepo::new(db_lock.conn());
        let snack = repo.cari_by_id(&b1_id).unwrap().unwrap();
        assert_eq!(snack.stok, 17.0);
    }
}
