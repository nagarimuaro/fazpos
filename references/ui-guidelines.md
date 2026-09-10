# Panduan UI — FAZPOS (Slint, Desktop Native)

Dokumen ini melengkapi `plan.md` dan `skema-database.md`. Isinya: prinsip
desain (anti-template/anti-AI-slop, diadaptasi untuk desktop native), layar
apa saja yang perlu dibuat, pola komponen, style tokens, arsitektur file, dan
cara verifikasi setelah membuat/mengubah UI. Baca ini sebelum membuat atau
mengubah tampilan apa pun di proyek ini.

> **Koreksi stack:** kalau ada referensi lama menyebut Svelte/SvelteKit/HTML —
> itu SALAH untuk proyek ini. FAZPOS pakai **Slint (pure Rust, no web tech,
> desktop native)**. Prinsip desain di bawah sudah diadaptasi dari sana ke
> konteks Slint; jangan mengambil instruksi teknis Svelte-spesifik (SSR,
> hydration, svelte stores, file `.svelte`) dari sumber manapun untuk proyek
> ini.

---

## 1. Prinsip Desain — Anti Template / Anti AI-Slop

Kamu adalah **Senior Product Designer + Senior Frontend Engineer** untuk
aplikasi kasir desktop, bukan generator UI generik. Tujuannya: interface
yang terasa dirancang untuk kebutuhan kasir sungguhan, bukan template.

### 1.1 Desain mengikuti konteks, bukan tren
Semua keputusan visual (warna, tipografi, spacing, radius, layout) harus
menyesuaikan: identitas FAZPOS, target pengguna (kasir toko retail,
kemungkinan pakai touchscreen), konteks bisnis (operasional cepat, data
padat), jenis layar (transaksi vs laporan vs pengaturan), dan alur kerja
kasir. Kalau style tokens di §3 sudah ada, KEMBANGKAN itu — jangan ganti
karakter visual hanya supaya "terlihat modern".

### 1.2 Hindari pola visual generik ("AI-slop")
Jangan otomatis pakai: gradient dekoratif, glassmorphism, blur berlebihan,
shadow berlebihan, elemen dekoratif mengambang, rounded corner di semua
elemen tanpa alasan, card untuk setiap info kecil, kumpulan statistic-card
yang tidak perlu, icon di setiap menu tanpa fungsi, ilustrasi/animasi
dekoratif, efek glow/neon, atau layout "dashboard generik" (sidebar + cards
+ chart tanpa mikir kebutuhan sebenarnya). Boleh dipakai KALAU memang sesuai
kebutuhan — bukan dilarang mutlak, tapi harus ada alasan.

### 1.3 Setiap elemen harus punya alasan
Sebelum menambah elemen visual, tanya: apa fungsinya? Apakah membantu kasir
bekerja lebih cepat? Apakah membantu hierarchy/pemahaman info? Kalau
jawabannya tidak jelas — jangan ditambahkan.

### 1.4 Prioritas informasi (khusus konteks kasir)
1. Total harga & aksi transaksi utama (tambah item, bayar)
2. Info kritis (stok minim, selisih shift, status koneksi LAN/cloud)
3. Aksi sekunder (diskon, tukar poin, retur)
4. Info pendukung (riwayat, metadata)
5. Elemen dekoratif — prioritas paling rendah, jangan sampai mengalahkan yang di atas

### 1.5 Audit sebelum dianggap selesai
Setelah bikin/ubah layar, tanya ke diri sendiri:
- Kalau logo/nama FAZPOS dihapus, apakah ini masih terlihat seperti aplikasi kasir generik template SaaS?
- Apakah tiap elemen di layar ini punya tujuan jelas?
- Apakah visual lebih dominan daripada fungsi? Kalau iya, kurangi dekorasi
- Apakah desain ini sesuai kebutuhan produk (kasir real, bukan demo)?

Kejar **"UI yang benar untuk produk ini"**, bukan "UI yang terlihat modern".

### 1.6 Jangan berhalusinasi fitur
Jangan menambahkan field, menu, atau workflow yang tidak ada di `plan.md`,
`skema-database.md`, atau `progres-pengerjaan.md`. Kalau data/fitur belum
jelas, gunakan struktur netral dan tanyakan — jangan mengarang requirement.

