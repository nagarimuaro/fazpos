# Spesifikasi Skema Database — POS (untuk Development Agent)

> Nama proyek: **FAZPOS**. "iB Retago 5" yang disebut di dokumen ini adalah SISTEM LAMA yang digantikan/di-import datanya — bukan nama proyek baru.

Dokumen ini adalah acuan teknis skema database. **Nama tabel & nama kolom
mengikuti sistem lama (iB Retago 5)** agar 1:1 dengan modul Import Legacy SQL
(lihat Plan bagian 5). Yang BARU hanya: kolom ekstensi arsitektur (cabang,
device/LAN, sync) dan beberapa tabel baru untuk fitur yang belum ada di
sistem lama (poin pelanggan, cabang, device).

Sistem lisensi (bagian 6 di plan) **tidak menyentuh tabel bisnis** — cukup
modul terpisah (aktivasi & verifikasi), tidak perlu direpresentasikan di
skema ini.

---

## 1. Prinsip Desain

1. **Nama tabel & kolom = sama seperti lama** (snake_case, Bahasa Indonesia) — memudahkan mapping saat import & memudahkan tim yang sudah familiar sistem lama
2. **Primary Key diganti** dari `kode`/`faktur` (string bisa collision) menjadi `id` (UUID/TEXT) — supaya aman digabung dari banyak cabang & banyak device via LAN tanpa bentrok
3. **Kolom `kode`/`faktur` asli tetap ada** sebagai kolom biasa (bukan PK) — dipakai untuk tampilan/nomor struk, unik **per cabang**, format disarankan: `{kode_cabang}-{nomor_urut}` (mis. `CB01-000123`)
4. **Kolom ekstensi standar** ditambahkan ke semua tabel transaksi & master (lihat §2)
5. Tipe data diperbaiki dari varchar-semua (sistem lama) menjadi tipe asli (INTEGER, DECIMAL, DATE, BOOLEAN)

---

## 2. Kolom Ekstensi Standar

Ditambahkan ke **semua tabel master & transaksi** (tidak ditulis ulang di tiap tabel di bawah, cukup diasumsikan ada):

| Kolom | Tipe | Keterangan |
|---|---|---|
| `id` | TEXT (UUID) | Primary key baru, pengganti kode/faktur |
| `cabang_id` | TEXT (FK → `cabang.id`) | Data ini milik cabang mana |
| `device_id` | TEXT (FK → `device.id`) | Device/terminal asal input (untuk histori LAN) |
| `created_at` | DATETIME | Waktu dibuat |
| `updated_at` | DATETIME | Waktu terakhir diubah |
| `sync_status` | TEXT (`local_only`\|`pending`\|`synced`) | Status outbox pattern untuk cloud sync |

---

## 3. Tabel Baru — Bagian Arsitektur (Cabang & Device)

### `cabang`
| Kolom | Tipe | Keterangan |
|---|---|---|
| `id` | TEXT PK | |
| `kode_cabang` | TEXT UNIQUE | Dipakai sbg prefix nomor faktur |
| `nama_cabang` | TEXT | |
| `alamat` | TEXT | |
| `created_at` | DATETIME | |

### `device`
| Kolom | Tipe | Keterangan |
|---|---|---|
| `id` | TEXT PK | |
| `cabang_id` | TEXT FK → `cabang.id` | |
| `nama_device` | TEXT | |
| `role` | TEXT (`server`\|`client`) | Sesuai arsitektur LAN (plan §3.1) |
| `last_seen` | DATETIME | |

---

## 4. Tabel Master

