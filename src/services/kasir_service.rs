use crate::db::Database;
use crate::domain::pelanggan::DPelanggan;
use crate::domain::transaksi::{KeranjangItem, TPenjualan, TPenjualanPending};
use crate::repository::barang_repo::BarangRepo;
use crate::repository::hutang_piutang_repo::HutangPiutangRepo;
use crate::repository::pelanggan_repo::PelangganRepo;
use crate::repository::pending_repo::PendingRepo;
use crate::repository::poin_repo::PoinRepo;
use crate::repository::shift_repo::ShiftRepo;
use crate::repository::transaksi_repo::TransaksiRepo;
use chrono::Utc;

pub struct KasirService {
    pub cabang_id: String,
    pub device_id: String,
    pub keranjang: Vec<KeranjangItem>,
    pub member_terpilih: Option<DPelanggan>,
    pub poin_ditukar: i64,
    pub nilai_tukar_poin: f64,
}

impl KasirService {
    pub fn new(cabang_id: impl Into<String>, device_id: impl Into<String>) -> Self {
        Self {
            cabang_id: cabang_id.into(),
            device_id: device_id.into(),
            keranjang: Vec::new(),
            member_terpilih: None,
            poin_ditukar: 0,
            nilai_tukar_poin: 0.0,
        }
    }

    /// Pasang pelanggan/member ke transaksi
    pub fn attach_member(&mut self, member: DPelanggan) {
        self.member_terpilih = Some(member);
        self.poin_ditukar = 0;
        self.nilai_tukar_poin = 0.0;
    }

    /// Lepas member dari transaksi kasir
    pub fn detach_member(&mut self) {
        self.member_terpilih = None;
        self.poin_ditukar = 0;
        self.nilai_tukar_poin = 0.0;
    }

    /// Tukar poin member menjadi potongan nilai belanja
    pub fn tukar_poin(&mut self, db: &Database, poin: i64) -> Result<f64, String> {
        let member = match &self.member_terpilih {
            Some(m) => m,
            None => return Err("Belum ada member yang dipilih".to_string()),
        };

        if poin <= 0 {
            self.poin_ditukar = 0;
            self.nilai_tukar_poin = 0.0;
            return Ok(0.0);
        }

        let poin_repo = PoinRepo::new(db.conn());
        let setting = poin_repo
            .ambil_pengaturan(&self.cabang_id)
            .map_err(|e| format!("Gagal membaca pengaturan poin: {}", e))?;

        if !setting.is_aktif {
            return Err("Sistem loyalty poin sedang dinonaktifkan".to_string());
        }

        if poin > member.poin_saldo {
            return Err(format!(
                "Poin tidak mencukupi! Saldo poin: {}, diminta: {}",
                member.poin_saldo, poin
            ));
        }

        if poin < setting.minimal_tukar {
            return Err(format!(
                "Minimal penukaran poin adalah {} poin",
                setting.minimal_tukar
            ));
        }

        let nilai = (poin as f64) * setting.nilai_tukar_poin;
        let subtotal = self.total_belanja();
        if nilai > subtotal {
            return Err(format!(
                "Nilai tukar poin (Rp {}) melebihi total belanja (Rp {})",
                nilai, subtotal
            ));
        }

        self.poin_ditukar = poin;
        self.nilai_tukar_poin = nilai;
        Ok(nilai)
    }

    /// Scan barcode atau ketik kode barang, tambahkan ke keranjang
    pub fn scan_barcode(&mut self, db: &Database, kode_atau_barcode: &str) -> Result<Option<KeranjangItem>, String> {
        let repo = BarangRepo::new(db.conn());
        let barang_opt = repo
            .cari_by_barcode_atau_kode(&self.cabang_id, kode_atau_barcode)
            .map_err(|e| format!("Database error: {}", e))?;

        let barang = match barang_opt {
            Some(b) => b,
            None => return Ok(None),
        };

        // Jika sudah ada di keranjang, tambah qty +1
        if let Some(existing) = self.keranjang.iter_mut().find(|i| i.barang_id == barang.id) {
            existing.jumlah += 1.0;
            existing.hitung_ulang_subtotal();
            return Ok(Some(existing.clone()));
        }

        // Jika belum ada, buat item baru
        let item = KeranjangItem::baru(
            barang.id,
            barang.kode,
            barang.nama,
            barang.satuan,
            barang.hargajual1,
            barang.hargapokok,
            1.0,
        );
        self.keranjang.push(item.clone());
        Ok(Some(item))
    }