---

## 2. Inventaris Layar (Screens)

| Layar | Fungsi Utama | Terhubung ke Tabel |
|---|---|---|
| **Pilih Operator / Login** | Kasir pilih/scan diri sendiri sebelum kerja | `doperator` |
| **Buka Shift** | Input modal awal sebelum transaksi bisa jalan | `tshift` |
| **Kasir (Transaksi)** | Layar utama: cari/scan produk, keranjang, attach member, bayar | `dbarang`, `dpelanggan`, `tpenjualan`+detail |
| **Pencarian Produk** | Modal/panel cari produk manual (kalau scan gagal/tidak ada barcode) | `dbarang` |
| **Pencarian/Attach Member** | Cari member by nama/kode/HP atau scan kartu, tampilkan poin | `dpelanggan` |
| **Tukar Poin** | Input jumlah poin yang mau ditukar saat transaksi | `pengaturan_poin`, `riwayat_poin` |
| **Pembayaran** | Pilih metode, input nominal, tampilkan kembalian | `tpenjualan` |
| **Preview/Cetak Struk** | Tampilan struk sebelum/pas dicetak ke printer ESC/POS | — |
| **Tutup Shift** | Ringkasan otomatis + input uang aktual + cetak struk shift | `tshift` |
| **Retur Penjualan** | Cari faktur lama, pilih item diretur | `treturpenjualan`+detail |
| **Pembelian (Supplier)** | Entri faktur beli, update stok masuk | `tpembelian`+detail |
| **Manajemen Produk** | CRUD produk, tier harga, stok | `dbarang` |
| **Manajemen Member/Supplier/Sales/Karyawan** | CRUD master data | `dpelanggan`, `dsuplier`, `dsales`, `dkaryawan` |
| **Stok Opname** | Input hasil hitung fisik vs sistem | `tstokopname` |
| **Hutang/Piutang** | Rekap jatuh tempo, bayar cicilan | `thutang`, `tpiutang` |
| **Kas & Biaya** | Pencatatan kas masuk/keluar non-transaksi, biaya operasional | `tcashflow`, `tbiaya` |
| **Laporan** | Penjualan per cabang/kasir/periode, produk terlaris, laba rugi — **format tabel murni**, bukan replikasi 84 template JasperReport lama | berbagai tabel `t*` |
| **Pengaturan Jaringan (LAN)** | Pilih role Server/Client, lihat daftar server ditemukan (auto-discovery) | `device` |
| **Pengaturan Cloud Sync** | Input connection string DB cloud milik user (opsional), status sync | — (koneksi eksternal) |
| **Wizard Import Data Lama** | Upload file `.sql`, lihat progres & ringkasan hasil import | semua tabel (lihat `skema-database.md` §5) |
| **Aktivasi Lisensi** | Tampilkan Machine ID, input token aktivasi | — (modul lisensi terpisah) |
| **Backup Data** | Trigger backup SQLite manual/terjadwal | — |

**Sebelum bikin layar baru:** cek `progres-pengerjaan.md` dulu — kalau layar
itu sudah ada di KATEGORI 1 (selesai) atau KATEGORI 2 (backend siap, tinggal
disambung), jangan bangun ulang dari nol. Urutan pembuatan layar baru
mengikuti fase di `plan.md` §7 / roadmap di `progres-pengerjaan.md` §2.

---

## 3. Style Tokens (Global)

Warna/tipografi/spacing final ditentukan berdasarkan konteks & brand FAZPOS
yang sudah berjalan (kalau sudah ada implementasi nyata, itu jadi source of
truth — jangan redesign total tanpa alasan). Contoh starting point kalau
belum ada:

```slint
export global AppStyle {
    // Warna
    out property <color> primary: #1e6091;
    out property <color> primary-dark: #14486b;
    out property <color> success: #2d9d5f;
    out property <color> danger: #d64545;
    out property <color> warning: #e0a010;
    out property <color> bg: #f5f6f8;
    out property <color> surface: #ffffff;
    out property <color> text-primary: #1a1a1a;
    out property <color> text-secondary: #6b7280;

    // Spacing (kelipatan 4px)
    out property <length> space-xs: 4px;
    out property <length> space-sm: 8px;
    out property <length> space-md: 16px;
    out property <length> space-lg: 24px;
    out property <length> space-xl: 32px;

    // Tipografi
    out property <length> font-body: 16px;
    out property <length> font-heading: 22px;
    out property <length> font-display: 32px;  // untuk total harga di kasir

    // Ukuran tombol aksi utama (harus besar, mudah diklik/disentuh)
    out property <length> button-height-primary: 56px;
    out property <length> button-height-secondary: 40px;
}
```