### `dbarang` (Produk) — **prioritas #1 saat import**
| Kolom | Tipe | Relasi/Keterangan |
|---|---|---|
| `kode` | TEXT | Kode produk asli, UNIQUE per `cabang_id` |
| `nama` | TEXT | |
| `kategori` | TEXT | |
| `stok` | INTEGER | |
| `stok_min` | INTEGER | |
| `satuan` | TEXT | |
| `harga_beli` | DECIMAL | |
| `qty_1..qty_5` | INTEGER | Ambang jumlah untuk tier harga |
| `harga_jual_1..harga_jual_5` | DECIMAL | Harga per tier |
| `suplier` | TEXT | FK longgar → `dsuplier.kode` |
| `diskon` | DECIMAL | |
| `lokasi` | TEXT | |
| `expired` | DATE | |
| *+ kolom ekstensi §2* | | |

### `dpelanggan`
| Kolom | Tipe | Keterangan |
|---|---|---|
| `kode`, `nama`, `alamat`, `no_hp`, `email`, `rekening`, `keterangan` | TEXT | sama seperti lama |
| `diskon_persen` | DECIMAL | |
| `poin_saldo` | INTEGER DEFAULT 0 | **BARU** — saldo poin berjalan |
| `id_kartu` | TEXT UNIQUE (per cabang), nullable | **BARU** — nilai yang di-print/encode di kartu member fisik (barcode/QR), berbeda dari `kode` internal. Ini yang dibaca saat kartu di-scan |
| *+ kolom ekstensi §2* | | |

**Pencarian member (kasir):** satu kotak input yang menerima — ketik manual (`nama`, `kode`, atau `no_hp`, dicocokkan pakai `LIKE`/partial match), **atau** scan kartu fisik (scanner HID otomatis mengetik nilai `id_kartu` persis, dicocokkan exact match). Tidak perlu mode terpisah — sistem cukup deteksi: kalau hasil ketik cocok persis dengan sebuah `id_kartu`, langsung anggap itu hasil scan.

### `dsuplier`, `dsales`, `dkaryawan`
Struktur identik satu sama lain (pola dari sistem lama): `kode`, `nama`, `alamat`, `no_hp`, `email`, `rekening`, `keterangan` + kolom ekstensi §2.

### `doperator` (User & Hak Akses)
| Kolom | Tipe | Keterangan |
|---|---|---|
| `kode` | TEXT | |
| `username` | TEXT UNIQUE | |
| `password` | TEXT | **Wajib di-hash** (bcrypt/argon2), tidak plaintext seperti sistem lama |
| `edit_penjualan`, `hapus_penjualan`, `tambah_penjualan`, ... (± seratus kolom hak akses per fitur) | BOOLEAN | Dipertahankan pola per-kolom sama seperti lama, tipe diubah jadi BOOLEAN asli (bukan `char(50)` string `'true'/'false'`) |
| `cabang_id` | TEXT FK, nullable | NULL = akses semua cabang (super-admin) |

---

## 5. Tabel Transaksi

### `tpenjualan`
| Kolom | Tipe | Relasi |
|---|---|---|
| `faktur` | TEXT | Nomor struk, unik per `cabang_id`, format `{kode_cabang}-{urutan}` |
| `tanggal` | DATE | |
| `waktu` | TIME | |
| `tunai_kredit` | TEXT | |
| `nhari` | INTEGER | |
| `jatuh_tempo` | DATE | |
| `kode_pelanggan` | TEXT | FK → `dpelanggan.kode` |
| `pelanggan` | TEXT | snapshot nama (seperti lama) |
| `kode_sales` | TEXT | FK → `dsales.kode` |
| `sales` | TEXT | snapshot nama |
| `total`, `diskon_persen`, `diskon`, `ppn`, `bayar`, `kembali`, `total_harga_beli`, `total_laba` | DECIMAL | |
| `operator` | TEXT | FK → `doperator.kode` |
| `poin_ditukar` | INTEGER DEFAULT 0 | **BARU** — jumlah poin yang ditukar pada transaksi ini |
| `nilai_tukar_poin` | DECIMAL DEFAULT 0 | **BARU** — nilai rupiah dari poin yang ditukar (mengurangi total bayar, terpisah dari `diskon` biasa) |