    /// Ubah jumlah item di keranjang
    pub fn ubah_qty(&mut self, index: usize, jumlah_baru: f64) -> Result<(), String> {
        if index >= self.keranjang.len() {
            return Err("Item tidak ditemukan di keranjang".to_string());
        }

        if jumlah_baru <= 0.0 {
            self.keranjang.remove(index);
        } else {
            self.keranjang[index].jumlah = jumlah_baru;
            self.keranjang[index].hitung_ulang_subtotal();
        }
        Ok(())
    }

    /// Hapus item dari keranjang
    pub fn hapus_item(&mut self, index: usize) -> Result<(), String> {
        if index >= self.keranjang.len() {
            return Err("Index di luar jangkauan".to_string());
        }
        self.keranjang.remove(index);
        Ok(())
    }

    /// Hitung subtotal seluruh keranjang
    pub fn total_belanja(&self) -> f64 {
        self.keranjang.iter().map(|i| i.subtotal).sum()
    }

    /// Kosongkan keranjang
    pub fn bersihkan_keranjang(&mut self) {
        self.keranjang.clear();
        self.member_terpilih = None;
        self.poin_ditukar = 0;
        self.nilai_tukar_poin = 0.0;
    }

    /// Simpan transaksi sebagai Pending (Hold Antrean)
    pub fn hold_transaksi(
        &mut self,
        db: &Database,
        operator_id: &str,
        keterangan: Option<String>,
    ) -> Result<String, String> {
        if self.keranjang.is_empty() {
            return Err("Keranjang kasir masih kosong, tidak ada yang perlu ditahan".to_string());
        }

        let shift_repo = ShiftRepo::new(db.conn());
        let shift_aktif = shift_repo
            .ambil_shift_aktif(&self.cabang_id, &self.device_id)
            .map_err(|e| format!("Gagal cek shift aktif: {}", e))?
            .ok_or_else(|| "Tidak ada shift kasir yang aktif.".to_string())?;

        let rand_suffix = &uuid::Uuid::new_v4().to_string()[..4].to_uppercase();
        let faktur = format!("HLD-{}-{}", Utc::now().format("%Y%m%d%H%M%S"), rand_suffix);

        let kode_pelanggan = self.member_terpilih.as_ref().map(|m| m.kode.clone());

        let (header, details) = TPenjualanPending::buat_pending(
            &self.cabang_id,
            &self.device_id,
            &shift_aktif.id,
            &faktur,
            operator_id,
            kode_pelanggan,
            &self.keranjang,
            keterangan,
        );

        let pending_repo = PendingRepo::new(db.conn());
        pending_repo
            .simpan_pending(&header, &details)
            .map_err(|e| format!("Gagal menyimpan pending kasir: {}", e))?;

        // Kosongkan keranjang setelah di-hold
        self.bersihkan_keranjang();

        Ok(faktur)
    }

    /// Panggil kembali transaksi yang di-hold (Recall)
    pub fn recall_transaksi(
        &mut self,
        db: &Database,
        pending_id: &str,
    ) -> Result<(), String> {
        let pending_repo = PendingRepo::new(db.conn());
        let details = pending_repo
            .ambil_detail(pending_id)
            .map_err(|e| format!("Gagal mengambil detail pending: {}", e))?;

        if details.is_empty() {
            return Err("Item pending tidak ditemukan atau sudah kosong".to_string());
        }

        // Ambil header pending untuk cek member
        let all_pending = pending_repo
            .semua_pending(&self.cabang_id, &self.device_id)
            .map_err(|e| format!("Gagal cek pending: {}", e))?;

        let header_opt = all_pending.into_iter().find(|p| p.id == pending_id);

        self.bersihkan_keranjang();

        for d in &details {
            self.keranjang.push(KeranjangItem::from(d));
        }

        if let Some(header) = header_opt {
            if let Some(kode_pel) = header.kode_pelanggan {
                let pel_repo = PelangganRepo::new(db.conn());
                if let Ok(Some(member)) = pel_repo.cari_by_kode(&self.cabang_id, &kode_pel) {
                    self.attach_member(member);
                }
            }
        }

        // Hapus dari pending setelah dipanggil ke keranjang
        pending_repo
            .hapus_pending(pending_id)
            .map_err(|e| format!("Gagal menghapus pending kasir: {}", e))?;

        Ok(())
    }

