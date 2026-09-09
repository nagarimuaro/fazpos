# Plan Pengembangan Aplikasi POS — Tauri + Rust

## 1. Ringkasan Proyek

| Aspek | Keterangan |
|---|---|
| Jenis usaha target | Retail/toko (barang fisik, stok) |
| Platform | Desktop (Windows, native app — bukan browser) |
| Model bisnis | Lisensi lifetime, sekali aktivasi, terikat ke hardware (SSD/HDD) |
| Skala | Multi-PC via LAN (dalam 1 toko) + Multi-cabang via Cloud (opsional) |
| Cloud | Bring-your-own-cloud — user pakai DB cloud sendiri, kita hanya sediakan integrasi |
| Prioritas | Kecepatan, setup semudah mungkin, tampilan (styling) fleksibel, tanpa gambar produk |

---

## 2. Stack Teknologi

| Layer | Pilihan | Alasan |
|---|---|---|
| UI Framework | **Slint** (pure Rust, no web tech) | Styling fleksibel (mirip QML/CSS-ish), native rendering, ringan, cocok untuk kios/kasir |
| Bahasa | Rust (100%, termasuk UI logic) | Performa tinggi, sulit di-reverse-engineer, binary kecil |
| Database lokal | SQLite (`rusqlite`/`sqlx`) | Wajib untuk offline-first, disimpan di PC yang berperan Server Lokal per toko |
| LAN server & discovery | `axum` (mini server tertanam) + `mdns-sd` (auto-discovery) | PC lain di 1 toko konek otomatis tanpa setting IP manual |
| Cloud DB (milik user) | PostgreSQL (BYO — Supabase/Neon/VPS user) | Hanya untuk gabungkan data antar cabang, kita tidak hosting |
| Sync ke cloud | Rust (`sqlx` + `tokio`) + outbox pattern | Server Lokal tiap cabang push/pull ke Cloud DB user saat online |
| Printer struk | ESC/POS via `serialport`/`escpos-rs` | Native, tanpa driver tambahan |
| Barcode scanner | Tidak perlu library khusus | Scanner USB umumnya bertindak sebagai keyboard (HID) |

---

## 3. Arsitektur Jaringan — 2 Tingkat (LAN & Cloud)

Supaya setup tetap simpel buat pengguna awam, sistem dibagi jadi 2 lapis independen — **LAN wajib**, **Cloud opsional**:

```
                    ┌─────────────────────────────────────────┐
                    │            1 TOKO / CABANG               │
                    │                                           │
                    │   [PC Kasir 1]        [PC Kasir 2]        │
                    │   role: SERVER   ◄──►  role: CLIENT       │
                    │   (SQLite utama)       (kirim/terima      │
                    │        ▲                via LAN)          │
                    │        │                    ▲             │
                    │   [PC Kasir 3]              │             │
                    │   role: CLIENT ──────────────┘             │
                    │   (auto-ditemukan via LAN discovery)      │
                    └───────────────────┬───────────────────────┘
                                         │  (opsional, kalau online)
                                         ▼
                         ┌───────────────────────────────┐
                         │   CLOUD DATABASE MILIK USER     │
                         │   (Postgres — Supabase/Neon/    │
                         │    VPS sendiri/apa saja)        │
                         │   → hanya dipakai utk gabungkan │
                         │     data ANTAR CABANG            │
                         └───────────────────────────────┘
                                         ▲
                                         │
                    ┌───────────────────┴───────────────────────┐
                    │            TOKO / CABANG LAIN               │
                    │        (struktur LAN sama seperti di atas)  │
                    └─────────────────────────────────────────────┘
```

### 3.1 Tingkat 1 — LAN (dalam 1 toko, multi-PC)