### `tpenjualandetail`
| Kolom | Tipe | Relasi |
|---|---|---|
| `faktur` | TEXT | FK → `tpenjualan.faktur` (scoped per `cabang_id`) |
| `kode` | TEXT | FK → `dbarang.kode` |
| `nama`, `satuan`, `jenis_harga` | TEXT | |
| `jumlah` | INTEGER | |
| `harga`, `diskon_persen`, `diskon`, `total`, `harga_beli`, `harga_jual_lama`, `laba` | DECIMAL | |

### Pola yang sama diterapkan ke:
- `tpenjualanpending` / `tpenjualanpendingdetail` — transaksi ditahan, struktur identik `tpenjualan`/`detail`
- `tpembelian` / `tpembeliandetail` — sama seperti penjualan tapi arah ke supplier (`kode_suplier`/`suplier` menggantikan `kode_pelanggan`/`pelanggan`, tanpa `sales`)
- `tpembelianpending` / `tpembelianpendingdetail` — versi ditahan dari pembelian
- `treturpenjualan` (`no_retur` PK asli, `faktur` FK → `tpenjualan.faktur`) / `treturpenjualandetail`
- `treturpembelian` (`no_retur` PK asli, `faktur` FK → `tpembelian.faktur`) / `treturpembeliandetail`

---

## 6. Tabel Keuangan

| Tabel | Relasi Kunci | Keterangan |
|---|---|---|
| `thutang` | `faktur` FK → `tpembelian.faktur`, `suplier` FK → `dsuplier.kode` | Hutang ke supplier |
| `tpiutang` | `faktur` FK → `tpenjualan.faktur`, `pelanggan` FK → `dpelanggan.kode` | Piutang dari pelanggan |
| `tcashflow` | `faktur` opsional, `kategori` → lookup kategori | Arus kas masuk/keluar |
| `tbiaya` | `kategori` → lookup kategori | Biaya operasional |

Kolom lain (`tanggal`, `waktu`, `tempo`, `jatuh_tempo`, `tagihan_awal`, `telah_dibayar`, `sisa`, `status`, `keterangan`, `operator`) dipertahankan sama seperti sistem lama, tipe disesuaikan (DATE/TIME/DECIMAL/TEXT).

---

## 7. Tabel Stok & Log Histori

| Tabel | Relasi Kunci | Sifat |
|---|---|---|
| `tstokopname` | `kode_barang` FK → `dbarang.kode` | Input manual saat opname fisik |
| `keluarmasuk` | `kode` FK → `dbarang.kode`, `faktur` → transaksi terkait | **Auto-generated** oleh sistem setiap ada transaksi (penjualan/pembelian/retur/opname) — bukan input manual |
| `perubahanharga` | `Kode` FK → `dbarang.kode` | **Auto-generated** setiap harga produk diubah — log histori harga beli/jual lama vs baru |

---

## 8. Tabel Konfigurasi & Lookup

Dipertahankan strukturnya sama seperti sistem lama, tapi **scoped per `cabang_id`**:

- `daplikasi` — 1 baris per cabang (info toko, header/footer struk, pengaturan notifikasi WA, dll)
- `aturkolom`, `checkboxediting`, `autokode`, `barcode` — pengaturan UI/perilaku aplikasi
- `combo*` (comboalasan, comboexpired, combojeniskas, combokategori, combokategoripemasukan, combokategoripengeluaran, combosatuan, combostatus, combostok, combotunaikredit, comboukurancetak) — isi dropdown, tetap sebagai tabel terpisah sesuai sistem lama
- `dpetunjuk`, `filtertanggal` — sama seperti lama

> Catatan: tabel-tabel ini murni pengaturan/lookup (bukan data transaksi inti), jadi dampaknya kecil terhadap logic sync — cukup disertakan di sync cabang seperti data master biasa.

---

## 9. Tabel Baru — Ekstensi Fitur Poin Pelanggan