    /// Eksekusi checkout transaksi kasir
    pub fn checkout(
        &mut self,
        db: &mut Database,
        operator_id: &str,
        bayar_tunai: f64,
        bayar_nontunai: f64,
        diskon_rp: f64,
        metode_bayar: &str,
    ) -> Result<TPenjualan, String> {
        if self.keranjang.is_empty() {
            return Err("Keranjang belanja masih kosong".to_string());
        }

        // Cek shift kasir aktif (aturan 6 SKILL.MD)
        let shift_repo = ShiftRepo::new(db.conn());
        let shift_aktif = shift_repo
            .ambil_shift_aktif(&self.cabang_id, &self.device_id)
            .map_err(|e| format!("Gagal cek shift aktif: {}", e))?
            .ok_or_else(|| "Tidak ada shift kasir yang aktif. Buka shift terlebih dahulu.".to_string())?;

        let total_subtotal = self.total_belanja();
        let total_akhir = (total_subtotal - diskon_rp - self.nilai_tukar_poin).max(0.0);
        let total_bayar = bayar_tunai + bayar_nontunai;

        if metode_bayar != "KREDIT" && total_bayar < total_akhir {
            return Err(format!(
                "Pembayaran kurang! Total: Rp {}, Dibayar: Rp {}",
                total_akhir, total_bayar
            ));
        }

        // Generate nomor faktur: format PJ-YYYYMMDDHHMMSS-XXXX (unik per milidetik)
        let rand_suffix = &uuid::Uuid::new_v4().to_string()[..4].to_uppercase();
        let faktur = format!("PJ-{}-{}", Utc::now().format("%Y%m%d%H%M%S"), rand_suffix);

        let kode_pelanggan = self
            .member_terpilih
            .as_ref()
            .map(|m| m.kode.as_str())
            .unwrap_or("UMUM");

        // Buat objek header dan detail
        let (mut header, details) = TPenjualan::buat_transaksi(
            &self.cabang_id,
            &self.device_id,
            &shift_aktif.id,
            &faktur,
            operator_id,
            kode_pelanggan,
            &self.keranjang,
            diskon_rp,
            self.nilai_tukar_poin,
            bayar_tunai,
            bayar_nontunai,
            metode_bayar,
        );

        header.poin_ditukar = self.poin_ditukar;
        header.nilai_tukar_poin = self.nilai_tukar_poin;

        // Loyalty Member Poin Processing
        let mut poin_dapat = 0i64;
        if let Some(member) = &self.member_terpilih {
            let poin_repo = PoinRepo::new(db.conn());
            if let Ok(setting) = poin_repo.ambil_pengaturan(&self.cabang_id) {
                if setting.is_aktif && setting.rupiah_per_poin > 0.0 {
                    poin_dapat = (total_akhir / setting.rupiah_per_poin) as i64;
                }
            }

            // Kurangi poin yang ditukar
            if self.poin_ditukar > 0 {
                let _ = poin_repo.kurangi_poin(
                    &self.cabang_id,
                    &member.id,
                    &faktur,
                    self.poin_ditukar,
                    &format!("Tukar poin pada transaksi {}", faktur),
                );
            }

            // Tambahkan poin yang didapat
            if poin_dapat > 0 {
                let _ = poin_repo.tambah_poin(
                    &self.cabang_id,
                    &member.id,
                    &faktur,
                    poin_dapat,
                    &format!("Poin dari belanja transaksi {}", faktur),
                );
            }
        }
        header.poin_didapat = poin_dapat;

        // Jika metode KREDIT, catat ke tpiutang
        if metode_bayar == "KREDIT" {
            let nama_pelanggan = self
                .member_terpilih
                .as_ref()
                .map(|m| m.nama.as_str())
                .unwrap_or("Pelanggan Umum");
            let pelanggan_id = self
                .member_terpilih
                .as_ref()
                .map(|m| m.id.as_str())
                .unwrap_or("UMUM");
            let hp_repo = HutangPiutangRepo::new(db.conn());
            hp_repo
                .catat_piutang_baru(
                    &self.cabang_id,
                    &faktur,
                    pelanggan_id,
                    nama_pelanggan,
                    total_akhir,
                    total_bayar,
                    &Utc::now().format("%Y-%m-%d").to_string(),
                    &format!("Piutang transaksi kasir {}", faktur),
                )
                .map_err(|e| format!("Gagal mencatat piutang: {}", e))?;
        }

        // Simpan atomik (tpenjualan, tpenjualandetail, update stok dbarang, log keluarmasuk, update tshift)
        let mut tx_repo = TransaksiRepo::new(db.conn_mut());
        tx_repo
            .simpan_transaksi_atomic(&header, &details)
            .map_err(|e| format!("Gagal menyimpan transaksi kasir: {}", e))?;

        // Kosongkan keranjang belanja setelah berhasil checkout
        self.bersihkan_keranjang();

        Ok(header)
    }
}