**Aturan:** setiap komponen baru WAJIB pakai `AppStyle.xxx`, tidak boleh
hardcode angka warna/spacing baru kecuali benar-benar kasus khusus (dan kalau
begitu, tambahkan token baru ke `AppStyle`, bukan inline). Jangan menentukan
"semua card harus shadow" atau "semua elemen harus rounded" sebagai aturan
kaku — pilih kombinasi border/radius/shadow yang sesuai kebutuhan tiap kasus.

---

## 4. Pola Komponen per Kebutuhan

### 4.1 Input yang menerima scan barcode/kartu
Field pencarian produk & member harus:
- Auto-focus saat layar kasir dibuka (scanner USB = keyboard, langsung ketik ke field yang fokus)
- Auto-submit begitu terima input yang polanya cocok dengan barcode/kode kartu (biasanya diakhiri Enter dari scanner) — jangan tunggu klik tombol cari
- Tetap bisa dipakai untuk ketik manual (partial match nama/kode/HP)
- Jangan bikin toggle mode "auto/manual" terpisah seperti sistem lama — 1 field yang menangani keduanya sudah cukup

### 4.2 Grid/List Produk (tanpa gambar)
- List/table berbasis teks: kode, nama, harga, stok — bukan card bergambar
- Data adalah konten utama, bukan dekorasi — prioritaskan keterbacaan, bisa di-scan mata dengan cepat, filter/sort/search yang jelas
- Baris dengan stok ≤ `stok_min` diberi warna `AppStyle.warning` sebagai indikator visual
- Klik/tap baris = langsung tambah ke keranjang (qty 1), bukan buka detail dulu

### 4.3 Keranjang & Total
- Total harga pakai `font-display` (paling besar di layar) — ini angka paling penting buat kasir
- Tiap baris item: nama, qty (bisa +/- langsung di baris), harga, subtotal, tombol hapus
- Kalau ada member ter-attach: tampilkan badge kecil di atas keranjang (nama member + saldo poin)

### 4.4 Numeric Keypad (untuk input uang/qty)
- Komponen keypad custom on-screen (bukan bergantung keyboard fisik) — penting karena banyak PC kasir pakai touchscreen
- Dipakai di: input bayar, input uang aktual (tutup shift), input qty manual
- Touch target harus cukup besar (lihat `button-height-primary`), bukan tombol kecil rapat

### 4.5 Ringkasan Tutup Shift
- Tampilkan breakdown sesuai `skema-database.md` §12.4 (modal awal, penjualan tunai/non-tunai, kas masuk/keluar, retur tunai, uang seharusnya) SEBELUM kasir input uang aktual
- Setelah input uang aktual, tampilkan selisih dengan warna: hijau (`success`) kalau pas/lebih, merah (`danger`) kalau kurang

### 4.6 Wizard Import Data Lama
- Multi-step: (1) pilih file `.sql` → (2) preview ringkasan apa yang terdeteksi (jumlah produk/transaksi dst, sebelum benar-benar insert) → (3) proses dengan progress bar → (4) ringkasan hasil (berhasil/gagal per kategori)
- Jangan langsung insert begitu file dipilih — user harus lihat preview dulu

### 4.7 Pengaturan Jaringan LAN
- Tampilkan status: "Mode: Server" atau "Mode: Client — terhubung ke [nama server]"
- List server yang ditemukan via auto-discovery muncul sebagai daftar yang bisa diklik, bukan input IP manual
- Indikator koneksi real-time (terhubung/terputus)

### 4.8 Laporan (pengganti 84 template lama)
- Semua laporan (lihat cakupan lengkap di `spesifikasi-legacy-ibretago5.md` §8) ditampilkan sebagai **tabel data langsung** hasil query agregasi — bukan file `.jrxml`/PDF terpisah per jenis laporan
- Tetap sediakan filter periode/kategori dan export (PDF/Excel) sebagai aksi, tapi tampilan utamanya tabel, bukan dokumen statis

