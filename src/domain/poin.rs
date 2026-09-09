use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PengaturanPoin {
    pub id: String,
    pub cabang_id: String,
    pub rupiah_per_poin: f64,
    pub nilai_tukar_poin: f64,
    pub minimal_tukar: i64,
    pub is_aktif: bool,
}

impl PengaturanPoin {
    pub fn default(cabang_id: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            cabang_id: cabang_id.into(),
            rupiah_per_poin: 10_000.0, // Tiap belanja Rp 10.000 dapat 1 poin
            nilai_tukar_poin: 100.0,   // 1 poin = Rp 100 potongan
            minimal_tukar: 10,         // Minimal tukar 10 poin (Rp 1.000)
            is_aktif: true,
        }
    }

    /// Hitung poin yang didapatkan dari total belanja bersih
    pub fn hitung_perolehan(&self, total_akhir: f64) -> i64 {
        if !self.is_aktif || self.rupiah_per_poin <= 0.0 || total_akhir <= 0.0 {
            return 0;
        }
        (total_akhir / self.rupiah_per_poin).floor() as i64
    }

    /// Validasi apakah poin bisa ditukarkan dan kembalikan nilai rupiahnya
    pub fn hitung_nilai_tukar(&self, saldo_poin: i64, poin_tukar: i64) -> Result<f64, String> {
        if !self.is_aktif {
            return Err("Sistem poin sedang tidak aktif".to_string());
        }
        if poin_tukar <= 0 {
            return Ok(0.0);
        }
        if poin_tukar > saldo_poin {
            return Err(format!(
                "Saldo poin tidak cukup! Saldo: {}, Ditukar: {}",
                saldo_poin, poin_tukar
            ));
        }
        if poin_tukar < self.minimal_tukar {
            return Err(format!(
                "Minimal penukaran poin adalah {} poin!",
                self.minimal_tukar
            ));
        }
        Ok(poin_tukar as f64 * self.nilai_tukar_poin)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiwayatPoin {
    pub id: String,
    pub cabang_id: String,
    pub pelanggan_id: String,
    pub faktur: Option<String>,
    pub tanggal: DateTime<Utc>,
    pub jenis: String, // "DAPAT" | "TUKAR" | "PENYESUAIAN"
    pub jumlah: i64,
    pub saldo_akhir: i64,
    pub keterangan: Option<String>,
}