### `pengaturan_poin` (BARU)
| Kolom | Tipe | Keterangan |
|---|---|---|
| `id` | TEXT PK | |
| `cabang_id` | TEXT FK → `cabang.id` | 1 baris per cabang (bisa beda rate tiap cabang, atau disamakan semua) |
| `rupiah_per_poin` | DECIMAL | Belanja berapa rupiah = dapat 1 poin (mis. `10000` → tiap kelipatan Rp10.000 belanja dapat 1 poin) |
| `nilai_tukar_poin` | DECIMAL | 1 poin setara berapa rupiah saat DITUKAR jadi potongan (mis. `100` → 1 poin = Rp100 potongan) |
| `minimal_poin_tukar` | INTEGER, nullable | Batas minimal poin yang boleh ditukar sekali transaksi (opsional) |

### `riwayat_poin` (BARU)
| Kolom | Tipe | Keterangan |
|---|---|---|
| `id` | TEXT PK | |
| `kode_pelanggan` | TEXT FK → `dpelanggan.kode` | |
| `faktur` | TEXT FK → `tpenjualan.faktur`, nullable | NULL kalau bukan dari transaksi (mis. adjustment manual) |
| `jenis` | TEXT (`dapat`\|`tukar`\|`adjustment`) | `dapat` = didapat dari belanja, `tukar` = dipakai sbg potongan |
| `jumlah_poin` | INTEGER | Positif untuk `dapat`, negatif (atau disimpan positif dgn `jenis`='tukar' sbg penanda arah) untuk `tukar` |
| `tanggal` | DATE | |
| *+ kolom ekstensi §2* | | |

---

## 10. Tabel Baru — Pergantian Shift Kasir

### `tshift` (BARU)
| Kolom | Tipe | Keterangan |
|---|---|---|
| `id` | TEXT PK | |
| `kode` | TEXT | Nomor shift, unik per `cabang_id`, format `{kode_cabang}-SHF-{urutan}` |
| `operator` | TEXT FK → `doperator.kode` | Kasir yang menjalankan shift |
| `device_id` | TEXT FK → `device.id` | Terminal tempat shift berjalan |
| `waktu_mulai` | DATETIME | |
| `waktu_selesai` | DATETIME, nullable | NULL selama shift masih berjalan |
| `modal_awal` | DECIMAL | Kas awal, diinput manual kasir saat buka shift |
| `total_penjualan_tunai` | DECIMAL | Dihitung otomatis dari `tpenjualan` (metode tunai) ber-`shift_id` ini |
| `total_penjualan_non_tunai` | DECIMAL | QRIS/kartu/transfer — tidak mempengaruhi kas fisik |
| `total_kas_masuk_lain` | DECIMAL | Dari `tcashflow` jenis pemasukan selama shift |
| `total_kas_keluar` | DECIMAL | Dari `tbiaya`/`tcashflow` jenis pengeluaran selama shift |
| `total_retur_tunai` | DECIMAL | Refund tunai selama shift (mengurangi kas) |
| `uang_seharusnya` | DECIMAL | Dihitung sistem: `modal_awal + total_penjualan_tunai + total_kas_masuk_lain − total_kas_keluar − total_retur_tunai` |
| `uang_aktual` | DECIMAL | Hasil hitung fisik oleh kasir saat tutup shift (input manual) |
| `selisih` | DECIMAL | `uang_aktual − uang_seharusnya` (bisa minus/plus) |
| `status` | TEXT (`berjalan`\|`selesai`) | |
| `keterangan` | TEXT | Catatan kasir kalau ada selisih, opsional |
| *+ kolom ekstensi §2* | | |

**Kolom tambahan `shift_id` (FK → `tshift.id`) ditambahkan ke tabel:** `tpenjualan`, `treturpenjualan`, `tbiaya`, `tcashflow` — supaya semua transaksi yang terjadi bisa dikelompokkan otomatis per shift saat penghitungan tutup shift.

