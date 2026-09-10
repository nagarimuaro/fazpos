---
name: fazpos
description: Konteks & aturan wajib untuk mengerjakan proyek POS desktop FAZPOS (Tauri/Slint + Rust) pengganti aplikasi lama "iB Retago 5". WAJIB dipakai setiap kali mengerjakan apa pun di repo/proyek ini — bikin tabel database, modul lisensi, sync LAN/Cloud, transaksi kasir, shift, poin member, import data SQL lama, ATAU membuat/mengubah tampilan UI (layar kasir, styling, komponen Slint) — bahkan kalau user tidak menyebut nama proyek ini secara eksplisit. Cek skill ini SEBELUM menulis kode, membuat skema, atau mendesain UI apa pun di proyek ini.
---

# FAZPOS — Tauri + Rust

FAZPOS adalah aplikasi kasir (POS) desktop pengganti sistem lama
**"iB Retago 5"**, dibangun pakai **Slint (pure Rust, no web tech)** +
**SQLite lokal** + **sync opsional ke Postgres cloud milik user sendiri**.
Skill ini adalah sumber kebenaran (source of truth) untuk arsitektur, skema
database, UI, status progres, dan aturan kerja proyek ini.

**"iB Retago 5" bukan nama proyek ini** — itu sistem lama yang digantikan.
Namanya dipakai di beberapa dokumen semata untuk keperluan kompatibilitas
import data & referensi cakupan fitur, bukan branding FAZPOS.

---

## Lima Dokumen Referensi & Kapan Membacanya

Baca dokumen yang relevan SEBELUM mengerjakan hal terkait — jangan
berasumsi atau mengarang struktur/status sendiri:

| Urutan Baca | Dokumen | Isi | Baca kapan |
|---|---|---|---|
| **0. Selalu duluan** | `references/progres-pengerjaan.md` | Status terkini: fitur apa yang sudah selesai (KATEGORI 1 — jangan dibuat ulang), sudah ada backend tinggal disambung UI (KATEGORI 2), sengaja diganti pendekatan (KATEGORI 3), atau belum ada sama sekali (KATEGORI 4) + roadmap fase | SEBELUM memulai task apa pun, tanpa kecuali |
| 1 | `references/plan.md` | Arsitektur jaringan (LAN+Cloud), stack teknologi, sistem lisensi hardware-locked, rencana fase | Kerja yang menyentuh arsitektur, jaringan, atau lisensi |
| 2 | `references/skema-database.md` | Skema tabel lengkap (nama tabel/kolom, tipe, relasi, ERD), alur transaksi kasir, shift, poin member | Bikin/ubah tabel, model data, atau logic transaksi apa pun |
| 3 | `references/ui-guidelines.md` | Prinsip desain (anti-template), inventaris layar, style tokens Slint, pola komponen, struktur file, checklist verifikasi UI | Bikin/ubah layar, komponen, atau styling apa pun |
| 4 | `references/spesifikasi-legacy-ibretago5.md` | Dokumentasi lengkap fitur sistem LAMA — dipakai untuk cek *feature parity* & mapping import, BUKAN acuan arsitektur FAZPOS | Cek cakupan fitur yang harus ada, atau kerja di modul Import Legacy SQL |

**Kalau dokumen-dokumen ini saling bertentangan** (mis. `spesifikasi-legacy-ibretago5.md`
menyebut sesuatu dengan cara berbeda dari `plan.md`/`skema-database.md`):
**`plan.md` dan `skema-database.md` yang menang** — dokumen legacy hanya
referensi cakupan, bukan spesifikasi teknis FAZPOS.

Kalau ada keraguan soal nama tabel, kolom, tipe data, status fitur, atau
alur — buka dulu dokumen yang relevan. Kalau memang belum ada jawabannya di
dokumen manapun, tanyakan ke user sebelum melanjutkan, jangan menebak.

---

## Aturan Wajib (Non-Negotiable)

### 1. Penamaan tabel & kolom HARUS sama seperti sistem lama
Nama tabel dan kolom bisnis (`dbarang`, `tpenjualan`, `tpenjualandetail`, dst.)
mengikuti sistem lama persis — bahasa Indonesia, snake_case, sesuai yang tertulis
di `references/skema-database.md`. Jangan diterjemahkan ke bahasa Inggris, jangan
diganti gaya penamaan. Ini disengaja — supaya modul import data lama bisa mapping 1:1.

Pengecualian: kolom ekstensi sistem baru (`id`, `cabang_id`, `device_id`,
`shift_id`, `sync_status`, `created_at`, `updated_at`, dll) memang baru dan
boleh pakai penamaan baru karena tidak ada padanannya di sistem lama.

