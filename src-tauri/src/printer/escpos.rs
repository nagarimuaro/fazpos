/// Builder byte stream ESC/POS native untuk printer thermal kasir (58mm / 80mm)
pub struct EscPosBuilder {
    buffer: Vec<u8>,
    lebar_karakter: usize, // 32 untuk 58mm, 48 untuk 80mm
}

impl EscPosBuilder {
    pub fn new_58mm() -> Self {
        let mut builder = Self {
            buffer: Vec::new(),
            lebar_karakter: 32,
        };
        builder.inisialisasi();
        builder
    }

    pub fn new_80mm() -> Self {
        let mut builder = Self {
            buffer: Vec::new(),
            lebar_karakter: 48,
        };
        builder.inisialisasi();
        builder
    }

    /// Reset / Inisialisasi printer (ESC @)
    pub fn inisialisasi(&mut self) -> &mut Self {
        self.buffer.extend_from_slice(&[0x1B, 0x40]);
        self
    }

    /// Rata kiri (ESC a 0)
    pub fn rata_kiri(&mut self) -> &mut Self {
        self.buffer.extend_from_slice(&[0x1B, 0x61, 0x00]);
        self
    }

    /// Rata tengah (ESC a 1)
    pub fn rata_tengah(&mut self) -> &mut Self {
        self.buffer.extend_from_slice(&[0x1B, 0x61, 0x01]);
        self
    }

    /// Rata kanan (ESC a 2)
    pub fn rata_kanan(&mut self) -> &mut Self {
        self.buffer.extend_from_slice(&[0x1B, 0x61, 0x02]);
        self
    }

    /// Mode tebal on/off (ESC E n)
    pub fn tebal(&mut self, aktif: bool) -> &mut Self {
        self.buffer.extend_from_slice(&[0x1B, 0x45, if aktif { 1 } else { 0 }]);
        self
    }

    /// Ukuran font ganda (GS ! n)
    pub fn ukuran_ganda(&mut self, aktif: bool) -> &mut Self {
        self.buffer.extend_from_slice(&[0x1D, 0x21, if aktif { 0x11 } else { 0x00 }]);
        self
    }

    /// Cetak teks biasa
    pub fn teks(&mut self, text: &str) -> &mut Self {
        self.buffer.extend_from_slice(text.as_bytes());
        self
    }

    /// Cetak teks diakhiri baris baru
    pub fn baris(&mut self, text: &str) -> &mut Self {
        self.buffer.extend_from_slice(text.as_bytes());
        self.buffer.push(b'\n');
        self
    }

    /// Baris baru kosong
    pub fn baris_baru(&mut self) -> &mut Self {
        self.buffer.push(b'\n');
        self
    }

    /// Cetak garis pemisah horizontal penuh sesuai lebar kertas
    pub fn garis_pemisah(&mut self) -> &mut Self {
        let garis = "-".repeat(self.lebar_karakter);
        self.baris(&garis);
        self
    }

    /// Cetak format 2 kolom: kiri rata kiri, kanan rata kanan (misal: "Total" ... "Rp 50.000")
    pub fn dua_kolom(&mut self, kiri: &str, kanan: &str) -> &mut Self {
        let len_kiri = kiri.chars().count();
        let len_kanan = kanan.chars().count();
        let total_len = len_kiri + len_kanan;

        if total_len >= self.lebar_karakter {
            // Jika melebihi lebar, cetak di baris terpisah
            self.baris(kiri);
            let spasi = " ".repeat(self.lebar_karakter.saturating_sub(len_kanan));
            self.baris(&format!("{}{}", spasi, kanan));
        } else {
            let spasi = " ".repeat(self.lebar_karakter - total_len);
            self.baris(&format!("{}{}{}", kiri, spasi, kanan));
        }
        self
    }

    /// Buka laci kasir (Cash Drawer Kick — ESC p 0 25 250)
    pub fn buka_laci_kasir(&mut self) -> &mut Self {
        self.buffer.extend_from_slice(&[0x1B, 0x70, 0x00, 0x19, 0xFA]);
        self
    }

    /// Feed kertas beberapa baris lalu potong kertas (GS V 66 n)
    pub fn potong_kertas(&mut self) -> &mut Self {
        self.buffer.extend_from_slice(&[b'\n', b'\n', b'\n', 0x1D, 0x56, 0x42, 0x00]);
        self
    }

    /// Ambil seluruh bytes ESC/POS
    pub fn ambil_bytes(&self) -> &[u8] {
        &self.buffer
    }
}