> **Catatan desain:** shift bersifat **per kasir per device**, bukan per toko — jadi kalau ada 3 kasir aktif bersamaan (multi-PC via LAN di bagian 3), masing-masing punya `tshift` sendiri dengan laci kas masing-masing.

---

## 11. Ringkasan Relasi Utama (ERD Tekstual)

```
cabang ─┬─< device
        ├─< dbarang, dpelanggan (+id_kartu, poin_saldo), dsuplier, dsales, dkaryawan, doperator
        ├─< pengaturan_poin (1:1 per cabang — rate konversi poin)
        ├─< tshift (per operator+device) ─┬─ tpenjualan (shift_id)
        │                                  ├─ treturpenjualan (shift_id)
        │                                  ├─ tbiaya (shift_id)
        │                                  └─ tcashflow (shift_id)
        ├─< tpenjualan (kode_pelanggan, poin_ditukar) >─< tpenjualandetail >─ dbarang
        │        │
        │        └─< tpiutang, riwayat_poin (via faktur/kode_pelanggan)
        ├─< tpembelian >─< tpembeliandetail >─ dbarang
        │        │
        │        └─< thutang (via faktur)
        ├─< treturpenjualan >─< treturpenjualandetail
        ├─< treturpembelian >─< treturpembeliandetail
        ├─< tstokopname ─ dbarang
        ├─< keluarmasuk ─ dbarang        (auto-generated log)
        ├─< perubahanharga ─ dbarang     (auto-generated log)
        ├─< tcashflow, tbiaya
        └─< daplikasi (1:1), aturkolom, checkboxediting, autokode,
            barcode, combo*, dpetunjuk, filtertanggal
```

**Aturan kunci untuk agent saat implementasi:**
- Semua `FK → kode` di atas adalah **relasi longgar per-cabang** (bukan FK database strict global), karena `kode` hanya unik dalam satu `cabang_id`, bukan unik global
- Uniqueness constraint yang benar: `UNIQUE(cabang_id, kode)` untuk tabel master, `UNIQUE(cabang_id, faktur)` untuk tabel transaksi
- Saat sync ke cloud, `cabang_id` jadi partition key alami — tidak perlu resolusi konflik rumit untuk data yang scope-nya sudah per-cabang

---

## 12. Alur Transaksi Kasir (State Machine)

### 12.1 Buka Shift (wajib sebelum transaksi)

```
1. Kasir login (pilih/scan operator di device ini)
2. App cek: ada tshift status 'berjalan' untuk operator+device ini?
   ├─ ADA  → langsung masuk ke halaman kasir, pakai shift itu
   └─ TIDAK ADA → wajib buka shift dulu:
        a. Kasir input modal_awal (uang kas fisik di laci saat mulai)
        b. App INSERT tshift baru: status='berjalan', waktu_mulai=now,
           modal_awal=input, cabang_id, device_id, operator
        c. Masuk ke halaman kasir
```

### 12.2 Transaksi Penjualan

