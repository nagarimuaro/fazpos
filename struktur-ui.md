# Dokumentasi Komprehensif & Spesifikasi Fitur iB Retago 5 - RTG02

Aplikasi **iB Retago 5 (Versi 5.2.210 - RTG02)** dikembangkan oleh iDea Brains sebagai software Point of Sale (POS), Manajemen Inventori, Keuangan, dan Pengendalian Usaha Retail, Grosir, Minimarket, serta Apotek.

---

## DAFTAR ISI
1. [Modul Master Data](#1-modul-master-data)
2. [Modul Transaksi Kasir & Point of Sale (POS)](#2-modul-transaksi-kasir--point-of-sale-pos)
3. [Modul Pembelian & Pengadaan Barang](#3-modul-pembelian--pengadaan-barang)
4. [Modul Retur (Penjualan & Pembelian)](#4-modul-retur-penjualan--pembelian)
5. [Modul Manajemen Stok, Inventori, & Stok Opname](#5-modul-manajemen-stok-inventori--stok-opname)
6. [Modul Keuangan, Arus Kas (Cashflow), & Biaya Operasional](#6-modul-keuangan-arus-kas-cashflow--biaya-operasional)
7. [Modul Manajemen Hutang & Piutang Dagang](#7-modul-manajemen-hutang--piutang-dagang)
8. [Detail Lengkap Seluruh Modul Laporan (84 Format JasperReports)](#8-detail-lengkap-seluruh-modul-laporan-84-format-jasperreports)
9. [Modul Barcode Printing & Cetak Label Rak](#9-modul-barcode-printing--cetak-label-rak)
10. [Detail Lengkap Seluruh Menu Pengaturan (Settings & Configuration)](#10-detail-lengkap-seluruh-menu-pengaturan-settings--configuration)
11. [Matriks Hak Akses Operator & Keamanan Sistem](#11-matriks-hak-akses-operator--keamanan-sistem)

---

## 1. Modul Master Data

### 1.1 Master Data Barang (`dbarang`)
- **Identitas Produk**: Kode Barang / Barcode unik, Nama Barang, Kategori Produk (`combokategori`), Lokasi Rak / Display fisik.
- **Manajemen Tanggal Kadaluarsa**: Field `expired` untuk monitoring otomatis masa simpan produk (khusus retail makanan/obat apotek).
- **Stok & Ambang Batas**: Pencatatan `stok` fisik dan batas peringatan `stok_min`.
- **Struktur Multi-Tier Pricing (Grosir Bertingkat 5 Level)**:
  - `harga_beli`: Harga pokok pembelian dari supplier.
  - `qty_1` & `harga_jual_1`: Level harga satuan eceran normal.
  - `qty_2` & `harga_jual_2`: Level harga grosir tier 1.
  - `qty_3` & `harga_jual_3`: Level harga grosir tier 2.
  - `qty_4` & `harga_jual_4`: Level harga partai besar / distributor tier 3.
  - `qty_5` & `harga_jual_5`: Level harga khusus / langganan tier 4.
- **Relasi & Diskon**: Relasi ke `suplier` utama dan default `diskon` per item.
- **Satuan Barang (`combosatuan`)**: Dukungan multi satuan (Pcs, Dus, Botol, Box, Pack, Lusin, dll).

### 1.2 Master Pelanggan (`dpelanggan`)
- Menyimpan Kode Pelanggan, Nama, Alamat, No. HP, Email, No. Rekening, Keterangan khusus, serta default `diskon_persen` member.

### 1.3 Master Supplier (`dsuplier`)
- Menyimpan Kode Supplier, Nama Instansi / Perusahaan, Alamat, No. HP Sales/Kantor, Email, Nomor Rekening Pembayaran, Catatan tempo.

### 1.4 Master Sales / Tenaga Pemasar (`dsales`)
- Tracking staf sales internal/eksternal untuk rekap komisi dan histori penjualan per tenaga penjual.

### 1.5 Master Karyawan (`dkaryawan`)
- Direktori seluruh pegawai/operator kasir beserta kontak dan rekening payroll.

### 1.6 Fitur Data Tools (Import / Export Excel)
- Setiap master data (Barang, Pelanggan, Supplier, Sales, Karyawan) dilengkapi fungsi **Export to Excel/CSV** dan **Import Data Massal** untuk migrasi data cepat.

---

## 2. Modul Transaksi Kasir & Point of Sale (POS)

### 2.1 Proses Transaksi Kasir (`tpenjualan`, `tpenjualandetail`)
- **Metode Pembayaran (`combotunaikredit`)**:
  - `Tunai` (Cash)
  - `Kredit` (Penjualan Tempo / Piutang dengan pencatatan `nhari` dan tanggal `jatuh_tempo`)
  - `Debit`
  - `Transfer`
  - `QRIS`
- **Pencarian Cepat**: Input otomatis scanner barcode (`auto`) atau mode pencarian manual nama/kode produk (`manual`).
- **Fitur Diskon Bertingkat**:
  - Diskon per item barang (% atau nominal).
  - Diskon faktur global (`diskon_faktur`, `dikson_persen`).
- **Pajak & Biaya**:
  - Kalkulasi PPN otomatis (`form_PPN`, `nilai_PPN`).
  - Penambahan form ongkos kirim (`form_ongkos`).
- **Analisis Profit Real-Time**:
  - Setiap faktur langsung menghitung `total_harga_beli` (HPP) dan `total_laba` (gross margin) kasir secara instan.
- **Fitur Pending Transaksi Kasir (`tpenjualanpending`, `tpenjualanpendingdetail`)**:
  - Menahan (Hold) antrean belanja pelanggan yang belum selesai dan memanggilnya kembali (Recall) tanpa kehilangan data keranjang.

---

## 3. Modul Pembelian & Pengadaan Barang

### 3.1 Transaksi Pembelian Supplier (`tpembelian`, `tpembeliandetail`)
- Pencatatan nota pembelian dari supplier dengan nomor faktur supplier asli.
- Pilihan pembayaran Tunai atau Kredit (Hutang Usaha) dengan termin jatuh tempo `nhari`.
- Perhitungan Diskon Faktur, PPN Masukan, dan Biaya Pengiriman.
- **Fitur Update Harga Otomatis**: Opsi untuk otomatis memperbarui `harga_beli` dan menaikkan `harga_jual` master barang saat terjadi kenaikan harga dari distributor.
- **Pending Pembelian (`tpembelianpending`)**: Draft penerimaan barang sementara sebelum faktur resmi disahkan.

---

## 4. Modul Retur (Penjualan & Pembelian)

### 4.1 Retur Penjualan (`treturpenjualan`, `treturpenjualandetail`)
- Pengembalian barang dari pelanggan karena cacat, salah beli, atau rusak.
- Pilihan penyelesaian retur:
  - Pengembalian uang tunai langsung (`total_dibayar`).
  - Pemotongan saldo piutang pelanggan (`total_mengurangi_piutang`).
- Cetak Nota Retur Penjualan (Thermal 58mm, 80mm, dan A4).

### 4.2 Retur Pembelian (`treturpembelian`, `treturpembeliandetail`)
- Pengembalian stok rusak ke supplier.
- Pilihan kompensasi:
  - Penerimaan uang tunai dari supplier (`total_dibayar`).
  - Pengurangan saldo hutang usaha ke supplier terkait (`total_mengurangi_hutang`).
- Cetak Nota Retur Pembelian resmi.

---

## 5. Modul Manajemen Stok, Inventori, & Stok Opname

### 5.1 Stok Opname Fisik (`tstokopname`)
- Form pencocokan stok komputer (`stok_komputer`) terhadap hasil hitung fisik gudang/rak (`stok_nyata`).
- Rekap otomatis kuantiti `selisih` dan valuasi `total_selisih` (Rupiah).
- Klasifikasi alasan selisih (`comboalasan`): `Hilang`, `Rusak`, atau `Dipakai Sendiri`.
- Fitur Import hasil opname dari Excel, Export rekapan, dan Cetak Berita Acara Stok Opname.

### 5.2 Kartu Stok & Aliran Mutasi (`keluarmasuk`)
- Audit trail pergerakan barang secara kronologis (Tanggal, Jam, No. Faktur, Masuk, Keluar, Saldo Akhir, Status Mutasi).

### 5.3 Filter & Monitoring Inventori Cepat
- **Filter Stok Kritis (`combostok`)**:
  - `Stok < stok_min`
  - `Stok = 0` (Kosong)
  - `Stok < 10`, `< 30`, `< 50`, `Stok > 50`
- **Filter Kadaluarsa (`comboexpired`)**:
  - `Expired` (Sudah lewat tanggal)
  - `Expired < 7 Hari`
  - `Expired < 30 Hari`
  - `Expired < 60 Hari`
  - `Expired < 90 Hari`

---

## 6. Modul Keuangan, Arus Kas (Cashflow), & Biaya Operasional

### 6.1 Buku Kas Utama (`tcashflow`)
- Pencatatan seluruh transaksi penerimaan (`pemasukan`) dan pengeluaran (`pengeluaran`) kas di luar penjualan/pembelian barang.
- Klasifikasi jenis kas: Pemasukan Lain-lain (`combokategoripemasukan`) dan Pengeluaran Beban (`combokategoripengeluaran`).
- Rekap saldo kas riil harian, mingguan, dan bulanan.

### 6.2 Pos Pengeluaran Biaya (`tbiaya`)
- Pencatatan detail beban operasional toko: Gaji Karyawan, Listrik/Air/Internet, Sewa Ruko, Konsumsi, Transportasi, ATK, Maintenance, dll.

---

## 7. Modul Manajemen Hutang & Piutang Dagang

### 7.1 Manajemen Hutang Supplier (`thutang`)
- Monitoring faktur pembelian kredit yang belum lunas.
- Menampilkan: No. Faktur, Tanggal, Supplier, Termin `tempo`, Tanggal `jatuh_tempo`, `tagihan_awal`, `telah_dibayar`, dan `sisa` hutang.
- Fitur **Bayar Hutang**: Pembayaran bertahap (cicilan) atau pelunasan total yang otomatis terhubung ke pemotongan Arus Kas (`tcashflow`).
- Histori detail pembayaran cicilan hutang.

### 7.2 Manajemen Piutang Pelanggan (`tpiutang`)
- Monitoring penjualan kredit ke pelanggan/member.
- Menampilkan: No. Faktur, Pelanggan, Jatuh Tempo, Tagihan Awal, Total Terbayar, dan Sisa Piutang Aktif.
- Fitur **Bayar Piutang**: Penerimaan cicilan/pelunasan piutang pelanggan dengan pencatatan langsung ke kas masuk.
- Cetak Bukti Tanda Terima Pembayaran Hutang/Piutang (Thermal 58mm, 65mm, 75mm, 90mm, 180mm).

---

## 8. Detail Lengkap Seluruh Modul Laporan (84 Format JasperReports)

Setiap laporan memiliki filter tanggal periode (`filtertanggal`), filter kategori, filter nama/kode, serta tombol **Preview, Cetak, dan Export PDF/Excel**.

### 8.1 Laporan Penjualan
1. `Laporan/Penjualan.jrxml`: Rekap umum seluruh faktur penjualan periode tertentu.
2. `Laporan/PenjualanKasir.jrxml`: Rekap performa kasir (jumlah faktur, omset, diskon, tunai vs kredit).
3. `Laporan/PenjualanPerBarang.jrxml`: Rincian kuantiti dan nilai penjualan per item produk.
4. `Laporan/PenjualanPerKategori.jrxml`: Analisis penjualan berdasarkan kelompok kategori produk.
5. `Laporan/PenjualanPerPelanggan.jrxml`: Rincian riwayat transaksi belanja per nama pelanggan.
6. `Laporan/PenjualanPerTanggal.jrxml`: Rekap rekapitulasi omset urut berdasarkan tanggal kalender.
7. `Laporan/PenjualanTotalPelanggan.jrxml`: Ranking kontribusi omset terbesar per pelanggan.
8. `Laporan/PenjualanTotalSales.jrxml`: Rekapitulasi omset dan diskon berdasarkan nama staf sales.
9. `Laporan/PenjualanPajak.jrxml`: Rekapitulasi penjualan khusus yang dikenakan PPN.
10. `Laporan/PenjualanRetur.jrxml`: Rekap seluruh faktur pengembalian barang dari pelanggan.
11. `Laporan/PenjualanReturDetail.jrxml`: Rincian item barang yang diretur oleh pelanggan.

### 8.2 Laporan Pembelian & Pengadaan
12. `Laporan/Pembelian.jrxml`: Rekapitulasi faktur pembelian dari seluruh supplier.
13. `Laporan/PembelianPerBarang.jrxml`: Rincian riwayat pengadaan barang per nama item.
14. `Laporan/PembelianPerSuplier.jrxml`: Rekapitulasi belanja stok yang dikelompokkan per supplier.
15. `Laporan/PembelianTotalSuplier.jrxml`: Akumulasi total nilai belanja per supplier.
16. `Laporan/PembelianRetur.jrxml`: Rekap transaksi pengembalian barang retur ke supplier.
17. `Laporan/PembelianReturDetail.jrxml`: Rincian item barang yang dikembalikan ke supplier.

### 8.3 Laporan Keuangan, Laba Rugi, & Arus Kas
18. `Laporan/labaRugi.jrxml`: Laporan laba/rugi komprehensif (Penjualan Bersih - HPP - Biaya Operasional = Laba Bersih Usaha).
19. `Laporan/Cashflow.jrxml`: Laporan mutasi arus kas masuk dan kas keluar.
20. `Laporan/Biaya.jrxml`: Rincian pengeluaran operasional toko per pos kategori beban.
21. `Laporan/Modal.jrxml`: Laporan estimasi nilai aset modal barang dagangan di toko/gudang.

### 8.4 Laporan Hutang & Piutang
22. `Laporan/Hutang.jrxml`: Rekap status tagihan hutang supplier (Lunas / Belum Lunas).
23. `Laporan/Piutang.jrxml`: Rekap status piutang pelanggan beserta sisa tagihan.
24. `Laporan/PiutangPelanggan.jrxml`: Rekap piutang yang dirinci per nama pelanggan.
25. `Laporan/histori_bayar_hutang_piutang.jrxml`: Riwayat log angsuran pembayaran hutang & piutang.

### 8.5 Laporan Inventori, Stok, & Pergerakan Barang
26. `Laporan/Barang.jrxml`: Master katalog barang lengkap beserta harga dan stok.
27. `Laporan/barang_stok_total.jrxml`: Rekapitulasi kuantiti dan valuasi total stok toko.
28. `Laporan/kartuStok.jrxml`: Laporan mutasi kartu stok barang individual.
29. `Laporan/barang_masuk.jrxml`: Rincian detail seluruh barang yang masuk gudang.
30. `Laporan/barang_keluar.jrxml`: Rincian detail seluruh barang yang keluar toko.
31. `Laporan/barang_keluar_masuk.jrxml`: Gabungan rekap arus masuk dan keluar barang.
32. `Laporan/barang_terlaris.jrxml`: Analisis produk Fast Moving / Produk Terlaris berdasarkan QTY terjual.
33. `Laporan/barang_tidak_laku.jrxml`: Analisis produk Dead Stock / Slow Moving yang tidak bergerak.

### 8.6 Laporan Grafik & Visualisasi Statistik Bisnis
34. `Laporan/GrafikFakturHarian.jrxml`: Grafik volume jumlah transaksi faktur per hari dalam 1 bulan.
35. `Laporan/GrafikFakturBulanan.jrxml`: Grafik perbandingan volume transaksi faktur antar bulan dalam 1 tahun.
36. `Laporan/GrafikOmsetLabaHarian.jrxml`: Grafik tren pergerakan Omset vs Laba Bersih harian.
37. `Laporan/GrafikOmsetLabaBulanan.jrxml`: Grafik tren pertumbuhan Omset vs Laba Bersih tahunan.

### 8.7 Template Struk Kasir, Faktur, & Dokumen Pengiriman
- **Ukuran Struk Thermal POS**:
  - `fPenjualan_58mm.jrxml`, `fPenjualan_58mm_2.jrxml`, `fPenjualan_58mm_3.jrxml` (Model 58mm variasi ringkas/lengkap)
  - `fPenjualan_65mm.jrxml`, `fPenjualan_65mm_2.jrxml`, `fPenjualan_65mm_3.jrxml`
  - `fPenjualan_75mm.jrxml`, `fPenjualan_75mm_2.jrxml`
  - `fPenjualan_80mm.jrxml`, `fPenjualan_90mm.jrxml`, `fPenjualan_100mm.jrxml`, `fPenjualan_120mm.jrxml`, `fPenjualan_180mm.jrxml`
- **Faktur Format Lebar & Continuous Dot Matrix**:
  - `fPenjualan_a4.jrxml`, `fPenjualan_a4_2.jrxml`, `fPenjualan_a4_3.jrxml`, `fPenjualan_a4_4.jrxml`
  - `fPenjualan_a4_LX.jrxml`, `fPenjualan_a4_2_LX.jrxml`, `fPenjualan_a4_3_LX.jrxml`, `fPenjualan_a4_4_LX.jrxml` (Format khusus printer pita Epson LX-300 / LX-310).
- **Surat Jalan & Nota Retur**:
  - `suratJalan_1.jrxml`, `suratJalan_1_LX.jrxml`, `suratJalan_2.jrxml`, `suratJalan_2_LX.jrxml`
  - `nota_retur_58mm.jrxml`, `nota_retur_80mm.jrxml`, `nota_retur_A4.jrxml`
  - `tHutangPiutang_58mm.jrxml` s/d `tHutangPiutang_180mm.jrxml`

---

## 9. Modul Barcode Printing & Cetak Label Rak

Menyediakan generator pencetakan barcode produk ke printer barcode thermal stiker dengan berbagai pilihan ukuran label (`comboukurancetak`):
- `3015.jrxml` s/d `3015x6.jrxml`: Ukuran label 30 x 15 mm (1, 2, 3, hingga 6 kolom per baris).
- `3020.jrxml` s/d `3020x3.jrxml`: Ukuran label 30 x 20 mm (1, 2, hingga 3 kolom).
- `4030.jrxml`, `4030x2.jrxml`: Ukuran label 40 x 30 mm (1 & 2 kolom).
- `5020.jrxml`, `5020x2.jrxml`: Ukuran label 50 x 20 mm (1 & 2 kolom).
- `5025.jrxml`: Ukuran label 50 x 25 mm.
- `7050.jrxml`: Ukuran label besar 70 x 50 mm.
- `label_rak.jrxml`: Cetak kartu label harga gantung / selipan rak display toko (Price Tag Rak).

---

## 10. Detail Lengkap Seluruh Menu Pengaturan (Settings & Configuration)

### 10.1 Pengaturan Profil Toko & Header Nota (`daplikasi`)
- `nama_toko`: Nama entitas toko / apotek / minimarket.
- `nohp` & `alamat`: Kontak WhatsApp toko dan alamat resmi yang tercetak pada nota.
- `header_nota`: Teks pembuka pada bagian atas struk kasir.
- `footer_nota`: Pesan penutup/syarat garansi nota (contoh: *"Barang yang sudah dibeli tidak dapat ditukar"*).
- `logoNota` & `logoAplikasi`: Pengaturan file logo grafis toko pada banner aplikasi dan kop cetak struk.
- `lokasi_database`: Path konfigurasi koneksi MariaDB / MySQL.

### 10.2 Pengaturan Notifikasi Otomatis WhatsApp (`daplikasi`)
- `nomor_wa_notif`: Nomor WhatsApp pemilik usaha / owner penerima notifikasi ringkasan penjualan.
- `waktu_kirim_pesan`: Pilihan trigger pengiriman (`Pukul` atau per interval `value_periode` misal tiap 1 jam).
- `pukul`: Set jam pengiriman laporan harian (contoh default: `18:00`).
- **Opsi Konten Laporan WA yang Dikirim**:
  - `text_header`: Menampilkan nama toko & tanggal.
  - `text_total_harian`: Menampilkan total omset rupiah hari ini.
  - `text_total_bulanan`: Menampilkan akumulasi omset rupiah bulan berjalan.
  - `banyak_transaksi_harian`: Menampilkan total jumlah nota hari ini.
  - `banyak_transaksi_bulanan`: Menampilkan total akumulasi transaksi bulan ini.

### 10.3 Pengaturan Integrasi Laporan Online Cloud (`daplikasi`)
- `nomor_wa_lap_online`: ID akun WhatsApp untuk autentikasi pelaporan online.
- `password_lap_online`: PIN / Password keamanan enkripsi laporan online.
- `sinkron_laporan`: Saklar on/off sinkronisasi laporan otomatis ke server cloud iDea Brains.

### 10.4 Kebijakan Operasional & Aturan Kasir (`checkboxediting`)
- `mengubah_harga`: Izin apakah kasir boleh mengubah harga jual secara bebas di tabel kasir.
- `mengubah_tanggal`: Izin apakah operator boleh melakukan backdate / memajukan tanggal transaksi.
- `memberikan_diskon`: Izin hak pemberian diskon item manual oleh kasir.
- `diskon_faktur`: Izin hak pemberian diskon total nota.
- `update_harga`: Opsi update otomatis harga beli master saat entry pembelian baru.
- `stok_minus`: Saklar keamanan apakah sistem memperbolehkan kasir menjual barang saat stok = 0 (minus).
- `pembulatan_harga` & `angka_pembulatan`: Aturan pembulatan nominal total belanja (misal dibulatkan ke kelipatan 100 atau 500).
- `laba_otomatis`: Kalkulasi mark-up margin keuntungan persentase otomatis saat input barang baru.
- `form_PPN` & `nilai_PPN`: Menampilkan kolom PPN dan nilai default (misal 11%).
- `form_ongkos`: Menampilkan input ongkos ekspedisi pada faktur penjualan.
- `pending_faktur`: Mengaktifkan fitur hold transaksi kasir.
- `print_preview`: Opsi menampilkan preview layar sebelum mencetak ke printer fisik.
- `auto` / `manual`: Default mode input barcode scanner (otomatis add ke baris setelah scan enter) atau mode manual.
- `konfirmasi_barang` & `konfirmasi_beli`: Menampilkan dialog peringatan verifikasi sebelum simpan transaksi.
- `konfirmasi_obat`: Mode khusus modul farmasi / apotek untuk konfirmasi dosis dan resep dokter (`harga_resep`).

### 10.5 Pengaturan Penomoran Otomatis Kode (`autokode`)
Konfigurasi prefix penomoran otomatis ID & No Faktur pada modul:
- `barang`, `suplier`, `pelanggan`, `sales`, `karyawan`, `operator`, `kategori`, `retur_pembelian`, `retur_penjualan`, `stokOpname`.

### 10.6 Kustomisasi Tampilan Kolom Grid (`aturkolom`)
Pengaturan visibilitas kolom tabel agar tampilan antarmuka kasir dan administrasi dapat disesuaikan dengan kebutuhan layar monitor:
- Atur kolom tabel Penjualan (`tPenjualan`), Pembelian (`tPembelian`), Master Barang (`dBarang`), Supplier, Pelanggan, Sales, Karyawan, Hutang, Piutang, Cashflow, Biaya, Stok Opname, dan Retur.

### 10.7 Pengaturan Printer & Perangkat Keras POS (`Printer.ini`, `comboukurancetak`)
- Konfigurasi printer kasir struk default.
- Konfigurasi printer barcode label default.
- Konfigurasi printer faktur laporan dot matrix (Epson ESC/POS).
- Pengaturan direct cut (auto cutter) & trigger pembuka laci kasir (Cash Drawer Kick).

### 10.8 Pengaturan Jaringan & Multi-User (`Client.ini`, `Database.ini`)
- `Client.ini`: Mode operasi stasiun kerja (`Master` / Server Utama atau `Client` / Komputer Kasir Tambahan).
- `Database.ini`: Konfigurasi Hostname/IP Server MariaDB (default `localhost`), Port Database (default `3309`), user, dan password.
- `Backup.ini`: Konfigurasi direktori tujuan otomatisasi backup database (default `D:/`).

---

## 11. Matriks Hak Akses Operator & Keamanan Sistem

Sistem keamanan multi-pengguna (`doperator`) mengontrol hak akses setiap user/kasir secara granular hingga tingkat fungsi tombol:

| Kategori Modul | Rincian Hak Akses per Tombol / Fitur |
|---|---|
| **Master Barang** | `jendela_barang`, `tambah_barang`, `edit_barang`, `hapus_barang`, `import_barang`, `export_barang`, `persediaan`, `barcode` |
| **Master Pelanggan** | `jendela_pelanggan`, `tambah_pelanggan`, `edit_pelanggan`, `hapus_pelanggan`, `import_pelanggan`, `export_pelanggan` |
| **Master Supplier** | `jendela_suplier`, `tambah_suplier`, `edit_suplier`, `hapus_suplier`, `import_suplier`, `export_suplier` |
| **Master Sales** | `jendela_sales`, `tambah_sales`, `edit_sales`, `hapus_sales`, `import_sales`, `export_sales` |
| **Master Karyawan** | `jendela_karyawan`, `tambah_karyawan`, `edit_karyawan`, `hapus_karyawan`, `import_karyawan`, `export_karyawan` |
| **Penjualan Kasir** | `jendela_penjualan`, `penjualan`, `tambah_penjualan`, `detail_penjualan`, `hapus_penjualan`, `cetak_penjualan`, `export_penjualan`, `export_penjualan_detail` |
| **Pembelian Supplier** | `jendela_pembelian`, `pembelian`, `tambah_pembelian`, `detail_pembelian`, `hapus_pembelian`, `cetak_pembelian`, `export_pembelian`, `export_pembelian_detail` |
| **Retur Transaksi** | `retur_penjualan`, `tambah_retur_penjualan`, `detail_retur_penjualan`, `hapus_retur_penjualan`, `retur_pembelian`, `tambah_retur_pembelian`, `detail_retur_pembelian`, `hapus_retur_pembelian` |
| **Stok Opname** | `stok_opname`, `tambah_stok_opname`, `detail_stok_opname`, `hapus_stok_opname`, `cetak_stok_opname`, `import_stok_opname`, `export_stok_opname` |
| **Hutang & Piutang** | `jendela_hutang`, `hutang`, `tambah_hutang`, `bayar_hutang`, `cetak_hutang`, `export_hutang`, `histori_hutang`, `jendela_piutang`, `piutang`, `tambah_piutang`, `bayar_piutang`, `cetak_piutang`, `export_piutang`, `histori_piutang` |
| **Buku Kas & Biaya** | `jendela_cashflow`, `cashflow`, `tambah_cashflow`, `detail_cashflow`, `hapus_cashflow`, `cetak_cashflow`, `export_cashflow`, `jendela_biaya`, `tambah_biaya`, `detail_biaya`, `hapus_biaya`, `cetak_biaya`, `export_biaya`, `keuangan` |
| **Laporan & Analisis** | `laporan` (Hak akses membuka seluruh menu rekapitulasi dan grafik eksekutif) |
| **Pengaturan & Sistem** | `pengaturan`, `operator`, `atur_kolom`, `editing`, `printer`, `aplikasi`, `aktivasi`, `updates`, `online`, `apotek`, `minimar`, `about` |