- Satu PC berperan sebagai **Server Lokal** (menyimpan SQLite utama untuk toko itu, jalankan mini web-server di background lewat `axum` + `tokio`)
- PC lain di toko yang sama berperan sebagai **Client Lokal**, terhubung ke Server Lokal via LAN (kabel/switch, tanpa perlu internet sama sekali)
- **Auto-discovery** — client tidak perlu input IP manual:
  - App broadcast/scan jaringan lokal pakai **mDNS** (crate `mdns-sd`) atau UDP broadcast sederhana
  - Saat app dibuka di PC baru, otomatis muncul daftar "Server ditemukan: Kasir-Utama (192.168.1.5)" → tinggal klik "Gabung"
  - Tidak perlu setting IP statis, port forwarding, atau config jaringan manual

### 3.2 Tingkat 2 — Cloud (antar cabang, opsional)

- Hanya dipakai kalau usaha punya **lebih dari 1 cabang**
- **Cloud di sini murni database** — bukan aplikasi/server yang kita host. User pakai cloud database miliknya sendiri (Postgres di Supabase, Neon, RDS, atau VPS pribadi)
- Yang kita bangun: **modul sync** di dalam app yang tinggal diisi *connection string* (host, port, nama DB, user, password) di halaman Settings → Cloud Sync
- Server Lokal tiap cabang yang bertugas push/pull data ke Cloud DB tersebut (bukan tiap PC kasir langsung konek ke cloud — biar lebih rapi & satu pintu per cabang)
- Kalau cabang cuma satu, fitur ini bisa di-skip total, aplikasi tetap jalan penuh cukup LAN + SQLite

### 3.3 Kenapa desain ini simpel buat user

- User **tidak perlu ngerti jaringan** — cukup pilih "Jadi Server" atau "Gabung sebagai Kasir" saat pertama buka app di tiap PC
- User **tidak perlu install database server terpisah** — SQLite sudah ikut menyatu di dalam app
- Kalau mau multi-cabang, user cuma perlu **1 kali isi connection string cloud** di 1 PC (Server Lokal), bukan di semua PC kasir
- Kalau tidak butuh cloud sama sekali, app tetap 100% fungsional (offline-first sejak awal)

**Prinsip kunci soal stok:** stok dikelola **per-cabang** (bukan stok global gabungan) untuk menghindari konflik sync antar cabang. Perpindahan barang antar cabang dilakukan lewat dokumen mutasi stok yang eksplisit, bukan otomatis digabung.

### 3.4 Catatan Keamanan Cloud (BYO-Cloud)

Karena koneksi ke cloud DB milik user bisa lewat internet terbuka, perlu diperhatikan:

| Risiko | Mitigasi |
|---|---|
| Kredensial DB di-embed di app | Simpan connection string terenkripsi di local storage, bukan plain text |
| Koneksi tanpa enkripsi | Wajibkan SSL/TLS untuk koneksi Postgres (`sslmode=require`) |
| DB terekspos publik | Anjurkan user pakai provider yang sudah kasih SSL + firewall/IP allowlist bawaan (Supabase/Neon sudah default aman) |
| Sync gagal parsial | Outbox pattern tetap dipakai — retry otomatis, tidak ada data hilang saat koneksi putus |

---

## 4. Cakupan Fungsional (garis besar)

Ini daftar area fitur yang akan dibahas detail satu per satu di tahap berikutnya — di sini baru garis besarnya:

- Master data (produk, pelanggan, cabang, user/kasir)
- Transaksi kasir & pembayaran
- Manajemen stok (termasuk mutasi antar cabang)
- Program poin pelanggan
- Laporan
- Sistem lisensi lifetime hardware-locked (dasarnya sudah dirancang di bagian 6)
- Sinkronisasi LAN & Cloud (dasarnya sudah dirancang di bagian 3)

---

## 5. Skema Database Baru & Kompatibilitas dengan Sistem Lama

### 5.1 Prinsip

- Aplikasi baru pakai skema baru yang **ternormalisasi & bertipe kuat** (integer/decimal/date/boolean asli, foreign key jelas) — bukan copy skema lama, karena skema lama (varchar semua, tanpa FK) tidak cocok untuk sync multi-cabang & performa
- Tapi client lama (pengguna aplikasi sebelumnya, terdeteksi: **iB Retago 5**, MariaDB) harus bisa **upload file `.sql` mereka dan datanya otomatis masuk** ke skema baru — tanpa mapping manual tiap kali

