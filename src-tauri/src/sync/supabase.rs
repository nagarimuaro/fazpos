use chrono::Utc;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;
use crate::db::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseConfig {
    pub url: String,
    pub api_key: String,
    pub is_bound: bool,
    pub auto_sync: bool,
    pub interval_menit: u32,
    pub last_sync_waktu: Option<String>,
    pub last_sync_status: Option<String>,
}

impl Default for SupabaseConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            api_key: String::new(),
            is_bound: false,
            auto_sync: true,
            interval_menit: 5,
            last_sync_waktu: None,
            last_sync_status: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncTableDetail {
    pub tabel: String,
    pub jumlah_dikirim: usize,
    pub sukses: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseSyncResult {
    pub sukses: bool,
    pub total_dikirim: usize,
    pub total_berhasil: usize,
    pub total_gagal: usize,
    pub detail_per_tabel: Vec<SyncTableDetail>,
    pub durasi_ms: u64,
    pub pesan: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncLogItem {
    pub id: String,
    pub waktu: String,
    pub tabel: String,
    pub jumlah_record: i64,
    pub status: String,
    pub pesan: Option<String>,
    pub durasi_ms: Option<i64>,
}

pub struct BatchTableData {
    pub rows: Vec<serde_json::Value>,
    pub ids: Vec<String>,
}

pub struct FullBatchData {
    pub cabang: BatchTableData,
    pub device: BatchTableData,
    pub barang: BatchTableData,
    pub pelanggan: BatchTableData,
    pub shift: BatchTableData,
    pub penjualan: BatchTableData,
    pub detail: BatchTableData,
    pub mutasi: BatchTableData,
}

pub struct SupabaseClient;

impl SupabaseClient {
    fn build_headers(key: &str) -> Result<HeaderMap, String> {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "apikey",
            HeaderValue::from_str(key).map_err(|e| e.to_string())?,
        );
        let auth_val = format!("Bearer {}", key);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_val).map_err(|e| e.to_string())?,
        );
        headers.insert(
            "Prefer",
            HeaderValue::from_static("resolution=merge-duplicates"),
        );
        Ok(headers)
    }

    /// Bersihkan URL Supabase agar tidak berakhiran slash
    fn normalize_url(url: &str) -> String {
        let trimmed = url.trim();
        if let Some(stripped) = trimmed.strip_suffix('/') {
            stripped.to_string()
        } else {
            trimmed.to_string()
        }
    }

    /// Tes koneksi ke Supabase PostgREST
    pub async fn test_connection(url: &str, key: &str) -> Result<u64, String> {
        let base_url = Self::normalize_url(url);
        if base_url.is_empty() {
            return Err("URL Supabase belum diisi".to_string());
        }
        if key.trim().is_empty() {
            return Err("API Key Supabase belum diisi".to_string());
        }

        let endpoint = format!("{}/rest/v1/cabang?select=count&limit=1", base_url);
        let headers = Self::build_headers(key)?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        let start = std::time::Instant::now();
        let resp = client
            .get(&endpoint)
            .headers(headers)
            .send()
            .await
            .map_err(|e| format!("Koneksi gagal: {}", e))?;

        let latency = start.elapsed().as_millis() as u64;

        match resp.status().as_u16() {
            200..=299 => Ok(latency),
            401 | 403 => Err("Otorisasi gagal: API Key tidak valid atau tidak memiliki izin".to_string()),
            404 => Err("Tabel belum ditemukan. Silakan jalankan Skema SQL di Supabase SQL Editor terlebih dahulu".to_string()),
            status => {
                let err_text = resp.text().await.unwrap_or_default();
                Err(format!("Supabase error (HTTP {}): {}", status, err_text))
            }
        }
    }

    /// Push batch array of records ke Supabase PostgREST table dengan upsert
    pub async fn push_records(
        url: &str,
        key: &str,
        table: &str,
        rows: &serde_json::Value,
    ) -> Result<(), String> {
        let base_url = Self::normalize_url(url);
        let endpoint = format!("{}/rest/v1/{}", base_url, table);
        let headers = Self::build_headers(key)?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|e| e.to_string())?;

        let resp = client
            .post(&endpoint)
            .headers(headers)
            .json(rows)
            .send()
            .await
            .map_err(|e| format!("Gagal kirim data ke {}: {}", table, e))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(format!("HTTP {} pada tabel {}: {}", status, table, body))
        }
    }

    /// Tarik data dari endpoint PostgREST Supabase
    pub async fn pull_records(
        url: &str,
        key: &str,
        table: &str,
        query: &str,
    ) -> Result<serde_json::Value, String> {
        let base_url = Self::normalize_url(url);
        let endpoint = if query.is_empty() {
            format!("{}/rest/v1/{}", base_url, table)
        } else {
            format!("{}/rest/v1/{}?{}", base_url, table, query)
        };

        let mut headers = HeaderMap::new();
        headers.insert(
            "apikey",
            HeaderValue::from_str(key).map_err(|e| e.to_string())?,
        );
        let auth_val = format!("Bearer {}", key);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_val).map_err(|e| e.to_string())?,
        );
        headers.insert(
            "accept",
            HeaderValue::from_static("application/json"),
        );

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|e| e.to_string())?;

        let resp = client
            .get(&endpoint)
            .headers(headers)
            .send()
            .await
            .map_err(|e| format!("Gagal ambil data dari {}: {}", table, e))?;

        if resp.status().is_success() {
            let json_val = resp.json::<serde_json::Value>().await.map_err(|e| e.to_string())?;
            Ok(json_val)
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(format!("HTTP {} pada tabel {}: {}", status, table, body))
        }
    }

    /// Ambil daftar cabang dari Supabase
    pub async fn pull_cabang_list(url: &str, key: &str) -> Result<Vec<serde_json::Value>, String> {
        let val = Self::pull_records(url, key, "cabang", "select=*&order=created_at.asc").await?;
        if let Some(arr) = val.as_array() {
            Ok(arr.clone())
        } else {
            Ok(Vec::new())
        }
    }

    /// Push cabang baru ke Supabase Cloud
    pub async fn push_cabang_to_cloud(
        url: &str,
        key: &str,
        c: &crate::domain::cabang::Cabang,
    ) -> Result<(), String> {
        let payload = serde_json::json!([{
            "id": c.id,
            "kode": c.kode,
            "nama": c.nama,
            "alamat": c.alamat,
            "telepon": c.telepon,
            "is_pusat": c.is_pusat,
            "updated_at": chrono::Utc::now().to_rfc3339()
        }]);
        Self::push_records(url, key, "cabang", &payload).await
    }

    /// Tarik master produk & pelanggan dari Supabase Cloud dan simpan ke SQLite lokal
    pub async fn pull_master_catalog(
        url: &str,
        key: &str,
        db_arc: &Arc<Mutex<Database>>,
        target_cabang_id: &str,
    ) -> Result<u32, String> {
        // 1. Ambil semua data via HTTP dari Supabase tanpa menyentuh DB SQLite
        let cabang_list = Self::pull_cabang_list(url, key).await.unwrap_or_default();
        let goods_val = Self::pull_records(url, key, "dbarang", "select=*").await?;
        let goods_arr = goods_val.as_array().cloned().unwrap_or_default();
        let cust_arr = match Self::pull_records(url, key, "dpelanggan", "select=*").await {
            Ok(v) => v.as_array().cloned().unwrap_or_default(),
            Err(_) => Vec::new(),
        };

        // 2. Simpan secara sinkronus ke DB SQLite dalam 1 blok lock cepat tanpa ada .await
        let mut total_imported = 0u32;
        {
            let db = db_arc.lock().map_err(|e| e.to_string())?;
            let conn = db.conn();

            let c_repo = crate::repository::cabang_repo::CabangRepo::new(conn);
            for c_val in &cabang_list {
                if let (Some(id), Some(kode), Some(nama)) = (
                    c_val.get("id").and_then(|v| v.as_str()),
                    c_val.get("kode").and_then(|v| v.as_str()),
                    c_val.get("nama").and_then(|v| v.as_str()),
                ) {
                    let is_pusat = c_val.get("is_pusat").and_then(|v| v.as_bool()).unwrap_or(false);
                    let mut c = crate::domain::cabang::Cabang::baru(kode, nama, is_pusat);
                    c.id = id.to_string();
                    c.alamat = c_val.get("alamat").and_then(|v| v.as_str()).map(|s| s.to_string());
                    c.telepon = c_val.get("telepon").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let _ = c_repo.simpan_cabang(&c);
                }
            }

            let b_repo = crate::repository::barang_repo::BarangRepo::new(conn);
            for g in &goods_arr {
                if let (Some(kode), Some(nama)) = (
                    g.get("kode").and_then(|v| v.as_str()),
                    g.get("nama").and_then(|v| v.as_str()),
                ) {
                    let hargapokok = g.get("hargapokok").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let hargajual1 = g.get("hargajual1").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let stok = g.get("stok").and_then(|v| v.as_f64()).unwrap_or(0.0);

                    let mut b = crate::domain::barang::DBarang::baru(target_cabang_id, kode, nama, hargapokok, hargajual1, stok);
                    if let Some(id) = g.get("id").and_then(|v| v.as_str()) {
                        b.id = id.to_string();
                    }
                    b.barcode = g.get("barcode").and_then(|v| v.as_str()).map(|s| s.to_string());
                    b.satuan = g.get("satuan").and_then(|v| v.as_str()).unwrap_or("PCS").to_string();
                    b.kategori = g.get("kategori").and_then(|v| v.as_str()).map(|s| s.to_string());
                    b.rak = g.get("rak").and_then(|v| v.as_str()).map(|s| s.to_string());
                    b.hargajual2 = g.get("hargajual2").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    b.hargajual3 = g.get("hargajual3").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    b.hargajual4 = g.get("hargajual4").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    b.hargapartai = g.get("hargapartai").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    b.stokminimum = g.get("stokminimum").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    b.is_aktif = g.get("is_aktif").and_then(|v| v.as_bool()).unwrap_or(true);
                    b.sync_status = "synced".to_string();

                    if b_repo.simpan(&b).is_ok() {
                        total_imported += 1;
                    }
                }
            }

            let p_repo = crate::repository::pelanggan_repo::PelangganRepo::new(conn);
            for c in &cust_arr {
                if let (Some(kode), Some(nama)) = (
                    c.get("kode").and_then(|v| v.as_str()),
                    c.get("nama").and_then(|v| v.as_str()),
                ) {
                    let mut p = crate::domain::pelanggan::DPelanggan::baru(target_cabang_id, kode, nama);
                    if let Some(id) = c.get("id").and_then(|v| v.as_str()) {
                        p.id = id.to_string();
                    }
                    p.alamat = c.get("alamat").and_then(|v| v.as_str()).map(|s| s.to_string());
                    p.telepon = c.get("telepon").and_then(|v| v.as_str()).map(|s| s.to_string());
                    p.id_kartu = c.get("id_kartu").and_then(|v| v.as_str()).map(|s| s.to_string());
                    p.plafonpiutang = c.get("plafonpiutang").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    p.poin_saldo = c.get("poin_saldo").and_then(|v| v.as_i64()).unwrap_or(0);
                    p.is_aktif = c.get("is_aktif").and_then(|v| v.as_bool()).unwrap_or(true);
                    p.sync_status = "synced".to_string();
                    let _ = p_repo.simpan(&p);
                }
            }

            Self::catat_log(conn, "dbarang", total_imported as usize, "sukses", Some("Tarik master produk dari Supabase"), None);
        }

        Ok(total_imported)
    }

    /// Kumpulkan semua record pending dari SQLite lokal secara sinkronus cepat
    fn kumpulkan_pending(conn: &Connection, cabang_id: &str) -> Result<FullBatchData, String> {
        // 1. Cabang
        let cabang = {
            let mut stmt = conn
                .prepare("SELECT id, kode, nama, alamat, telepon, is_pusat FROM cabang WHERE sync_status = 'pending' LIMIT 100")
                .map_err(|e| e.to_string())?;
            let mut rows = Vec::new();
            let mut ids = Vec::new();
            let iter = stmt
                .query_map([], |r| {
                    let id: String = r.get(0)?;
                    let is_pusat: i32 = r.get(5)?;
                    Ok((
                        id.clone(),
                        json!({
                            "id": id,
                            "kode": r.get::<_, String>(1)?,
                            "nama": r.get::<_, String>(2)?,
                            "alamat": r.get::<_, Option<String>>(3)?,
                            "telepon": r.get::<_, Option<String>>(4)?,
                            "is_pusat": is_pusat == 1,
                        }),
                    ))
                })
                .map_err(|e| e.to_string())?;
            for item in iter.flatten() {
                ids.push(item.0);
                rows.push(item.1);
            }
            BatchTableData { rows, ids }
        };

        // 2. Device
        let device = {
            let mut stmt = conn
                .prepare("SELECT id, cabang_id, kode, nama, role, machine_id, ip_address, is_active FROM device WHERE cabang_id = ? LIMIT 100")
                .map_err(|e| e.to_string())?;
            let mut rows = Vec::new();
            let mut ids = Vec::new();
            let iter = stmt
                .query_map([cabang_id], |r| {
                    let id: String = r.get(0)?;
                    let is_active: i32 = r.get(7)?;
                    Ok((
                        id.clone(),
                        json!({
                            "id": id,
                            "cabang_id": r.get::<_, String>(1)?,
                            "kode": r.get::<_, String>(2)?,
                            "nama": r.get::<_, String>(3)?,
                            "role": r.get::<_, String>(4)?,
                            "machine_id": r.get::<_, String>(5)?,
                            "ip_address": r.get::<_, Option<String>>(6)?,
                            "is_active": is_active == 1,
                        }),
                    ))
                })
                .map_err(|e| e.to_string())?;
            for item in iter.flatten() {
                ids.push(item.0);
                rows.push(item.1);
            }
            BatchTableData { rows, ids }
        };

        // 3. dbarang
        let barang = {
            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT id, cabang_id, kode, barcode, nama, satuan, kategori, rak,
                           hargapokok, hargajual1, hargajual2, hargajual3, hargajual4,
                           hargapartai, stok, stokminimum, is_aktif
                    FROM dbarang
                    WHERE cabang_id = ? AND sync_status = 'pending'
                    LIMIT 200
                    "#,
                )
                .map_err(|e| e.to_string())?;
            let mut rows = Vec::new();
            let mut ids = Vec::new();
            let iter = stmt
                .query_map([cabang_id], |r| {
                    let id: String = r.get(0)?;
                    let is_aktif: i32 = r.get(16)?;
                    Ok((
                        id.clone(),
                        json!({
                            "id": id,
                            "cabang_id": r.get::<_, String>(1)?,
                            "kode": r.get::<_, String>(2)?,
                            "barcode": r.get::<_, Option<String>>(3)?,
                            "nama": r.get::<_, String>(4)?,
                            "satuan": r.get::<_, String>(5)?,
                            "kategori": r.get::<_, Option<String>>(6)?,
                            "rak": r.get::<_, Option<String>>(7)?,
                            "hargapokok": r.get::<_, f64>(8)?,
                            "hargajual1": r.get::<_, f64>(9)?,
                            "hargajual2": r.get::<_, f64>(10)?,
                            "hargajual3": r.get::<_, f64>(11)?,
                            "hargajual4": r.get::<_, f64>(12)?,
                            "hargapartai": r.get::<_, f64>(13)?,
                            "stok": r.get::<_, f64>(14)?,
                            "stokminimum": r.get::<_, f64>(15)?,
                            "is_aktif": is_aktif == 1,
                        }),
                    ))
                })
                .map_err(|e| e.to_string())?;
            for item in iter.flatten() {
                ids.push(item.0);
                rows.push(item.1);
            }
            BatchTableData { rows, ids }
        };

        // 4. dpelanggan
        let pelanggan = {
            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT id, cabang_id, kode, nama, alamat, telepon, plafonpiutang, poin_saldo, id_kartu, is_aktif
                    FROM dpelanggan
                    WHERE cabang_id = ? AND sync_status = 'pending'
                    LIMIT 200
                    "#,
                )
                .map_err(|e| e.to_string())?;
            let mut rows = Vec::new();
            let mut ids = Vec::new();
            let iter = stmt
                .query_map([cabang_id], |r| {
                    let id: String = r.get(0)?;
                    let is_aktif: i32 = r.get(9)?;
                    Ok((
                        id.clone(),
                        json!({
                            "id": id,
                            "cabang_id": r.get::<_, String>(1)?,
                            "kode": r.get::<_, String>(2)?,
                            "nama": r.get::<_, String>(3)?,
                            "alamat": r.get::<_, Option<String>>(4)?,
                            "telepon": r.get::<_, Option<String>>(5)?,
                            "plafonpiutang": r.get::<_, f64>(6)?,
                            "poin_saldo": r.get::<_, i64>(7)?,
                            "id_kartu": r.get::<_, Option<String>>(8)?,
                            "is_aktif": is_aktif == 1,
                        }),
                    ))
                })
                .map_err(|e| e.to_string())?;
            for item in iter.flatten() {
                ids.push(item.0);
                rows.push(item.1);
            }
            BatchTableData { rows, ids }
        };

        // 5. tshift
        let shift = {
            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT id, cabang_id, device_id, operator_id, waktu_buka, waktu_tutup,
                           modal_awal, total_penjualan_tunai, total_penjualan_nontunai,
                           total_retur, total_biaya, uang_seharusnya, uang_aktual,
                           selisih, status, catatan, kode, total_kas_masuk_lain,
                           total_kas_keluar, total_retur_tunai
                    FROM tshift
                    WHERE cabang_id = ? AND sync_status = 'pending'
                    LIMIT 200
                    "#,
                )
                .map_err(|e| e.to_string())?;
            let mut rows = Vec::new();
            let mut ids = Vec::new();
            let iter = stmt
                .query_map([cabang_id], |r| {
                    let id: String = r.get(0)?;
                    Ok((
                        id.clone(),
                        json!({
                            "id": id,
                            "cabang_id": r.get::<_, String>(1)?,
                            "device_id": r.get::<_, String>(2)?,
                            "operator_id": r.get::<_, String>(3)?,
                            "waktu_buka": r.get::<_, String>(4)?,
                            "waktu_tutup": r.get::<_, Option<String>>(5)?,
                            "modal_awal": r.get::<_, f64>(6)?,
                            "total_penjualan_tunai": r.get::<_, f64>(7)?,
                            "total_penjualan_nontunai": r.get::<_, f64>(8)?,
                            "total_retur": r.get::<_, f64>(9)?,
                            "total_biaya": r.get::<_, f64>(10)?,
                            "uang_seharusnya": r.get::<_, f64>(11)?,
                            "uang_aktual": r.get::<_, Option<f64>>(12)?,
                            "selisih": r.get::<_, Option<f64>>(13)?,
                            "status": r.get::<_, Option<String>>(14)?,
                            "catatan": r.get::<_, Option<String>>(15)?,
                            "kode": r.get::<_, Option<String>>(16)?,
                            "total_kas_masuk_lain": r.get::<_, Option<f64>>(17)?.unwrap_or(0.0),
                            "total_kas_keluar": r.get::<_, Option<f64>>(18)?.unwrap_or(0.0),
                            "total_retur_tunai": r.get::<_, Option<f64>>(19)?.unwrap_or(0.0),
                        }),
                    ))
                })
                .map_err(|e| e.to_string())?;
            for item in iter.flatten() {
                ids.push(item.0);
                rows.push(item.1);
            }
            BatchTableData { rows, ids }
        };

        // 6. tpenjualan
        let penjualan = {
            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT id, cabang_id, device_id, shift_id, faktur, tanggal,
                           kode_pelanggan, operator_id, subtotal, diskon_rp, total_akhir,
                           bayar_tunai, bayar_nontunai, kembalian, metode_bayar, status,
                           poin_didapat, poin_ditukar, nilai_tukar_poin
                    FROM tpenjualan
                    WHERE cabang_id = ? AND sync_status = 'pending'
                    LIMIT 200
                    "#,
                )
                .map_err(|e| e.to_string())?;
            let mut rows = Vec::new();
            let mut ids = Vec::new();
            let iter = stmt
                .query_map([cabang_id], |r| {
                    let id: String = r.get(0)?;
                    Ok((
                        id.clone(),
                        json!({
                            "id": id,
                            "cabang_id": r.get::<_, String>(1)?,
                            "device_id": r.get::<_, String>(2)?,
                            "shift_id": r.get::<_, String>(3)?,
                            "faktur": r.get::<_, String>(4)?,
                            "tanggal": r.get::<_, String>(5)?,
                            "kode_pelanggan": r.get::<_, String>(6)?,
                            "operator_id": r.get::<_, String>(7)?,
                            "subtotal": r.get::<_, f64>(8)?,
                            "diskon_rp": r.get::<_, f64>(9)?,
                            "total_akhir": r.get::<_, f64>(10)?,
                            "bayar_tunai": r.get::<_, f64>(11)?,
                            "bayar_nontunai": r.get::<_, f64>(12)?,
                            "kembalian": r.get::<_, f64>(13)?,
                            "metode_bayar": r.get::<_, String>(14)?,
                            "status": r.get::<_, String>(15)?,
                            "poin_didapat": r.get::<_, i64>(16)?,
                            "poin_ditukar": r.get::<_, i64>(17)?,
                            "nilai_tukar_poin": r.get::<_, f64>(18)?,
                        }),
                    ))
                })
                .map_err(|e| e.to_string())?;
            for item in iter.flatten() {
                ids.push(item.0);
                rows.push(item.1);
            }
            BatchTableData { rows, ids }
        };

        // 7. tpenjualandetail
        let detail = {
            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT id, penjualan_id, cabang_id, barang_id, kode_barang, nama_barang,
                           jumlah, satuan, hargajual, hargapokok, diskon_persen, diskon_rp, subtotal
                    FROM tpenjualandetail
                    WHERE cabang_id = ? AND sync_status = 'pending'
                    LIMIT 500
                    "#,
                )
                .map_err(|e| e.to_string())?;
            let mut rows = Vec::new();
            let mut ids = Vec::new();
            let iter = stmt
                .query_map([cabang_id], |r| {
                    let id: String = r.get(0)?;
                    Ok((
                        id.clone(),
                        json!({
                            "id": id,
                            "penjualan_id": r.get::<_, String>(1)?,
                            "cabang_id": r.get::<_, String>(2)?,
                            "barang_id": r.get::<_, String>(3)?,
                            "kode_barang": r.get::<_, String>(4)?,
                            "nama_barang": r.get::<_, String>(5)?,
                            "jumlah": r.get::<_, f64>(6)?,
                            "satuan": r.get::<_, String>(7)?,
                            "hargajual": r.get::<_, f64>(8)?,
                            "hargapokok": r.get::<_, f64>(9)?,
                            "diskon_persen": r.get::<_, f64>(10)?,
                            "diskon_rp": r.get::<_, f64>(11)?,
                            "subtotal": r.get::<_, f64>(12)?,
                        }),
                    ))
                })
                .map_err(|e| e.to_string())?;
            for item in iter.flatten() {
                ids.push(item.0);
                rows.push(item.1);
            }
            BatchTableData { rows, ids }
        };

        // 8. keluarmasuk
        let mutasi = {
            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT id, cabang_id, device_id, barang_id, kode_barang,
                           tanggal, jenis, referensi, masuk, keluar, sisa, keterangan
                    FROM keluarmasuk
                    WHERE cabang_id = ? AND sync_status = 'pending'
                    LIMIT 200
                    "#,
                )
                .map_err(|e| e.to_string())?;
            let mut rows = Vec::new();
            let mut ids = Vec::new();
            let iter = stmt
                .query_map([cabang_id], |r| {
                    let id: String = r.get(0)?;
                    Ok((
                        id.clone(),
                        json!({
                            "id": id,
                            "cabang_id": r.get::<_, String>(1)?,
                            "device_id": r.get::<_, String>(2)?,
                            "barang_id": r.get::<_, String>(3)?,
                            "kode_barang": r.get::<_, String>(4)?,
                            "tanggal": r.get::<_, String>(5)?,
                            "jenis": r.get::<_, String>(6)?,
                            "referensi": r.get::<_, String>(7)?,
                            "masuk": r.get::<_, f64>(8)?,
                            "keluar": r.get::<_, f64>(9)?,
                            "sisa": r.get::<_, f64>(10)?,
                            "keterangan": r.get::<_, Option<String>>(11)?,
                        }),
                    ))
                })
                .map_err(|e| e.to_string())?;
            for item in iter.flatten() {
                ids.push(item.0);
                rows.push(item.1);
            }
            BatchTableData { rows, ids }
        };

        Ok(FullBatchData {
            cabang,
            device,
            barang,
            pelanggan,
            shift,
            penjualan,
            detail,
            mutasi,
        })
    }

    /// Orkestrasi kirim seluruh data pending (8 tabel) ke Supabase Cloud
    pub async fn push_full_batch(
        url: &str,
        key: &str,
        db_arc: &Arc<Mutex<Database>>,
        cabang_id: &str,
    ) -> Result<SupabaseSyncResult, String> {
        let start_time = std::time::Instant::now();
        let mut total_dikirim = 0;
        let mut total_berhasil = 0;
        let mut total_gagal = 0;
        let mut details = Vec::new();

        // 1. Ekstrak data pending secara sinkronus cepat tanpa menahan lock saat HTTP
        let batch = {
            let db = db_arc.lock().map_err(|e| e.to_string())?;
            Self::kumpulkan_pending(db.conn(), cabang_id)?
        };

        // Helper untuk kirim per tabel dan update status
        let tables_to_push = [
            ("cabang", batch.cabang),
            ("device", batch.device),
            ("dbarang", batch.barang),
            ("dpelanggan", batch.pelanggan),
            ("tshift", batch.shift),
            ("tpenjualan", batch.penjualan),
            ("tpenjualandetail", batch.detail),
            ("keluarmasuk", batch.mutasi),
        ];

        for (table_name, data) in tables_to_push {
            if !data.rows.is_empty() {
                let count = data.rows.len();
                total_dikirim += count;
                match Self::push_records(url, key, table_name, &json!(data.rows)).await {
                    Ok(_) => {
                        total_berhasil += count;
                        if let Ok(db) = db_arc.lock() {
                            Self::tandai_synced(db.conn(), table_name, &data.ids);
                            Self::catat_log(db.conn(), table_name, count, "sukses", None, None);
                        }
                        details.push(SyncTableDetail {
                            tabel: table_name.to_string(),
                            jumlah_dikirim: count,
                            sukses: true,
                            error: None,
                        });
                    }
                    Err(e) => {
                        total_gagal += count;
                        if let Ok(db) = db_arc.lock() {
                            Self::catat_log(db.conn(), table_name, count, "gagal", Some(&e), None);
                        }
                        details.push(SyncTableDetail {
                            tabel: table_name.to_string(),
                            jumlah_dikirim: count,
                            sukses: false,
                            error: Some(e),
                        });
                    }
                }
            }
        }

        let durasi_ms = start_time.elapsed().as_millis() as u64;
        let sukses = total_gagal == 0;
        let pesan = if total_dikirim == 0 {
            "Semua data lokal sudah tersinkronisasi (0 pending)".to_string()
        } else if sukses {
            format!("Berhasil sinkronisasi {} record ke Supabase Cloud", total_berhasil)
        } else {
            format!(
                "Sinkronisasi sebagian: {} berhasil, {} gagal",
                total_berhasil, total_gagal
            )
        };

        Ok(SupabaseSyncResult {
            sukses,
            total_dikirim,
            total_berhasil,
            total_gagal,
            detail_per_tabel: details,
            durasi_ms,
            pesan,
        })
    }

    /// Update status baris lokal menjadi 'synced'
    fn tandai_synced(conn: &Connection, table: &str, ids: &[String]) {
        if ids.is_empty() || table == "device" {
            return;
        }
        let now = Utc::now().to_rfc3339();
        for chunk in ids.chunks(50) {
            let placeholders: Vec<String> = chunk.iter().map(|_| "?".to_string()).collect();
            let sql = format!(
                "UPDATE {} SET sync_status = 'synced', sync_at = ? WHERE id IN ({})",
                table,
                placeholders.join(",")
            );
            let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
            params.push(&now);
            for id in chunk {
                params.push(id);
            }
            let _ = conn.execute(&sql, params.as_slice());
        }
    }

    /// Catat riwayat ke tabel sync_log
    pub fn catat_log(
        conn: &Connection,
        table: &str,
        count: usize,
        status: &str,
        pesan: Option<&str>,
        durasi: Option<i64>,
    ) {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let _ = conn.execute(
            "INSERT INTO sync_log (id, waktu, tabel, jumlah_record, status, pesan, durasi_ms) VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![id, now, table, count as i64, status, pesan, durasi],
        );
    }

    /// Hitung total record lokal yang berstatus pending sync
    pub fn count_pending_records(conn: &Connection, cabang_id: &str) -> u64 {
        let tables = [
            ("cabang", "WHERE sync_status = 'pending'"),
            ("dbarang", "WHERE cabang_id = ? AND sync_status = 'pending'"),
            ("dpelanggan", "WHERE cabang_id = ? AND sync_status = 'pending'"),
            ("tshift", "WHERE cabang_id = ? AND sync_status = 'pending'"),
            ("tpenjualan", "WHERE cabang_id = ? AND sync_status = 'pending'"),
            ("tpenjualandetail", "WHERE cabang_id = ? AND sync_status = 'pending'"),
            ("keluarmasuk", "WHERE cabang_id = ? AND sync_status = 'pending'"),
        ];

        let mut total = 0u64;
        for (tbl, filter) in tables {
            let sql = format!("SELECT COUNT(*) FROM {} {}", tbl, filter);
            let count: i64 = if filter.contains('?') {
                conn.query_row(&sql, [cabang_id], |r| r.get(0)).unwrap_or(0)
            } else {
                conn.query_row(&sql, [], |r| r.get(0)).unwrap_or(0)
            };
            total += count.max(0) as u64;
        }
        total
    }

    /// Ambil riwayat log sinkronisasi terakhir
    pub fn ambil_sync_log(conn: &Connection, limit: usize) -> Vec<SyncLogItem> {
        let sql = "SELECT id, waktu, tabel, jumlah_record, status, pesan, durasi_ms FROM sync_log ORDER BY waktu DESC LIMIT ?";
        let mut stmt = match conn.prepare(sql) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let iter = stmt.query_map([limit as i64], |r| {
            Ok(SyncLogItem {
                id: r.get(0)?,
                waktu: r.get(1)?,
                tabel: r.get(2)?,
                jumlah_record: r.get(3)?,
                status: r.get(4)?,
                pesan: r.get(5)?,
                durasi_ms: r.get(6)?,
            })
        });

        match iter {
            Ok(mapped) => mapped.flatten().collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Ambil template SQL DDL untuk Supabase
    pub fn get_supabase_ddl() -> &'static str {
        include_str!("../../../references/supabase-schema.sql")
    }
}
