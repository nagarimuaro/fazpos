> **Dokumen ini WAJIB dicek SEBELUM memulai task apa pun**, dan **WAJIB
> diupdate SETELAH task selesai & lolos test** (lihat aturan testing di
> `SKILL.md`). Ini status kebenaran terkini soal apa yang sudah jalan, apa
> yang tinggal disambung ke UI, apa yang sengaja tidak dibuat ulang, dan apa
> yang belum ada sama sekali. Jangan membangun ulang sesuatu yang sudah ada
> di KATEGORI 1, dan jangan pakai pendekatan lama yang sudah sengaja
> digantikan di KATEGORI 3.

# Matrix Analisis & Progres Pengerjaan: iB Retago 5 vs Backend FAZPOS

Dokumen ini memetakan seluruh modul dan fitur dari spesifikasi
**iB Retago 5** (`spesifikasi-legacy-ibretago5.md`) terhadap kondisi
**Backend Rust (fazpos)** dan **Frontend Slint (ui/)**.

---

## 1. Status Pemetaan Fitur

### KATEGORI 1: SUDAH ADA DI BACKEND & SUDAH TERSAMBUNG KE UI (JANGAN DIBUAT LAGI)
Fitur-fitur ini sudah berjalan penuh, teruji di 29 unit/integration test, dan memiliki tampilan modular:

| Modul Retago | Fitur di FAZPOS | File Backend | File UI | Status |
|---|---|---|---|---|
| **1.1 Master Barang** | Grid data-dense, stok, stok minimum, highlight stok kritis (amber), tambah barang, hapus barang, cari barang | `barang_repo.rs`, `domain::barang` | `views/barang_view.slint` | Selesai |
| **1.2 Master Pelanggan** | Grid pelanggan, saldo poin member, alamat, telepon, pencarian | `pelanggan_repo.rs`, `domain::pelanggan` | `views/pelanggan_view.slint` | Selesai |
| **1.3 Master Supplier** | CRUD Direktori supplier pengadaan barang dan hutang | `supplier_repo.rs`, `domain` | `views/supplier_view.slint` | Selesai |
| **2.1 Kasir POS** | Scan barcode, grid keranjang, ubah qty, hapus item, bayar tunai/nontunai/kredit, hitung kembalian, auto potong stok | `kasir_service.rs`, `transaksi_repo.rs` | `views/kasir_view.slint` | Selesai |
| **2.1 Pending Kasir (Hold/Recall)** | Tahan keranjang antrean kasir dan panggil kembali | `pending_repo.rs`, `kasir_service.rs` | `views/kasir_view.slint` | Selesai |
| **Poin & Loyalty Kasir** | Scan kartu member fisik, tukar poin diskon, tambah poin belanja otomatis | `poin_repo.rs`, `pelanggan_repo.rs`, `kasir_service.rs` | `views/kasir_view.slint` | Selesai |
| **2.1 Riwayat Penjualan** | Grid transaksi, total omzet, no faktur, tanggal, nama kasir, tombol cetak ESC/POS | `tpenjualan` query | `views/riwayat_view.slint` | Selesai |
| **3.1 Pembelian Barang** | Entri faktur beli supplier, auto tambah stok barang, auto catat hutang tempo | `pembelian_repo.rs` | `views/pembelian_view.slint` | Selesai |
| **4.1 Retur Penjualan** | Validasi faktur asal, restock otomatis, potong kas shift | `retur_repo.rs` | `views/retur_view.slint` | Selesai |
| **5.1 Stok Opname** | Hitung fisik vs sistem, penyesuaian stok otomatis, log selisih | `opname_repo.rs` | `views/stok_opname_view.slint` | Selesai |
| **6.1 Shift Kasir** | Buka shift (modal awal), rekap tunai/non-tunai/retur/biaya, tutup shift | `shift_service.rs`, `shift_repo.rs` | `views/shift_view.slint` | Selesai |
| **6.1 & 6.2 Buku Kas & Biaya** | Arus kas operasional (masuk/keluar), biaya operasional (beban toko) | `buku_kas_repo.rs` | `views/buku_kas_view.slint` | Selesai |
| **7.1 & 7.2 Hutang / Piutang** | Rekap jatuh tempo, cicilan hutang pembelian, piutang kasir kredit | `hutang_piutang_repo.rs` | `views/hutang_piutang_view.slint` | Selesai |
| **8. Laporan Bisnis** | Ringkasan omzet, HPP, laba kotor, dan Top 10 produk terlaris (**Format Tabel Murni**) | `sales_report.rs` | `views/laporan_view.slint` | Selesai |
| **10. Pengaturan Lisensi** | Aktivasi offline hardware-locked Ed25519, machine ID unik | `license/verification.rs`, `machine_id.rs` | `views/pengaturan_view.slint` | Selesai |
| **10.8 Backup Data** | Online backup SQLite database (mode WAL aman saat kasir aktif) | `backup/sqlite_backup.rs` | `views/pengaturan_view.slint` | Selesai |
| **Migrasi Data Lama** | Parser dan import dump query SQL `INSERT` iB Retago 5 | `importer/import_service.rs` | `views/pengaturan_view.slint` | Selesai |