### 5.2 Ringkasan Skema Lama (hasil analisis file yang diupload)

| Kategori | Tabel |
|---|---|
| Master data | `dbarang` (produk + 5 tier harga), `dpelanggan`, `dsuplier`, `dsales`, `dkaryawan`, `doperator` (user + izin akses per-kolom) |
| Transaksi | `tpenjualan`/`tpembelian` + detail, versi *pending*, retur penjualan/pembelian + detail |
| Keuangan | `tcashflow`, `tbiaya`, `thutang`, `tpiutang` |
| Stok | `tstokopname`, `keluarmasuk` (log stok masuk/keluar), `perubahanharga` (log histori harga) |
| Konfigurasi | `daplikasi` (info toko/struk), `aturkolom`, `checkboxediting`, `autokode`, tabel `combo*` (dropdown), `barcode`, `dpetunjuk` |

**Karakteristik yang mempengaruhi desain import:** semua kolom bertipe varchar/text (termasuk angka, tanggal, boolean sebagai string `'true'`/`'false'`); tidak ada foreign key eksplisit (relasi hanya lewat kolom `kode`); harga bertingkat (qty_1..5) disimpan flat per baris produk; hak akses user berupa ±100 kolom terpisah, bukan tabel permission relasional.

### 5.3 Modul Import Legacy SQL

```
1. User upload file .sql dari aplikasi lama
2. App PARSE file dump (baca teks, ekstrak nama tabel + isi INSERT)
   — cukup baca file, tidak perlu koneksi live ke MySQL/MariaDB
3. App cocokkan tiap tabel lama ke MAPPING yang sudah didefinisikan
   (mapping disimpan terpisah dari core app, mudah diperbarui kalau
   ada variasi versi skema lama di kemudian hari)
4. App KONVERSI tipe data: varchar → integer/decimal/date/boolean asli,
   dan urai kolom harga bertingkat (qty_1..5) jadi baris terpisah
5. App INSERT ke skema baru, jaga relasi (kode produk/pelanggan diubah
   jadi ID relasional yang konsisten)
6. App tampilkan ringkasan hasil: jumlah produk/transaksi/pelanggan
   yang berhasil masuk, dan baris yang gagal (dicatat di log, tidak
   menggagalkan keseluruhan proses)
```

### 5.4 Cakupan Data yang Diimpor (berurutan sesuai prioritas)

1. **Produk** — prioritas utama. Begitu file `.sql` diupload, data produk (`dbarang`, termasuk tier harga qty_1..5) harus langsung tersedia dan siap dipakai transaksi di app baru, tanpa langkah manual tambahan
2. Master lain: pelanggan, supplier, sales, karyawan
3. Histori transaksi: penjualan, pembelian, retur (beserta detail item)
4. Keuangan: hutang, piutang, cashflow, biaya
5. Stok: hasil stok opname, log mutasi stok
6. User & hak akses (dipetakan ke sistem permission baru yang lebih terstruktur)
7. Konfigurasi toko (nama, alamat, header/footer struk, dll dari `daplikasi`)

Semua kategori dipetakan pakai mekanisme mapping yang sama (lihat 5.3) — produk hanya didahulukan urutan prosesnya karena itu yang harus langsung bisa dipakai kasir setelah import selesai.

### 5.5 Pertimbangan Teknis

| Hal | Pendekatan |
|---|---|
| Parsing file SQL dump | Crate `sqlparser` (parse AST) atau extractor berbasis regex khusus statement `INSERT` — cukup untuk dump data, tidak perlu eksekusi query |
| Encoding karakter | Beberapa tabel lama pakai charset `ascii`/`latin1` → wajib dikonversi ke UTF-8 saat insert |
| Data tidak konsisten | Baris yang gagal dikonversi (angka kosong, format tanggal aneh) dicatat di log import, proses tetap lanjut untuk baris lain |
| Sifat proses | Sekali jalan saat migrasi awal client pindah ke app baru — bukan proses berjalan terus-menerus |