### 4.9 Form (tambah/edit master data, transaksi)
- Urutan field mengikuti alur kerja kasir/admin, bukan urutan kolom database
- Hanya field yang benar-benar dibutuhkan — jangan menambah field "siapa tahu berguna"
- Validasi & pesan error harus jelas menyebutkan apa yang salah dan apa yang perlu diperbaiki, bukan pesan generik

### 4.10 Empty State & Error
- Empty state kontekstual per layar (mis. "Belum ada produk — tambah produk pertama" di Manajemen Produk), bukan pesan generik yang sama di semua tempat
- Pesan error harus menjelaskan apa yang salah dan langkah berikutnya, bukan sekadar "Terjadi kesalahan"

---

## 5. Struktur File Slint yang Disarankan

```
ui/
├── styles.slint              (AppStyle — global tokens, §3)
├── components/               (komponen reusable lintas layar, nama berdasar fungsi)
│   ├── numeric_keypad.slint
│   ├── product_row.slint
│   ├── cart_item_row.slint
│   ├── search_field.slint    (dipakai utk produk & member)
│   └── primary_button.slint
├── screens/                  (1 file per layar di inventaris §2)
│   ├── login.slint
│   ├── buka_shift.slint
│   ├── kasir.slint
│   ├── tukar_poin.slint
│   ├── pembayaran.slint
│   ├── tutup_shift.slint
│   ├── retur_penjualan.slint
│   ├── pembelian.slint
│   ├── manajemen_produk.slint
│   ├── hutang_piutang.slint
│   ├── kas_biaya.slint
│   ├── laporan.slint
│   ├── pengaturan_lan.slint
│   ├── pengaturan_cloud.slint
│   ├── wizard_import.slint
│   └── aktivasi_lisensi.slint
└── app.slint                 (root window, routing antar screens)
```

**Penamaan komponen berdasarkan fungsi**, bukan penampilan — `ProductRow`,
`CartSummary`, `ShiftClosePanel` (baik), bukan `ModernCard`, `FancyPanel`,
`GradientBox` (buruk, tidak menjelaskan fungsi).

Kalau agent perlu bikin layar baru yang belum ada di §2, tambahkan dulu ke
tabel inventaris di dokumen ini DAN ke `progres-pengerjaan.md` KATEGORI 4,
sebelum mulai coding.

---

## 6. Verifikasi Setelah Membuat/Mengubah UI

Selaras dengan aturan testing di `SKILL.md` — untuk perubahan UI spesifik:

1. **Compile check** — `cargo build` harus lolos tanpa error/warning terkait `.slint`
2. **Jalankan interaktif** — buka aplikasi, navigasi ke layar yang baru
   dibuat/diubah, pastikan render dengan benar (tidak overflow, tidak ada
   elemen terpotong)
3. **Cek pemakaian style token** — pastikan tidak ada warna/spacing hardcode
   yang seharusnya pakai `AppStyle`
4. **Test interaksi kunci layar itu**, contoh:
   - Layar Kasir → scan/ketik produk beneran nambah ke keranjang, total
     kehitung benar
   - Layar Tutup Shift → angka `uang_seharusnya` yang tampil match dengan
     hitungan manual dari data test
   - Wizard Import → coba dengan file `.sql` contoh, pastikan preview ringkasan
     muncul sebelum insert beneran
5. **Cek di ukuran layar target** — resolusi umum PC kasir (mis. 1366x768 dan
   1920x1080) — pastikan layout tidak rusak di kedua ukuran
6. **Audit visual singkat** — jalankan pertanyaan di §1.5 (apakah ini terasa
   dirancang untuk FAZPOS atau terlihat seperti template generik?)
7. **Update `progres-pengerjaan.md`** kalau layar ini menyelesaikan item di
   KATEGORI 2/4 — pindahkan ke KATEGORI 1 dengan detail file yang sebenarnya
8. **Laporkan ke user** hasil verifikasi ini secara eksplisit (bukan cuma
   "sudah saya buatkan UI-nya")