---

### KATEGORI 2: SUDAH ADA DI BACKEND, BELUM TERSAMBUNG KE UI (TINGGAL INTEGRASI UI)
Fitur backend sudah siap dan teruji test, hanya butuh penambahan antarmuka / sub-tab:

| Modul Retago | Fitur Backend Tersedia | File Backend | Rencana Integrasi UI |
|---|---|---|---|
| **8.7 Cetak Struk Fisik** | Generator ESC/POS thermal 58mm/80mm (cut paper, cash drawer, format struk) | `printer/escpos.rs`, `printer/struk.rs` | Hubungkan tombol `Cetak Struk` ke port printer lokal via USB/Raw |
| **Jaringan Multi-Kasir LAN** | Server Axum embedded, sinkronisasi checkout client-server | `lan/server.rs`, `lan/protocol.rs` | Pengaturan IP Server/Client di `PengaturanView` |
| **Sinkronisasi Cloud Outbox** | Outbox pattern log transaksi untuk sync multi-cabang | `sync/outbox.rs`, `sync/payload.rs` | Tampilan status sinkronisasi di Status Bar |

---

### KATEGORI 3: FITUR MIRIP YANG SUDAH DILAYANI DESAIN MODERN (JANGAN DIBUAT ULANG)
Untuk efisiensi, kompleksitas sistem lama yang usang digantikan oleh arsitektur baru:

| Modul di iB Retago 5 | Pendekatan Sistem Lama | Pendekatan FAZPOS (Modern) | Alasan Tidak Dibuat Ulang |
|---|---|---|---|
| **Multi-Tier Harga** | 5 level field terpisah (`qty_1` s/d `qty_5`, `harga_jual_1` s/d `5`) | Multi-tier harga di `dbarang`: Eceran (1), Grosir 1 (2), Grosir 2 (3), Partai | Backend & UI sudah menampung tier harga secara efisien |
| **84 Format JasperReports** | 84 template file `.jrxml` eksternal terpisah | Query agregasi SQLite langsung ditampilkan dalam **Format Tabel Murni** di UI | Sesuai instruksi: semua laporan dijadikan tabel data, cepat tanpa runtime Java/Jasper |
| **Scanner Barcode Auto/Manual** | 2 mode terpisah dengan konfigurasi checkbox | 1 Barcode TextInput yang auto-enter scanner HID sekaligus support manual | Barcode scanner USB standar langsung mengirim sinyal `Enter` |
| **Penomoran Faktur (autokode)** | Konfigurasi prefix tabel manual | Prefix deterministik otomatis `{kode_cabang}-{faktur}` | Mencegah collision transaksi multi-cabang / multi-device |
| **Database Server** | MySQL / MariaDB terpisah (port 3309) | SQLite lokal (WAL Mode) per toko | Zero configuration, tidak perlu instalasi service DB terpisah |