---

## 6. Sistem Lisensi — Hardware-Locked, Offline, Anti-Reverse

### 6.1 Konsep Dasar

- Setiap instalasi menghasilkan **Machine ID** unik dari hardware disk (SSD/HDD, auto-detect) — dikombinasikan dengan identifier tambahan (volume ID, dsb.) agar tidak mudah dispoof
- Aktivasi bersifat **offline**: user kirim Machine ID ke vendor → vendor generate token → user input token ke app
- Token di-generate menggunakan **kriptografi asimetris (Ed25519)** — bukan HMAC shared-secret

### 6.2 Alur Aktivasi

```
1. App baca Hardware ID (disk serial + fallback identifier)
2. App hasilkan Machine ID (hash dari kombinasi identifier)
3. User kirim Machine ID ke vendor (WA/email)
4. Vendor sign Machine ID pakai Private Key (tool CLI internal)
5. Vendor kirim Activation Token ke user
6. App verifikasi token pakai Public Key yang tertanam
7. Jika valid → status "activated" tersimpan permanen, terikat ke Machine ID tsb
```

### 6.3 Kenapa Ed25519 (asymmetric), bukan shared-secret

- **1 Private Key** dipegang vendor sendiri, tidak pernah didistribusikan
- **1 Public Key** ditanam di semua copy aplikasi (aman untuk terekspos)
- Meski app di-reverse-engineer total, penyerang hanya menemukan Public Key — yang **tidak bisa** dipakai untuk generate token baru
- Setiap Machine ID menghasilkan token berbeda meski private key yang dipakai sama (analog: satu tanda tangan asli, dipakai menandatangani banyak dokumen berbeda, tiap hasil unik)

### 6.4 Lapisan Proteksi Tambahan

| Lapis | Tujuan | Implementasi |
|---|---|---|
| 1. Cryptographic signing | Cegah pemalsuan token (matematis, paling kuat) | Ed25519 (`ed25519-dalek`) |
| 2. Anti-tamper status lokal | Cegah edit manual file status aktivasi | Simpan HMAC/signature dari (machine_id + flag), bukan flag polos |
| 3. Binary hardening | Perlambat reverse engineering | Strip symbol, `obfstr` untuk string, anti-debugging check |
| 4. Machine ID robust | Cegah spoofing identifier | Kombinasi beberapa hardware identifier, di-hash |

**Catatan realistis:** tidak ada proteksi yang 100% tidak bisa dibobol — tujuannya membuat crack memakan waktu/effort yang tidak sepadan dibanding harga lisensi, sambil menutup celah pemalsuan token secara matematis lewat asymmetric signing.

### 6.5 Komponen yang Perlu Dibangun

1. **Machine ID generator** — baca disk serial, hashing (`sha2`/`blake3`)
2. **Tool CLI signing** (terpisah, dipegang vendor) — input Machine ID → output token
3. **Modul verifikasi + local activation storage** (tertanam di app POS) — Public Key + anti-tamper

---

## 7. Rencana Fase Pengembangan

| Fase | Fokus |
|---|---|
| 1 | Setup project Slint + SQLite; CRUD produk & kasir dasar (single device, offline) |
| 2 | Sistem lisensi hardware-locked (Machine ID, signing tool, verifikasi) |
| 3 | Modul Import Legacy SQL (parsing dump, mapping, konversi tipe data) |
| 4 | Transaksi lengkap + cetak struk (ESC/POS) + scan barcode |
| 5 | Sistem poin pelanggan |
| 6 | Mode multi-PC via LAN: role Server/Client + auto-discovery (mDNS) |
| 7 | Modul Cloud Sync (BYO-cloud): setting connection string, push/pull antar cabang |
| 8 | Resolusi konflik stok antar cabang, laporan & dashboard gabungan |
| 9 | Hardening: binary protection, retry sync, backup lokal |