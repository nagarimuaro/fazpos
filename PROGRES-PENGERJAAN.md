# Matrix Analisis & Progres Pengerjaan: iB Retago 5 vs Backend FAZPOS

Dokumen ini memetakan seluruh modul dan fitur dari spesifikasi **iB Retago 5 (struktur-ui.md)** terhadap kondisi **Backend Rust (fazpos)** dan **Frontend Slint (ui/)**.

---

## 1. Status Pemetaan Fitur

### KATEGORI 1: SUDAH ADA DI BACKEND & SUDAH TERSAMBUNG KE UI (JANGAN DIBUAT LAGI)
Fitur-fitur ini sudah berjalan penuh, teruji di unit test, dan memiliki tampilan modular:

| Modul Retago | Fitur di FAZPOS | File Backend | File UI | Status |
|---|---|---|---|---|
| **1.1 Master Barang** | Grid data-dense, stok, stok minimum, highlight stok kritis (amber), tambah barang, hapus barang, cari barang | `barang_repo.rs`, `domain::barang` | `views/barang_view.slint` | Selesai |
| **1.2 Master Pelanggan** | Grid pelanggan, saldo poin member, alamat, telepon, pencarian | `dpelanggan` query, `domain::pelanggan` | `views/pelanggan_view.slint` | Selesai |
| **2.1 Kasir POS** | Scan barcode, grid keranjang, ubah qty (+/-), hapus item, bayar tunai, hitung kembalian, reset transaksi, auto potong stok | `kasir_service.rs`, `transaksi_repo.rs` | `views/kasir_view.slint` | Selesai |
| **2.1 Riwayat Penjualan** | Grid transaksi, total omzet, no faktur, tanggal, nama kasir, tombol cetak | `tpenjualan` query | `views/riwayat_view.slint` | Selesai |
| **6.1 Shift Kasir** | Buka shift (modal awal), rekap tunai/non-tunai/retur, tutup shift, validasi status shift aktif | `shift_service.rs`, `shift_repo.rs` | `views/shift_view.slint` | Selesai |
| **8. Laporan Bisnis** | Ringkasan omzet, HPP, laba kotor, dan Top 10 produk terlaris (**Format Tabel Murni**) | `sales_report.rs` | `views/laporan_view.slint` | Selesai |
| **10. Pengaturan Lisensi** | Aktivasi offline hardware-locked Ed25519, machine ID unik | `license/verification.rs`, `machine_id.rs` | `views/pengaturan_view.slint` | Selesai |
| **10.8 Backup Data** | Online backup SQLite database (mode WAL aman saat kasir aktif) | `backup/sqlite_backup.rs` | `views/pengaturan_view.slint` | Selesai |
| **Migrasi Data Lama** | Parser dan import dump query SQL `INSERT` iB Retago 5 | `importer/import_service.rs` | `views/pengaturan_view.slint` | Selesai |

---

### KATEGORI 2: SUDAH ADA DI BACKEND, BELUM TERSAMBUNG KE UI (TINGGAL INTEGRASI UI)
Fitur backend sudah siap dan teruji test, hanya butuh penambahan antarmuka / sub-tab:

| Modul Retago | Fitur Backend Tersedia | File Backend | Rencana Integrasi UI |
|---|---|---|---|
| **4.1 Retur Penjualan** | Model retur, validasi faktur asal, pengembalian stok otomatis, pencatatan di shift | `repository/retur_repo.rs`, `domain::transaksi::ReturPenjualan` | Tambah Sub-Menu di Transaksi: `Retur Penjualan` |
| **Poin & Loyalty Member** | Akumulasi poin belanja, tukar poin jadi diskon belanja, log riwayat poin | `repository/poin_repo.rs`, `domain::poin` | Tambah input tukar poin di `KasirView` dan kolom poin di `PelangganView` |
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

### KATEGORI 4: BELUM ADA DI BACKEND (PERLU DIBUAT BARU SECARA BERTAHAP)
Fitur bisnis esensial yang belum diimplementasikan di backend maupun UI:

| Modul Retago | Fitur yang Diperlukan | Kebutuhan Backend | Kebutuhan UI | Prioritas |
|---|---|---|---|---|
| **3.1 Pembelian Barang** | Entri faktur beli dari supplier, update stok masuk, histori HPP | Tabel `tpembelian`, `tpembeliandetail`, `PembelianRepo` | View Pembelian & Form Masuk Barang | **Tinggi (Fase 1)** |
| **1.3 Master Supplier** | Direktori supplier untuk pengadaan barang dan hutang | Tabel `dsuplier`, `SupplierRepo` | View Master Supplier di Tab Master | **Tinggi (Fase 1)** |
| **7.1 & 7.2 Hutang / Piutang** | Pembayaran tempo, jatuh tempo, cicilan, buku hutang-piutang | Tabel `thutang`, `tpiutang`, `HutangPiutangRepo` | View Hutang/Piutang di Tab Transaksi/Keuangan | **Sedang (Fase 2)** |
| **5.1 Stok Opname** | Pencocokan fisik vs komputer, selisih stok, penyesuaian | Tabel `tstokopname`, `OpnameRepo` | View Stok Opname di Tab Master/Stok | **Sedang (Fase 3)** |
| **6.1 & 6.2 Buku Kas / Biaya** | Arus kas operasional (listrik, gaji, sewa), pemasukan non-penjualan | Tabel `tcashflow`, `tbiaya`, `CashflowRepo` | View Kas & Operasional | **Sedang (Fase 4)** |
| **2.1 Pending Kasir (Hold/Recall)** | Tahan keranjang antrean kasir dan panggil kembali | Penyimpanan in-memory / tabel `tpenjualanpending` | Tombol `Hold` & `Recall` di KasirView | **Rendah (Fase 5)** |

---

## 2. Roadmap & Urutan Progres Pengerjaan

```
[FASE 0: Verifikasi Fondasi]  --> SUDAH SELESAI (Modular Slint, 22/22 Tes Rust Lolos)
           │
           ▼
[FASE 1: Integrasi Aset Backend yang Ada ke UI]
  ├── Sambungkan Retur Penjualan ke UI Transaksi
  ├── Tampilkan fitur Poin Member di Kasir & Pelanggan
  └── Pasang trigger ESC/POS thermal printing pada Riwayat Transaksi
           │
           ▼
[FASE 2: Pengadaan & Supplier (Pembelian)]
  ├── Backend: Tabel & Repo `dsuplier` + `tpembelian` + `tpembeliandetail`
  ├── UI: Sub-Menu Master Supplier & Transaksi Pembelian
  └── Logic: Auto-tambah stok dan catat log mutasi `keluarmasuk`
           │
           ▼
[FASE 3: Hutang & Piutang Dagang]
  ├── Backend: Tabel & Repo `thutang` (dari pembelian tempo) & `tpiutang` (dari kasir kredit)
  └── UI: Rekapitulasi jatuh tempo & form pembayaran cicilan (format tabel)
           │
           ▼
[FASE 4: Stok Opname & Manajemen Inventori]
  ├── Backend: Tabel `tstokopname` + penyesuaian stok otomatis
  └── UI: Tabel cek stok fisik vs sistem dengan indikator selisih
           │
           ▼
[FASE 5: Buku Kas & Pengeluaran Operasional]
  ├── Backend: Tabel `tcashflow` & `tbiaya`
  └── UI: Pencatatan beban toko (gaji, listrik, sewa) & rekap kas operasional
```
