use crate::domain::operator::DOperator;
use crate::importer::types::konversi_bool;

pub struct OperatorMapper;

impl OperatorMapper {
    pub fn petakan(
        cabang_id: &str,
        kolom_names: Option<&[String]>,
        row_values: &[String],
    ) -> Result<DOperator, String> {
        if row_values.is_empty() {
            return Err("Baris operator kosong".to_string());
        }

        let mut kode = String::new();
        let mut nama = String::new();
        let mut password_hash = "default123".to_string();
        let mut role = "kasir".to_string();
        let mut is_aktif = true;

        if let Some(cols) = kolom_names {
            for (idx, col) in cols.iter().enumerate() {
                if idx >= row_values.len() {
                    break;
                }
                let val = &row_values[idx];
                match col.as_str() {
                    "kode" | "user" | "username" => kode = val.clone(),
                    "nama" | "nama_lengkap" => nama = val.clone(),
                    "password" | "password_hash" => if !val.is_empty() { password_hash = val.clone(); },
                    "role" | "jabatan" => if !val.is_empty() { role = val.to_lowercase(); },
                    "is_aktif" | "aktif" => is_aktif = konversi_bool(val),
                    _ => {}
                }
            }
        } else if row_values.len() >= 2 {
            kode = row_values[0].clone();
            nama = row_values[1].clone();
            if row_values.len() > 2 { password_hash = row_values[2].clone(); }
        }

        if kode.trim().is_empty() {
            return Err("Kode/username operator tidak boleh kosong".to_string());
        }

        let mut op = DOperator::baru(cabang_id, kode, nama, password_hash, role);
        op.is_aktif = is_aktif;
        Ok(op)
    }
}
