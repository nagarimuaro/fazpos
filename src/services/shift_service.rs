use crate::db::Database;
use crate::domain::shift::TShift;
use crate::repository::shift_repo::ShiftRepo;

pub struct ShiftService {
    pub cabang_id: String,
    pub device_id: String,
}

impl ShiftService {
    pub fn new(cabang_id: impl Into<String>, device_id: impl Into<String>) -> Self {
        Self {
            cabang_id: cabang_id.into(),
            device_id: device_id.into(),
        }
    }

    /// Ambil shift yang saat ini sedang aktif
    pub fn ambil_shift_aktif(&self, db: &Database) -> Result<Option<TShift>, String> {
        let repo = ShiftRepo::new(db.conn());
        repo.ambil_shift_aktif(&self.cabang_id, &self.device_id)
            .map_err(|e| format!("Gagal membaca status shift: {}", e))
    }

    /// Buka shift baru untuk kasir di device ini
    pub fn buka_shift(
        &self,
        db: &Database,
        operator_id: &str,
        modal_awal: f64,
    ) -> Result<TShift, String> {
        let repo = ShiftRepo::new(db.conn());

        // Pastikan tidak ada shift yang masih menggantung/open
        if let Some(aktif) = repo
            .ambil_shift_aktif(&self.cabang_id, &self.device_id)
            .map_err(|e| format!("Database error: {}", e))?
        {
            return Err(format!(
                "Shift masih aktif (ID: {}). Tutup shift lama terlebih dahulu.",
                aktif.id
            ));
        }

        let shift = TShift::buka(&self.cabang_id, &self.device_id, operator_id, modal_awal);
        repo.buka_shift(&shift)
            .map_err(|e| format!("Gagal membuka shift: {}", e))?;

        Ok(shift)
    }

    /// Tutup shift dengan input uang aktual kasir
    pub fn tutup_shift(
        &self,
        db: &Database,
        shift_id: &str,
        uang_aktual: f64,
        catatan: Option<&str>,
    ) -> Result<TShift, String> {
        let repo = ShiftRepo::new(db.conn());

        let mut shift = repo
            .ambil_shift_aktif(&self.cabang_id, &self.device_id)
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "Tidak ditemukan shift aktif untuk ditutup".to_string())?;

        if shift.id != shift_id {
            return Err("ID shift tidak sesuai dengan shift yang sedang aktif".to_string());
        }

        shift.tutup(uang_aktual, catatan.map(|s| s.to_string()));

        repo.tutup_shift(
            &shift.id,
            uang_aktual,
            shift.uang_seharusnya,
            shift.selisih.unwrap_or(0.0),
            catatan,
        )
        .map_err(|e| format!("Gagal mengupdate penutupan shift: {}", e))?;

        Ok(shift)
    }
}