---

### KATEGORI 4: FITUR TAMBAHAN TAHAP LANJUT (OPSIONAL)
Fitur lanjutan untuk fase penyempurnaan:

| Modul Retago | Fitur yang Diperlukan | Kebutuhan Backend | Kebutuhan UI | Prioritas |
|---|---|---|---|---|
| **Multi-Kasir LAN Discovery** | Auto-discovery mDNS untuk pencarian server kasir otomatis | `lan/protocol.rs` | Tombol cari server otomatis di PengaturanView | **Rendah** |
| **Cloud Sync Background Worker** | Auto-push log outbox ke Postgres Cloud | `sync/outbox.rs` | Indikator status sync di header/statusbar | **Rendah** |

---

## 2. Roadmap & Urutan Progres Pengerjaan

```
[FASE 0: Verifikasi Fondasi]  --> SUDAH SELESAI (Modular Slint, 22/22 Tes Lolos)
           │
           ▼
[FASE 1: Integrasi Modul & Poin Kasir] --> SUDAH SELESAI
  ├── Sambungkan Retur Penjualan ke UI Transaksi
  ├── Fitur Poin & Tukar Poin Member di Kasir & Pelanggan
  └── Pasang trigger ESC/POS thermal printing pada Riwayat Transaksi
           │
           ▼
[FASE 2: Pengadaan & Supplier (Pembelian)] --> SUDAH SELESAI
  ├── Backend: Tabel & Repo `dsuplier` + `tpembelian` + `tpembeliandetail`
  ├── UI: Sub-Menu Master Supplier & Transaksi Pembelian
  └── Logic: Auto-tambah stok dan catat log mutasi `keluarmasuk`
           │
           ▼
[FASE 3: Hutang & Piutang Dagang] --> SUDAH SELESAI
  ├── Backend: Tabel & Repo `thutang` (dari pembelian tempo) & `tpiutang` (dari kasir kredit)
  └── UI: Rekapitulasi jatuh tempo & form pembayaran cicilan (format tabel)
           │
           ▼
[FASE 4: Stok Opname & Manajemen Inventori] --> SUDAH SELESAI
  ├── Backend: Tabel `tstokopname` + penyesuaian stok otomatis
  └── UI: Tabel cek stok fisik vs sistem dengan indikator selisih
           │
           ▼
[FASE 5: Buku Kas, Pengeluaran & Pending Antrean] --> SUDAH SELESAI
  ├── Backend: Tabel `tcashflow`, `tbiaya`, `tpenjualanpending` + `tpenjualanpendingdetail`
  ├── Logic: Hold/Recall antrean kasir, shift tagging pada cashflow/biaya
  └── UI: View Buku Kas operasional & Tombol Hold/Recall pada KasirView
```

---

## 3. Cara Menjaga Dokumen Ini Tetap Akurat

Setiap kali agent menyelesaikan & meloloskan test untuk sebuah task (lihat
aturan "WAJIB TEST" di `SKILL.md`):

1. Kalau task itu **menyelesaikan item dari KATEGORI 2 atau 4** — pindahkan
   baris item tersebut ke KATEGORI 1, isi kolom File Backend/UI sebenarnya,
   dan update status roadmap fase terkait
2. Kalau task itu **memulai tapi belum menuntaskan** item KATEGORI 4 —
   jangan pindahkan ke KATEGORI 1 dulu, cukup catat progres parsial di
   kolom Prioritas (mis. "Tinggi (Fase 1) — backend selesai, UI belum")
3. Kalau ternyata muncul kebutuhan fitur baru yang tidak ada di
   `spesifikasi-legacy-ibretago5.md` maupun tabel di atas — tambahkan baris
   baru ke KATEGORI 4 dulu sebelum mulai kerja, jangan langsung coding tanpa
   tercatat di sini