```
1. Tambah item ke keranjang — scan barcode (HID input) atau cari manual
2. Untuk tiap item: tentukan harga sesuai tier (qty_1..5 di dbarang),
   bisa override diskon per item (jika hak akses operator izinkan)
3. (opsional) Attach Member:
   a. Kasir ketik nama/kode/no_hp DI KOTAK YANG SAMA, atau scan kartu
      member fisik (scanner otomatis isi nilai persis id_kartu)
   b. App cari: exact match ke id_kartu → langsung ketemu (hasil scan),
      atau partial match ke nama/kode/no_hp → tampilkan pilihan
   c. Setelah member ter-attach: tampilkan poin_saldo & diskon_persen
      member ke kasir
4. (opsional) Tukar Poin — hanya bisa jika member sudah ter-attach:
   a. Kasir pilih "Tukar Poin", input jumlah poin yang mau dipakai
      (tidak boleh > poin_saldo member, dan >= minimal_poin_tukar
      jika diset di pengaturan_poin)
   b. App hitung nilai_tukar_poin = jumlah_poin × nilai_tukar_poin
      (dari pengaturan_poin cabang ini)
   c. Nilai ini mengurangi total yang harus dibayar (terpisah dari
      diskon_persen/diskon biasa)
5. (opsional) Simpan sebagai Pending → INSERT tpenjualanpending
   (ditahan, belum mempengaruhi stok/kas/poin)
6. Proses Bayar:
   a. Pilih metode: tunai / kredit / non-tunai (qris, kartu, dll)
   b. Input jumlah bayar → hitung kembalian (khusus tunai)
   c. Commit transaksi:
      - INSERT tpenjualan (+ shift_id, kode_pelanggan jika ada member,
        poin_ditukar & nilai_tukar_poin jika ada tukar poin)
      - INSERT tpenjualandetail per item
      - UPDATE dbarang.stok (kurangi sesuai jumlah terjual)
      - INSERT keluarmasuk (log otomatis, kolom `keluar`)
      - Jika metode kredit → INSERT tpiutang
      - Jika ada member (poin aktif di pengaturan_poin):
          · Jika ada tukar poin → UPDATE dpelanggan.poin_saldo
            (kurangi), INSERT riwayat_poin (jenis='tukar')
          · Hitung poin didapat dari total belanja ÷ rupiah_per_poin
            → UPDATE dpelanggan.poin_saldo (tambah), INSERT
            riwayat_poin (jenis='dapat')
   d. Cetak struk (ESC/POS) — tampilkan juga saldo poin terbaru & poin
      yang baru didapat/ditukar di struk
```

### 12.3 Retur Penjualan

```
1. Cari faktur asli (tpenjualan) → pilih item yang diretur
2. INSERT treturpenjualan (+ shift_id) & treturpenjualandetail
3. UPDATE dbarang.stok (tambahkan balik sesuai jumlah retur)
4. INSERT keluarmasuk (log otomatis, kolom `masuk`)
5. Jika retur mempengaruhi tpiutang → UPDATE tpiutang.sisa
6. Jika refund tunai → tercatat sebagai pengurang kas
   (masuk hitungan total_retur_tunai di tshift)
```

### 12.4 Tutup Shift

```
1. Kasir pilih "Tutup Shift"
2. App HITUNG OTOMATIS dari semua transaksi ber-shift_id = shift ini:
   - total_penjualan_tunai (dari tpenjualan, metode tunai)
   - total_penjualan_non_tunai
   - total_kas_masuk_lain (dari tcashflow, jenis pemasukan)
   - total_kas_keluar (dari tbiaya + tcashflow jenis pengeluaran)
   - total_retur_tunai (dari treturpenjualan, refund tunai)
   - uang_seharusnya = modal_awal + total_penjualan_tunai
                        + total_kas_masuk_lain
                        − total_kas_keluar − total_retur_tunai
3. Tampilkan ringkasan ke kasir
4. Kasir hitung uang fisik di laci → input uang_aktual
5. App hitung selisih = uang_aktual − uang_seharusnya
6. UPDATE tshift: waktu_selesai=now, status='selesai',
   simpan semua total & selisih di atas
7. CETAK STRUK PERGANTIAN SHIFT, isi:
   - Info: nama kasir, device/terminal, cabang, waktu mulai–selesai
   - Modal awal
   - Rincian: total penjualan tunai, non-tunai, kas masuk lain,
     kas keluar, retur tunai
   - Uang seharusnya vs uang aktual
   - Selisih (ditandai jelas kalau minus/kurang)
8. Shift berikutnya (kasir sama atau ganti kasir) wajib buka shift baru
   dengan modal_awal baru (biasanya = uang_aktual shift sebelumnya,
   tapi tetap diinput manual untuk verifikasi fisik ulang)
```