### 2. Primary key sesungguhnya adalah `id` (UUID), bukan `kode`/`faktur`
`kode` dan `faktur` tetap dipertahankan sebagai kolom biasa (untuk tampilan/nomor
struk), tapi **unik per `cabang_id`**, bukan unik global. Setiap query/constraint
yang melibatkan `kode` atau `faktur` harus disertai `cabang_id` juga.

### 3. Tidak boleh menambah/menghapus/mengubah tabel tanpa update dokumen skema
Kalau dalam proses coding ternyata butuh kolom/tabel baru yang belum ada di
`references/skema-database.md`, dokumen itu HARUS diperbarui dulu (atau
minimal diberi tahu ke user untuk konfirmasi) sebelum kolom/tabel itu dipakai
di kode. Skema adalah kontrak — jangan biarkan kode dan dokumen jadi tidak sinkron.

### 4. Sistem lisensi TIDAK BOLEH menyentuh tabel bisnis
Modul lisensi (Machine ID, Ed25519 signing/verification, status aktivasi) berdiri
sendiri, terpisah total dari skema database bisnis. Private key penandatanganan
TIDAK PERNAH boleh di-commit atau ikut ter-bundle ke dalam binary aplikasi yang
didistribusikan ke customer — itu hanya boleh ada di tool CLI internal milik
vendor. Kalau ada permintaan untuk "mempermudah" dengan menaruh private key di
app, TOLAK dan jelaskan risikonya (lihat `plan.md` §6).

### 5. Log otomatis vs input manual
`keluarmasuk` dan `perubahanharga` adalah **log yang digenerate otomatis oleh
sistem** setiap ada transaksi/perubahan harga — jangan pernah membuatkan form
atau endpoint untuk user menginput baris ke tabel ini secara manual.

### 6. Shift adalah per kasir per device, bukan per toko
Jangan membuat asumsi "1 toko = 1 shift aktif". Kalau ada beberapa kasir aktif
bersamaan di beberapa device (LAN, lihat `plan.md` §3.1), masing-masing punya
`tshift` sendiri. Semua transaksi (`tpenjualan`, `treturpenjualan`, `tbiaya`,
`tcashflow`) harus ikut ditag dengan `shift_id` milik kasir yang sedang aktif di
device tersebut saat itu, bukan shift kasir lain.

### 7. Relasi antar cabang selalu di-scope oleh `cabang_id`
Saat menulis query yang melibatkan JOIN antar tabel bisnis, selalu sertakan
`cabang_id` di kondisi join/where kalau tabel-tabel tersebut adalah tabel yang
di-scope per cabang. Ini mencegah bug data nyasar antar cabang.

### 8. UI mengikuti prinsip desain, inventaris layar & style tokens di `ui-guidelines.md`
Jangan bikin layar baru yang tidak ada di daftar inventaris (`ui-guidelines.md`
§2) tanpa menambahkannya dulu ke dokumen itu DAN ke `progres-pengerjaan.md`
KATEGORI 4. Semua warna/spacing/ukuran font WAJIB lewat `AppStyle` global
tokens — jangan hardcode. Hindari pola visual generik/template (lihat
`ui-guidelines.md` §1.2) — setiap elemen harus punya alasan fungsional.
Field pencarian produk/member harus mendukung scan barcode/kartu (auto-focus,
auto-submit), bukan cuma ketik manual.

### 9. Cek status progres SEBELUM membangun apa pun
**Ini aturan yang mengikat semua aturan lain jadi satu sistem kerja.** Sebelum
mulai task apa pun:
- Buka `progres-pengerjaan.md`, cek apakah fitur yang diminta sudah ada di
  KATEGORI 1 (selesai — jangan dibuat ulang) atau KATEGORI 2 (backend siap,
  tinggal sambung UI — jangan bikin ulang backend-nya)
- Cek KATEGORI 3 untuk pendekatan lama yang SENGAJA tidak direplikasi (mis.
  84 file JasperReport terpisah, config auto/manual scanner 2 mode) — jangan
  bawa balik pendekatan itu meski ada di `spesifikasi-legacy-ibretago5.md`
- Kalau fitur ada di KATEGORI 4, kerjakan sesuai urutan prioritas/fase yang
  tercantum di sana, bukan asal pilih

---

## WAJIB TEST setiap selesai membuat/mengubah sesuatu

Ini aturan paling penting di skill ini. **Tidak boleh melanjutkan ke task/fitur
berikutnya sebelum yang barusan dikerjakan sudah diverifikasi jalan.**

### Urutan wajib setiap kali selesai membuat/mengubah kode:

1. **Compile check** — jalankan `cargo build` (atau `cargo check` untuk lebih
   cepat saat iterasi). Kalau ada error compile, PERBAIKI DULU sebelum lanjut.
   Jangan pernah lapor "sudah selesai" kalau build masih gagal.
