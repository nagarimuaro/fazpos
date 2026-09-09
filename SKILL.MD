---
name: fazpos
description: Konteks & aturan wajib untuk mengerjakan proyek POS desktop (Tauri/Slint + Rust) FAZPOS pengganti aplikasi lama "iB Retago 5". WAJIB dipakai setiap kali mengerjakan apa pun di repo/proyek ini — bikin tabel database, modul lisensi, sync LAN/Cloud, transaksi kasir, shift, poin member, atau import data SQL lama — bahkan kalau user tidak menyebut nama proyek ini secara eksplisit. Cek skill ini SEBELUM menulis kode, membuat skema, atau mengubah struktur data apa pun di proyek ini.
---

# FAZPOS — Tauri + Rust

Proyek ini adalah aplikasi kasir (POS) desktop FAZPOS pengganti sistem lama "iB Retago 5",
dibangun pakai **Slint (pure Rust, no web tech)** + **SQLite lokal** + **sync opsional
ke Postgres cloud milik user sendiri**. Skill ini adalah sumber kebenaran (source of
truth) untuk arsitektur, skema database, dan aturan kerja proyek ini.

**Dua dokumen referensi WAJIB dibaca sebelum mengerjakan hal terkait:**

| Dokumen | Isi | Baca kapan |
|---|---|---|
| `references/plan.md` | Arsitektur jaringan (LAN+Cloud), stack teknologi, sistem lisensi hardware-locked, rencana fase | Sebelum kerja apa pun yang menyentuh arsitektur, jaringan, atau lisensi |
| `references/skema-database.md` | Skema tabel lengkap (nama tabel/kolom, tipe, relasi, ERD), alur transaksi kasir, shift, poin member | Sebelum bikin/ubah tabel, model data, atau logic transaksi apa pun |

Jangan berasumsi atau mengarang struktur sendiri — kalau ada keraguan soal nama
tabel, kolom, tipe data, atau alur, **buka dulu dokumen referensi di atas**. Kalau
sesuatu memang belum ada di dokumen, tanyakan ke user sebelum melanjutkan, jangan
menebak.

---

## Aturan Wajib (Non-Negotiable)

### 1. Penamaan tabel & kolom HARUS sama seperti sistem lama
Nama tabel dan kolom bisnis (`dbarang`, `tpenjualan`, `tpenjualandetail`, dst.)
mengikuti sistem lama persis — bahasa Indonesia, snake_case, sesuai yang tertulis
di `references/skema-database.md`. Jangan diterjemahkan ke bahasa Inggris, jangan
diganti gaya penamaan, walau secara konvensi Rust/SQL modern "lebih baik" begini-begitu.
Ini disengaja — supaya modul import data lama bisa mapping 1:1.

Pengecualian: kolom ekstensi sistem baru (`id`, `cabang_id`, `device_id`,
`shift_id`, `sync_status`, `created_at`, `updated_at`, dll — didaftarkan lengkap
di §2 `skema-database.md`) memang baru dan boleh pakai penamaan baru karena tidak
ada padanannya di sistem lama.

### 2. Primary key sesungguhnya adalah `id` (UUID), bukan `kode`/`faktur`
`kode` dan `faktur` tetap dipertahankan sebagai kolom biasa (untuk tampilan/nomor
struk), tapi **unik per `cabang_id`**, bukan unik global. Setiap query/constraint
yang melibatkan `kode` atau `faktur` harus disertai `cabang_id` juga. Jangan buat
`kode`/`faktur` sebagai primary key literal di tabel manapun.

### 3. Tidak boleh menambah/menghapus/mengubah tabel tanpa update dokumen skema
Kalau dalam proses coding ternyata butuh kolom/tabel baru yang belum ada di
`references/skema-database.md`, tabel dokumen itu HARUS diperbarui dulu (atau
minimal diberi tahu ke user untuk konfirmasi) sebelum kolom/tabel itu dipakai di
kode. Skema adalah kontrak — jangan biarkan kode dan dokumen jadi tidak sinkron.

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
di-scope per cabang (lihat aturan di `skema-database.md` §11 bagian "Aturan kunci
untuk agent"). Ini mencegah bug data nyasar antar cabang.

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
   yang diharapkan (uang_seharusnya, poin_saldo, stok berkurang, dst).
5. **Laporkan hasil test ke user secara eksplisit** — jangan cuma bilang
   "sudah saya buat", tapi sertakan ringkasan hasil test: apa yang dites,
   lolos/gagal, dan kalau gagal, apa yang masih perlu diperbaiki.

### Kalau test gagal
Jangan lanjut ke task berikutnya. Perbaiki dulu sampai lolos, atau kalau
butuh keputusan dari user (misal ambiguitas requirement), tanyakan dulu —
jangan menebak lalu lanjut membangun di atas fondasi yang belum tervalidasi.

### Task besar → pecah jadi checkpoint
Untuk fitur besar (misal seluruh modul shift, atau seluruh modul import legacy
SQL), jangan tulis semua kode dulu baru ditest di akhir. Pecah jadi
checkpoint kecil (misal: 1) skema tabel → test insert/select, 2) logic
perhitungan → unit test, 3) integrasi ke UI → test manual alur), dan test tiap
checkpoint sebelum lanjut ke checkpoint berikutnya.

---

## Referensi Cepat — Fase Pengembangan

Ikuti urutan fase dari `plan.md` §7 kecuali user secara eksplisit minta lompat
urutan. Fase 1-2 (setup dasar + lisensi) adalah fondasi yang harus solid dan
teruji sebelum lanjut ke fase yang bergantung padanya.