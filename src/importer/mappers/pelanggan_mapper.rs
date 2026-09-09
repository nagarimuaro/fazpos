use crate::domain::pelanggan::DPelanggan;
use crate::importer::types::{konversi_bool, konversi_f64, konversi_i64};

pub struct PelangganMapper;

impl PelangganMapper {
    pub fn petakan(
        cabang_id: &str,
        kolom_names: Option<&[String]>,
        row_values: &[String],
    ) -> Result<DPelanggan, String> {
        if row_values.is_empty() {
            return Err("Baris pelanggan kosong".to_string());
        }

        let mut kode = String::new();
        let mut nama = String::new();
        let mut alamat = None;
        let mut telepon = None;
        let mut plafonpiutang = 0.0;
        let mut poin_saldo = 0;
        let mut id_kartu = None;
        let mut is_aktif = true;

        if let Some(cols) = kolom_names {
            for (idx, col) in cols.iter().enumerate() {
                if idx >= row_values.len() {
                    break;
                }
                let val = &row_values[idx];
                match col.as_str() {
                    "kode" => kode = val.clone(),
                    "nama" => nama = val.clone(),
                    "alamat" => if !val.is_empty() { alamat = Some(val.clone()); },
                    "telepon" | "no_hp" | "hp" => if !val.is_empty() { telepon = Some(val.clone()); },
                    "plafonpiutang" | "plafon" => plafonpiutang = konversi_f64(val),
                    "poin_saldo" | "poin" => poin_saldo = konversi_i64(val),
                    "id_kartu" | "kartu" => if !val.is_empty() { id_kartu = Some(val.clone()); },
                    "is_aktif" | "aktif" => is_aktif = konversi_bool(val),
                    _ => {}
                }
            }
        } else if row_values.len() >= 2 {
            kode = row_values[0].clone();
            nama = row_values[1].clone();
            if row_values.len() > 2 && !row_values[2].is_empty() { alamat = Some(row_values[2].clone()); }
            if row_values.len() > 3 && !row_values[3].is_empty() { telepon = Some(row_values[3].clone()); }
        }

        if kode.trim().is_empty() {
            return Err("Kode pelanggan tidak boleh kosong".to_string());
        }

        let mut p = DPelanggan::baru(cabang_id, kode, nama);
        p.alamat = alamat;
        p.telepon = telepon;
        p.plafonpiutang = plafonpiutang;
        p.poin_saldo = poin_saldo;
        p.id_kartu = id_kartu;
        p.is_aktif = is_aktif;

        Ok(p)
    }
}
