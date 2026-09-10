use rusqlite::{Connection, Result};

/// Inisialisasi seluruh skema database SQLite sesuai `skema-database.MD`
pub fn jalankan_migrasi(conn: &Connection) -> Result<()> {
    // 1. Tabel Cabang
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS cabang (
            id TEXT PRIMARY KEY,
            kode TEXT UNIQUE NOT NULL,
            nama TEXT NOT NULL,
            alamat TEXT,
            telepon TEXT,
            is_pusat INTEGER DEFAULT 0,
            sync_status TEXT DEFAULT 'pending',
            sync_at TIMESTAMP,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- 2. Tabel Device
        CREATE TABLE IF NOT EXISTS device (
            id TEXT PRIMARY KEY,
            cabang_id TEXT NOT NULL,
            kode TEXT NOT NULL,
            nama TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'client',
            machine_id TEXT NOT NULL,
            ip_address TEXT,
            last_seen TIMESTAMP,
            is_active INTEGER DEFAULT 1,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(cabang_id, kode)
        );

        -- 3. Tabel Master Barang (dbarang)
        CREATE TABLE IF NOT EXISTS dbarang (
            id TEXT PRIMARY KEY,
            cabang_id TEXT NOT NULL,
            kode TEXT NOT NULL,
            barcode TEXT,
            nama TEXT NOT NULL,
            satuan TEXT DEFAULT 'PCS',
            kategori TEXT,
            rak TEXT,
            hargapokok REAL DEFAULT 0,
            hargajual1 REAL DEFAULT 0,
            hargajual2 REAL DEFAULT 0,
            hargajual3 REAL DEFAULT 0,
            hargajual4 REAL DEFAULT 0,
            hargapartai REAL DEFAULT 0,
            stok REAL DEFAULT 0,
            stokminimum REAL DEFAULT 0,
            is_aktif INTEGER DEFAULT 1,
            sync_status TEXT DEFAULT 'pending',
            sync_at TIMESTAMP,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(cabang_id, kode)
        );
        CREATE INDEX IF NOT EXISTS idx_dbarang_barcode ON dbarang(cabang_id, barcode);
        CREATE INDEX IF NOT EXISTS idx_dbarang_nama ON dbarang(cabang_id, nama);

        -- 4. Tabel Pelanggan (dpelanggan)
        CREATE TABLE IF NOT EXISTS dpelanggan (
            id TEXT PRIMARY KEY,
            cabang_id TEXT NOT NULL,
            kode TEXT NOT NULL,
            nama TEXT NOT NULL,
            alamat TEXT,
            telepon TEXT,
            plafonpiutang REAL DEFAULT 0,
            poin_saldo INTEGER DEFAULT 0,
            id_kartu TEXT,
            is_aktif INTEGER DEFAULT 1,
            sync_status TEXT DEFAULT 'pending',
            sync_at TIMESTAMP,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(cabang_id, kode)
        );
        CREATE INDEX IF NOT EXISTS idx_dpelanggan_kartu ON dpelanggan(cabang_id, id_kartu);

        -- 5. Tabel Operator / Kasir (doperator)
        CREATE TABLE IF NOT EXISTS doperator (
            id TEXT PRIMARY KEY,
            cabang_id TEXT NOT NULL,
            kode TEXT NOT NULL,
            nama TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'kasir',
            is_aktif INTEGER DEFAULT 1,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(cabang_id, kode)
        );

        -- 6. Tabel Shift Kasir (tshift)
        CREATE TABLE IF NOT EXISTS tshift (
            id TEXT PRIMARY KEY,
            cabang_id TEXT NOT NULL,
            device_id TEXT NOT NULL,
            operator_id TEXT NOT NULL,
            waktu_buka TIMESTAMP NOT NULL,
            waktu_tutup TIMESTAMP,
            modal_awal REAL NOT NULL DEFAULT 0,
            total_penjualan_tunai REAL DEFAULT 0,
            total_penjualan_nontunai REAL DEFAULT 0,
            total_retur REAL DEFAULT 0,
            total_biaya REAL DEFAULT 0,
            uang_seharusnya REAL DEFAULT 0,
            uang_aktual REAL,
            selisih REAL,
            status TEXT DEFAULT 'open',
            catatan TEXT,
            sync_status TEXT DEFAULT 'pending',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_tshift_status ON tshift(cabang_id, device_id, status);

        -- 7. Tabel Transaksi Penjualan Header (tpenjualan)
        CREATE TABLE IF NOT EXISTS tpenjualan (
            id TEXT PRIMARY KEY,
            cabang_id TEXT NOT NULL,
            device_id TEXT NOT NULL,
            shift_id TEXT NOT NULL,
            faktur TEXT NOT NULL,
            tanggal TIMESTAMP NOT NULL,
            kode_pelanggan TEXT DEFAULT 'UMUM',
            operator_id TEXT NOT NULL,
            subtotal REAL NOT NULL DEFAULT 0,
            diskon_rp REAL DEFAULT 0,
            total_akhir REAL NOT NULL DEFAULT 0,
            bayar_tunai REAL DEFAULT 0,
            bayar_nontunai REAL DEFAULT 0,
            kembalian REAL DEFAULT 0,
            metode_bayar TEXT DEFAULT 'TUNAI',
            status TEXT DEFAULT 'selesai',
            poin_didapat INTEGER DEFAULT 0,
            poin_ditukar INTEGER DEFAULT 0,
            nilai_tukar_poin REAL DEFAULT 0,
            sync_status TEXT DEFAULT 'pending',
            sync_at TIMESTAMP,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(cabang_id, faktur)
        );
        CREATE INDEX IF NOT EXISTS idx_tpenjualan_shift ON tpenjualan(shift_id);
        CREATE INDEX IF NOT EXISTS idx_tpenjualan_tanggal ON tpenjualan(cabang_id, tanggal);

        -- 8. Tabel Detail Transaksi Penjualan (tpenjualandetail)
        CREATE TABLE IF NOT EXISTS tpenjualandetail (
            id TEXT PRIMARY KEY,
            penjualan_id TEXT NOT NULL,
            cabang_id TEXT NOT NULL,
            barang_id TEXT NOT NULL,
            kode_barang TEXT NOT NULL,
            nama_barang TEXT NOT NULL,
            jumlah REAL NOT NULL DEFAULT 1,
            satuan TEXT DEFAULT 'PCS',
            hargajual REAL NOT NULL DEFAULT 0,
            hargapokok REAL NOT NULL DEFAULT 0,
            diskon_persen REAL DEFAULT 0,
            diskon_rp REAL DEFAULT 0,
            subtotal REAL NOT NULL DEFAULT 0,
            sync_status TEXT DEFAULT 'pending',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_tpenjualandetail_faktur ON tpenjualandetail(penjualan_id);

        -- 9. Log Otomatis Keluar Masuk Stok (keluarmasuk)
        CREATE TABLE IF NOT EXISTS keluarmasuk (
            id TEXT PRIMARY KEY,
            cabang_id TEXT NOT NULL,
            device_id TEXT NOT NULL,
            barang_id TEXT NOT NULL,
            kode_barang TEXT NOT NULL,
            tanggal TIMESTAMP NOT NULL,
            jenis TEXT NOT NULL,
            referensi TEXT NOT NULL,
            masuk REAL DEFAULT 0,
            keluar REAL DEFAULT 0,
            sisa REAL NOT NULL,
            keterangan TEXT,
            sync_status TEXT DEFAULT 'pending',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_keluarmasuk_barang ON keluarmasuk(cabang_id, barang_id, tanggal);

        -- 10. Log Otomatis Perubahan Harga (perubahanharga)
        CREATE TABLE IF NOT EXISTS perubahanharga (
            id TEXT PRIMARY KEY,
            cabang_id TEXT NOT NULL,
            barang_id TEXT NOT NULL,
            tanggal TIMESTAMP NOT NULL,
            operator_id TEXT,
            hargapokok_lama REAL,
            hargapokok_baru REAL,
            hargajual1_lama REAL,
            hargajual1_baru REAL,
            keterangan TEXT,
            sync_status TEXT DEFAULT 'pending',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- 11. Tabel Retur Penjualan Header (treturpenjualan)
        CREATE TABLE IF NOT EXISTS treturpenjualan (
            id TEXT PRIMARY KEY,
            cabang_id TEXT NOT NULL,
            device_id TEXT NOT NULL,
            shift_id TEXT NOT NULL,
            faktur TEXT NOT NULL,
            tanggal TIMESTAMP NOT NULL,
            faktur_penjualan TEXT NOT NULL,
            operator_id TEXT NOT NULL,
            subtotal REAL NOT NULL DEFAULT 0,
            diskon_rp REAL DEFAULT 0,
            total_akhir REAL NOT NULL DEFAULT 0,
            sync_status TEXT DEFAULT 'pending',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(cabang_id, faktur)
        );
        CREATE INDEX IF NOT EXISTS idx_treturpenjualan_faktur ON treturpenjualan(cabang_id, faktur_penjualan);

        -- 12. Tabel Detail Retur Penjualan (treturpenjualandetail)
        CREATE TABLE IF NOT EXISTS treturpenjualandetail (
            id TEXT PRIMARY KEY,
            retur_id TEXT NOT NULL,
            cabang_id TEXT NOT NULL,
            barang_id TEXT NOT NULL,
            kode_barang TEXT NOT NULL,
            nama_barang TEXT NOT NULL,
            jumlah REAL NOT NULL DEFAULT 1,
            satuan TEXT DEFAULT 'PCS',
            hargajual REAL NOT NULL DEFAULT 0,
            subtotal REAL NOT NULL DEFAULT 0,
            sync_status TEXT DEFAULT 'pending',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- 13. Tabel Pengaturan Poin Member (pengaturan_poin)
        CREATE TABLE IF NOT EXISTS pengaturan_poin (
            id TEXT PRIMARY KEY,
            cabang_id TEXT NOT NULL,
            rupiah_per_poin REAL DEFAULT 10000,
            nilai_tukar_poin REAL DEFAULT 100,
            minimal_tukar INTEGER DEFAULT 10,
            is_aktif INTEGER DEFAULT 1,
            sync_status TEXT DEFAULT 'pending',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(cabang_id)
        );

        -- 14. Tabel Riwayat Poin Member (riwayat_poin)
        CREATE TABLE IF NOT EXISTS riwayat_poin (
            id TEXT PRIMARY KEY,
            cabang_id TEXT NOT NULL,
            pelanggan_id TEXT NOT NULL,
            faktur TEXT,
            tanggal TIMESTAMP NOT NULL,
            jenis TEXT NOT NULL,
            jumlah INTEGER NOT NULL,
            saldo_akhir INTEGER NOT NULL,
            keterangan TEXT,
            sync_status TEXT DEFAULT 'pending',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_riwayat_poin_pelanggan ON riwayat_poin(cabang_id, pelanggan_id, tanggal);

        -- 15. Tabel Master Supplier (dsuplier)
        CREATE TABLE IF NOT EXISTS dsuplier (
            id TEXT PRIMARY KEY,
            cabang_id TEXT NOT NULL,
            kode TEXT NOT NULL,
            nama TEXT NOT NULL,
            alamat TEXT,
            telepon TEXT,
            rekening TEXT,
            keterangan TEXT,
            is_aktif INTEGER DEFAULT 1,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(cabang_id, kode)
        );

        -- 16. Tabel Pembelian Header (tpembelian)
        CREATE TABLE IF NOT EXISTS tpembelian (
            id TEXT PRIMARY KEY,
            cabang_id TEXT NOT NULL,
            faktur TEXT NOT NULL,
            faktur_supplier TEXT,
            tanggal TIMESTAMP NOT NULL,
            suplier_id TEXT NOT NULL,
            nama_suplier TEXT NOT NULL,
            operator_id TEXT NOT NULL,
            total REAL NOT NULL DEFAULT 0,
            diskon_rp REAL DEFAULT 0,
            total_akhir REAL NOT NULL DEFAULT 0,
            bayar REAL DEFAULT 0,
            sisa REAL DEFAULT 0,
            tunai_kredit TEXT DEFAULT 'TUNAI',
            jatuh_tempo TIMESTAMP,
            status TEXT DEFAULT 'selesai',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(cabang_id, faktur)
        );

        -- 17. Tabel Detail Pembelian (tpembeliandetail)
        CREATE TABLE IF NOT EXISTS tpembeliandetail (
            id TEXT PRIMARY KEY,
            pembelian_id TEXT NOT NULL,
            cabang_id TEXT NOT NULL,
            barang_id TEXT NOT NULL,
            kode_barang TEXT NOT NULL,
            nama_barang TEXT NOT NULL,
            jumlah REAL NOT NULL DEFAULT 1,
            satuan TEXT DEFAULT 'PCS',
            harga_beli REAL NOT NULL DEFAULT 0,
            subtotal REAL NOT NULL DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- 18. Tabel Hutang Dagang (thutang)
        CREATE TABLE IF NOT EXISTS thutang (
            id TEXT PRIMARY KEY,
            cabang_id TEXT NOT NULL,
            faktur_beli TEXT NOT NULL,
            suplier_id TEXT NOT NULL,
            nama_suplier TEXT NOT NULL,
            tanggal TIMESTAMP NOT NULL,
            jatuh_tempo TIMESTAMP,
            tagihan_awal REAL NOT NULL,
            telah_dibayar REAL DEFAULT 0,
            sisa REAL NOT NULL,
            status TEXT DEFAULT 'belum_lunas',
            keterangan TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- 19. Tabel Piutang Dagang (tpiutang)
        CREATE TABLE IF NOT EXISTS tpiutang (
            id TEXT PRIMARY KEY,
            cabang_id TEXT NOT NULL,
            faktur_jual TEXT NOT NULL,
            pelanggan_id TEXT NOT NULL,
            nama_pelanggan TEXT NOT NULL,
            tanggal TIMESTAMP NOT NULL,
            jatuh_tempo TIMESTAMP,
            tagihan_awal REAL NOT NULL,
            telah_dibayar REAL DEFAULT 0,
            sisa REAL NOT NULL,
            status TEXT DEFAULT 'belum_lunas',
            keterangan TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- 20. Tabel Stok Opname (tstokopname)
        CREATE TABLE IF NOT EXISTS tstokopname (
            id TEXT PRIMARY KEY,
            cabang_id TEXT NOT NULL,
            barang_id TEXT NOT NULL,
            kode_barang TEXT NOT NULL,
            nama_barang TEXT NOT NULL,
            tanggal TIMESTAMP NOT NULL,
            stok_komputer REAL NOT NULL,
            stok_nyata REAL NOT NULL,
            selisih REAL NOT NULL,
            total_selisih REAL NOT NULL,
            alasan TEXT DEFAULT 'penyesuaian',
            operator_id TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- 21. Tabel Buku Kas / Cashflow (tcashflow)
        CREATE TABLE IF NOT EXISTS tcashflow (
            id TEXT PRIMARY KEY,
            cabang_id TEXT NOT NULL,
            shift_id TEXT,
            tanggal TIMESTAMP NOT NULL,
            jenis TEXT NOT NULL,
            kategori TEXT NOT NULL,
            nominal REAL NOT NULL,
            keterangan TEXT,
            operator_id TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- 22. Tabel Biaya Operasional (tbiaya)
        CREATE TABLE IF NOT EXISTS tbiaya (
            id TEXT PRIMARY KEY,
            cabang_id TEXT NOT NULL,
            shift_id TEXT,
            tanggal TIMESTAMP NOT NULL,
            kategori TEXT NOT NULL,
            nominal REAL NOT NULL,
            keterangan TEXT,
            operator_id TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- 23. Tabel Pending Kasir (tpenjualanpending - Hold/Recall Antrean)
        CREATE TABLE IF NOT EXISTS tpenjualanpending (
            id TEXT PRIMARY KEY,
            cabang_id TEXT NOT NULL,
            device_id TEXT NOT NULL,
            shift_id TEXT NOT NULL,
            faktur TEXT NOT NULL,
            tanggal TIMESTAMP NOT NULL,
            kode_pelanggan TEXT,
            operator_id TEXT NOT NULL,
            subtotal REAL NOT NULL DEFAULT 0.0,
            diskon_rp REAL NOT NULL DEFAULT 0.0,
            total_akhir REAL NOT NULL DEFAULT 0.0,
            keterangan TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- 24. Tabel Detail Pending Kasir (tpenjualanpendingdetail)
        CREATE TABLE IF NOT EXISTS tpenjualanpendingdetail (
            id TEXT PRIMARY KEY,
            pending_id TEXT NOT NULL,
            cabang_id TEXT NOT NULL,
            barang_id TEXT NOT NULL,
            kode_barang TEXT NOT NULL,
            nama_barang TEXT NOT NULL,
            jumlah REAL NOT NULL,
            satuan TEXT NOT NULL,
            hargajual REAL NOT NULL,
            hargapokok REAL NOT NULL DEFAULT 0.0,
            diskon_persen REAL NOT NULL DEFAULT 0.0,
            diskon_rp REAL NOT NULL DEFAULT 0.0,
            subtotal REAL NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(pending_id) REFERENCES tpenjualanpending(id) ON DELETE CASCADE
        );
        "#,
    )?;

    // Migrasi defensif untuk database yang sudah ada sebelumnya
    let _ = conn.execute("ALTER TABLE tcashflow ADD COLUMN shift_id TEXT;", []);
    let _ = conn.execute("ALTER TABLE tbiaya ADD COLUMN shift_id TEXT;", []);

    Ok(())
}
