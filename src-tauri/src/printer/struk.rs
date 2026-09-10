use crate::domain::cabang::Cabang;
use crate::domain::transaksi::{TPenjualan, TPenjualanDetail};
use crate::printer::escpos::EscPosBuilder;

pub struct StrukKasir;

impl StrukKasir {
    /// Format receipt lengkap transaksi penjualan ke dalam byte stream ESC/POS
    pub fn buat_struk(
        cabang: &Cabang,
        penjualan: &TPenjualan,
        details: &[TPenjualanDetail],
        is_58mm: bool,
    ) -> Vec<u8> {
        let mut builder = if is_58mm {
            EscPosBuilder::new_58mm()
        } else {
            EscPosBuilder::new_80mm()
        };

        // Buka laci kasir otomatis
        builder.buka_laci_kasir();

        // Header Toko (Rata Tengah & Tebal)
        builder
            .rata_tengah()
            .ukuran_ganda(true)
            .tebal(true)
            .baris(&cabang.nama)
            .ukuran_ganda(false)
            .tebal(false);

        if let Some(ref alamat) = cabang.alamat {
            builder.baris(alamat);
        }
        if let Some(ref telp) = cabang.telepon {
            builder.baris(&format!("Telp: {}", telp));
        }

        // Info Transaksi
        builder.garis_pemisah().rata_kiri();
        builder.dua_kolom("No. Faktur", &penjualan.faktur);
        builder.dua_kolom(
            "Tanggal",
            &penjualan.tanggal.format("%d/%m/%Y %H:%M").to_string(),
        );
        builder.dua_kolom("Kasir", &penjualan.operator_id);
        builder.dua_kolom("Pelanggan", &penjualan.kode_pelanggan);
        builder.garis_pemisah();

        // Daftar Barang
        for item in details {
            builder.baris(&item.nama_barang);
            let qty_harga = format!("{} {} x {}", item.jumlah, item.satuan, item.hargajual);
            let subtotal = format!("Rp {}", item.subtotal);
            builder.dua_kolom(&format!("  {}", qty_harga), &subtotal);

            if item.diskon_rp > 0.0 {
                builder.dua_kolom("  Diskon", &format!("-Rp {}", item.diskon_rp));
            }
        }

        // Ringkasan Pembayaran
        builder.garis_pemisah();
        builder.dua_kolom("Subtotal", &format!("Rp {}", penjualan.subtotal));

        if penjualan.diskon_rp > 0.0 {
            builder.dua_kolom("Diskon Transaksi", &format!("-Rp {}", penjualan.diskon_rp));
        }
        if penjualan.nilai_tukar_poin > 0.0 {
            builder.dua_kolom("Tukar Poin", &format!("-Rp {}", penjualan.nilai_tukar_poin));
        }

        builder
            .tebal(true)
            .dua_kolom("TOTAL", &format!("Rp {}", penjualan.total_akhir))
            .tebal(false);

        builder.dua_kolom(
            &format!("Bayar ({})", penjualan.metode_bayar),
            &format!("Rp {}", penjualan.bayar_tunai + penjualan.bayar_nontunai),
        );
        builder.dua_kolom("Kembalian", &format!("Rp {}", penjualan.kembalian));

        // Footer Struk
        builder
            .garis_pemisah()
            .rata_tengah()
            .baris("Terima Kasih Atas Kunjungan Anda")
            .baris("Barang yang dibeli tidak dapat ditukar")
            .baris_baru()
            .potong_kertas();

        builder.ambil_bytes().to_vec()
    }
}
