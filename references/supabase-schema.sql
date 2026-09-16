-- =============================================================
-- FAZPOS Supabase Cloud Schema
-- Jalankan SQL ini 1x di Supabase SQL Editor (Dashboard Supabase)
-- =============================================================

-- 1. Tabel Cabang
CREATE TABLE IF NOT EXISTS cabang (
    id TEXT PRIMARY KEY,
    kode TEXT UNIQUE NOT NULL,
    nama TEXT NOT NULL,
    alamat TEXT,
    telepon TEXT,
    is_pusat BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 2. Tabel Device / Terminal Kasir
CREATE TABLE IF NOT EXISTS device (
    id TEXT PRIMARY KEY,
    cabang_id TEXT NOT NULL REFERENCES cabang(id),
    kode TEXT NOT NULL,
    nama TEXT NOT NULL,
    role TEXT DEFAULT 'client',
    machine_id TEXT NOT NULL,
    ip_address TEXT,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(cabang_id, kode)
);

-- 3. Master Barang & Stok
CREATE TABLE IF NOT EXISTS dbarang (
    id TEXT PRIMARY KEY,
    cabang_id TEXT NOT NULL,
    kode TEXT NOT NULL,
    barcode TEXT,
    nama TEXT NOT NULL,
    satuan TEXT DEFAULT 'PCS',
    kategori TEXT,
    rak TEXT,
    hargapokok DOUBLE PRECISION DEFAULT 0,
    hargajual1 DOUBLE PRECISION DEFAULT 0,
    hargajual2 DOUBLE PRECISION DEFAULT 0,
    hargajual3 DOUBLE PRECISION DEFAULT 0,
    hargajual4 DOUBLE PRECISION DEFAULT 0,
    hargapartai DOUBLE PRECISION DEFAULT 0,
    stok DOUBLE PRECISION DEFAULT 0,
    stokminimum DOUBLE PRECISION DEFAULT 0,
    is_aktif BOOLEAN DEFAULT TRUE,
    sync_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(cabang_id, kode)
);
CREATE INDEX IF NOT EXISTS idx_dbarang_cabang ON dbarang(cabang_id);
CREATE INDEX IF NOT EXISTS idx_dbarang_barcode ON dbarang(cabang_id, barcode);

-- 4. Master Pelanggan & Loyalitas
CREATE TABLE IF NOT EXISTS dpelanggan (
    id TEXT PRIMARY KEY,
    cabang_id TEXT NOT NULL,
    kode TEXT NOT NULL,
    nama TEXT NOT NULL,
    alamat TEXT,
    telepon TEXT,
    plafonpiutang DOUBLE PRECISION DEFAULT 0,
    poin_saldo INTEGER DEFAULT 0,
    id_kartu TEXT,
    is_aktif BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(cabang_id, kode)
);
CREATE INDEX IF NOT EXISTS idx_dpelanggan_cabang ON dpelanggan(cabang_id);

-- 5. Header Transaksi Penjualan
CREATE TABLE IF NOT EXISTS tpenjualan (
    id TEXT PRIMARY KEY,
    cabang_id TEXT NOT NULL,
    device_id TEXT NOT NULL,
    shift_id TEXT NOT NULL,
    faktur TEXT NOT NULL,
    tanggal TIMESTAMPTZ NOT NULL,
    kode_pelanggan TEXT DEFAULT 'UMUM',
    operator_id TEXT NOT NULL,
    subtotal DOUBLE PRECISION DEFAULT 0,
    diskon_rp DOUBLE PRECISION DEFAULT 0,
    total_akhir DOUBLE PRECISION DEFAULT 0,
    bayar_tunai DOUBLE PRECISION DEFAULT 0,
    bayar_nontunai DOUBLE PRECISION DEFAULT 0,
    kembalian DOUBLE PRECISION DEFAULT 0,
    metode_bayar TEXT DEFAULT 'TUNAI',
    status TEXT DEFAULT 'selesai',
    poin_didapat INTEGER DEFAULT 0,
    poin_ditukar INTEGER DEFAULT 0,
    nilai_tukar_poin DOUBLE PRECISION DEFAULT 0,
    sync_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(cabang_id, faktur)
);
CREATE INDEX IF NOT EXISTS idx_tpenjualan_cabang_tgl ON tpenjualan(cabang_id, tanggal);

-- 6. Detail Transaksi Penjualan
CREATE TABLE IF NOT EXISTS tpenjualandetail (
    id TEXT PRIMARY KEY,
    penjualan_id TEXT NOT NULL REFERENCES tpenjualan(id) ON DELETE CASCADE,
    cabang_id TEXT NOT NULL,
    barang_id TEXT NOT NULL,
    kode_barang TEXT NOT NULL,
    nama_barang TEXT NOT NULL,
    jumlah DOUBLE PRECISION DEFAULT 1,
    satuan TEXT DEFAULT 'PCS',
    hargajual DOUBLE PRECISION DEFAULT 0,
    hargapokok DOUBLE PRECISION DEFAULT 0,
    diskon_persen DOUBLE PRECISION DEFAULT 0,
    diskon_rp DOUBLE PRECISION DEFAULT 0,
    subtotal DOUBLE PRECISION DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_tpenjualandetail_penjualan ON tpenjualandetail(penjualan_id);

-- 7. Sesi Shift Kasir
CREATE TABLE IF NOT EXISTS tshift (
    id TEXT PRIMARY KEY,
    cabang_id TEXT NOT NULL,
    device_id TEXT NOT NULL,
    operator_id TEXT NOT NULL,
    waktu_buka TIMESTAMPTZ NOT NULL,
    waktu_tutup TIMESTAMPTZ,
    modal_awal DOUBLE PRECISION DEFAULT 0,
    total_penjualan_tunai DOUBLE PRECISION DEFAULT 0,
    total_penjualan_nontunai DOUBLE PRECISION DEFAULT 0,
    total_retur DOUBLE PRECISION DEFAULT 0,
    total_biaya DOUBLE PRECISION DEFAULT 0,
    total_kas_masuk_lain DOUBLE PRECISION DEFAULT 0,
    total_kas_keluar DOUBLE PRECISION DEFAULT 0,
    total_retur_tunai DOUBLE PRECISION DEFAULT 0,
    uang_seharusnya DOUBLE PRECISION DEFAULT 0,
    uang_aktual DOUBLE PRECISION,
    selisih DOUBLE PRECISION,
    status TEXT DEFAULT 'open',
    catatan TEXT,
    kode TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_tshift_cabang ON tshift(cabang_id, waktu_buka);

-- 8. Kartu Mutasi Stok (Keluar/Masuk)
CREATE TABLE IF NOT EXISTS keluarmasuk (
    id TEXT PRIMARY KEY,
    cabang_id TEXT NOT NULL,
    device_id TEXT NOT NULL,
    barang_id TEXT NOT NULL,
    kode_barang TEXT NOT NULL,
    tanggal TIMESTAMPTZ NOT NULL,
    jenis TEXT NOT NULL,
    referensi TEXT NOT NULL,
    masuk DOUBLE PRECISION DEFAULT 0,
    keluar DOUBLE PRECISION DEFAULT 0,
    sisa DOUBLE PRECISION NOT NULL,
    keterangan TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_keluarmasuk_cabang_tgl ON keluarmasuk(cabang_id, tanggal);
