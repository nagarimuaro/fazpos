use crate::domain::barang::DBarang;
use crate::importer::types::{konversi_bool, konversi_f64};

pub struct BarangMapper;

impl BarangMapper {
    /// Petakan baris data legacy dbarang ke struct domain DBarang sistem baru
    pub fn petakan(
        cabang_id: &str,
        kolom_names: Option<&[String]>,
        row_values: &[String],
    ) -> Result<DBarang, String> {
        if row_values.is_empty() {
            return Err("Baris data barang kosong".to_string());
        }

        let mut kode = String::new();
        let mut barcode = None;
        let mut nama = String::new();
        let mut satuan = "PCS".to_string();
        let mut kategori = None;
        let mut rak = None;
        let mut hargapokok = 0.0;
        let mut hargajual1 = 0.0;
        let mut hargajual2 = 0.0;
        let mut hargajual3 = 0.0;
        let mut hargajual4 = 0.0;
        let mut hargapartai = 0.0;
        let mut stok = 0.0;
        let mut stokminimum = 0.0;
        let mut is_aktif = true;

        if let Some(cols) = kolom_names {
            // Mapping berdasarkan nama kolom
            for (idx, col_name) in cols.iter().enumerate() {
                if idx >= row_values.len() {
                    break;
                }
                let val = &row_values[idx];
                match col_name.as_str() {
                    "kode" => kode = val.clone(),
                    "barcode" => if !val.is_empty() { barcode = Some(val.clone()); },
                    "nama" | "namabarang" => nama = val.clone(),
                    "satuan" => if !val.is_empty() { satuan = val.clone(); },
                    "kategori" | "kategoribarang" => if !val.is_empty() { kategori = Some(val.clone()); },
                    "rak" | "lokasi" => if !val.is_empty() { rak = Some(val.clone()); },
                    "hargapokok" | "hargabeli" | "harga_beli" => hargapokok = konversi_f64(val),
                    "hargajual" | "hargajual1" | "harga_jual_1" => hargajual1 = konversi_f64(val),
                    "hargajual2" | "harga_jual_2" => hargajual2 = konversi_f64(val),
                    "hargajual3" | "harga_jual_3" => hargajual3 = konversi_f64(val),
                    "hargajual4" | "harga_jual_4" => hargajual4 = konversi_f64(val),
                    "hargapartai" | "hargajual5" | "harga_jual_5" => hargapartai = konversi_f64(val),
                    "stok" => stok = konversi_f64(val),
                    "stokminimum" | "stok_min" => stokminimum = konversi_f64(val),
                    "is_aktif" | "aktif" => is_aktif = konversi_bool(val),
                    _ => {}
                }
            }
        } else {
            // Mapping posisi standar iB Retago 5 jika tanpa header kolom
            if row_values.len() >= 3 {
                kode = row_values[0].clone();
                nama = row_values[1].clone();
                if row_values.len() > 2 { hargapokok = konversi_f64(&row_values[2]); }
                if row_values.len() > 3 { hargajual1 = konversi_f64(&row_values[3]); }
                if row_values.len() > 4 { stok = konversi_f64(&row_values[4]); }
                if row_values.len() > 5 { satuan = row_values[5].clone(); }
                if row_values.len() > 6 { barcode = Some(row_values[6].clone()); }
            }
        }

        if kode.trim().is_empty() {
            return Err("Kode barang tidak boleh kosong".to_string());
        }
        if nama.trim().is_empty() {
            nama = format!("Produk [{}]", kode);
        }

        // Jika tier harga 2..4 nol, fallback ke hargajual1
        if hargajual2 == 0.0 { hargajual2 = hargajual1; }
        if hargajual3 == 0.0 { hargajual3 = hargajual1; }
        if hargajual4 == 0.0 { hargajual4 = hargajual1; }
        if hargapartai == 0.0 { hargapartai = hargajual1; }

        let mut b = DBarang::baru(cabang_id, kode, nama, hargapokok, hargajual1, stok);
        b.barcode = barcode;
        b.satuan = satuan;
        b.kategori = kategori;
        b.rak = rak;
        b.hargajual2 = hargajual2;
        b.hargajual3 = hargajual3;
        b.hargajual4 = hargajual4;
        b.hargapartai = hargapartai;
        b.stokminimum = stokminimum;
        b.is_aktif = is_aktif;

        Ok(b)
    }
}