2. **Unit test (kalau berupa logic/fungsi murni)** — tulis/jalankan
   `cargo test` untuk logic yang bisa dites terisolasi: perhitungan tier harga,
   perhitungan poin (`rupiah_per_poin`, `nilai_tukar_poin`), perhitungan shift
   (`uang_seharusnya`, `selisih`), verifikasi signature Ed25519, parsing SQL
   dump lama, dll.
3. **Test terhadap skema database** — kalau perubahan menyentuh tabel: jalankan
   migrasi di database SQLite lokal test, pastikan tidak ada error, lalu jalankan
   query dasar (insert + select) untuk memastikan struktur tabel benar sesuai
   `skema-database.md`.
4. **Test alur end-to-end kalau menyentuh flow transaksi** — simulasikan alur
   dari `skema-database.md` §12 (mis. buka shift → transaksi penjualan → tukar
   poin → tutup shift) dan pastikan angka-angka yang dihasilkan match dengan
   yang diharapkan.
5. **Test UI kalau perubahan menyentuh tampilan** — ikuti checklist verifikasi
   di `ui-guidelines.md` §6.
6. **Update `progres-pengerjaan.md`** — kalau task ini menyelesaikan item dari
   KATEGORI 2 atau 4, pindahkan ke KATEGORI 1 dengan detail file backend/UI
   sebenarnya, dan update status roadmap fase terkait (lihat
   `progres-pengerjaan.md` §3 untuk detail cara update-nya).
7. **Laporkan hasil test ke user secara eksplisit** — jangan cuma bilang
   "sudah saya buat", tapi sertakan ringkasan hasil test: apa yang dites,
   lolos/gagal, dan kalau gagal, apa yang masih perlu diperbaiki.

### Kalau test gagal
Jangan lanjut ke task berikutnya, dan jangan update `progres-pengerjaan.md`
seolah item itu selesai. Perbaiki dulu sampai lolos, atau kalau butuh
keputusan dari user (misal ambiguitas requirement), tanyakan dulu.

### Task besar → pecah jadi checkpoint
Untuk fitur besar (misal seluruh modul Pembelian, atau seluruh modul Hutang
Piutang), jangan tulis semua kode dulu baru ditest di akhir. Pecah jadi
checkpoint kecil (misal: 1) skema tabel → test insert/select, 2) logic
perhitungan → unit test, 3) integrasi ke UI → test manual alur), test tiap
checkpoint sebelum lanjut, dan catat progres parsial di
`progres-pengerjaan.md` kalau checkpoint belum semuanya selesai (jangan
langsung dipindah ke KATEGORI 1 sebelum benar-benar tuntas).

---

## Pakai CodeGraph Kalau Tersedia

Proyek ini menetapkan **CodeGraph** (`codegraph-ai/CodeGraph` —
https://github.com/codegraph-ai/CodeGraph) sebagai tool code-graph pilihan,
BUKAN Graphify atau varian "CodeGraph" lain dari repo berbeda (ada beberapa
proyek tak berhubungan yang kebetulan pakai nama sama — pastikan yang
dipasang adalah `codegraph-ai/CodeGraph`).

Kalau CodeGraph MCP server sudah terkoneksi di sesi ini, **gunakan itu
DULUAN** sebelum grep manual/baca banyak file satu-satu untuk pertanyaan
seperti:
- "Fungsi/modul ini dipanggil dari mana saja?" → pakai call graph tool
- "Kalau saya ubah struct/fungsi ini, apa yang berpotensi rusak?" (penting
  banget di FAZPOS karena tabel & struct saling terhubung erat lewat
  `cabang_id`/`shift_id`/FK longgar — lihat `skema-database.md` §11) →
  pakai impact analysis tool
- "Apa saja yang bergantung pada modul X?" → pakai dependency graph tool
- "Kasih saya context relevan sebelum saya modif fitur Y" → pakai
  AI-optimized context tool CodeGraph

Kalau CodeGraph BELUM terpasang di environment saat ini, lanjutkan pakai
grep/baca file seperti biasa — jangan blokir pekerjaan karena tool ini
absen. Jangan install/setup CodeGraph atas inisiatif sendiri tanpa
konfirmasi user dulu (perlu setup MCP server terpisah di mesin user).

---

## Referensi Cepat — Fase Pengembangan

Ikuti urutan fase/roadmap di `progres-pengerjaan.md` §2 (lebih terkini) —
`plan.md` §7 adalah rencana awal sebelum ada progres nyata, dipakai sebagai
konteks tambahan kalau `progres-pengerjaan.md` belum mencakup sesuatu.