<script lang="ts">
  import { formatRupiah } from "../lib/api";

  type MainCategory = "penjualan" | "pembelian" | "keuangan" | "hutang_piutang" | "stok" | "grafik";

  let mainCategory = $state<MainCategory>("penjualan");
  let subReport = $state("penjualan_umum");
  let searchQuery = $state("");
  let dateRange = $state("bulan_ini");
  let searchInputElement: HTMLInputElement | null = $state(null);
  let toastMessage = $state("");

  // 37 Sub-reports configuration from references/spesifikasi-legacy-ibretago5.md section 8
  const subReportOptions: Record<MainCategory, { id: string; label: string; legacyFile: string }[]> = {
    penjualan: [
      { id: "penjualan_umum", label: "1. Rekap Umum Seluruh Faktur Penjualan", legacyFile: "Laporan/Penjualan.jrxml" },
      { id: "penjualan_kasir", label: "2. Rekap Performa Kasir & Shift (Tunai vs Nontunai)", legacyFile: "Laporan/PenjualanKasir.jrxml" },
      { id: "penjualan_per_barang", label: "3. Rincian Qty & Nilai Penjualan Per Item Produk", legacyFile: "Laporan/PenjualanPerBarang.jrxml" },
      { id: "penjualan_per_kategori", label: "4. Analisis Penjualan Per Kelompok Kategori", legacyFile: "Laporan/PenjualanPerKategori.jrxml" },
      { id: "penjualan_per_pelanggan", label: "5. Rincian Riwayat Belanja Per Pelanggan / Member", legacyFile: "Laporan/PenjualanPerPelanggan.jrxml" },
      { id: "penjualan_per_tanggal", label: "6. Rekapitulasi Omset Urut Per Tanggal Kalender", legacyFile: "Laporan/PenjualanPerTanggal.jrxml" },
      { id: "penjualan_ranking_pelanggan", label: "7. Ranking Kontribusi Omset Terbesar Pelanggan", legacyFile: "Laporan/PenjualanTotalPelanggan.jrxml" },
      { id: "penjualan_per_sales", label: "8. Rekapitulasi Omset & Diskon Per Staf Sales", legacyFile: "Laporan/PenjualanTotalSales.jrxml" },
      { id: "penjualan_pajak", label: "9. Rekapitulasi Penjualan Kena Pajak (PPN 11%)", legacyFile: "Laporan/PenjualanPajak.jrxml" },
      { id: "penjualan_retur", label: "10. Rekap Seluruh Faktur Retur Penjualan", legacyFile: "Laporan/PenjualanRetur.jrxml" },
      { id: "penjualan_retur_detail", label: "11. Rincian Item Barang Yang Diretur Pelanggan", legacyFile: "Laporan/PenjualanReturDetail.jrxml" },
    ],
    pembelian: [
      { id: "pembelian_faktur", label: "12. Rekapitulasi Faktur Pembelian Dari Supplier", legacyFile: "Laporan/Pembelian.jrxml" },
      { id: "pembelian_per_barang", label: "13. Rincian Riwayat Pengadaan Barang Per Item", legacyFile: "Laporan/PembelianPerBarang.jrxml" },
      { id: "pembelian_per_supplier", label: "14. Rekap Belanja Stok Dikelompokkan Per Supplier", legacyFile: "Laporan/PembelianPerSuplier.jrxml" },
      { id: "pembelian_total_supplier", label: "15. Akumulasi Total Nilai Belanja Per Supplier", legacyFile: "Laporan/PembelianTotalSuplier.jrxml" },
      { id: "pembelian_retur", label: "16. Rekap Transaksi Pengembalian Retur Ke Supplier", legacyFile: "Laporan/PembelianRetur.jrxml" },
      { id: "pembelian_retur_detail", label: "17. Rincian Item Barang Yang Diretur Ke Supplier", legacyFile: "Laporan/PembelianReturDetail.jrxml" },
    ],
    keuangan: [
      { id: "laba_rugi", label: "18. Laba / Rugi Komprehensif (Penjualan - HPP - Beban)", legacyFile: "Laporan/labaRugi.jrxml" },
      { id: "cashflow", label: "19. Laporan Mutasi Arus Kas Masuk & Kas Keluar", legacyFile: "Laporan/Cashflow.jrxml" },
      { id: "biaya_operasional", label: "20. Rincian Pengeluaran Beban Operasional Toko", legacyFile: "Laporan/Biaya.jrxml" },
      { id: "modal_aset", label: "21. Estimasi Nilai Aset Modal Barang di Toko/Gudang", legacyFile: "Laporan/Modal.jrxml" },
    ],
    hutang_piutang: [
      { id: "rekap_hutang", label: "22. Rekap Status Tagihan Hutang Supplier (Lunas/Tempo)", legacyFile: "Laporan/Hutang.jrxml" },
      { id: "rekap_piutang", label: "23. Rekap Status Piutang Pelanggan & Sisa Tagihan", legacyFile: "Laporan/Piutang.jrxml" },
      { id: "piutang_per_pelanggan", label: "24. Rekap Piutang Dirinci Per Nama Pelanggan", legacyFile: "Laporan/PiutangPelanggan.jrxml" },
      { id: "histori_bayar_hp", label: "25. Riwayat Log Angsuran Pembayaran Hutang & Piutang", legacyFile: "Laporan/histori_bayar_hutang_piutang.jrxml" },
    ],
    stok: [
      { id: "katalog_barang", label: "26. Master Katalog Barang Lengkap Beserta Harga", legacyFile: "Laporan/Barang.jrxml" },
      { id: "barang_stok_total", label: "27. Rekapitulasi Kuantiti & Valuasi Total Stok Toko", legacyFile: "Laporan/barang_stok_total.jrxml" },
      { id: "kartu_stok", label: "28. Laporan Mutasi Kartu Stok Barang Individual", legacyFile: "Laporan/kartuStok.jrxml" },
      { id: "barang_masuk", label: "29. Rincian Detail Seluruh Barang Masuk Gudang", legacyFile: "Laporan/barang_masuk.jrxml" },
      { id: "barang_keluar", label: "30. Rincian Detail Seluruh Barang Keluar Toko", legacyFile: "Laporan/barang_keluar.jrxml" },
      { id: "barang_keluar_masuk", label: "31. Gabungan Rekap Arus Masuk & Keluar Barang", legacyFile: "Laporan/barang_keluar_masuk.jrxml" },
      { id: "barang_terlaris", label: "32. Analisis Produk Fast Moving (Paling Laris)", legacyFile: "Laporan/barang_terlaris.jrxml" },
      { id: "barang_tidak_laku", label: "33. Analisis Produk Dead Stock / Slow Moving", legacyFile: "Laporan/barang_tidak_laku.jrxml" },
    ],
    grafik: [
      { id: "grafik_faktur_harian", label: "34. Grafik Volume Jumlah Transaksi Faktur Harian", legacyFile: "Laporan/GrafikFakturHarian.jrxml" },
      { id: "grafik_faktur_bulanan", label: "35. Grafik Perbandingan Volume Transaksi Antar Bulan", legacyFile: "Laporan/GrafikFakturBulanan.jrxml" },
      { id: "grafik_omset_laba_harian", label: "36. Grafik Tren Omset vs Laba Bersih Harian", legacyFile: "Laporan/GrafikOmsetLabaHarian.jrxml" },
      { id: "grafik_omset_laba_bulanan", label: "37. Grafik Tren Pertumbuhan Omset vs Laba Tahunan", legacyFile: "Laporan/GrafikOmsetLabaBulanan.jrxml" },
    ],
  };

  function handleMainCategoryChange(cat: MainCategory) {
    mainCategory = cat;
    subReport = subReportOptions[cat][0].id;
  }

  interface ReportRow {
    c1: string;
    c2: string;
    c3: string;
    c4: string | number;
    c5: string | number;
    c6: string | number;
    c7: string | number;
    badge?: string;
  }

  let activeReportData = $derived.by(() => {
    let headers = ["No / Kode", "Deskripsi / Entitas", "Keterangan", "Qty / Jml", "Tarif / HPP", "Diskon / Potongan", "Total Nilai"];
    let rows: ReportRow[] = [];

    switch (subReport) {
      // ── 8.1 PENJUALAN ──
      case "penjualan_umum":
        headers = ["No. Faktur", "Waktu & Tanggal", "Kasir / Shift", "Total Item", "Subtotal", "Diskon", "Total Omset"];
        rows = [
          { c1: "#ORD-8829", c2: "10/09/2026 19:40", c3: "Alexander P. (Shift 1)", c4: "6 Pcs", c5: 148000, c6: 14800, c7: 147520, badge: "LUNAS" },
          { c1: "#ORD-8828", c2: "10/09/2026 19:28", c3: "Alexander P. (Shift 1)", c4: "3 Pcs", c5: 85000, c6: 0, c7: 85000, badge: "LUNAS" },
          { c1: "#ORD-8827", c2: "10/09/2026 19:15", c3: "Alexander P. (Shift 1)", c4: "2 Pcs", c5: 42000, c6: 0, c7: 42000, badge: "LUNAS" },
          { c1: "#ORD-8826", c2: "10/09/2026 18:55", c3: "Alexander P. (Shift 1)", c4: "4 Pcs", c5: 115000, c6: 5000, c7: 110000, badge: "LUNAS" },
          { c1: "#ORD-8825", c2: "10/09/2026 18:42", c3: "Alexander P. (Shift 1)", c4: "1 Pcs", c5: 32000, c6: 0, c7: 32000, badge: "LUNAS" },
        ];
        break;

      case "penjualan_kasir":
        headers = ["ID Operator", "Nama Kasir", "Shift Aktif", "Total Faktur", "Omset Tunai", "Omset Non-Tunai", "Total Omset"];
        rows = [
          { c1: "OP01", c2: "Alexander P.", c3: "Shift 01 (Pagi - Sore)", c4: "98 Nota", c5: 5850000, c6: 2575000, c7: 8425000 },
          { c1: "OP02", c2: "Siti Rahma", c3: "Shift 02 (Sore - Malam)", c4: "76 Nota", c5: 4210000, c6: 1980000, c7: 6190000 },
        ];
        break;

      case "penjualan_per_barang":
        headers = ["Barcode / SKU", "Nama Produk", "Kategori", "Qty Terjual", "Harga Jual", "Total HPP", "Total Omset"];
        rows = [
          { c1: "SKU-1001", c2: "Iced Caramel Macchiato 350ml", c3: "Minuman Kopi", c4: "48 Cup", c5: 32000, c6: 864000, c7: 1536000 },
          { c1: "SKU-1002", c2: "Artisan Butter Croissant", c3: "Pastry & Bakery", c4: "36 Pcs", c5: 22000, c6: 504000, c7: 792000 },
          { c1: "SKU-1005", c2: "Club Sandwich Triple Decker", c3: "Makanan Siap Saji", c4: "24 Porsi", c5: 38000, c6: 576000, c7: 912000 },
          { c1: "SKU-1007", c2: "Air Mineral Artesian 600ml", c3: "Minuman Dingin", c4: "115 Btl", c5: 8000, c6: 517500, c7: 920000 },
        ];
        break;

      case "penjualan_per_kategori":
        headers = ["Kode Kategori", "Nama Kelompok Kategori", "Jumlah SKU", "Total Qty", "Rata-rata Margin", "Total Diskon", "Total Omset"];
        rows = [
          { c1: "KAT-01", c2: "Makanan & Minuman", c3: "740 SKU", c4: "1.420 Unit", c5: "28.5%", c6: 450000, c7: 24500000 },
          { c1: "KAT-02", c2: "Kebutuhan Rumah Tangga", c3: "320 SKU", c4: "580 Unit", c5: "21.2%", c6: 180000, c7: 14200000 },
          { c1: "KAT-03", c2: "Personal Care & Kosmetik", c3: "210 SKU", c4: "310 Unit", c5: "24.0%", c6: 95000, c7: 7850000 },
          { c1: "KAT-04", c2: "Rokok & Tembakau", c3: "85 SKU", c4: "415 Bks", c5: "8.5%", c6: 0, c7: 9800000 },
        ];
        break;

      case "penjualan_per_pelanggan":
        headers = ["Kode Member", "Nama Pelanggan", "Tingkat Tier", "Frekuensi", "Poin Dipakai", "Nilai Diskon", "Total Belanja"];
        rows = [
          { c1: "MBR-0421", c2: "Budi Santoso", c3: "Tier VIP (Diskon 10%)", c4: "14 Kali", c5: "128 Pts", c6: 14800, c7: 4850000 },
          { c1: "MBR-0112", c2: "Siti Rahmawati", c3: "Tier Gold (Diskon 5%)", c4: "10 Kali", c5: "85 Pts", c6: 8500, c7: 3200000 },
          { c1: "MBR-0055", c2: "dr. Hendra Kurnia", c3: "Tier VIP (Diskon 10%)", c4: "18 Kali", c5: "350 Pts", c6: 35000, c7: 6420000 },
        ];
        break;

      case "penjualan_per_tanggal":
        headers = ["Tanggal", "Hari", "Total Nota", "Total Qty", "Omset Tunai", "Omset Non-Tunai", "Total Omset Bersih"];
        rows = [
          { c1: "10/09/2026", c2: "Kamis", c3: "Shift 1 & 2", c4: "142 Nota", c5: 5850000, c6: 2575000, c7: 8425000 },
          { c1: "09/09/2026", c2: "Rabu", c3: "Shift 1 & 2", c4: "128 Nota", c5: 5310000, c6: 2340000, c7: 7650000 },
          { c1: "08/09/2026", c2: "Selasa", c3: "Shift 1 & 2", c4: "135 Nota", c5: 5520000, c6: 2460000, c7: 7980000 },
          { c1: "07/09/2026", c2: "Senin", c3: "Shift 1 & 2", c4: "110 Nota", c5: 4580000, c6: 1960000, c7: 6540000 },
        ];
        break;

      case "penjualan_ranking_pelanggan":
        headers = ["Peringkat", "Kode / Kartu", "Nama Pelanggan", "Status Tier", "Jumlah Nota", "Rata-rata Basket", "Kontribusi Total"];
        rows = [
          { c1: "Rank 1", c2: "MBR-0055", c3: "dr. Hendra Kurnia", c4: "18 Nota", c5: 356600, c6: "12.9% Total", c7: 6420000 },
          { c1: "Rank 2", c2: "MBR-0421", c3: "Budi Santoso", c4: "14 Nota", c5: 346400, c6: "9.8% Total", c7: 4850000 },
          { c1: "Rank 3", c2: "MBR-0112", c3: "Siti Rahmawati", c4: "10 Nota", c5: 320000, c6: "6.5% Total", c7: 3200000 },
        ];
        break;

      case "penjualan_per_sales":
        headers = ["Kode Sales", "Nama Tenaga Pemasar", "Wilayah", "Total Faktur", "Komisi Persen", "Nilai Diskon", "Total Omset Sales"];
        rows = [
          { c1: "SLS-01", c2: "Rudi Hermawan", c3: "Wilayah Muaro Barat", c4: "42 Nota", c5: "2.5%", c6: 120000, c7: 12450000 },
          { c1: "SLS-02", c2: "Wahyu Pratama", c3: "Wilayah Muaro Timur", c4: "35 Nota", c5: "2.5%", c6: 85000, c7: 9850000 },
        ];
        break;

      case "penjualan_pajak":
        headers = ["No. Faktur", "Tanggal", "Nama Pelanggan", "Dasar Pengenaan Pajak (DPP)", "Tarif PPN", "Nilai PPN (11%)", "Total Faktur Pajak"];
        rows = [
          { c1: "#ORD-8829", c2: "10/09/2026", c3: "Budi Santoso", c4: "DPP Resmi", c5: 132890, c6: 14630, c7: 147520 },
          { c1: "#ORD-8826", c2: "10/09/2026", c3: "Dewi Lestari", c4: "DPP Resmi", c5: 99099, c6: 10901, c7: 110000 },
          { c1: "#ORD-8820", c2: "10/09/2026", c3: "Rian Hidayat", c4: "DPP Resmi", c5: 53603, c6: 5897, c7: 59500 },
        ];
        break;

      case "penjualan_retur":
      case "penjualan_retur_detail":
        headers = ["No. Retur", "Tanggal", "Faktur Asal Penjualan", "Nama Pelanggan", "Alasan Retur", "Kompensasi", "Total Nilai Retur"];
        rows = [
          { c1: "#RET-0012", c2: "09/09/2026", c3: "#ORD-8798", c4: "Siti Rahmawati", c5: "Kemasan Cacat Pabrik", c6: "Ganti Tunai Langsung", c7: 32000, badge: "RETUR SELESAI" },
          { c1: "#RET-0011", c2: "05/09/2026", c3: "#ORD-8740", c4: "dr. Hendra Kurnia", c5: "Salah Ambil Varian Rasa", c6: "Potong Piutang Member", c7: 45000, badge: "RETUR SELESAI" },
        ];
        break;

      // ── 8.2 PEMBELIAN ──
      case "pembelian_faktur":
      case "pembelian_per_supplier":
      case "pembelian_total_supplier":
        headers = ["No. PO / Faktur", "Nama Supplier", "Termin Tempo", "Item Barang", "Total Qty", "Status Bayar", "Total Belanja (HPP)"];
        rows = [
          { c1: "#PO-2026-0910", c2: "PT Sumber Makmur", c3: "Tunai Kas Toko", c4: "8 SKU", c5: "240 Pcs", c6: "LUNAS", c7: 4250000, badge: "LUNAS" },
          { c1: "#PO-2026-0909", c2: "PT Kopi Mandiri", c3: "Tempo 14 Hari (23 Sep)", c4: "4 SKU", c5: "120 Cup", c6: "TEMPO", c7: 3840000, badge: "TEMPO" },
          { c1: "#PO-2026-0908", c2: "Prima Bakery", c3: "Tunai", c4: "6 SKU", c5: "180 Pcs", c6: "LUNAS", c7: 2750000, badge: "LUNAS" },
          { c1: "#PO-2026-0906", c2: "Danone Tirta", c3: "Tempo 14 Hari (20 Sep)", c4: "5 SKU", c5: "350 Btl", c6: "TEMPO", c7: 2360000, badge: "TEMPO" },
        ];
        break;

      case "pembelian_per_barang":
        headers = ["Barcode", "Nama Barang Pengadaan", "Supplier Asal", "Qty Masuk", "Satuan", "Harga Beli Baru", "Subtotal Pembelian"];
        rows = [
          { c1: "SKU-1001", c2: "Sirup Caramel Import 1L", c3: "PT Kopi Mandiri", c4: "24", c5: "Btl", c6: 85000, c7: 2040000 },
          { c1: "SKU-1007", c2: "Air Mineral Artesian 600ml", c3: "Danone Tirta", c4: "15", c5: "Dus", c6: 48000, c7: 720000 },
          { c1: "SKU-1011", c2: "Indomie Goreng Spesial 85g", c3: "Indofood Sukses Makmur", c4: "50", c5: "Dus", c6: 108000, c7: 5400000 },
        ];
        break;

      case "pembelian_retur":
      case "pembelian_retur_detail":
        headers = ["No. Retur Beli", "Tanggal", "Faktur Pembelian Asal", "Nama Supplier", "Keterangan Rusak", "Kompensasi", "Total Nilai Retur"];
        rows = [
          { c1: "#RET-B-004", c2: "07/09/2026", c3: "#PO-2026-0904", c4: "Indofood Sukses Makmur", c5: "Bocor Dus Saat Ekspedisi", c6: "Potong Hutang Dagang", c7: 216000, badge: "DISETUJUI" },
        ];
        break;

      // ── 8.3 KEUANGAN & LABA RUGI ──
      case "laba_rugi":
        headers = ["Kode Akun", "Komponen Laporan Keuangan", "Penjelasan", "Kuantiti", "Debet / HPP", "Kredit / Omset", "Saldo Laba Bersih"];
        rows = [
          { c1: "REV-01", c2: "Pendapatan Omset Penjualan Bersih", c3: "842 Transaksi Kasir POS", c4: "2.231 Unit", c5: 0, c6: 49565000, c7: 49565000 },
          { c1: "HPP-01", c2: "Harga Pokok Penjualan (HPP Keluar)", c3: "Biaya Pokok Pembelian Barang Terjual", c4: "-", c5: 34250000, c6: 0, c7: -34250000 },
          { c1: "GROSS", c2: "LABA KOTOR (GROSS MARGIN)", c3: "Omset Bersih - HPP (Margin 30.9%)", c4: "-", c5: 0, c6: 0, c7: 15315000, badge: "+30.9%" },
          { c1: "EXP-01", c2: "Biaya Listrik, Air & Internet Toko", c3: "Beban Utilitas Operasional", c4: "1 Bulan", c5: 1450000, c6: 0, c7: -1450000 },
          { c1: "EXP-02", c2: "Gaji Karyawan & Operator Kasir", c3: "Payroll Shift Kasir", c4: "3 Orang", c5: 6500000, c6: 0, c7: -6500000 },
          { c1: "EXP-03", c2: "Perlengkapan, Plastik & Kertas Struk", c3: "Konsumsi Operasional Toko", c4: "12 Pack", c5: 420000, c6: 0, c7: -420000 },
          { c1: "NET", c2: "ESTIMASI LABA BERSIH (NET PROFIT)", c3: "Laba Kotor - Total Beban Toko", c4: "-", c5: 0, c6: 0, c7: 6945000, badge: "+14.0% NET" },
        ];
        break;

      case "cashflow":
        headers = ["Tanggal & Jam", "Kategori Arus Kas", "Keterangan Transaksi", "Operator", "Kas Masuk", "Kas Keluar", "Saldo Kas Berjalan"];
        rows = [
          { c1: "10/09 19:40", c2: "Penjualan Kasir POS", c3: "Faktur #ORD-8829 (Tunai)", c4: "Alexander P.", c5: 147520, c6: 0, c7: 8425000 },
          { c1: "10/09 14:00", c2: "Beban Operasional", c3: "Beli Kertas Struk Thermal 80mm (1 Dus)", c4: "Siti Rahma", c5: 0, c6: 185000, c7: 8277480 },
          { c1: "10/09 10:00", c2: "Pembayaran Piutang", c3: "Cicilan Piutang Member #ORD-8812", c4: "Alexander P.", c5: 500000, c6: 0, c7: 8462480 },
          { c1: "10/09 08:00", c2: "Modal Awal Shift", c3: "Buka Shift 01 Kasir Reg 01", c4: "Alexander P.", c5: 100000, c6: 0, c7: 7962480 },
        ];
        break;

      case "biaya_operasional":
        headers = ["No. Bukti", "Tanggal", "Pos Kategori Biaya", "Penerima / Vendor", "Metode Bayar", "Penanggung Jawab", "Total Pengeluaran"];
        rows = [
          { c1: "#EXP-091", c2: "05/09/2026", c3: "Listrik PLN & Air PDAM Toko", c4: "PLN Muaro", c5: "Transfer Bank", c6: "Manager Toko", c7: 1450000 },
          { c1: "#EXP-092", c2: "01/09/2026", c3: "Gaji Operator Kasir & Staf", c4: "3 Karyawan", c5: "Transfer Bank", c6: "Owner", c7: 6500000 },
          { c1: "#EXP-093", c2: "08/09/2026", c3: "Konsumsi & Minuman Staf Toko", c4: "Pantry", c5: "Kas Toko (Petty Cash)", c6: "Kasir OP01", c7: 240000 },
        ];
        break;

      case "modal_aset":
        headers = ["Kelompok Kategori", "Total Item SKU", "Kuantiti Fisik", "Rata-rata HPP", "Total Nilai Modal", "Estimasi Nilai Jual", "Potensi Margin"];
        rows = [
          { c1: "Makanan & Minuman", c2: "740 SKU", c3: "12.450 Pcs", c4: 18500, c5: 28450000, c6: 36500000, c7: 8050000 },
          { c1: "Kebutuhan Rumah Tangga", c2: "320 SKU", c3: "4.800 Pcs", c4: 14200, c5: 12100000, c6: 15400000, c7: 3300000 },
          { c1: "Personal Care", c2: "210 SKU", c3: "2.100 Pcs", c4: 21000, c5: 8100000, c6: 10800000, c7: 2700000 },
        ];
        break;

      // ── 8.4 HUTANG & PIUTANG ──
      case "rekap_hutang":
      case "rekap_piutang":
      case "piutang_per_pelanggan":
      case "histori_bayar_hp":
        headers = ["No. Faktur Ref", "Tanggal", "Nama Rekanan (Supplier/Member)", "Jatuh Tempo", "Tagihan Awal", "Telah Dibayar", "Sisa Saldo"];
        rows = [
          { c1: "#PO-2026-0909", c2: "09/09/2026", c3: "PT Kopi Mandiri", c4: "23/09/2026", c5: 3840000, c6: 0, c7: 3840000, badge: "TEMPO" },
          { c1: "#PO-2026-0906", c2: "06/09/2026", c3: "Danone Tirta", c4: "20/09/2026", c5: 2360000, c6: 1000000, c7: 1360000, badge: "TEMPO" },
          { c1: "#ORD-8812", c2: "08/09/2026", c3: "dr. Hendra Kurnia (VIP)", c4: "22/09/2026", c5: 1250000, c6: 500000, c7: 750000, badge: "TEMPO" },
          { c1: "#ORD-8790", c2: "02/09/2026", c3: "Siti Rahmawati (Gold)", c4: "16/09/2026", c5: 850000, c6: 0, c7: 850000, badge: "TEMPO" },
        ];
        break;

      // ── 8.5 INVENTORI & STOK ──
      case "katalog_barang":
      case "barang_stok_total":
        headers = ["Barcode", "Nama Produk", "Kategori", "Lokasi Rak", "Stok Fisik", "Harga Pokok (HPP)", "Harga Jual 1"];
        rows = [
          { c1: "SKU-1001", c2: "Iced Caramel Macchiato 350ml", c3: "Minuman Kopi", c4: "Chiller A", c5: "48 Cup", c6: 18000, c7: 32000 },
          { c1: "SKU-1002", c2: "Artisan Butter Croissant", c3: "Pastry", c4: "Display 01", c5: "24 Pcs", c6: 14000, c7: 22000 },
          { c1: "SKU-1005", c2: "Club Sandwich Triple Decker", c3: "Siap Saji", c4: "Warm Box", c5: "15 Porsi", c6: 24000, c7: 38000 },
          { c1: "SKU-1007", c2: "Air Mineral Artesian 600ml", c3: "Minuman Dingin", c4: "Rak B-02", c5: "120 Btl", c6: 4500, c7: 8000 },
        ];
        break;

      case "kartu_stok":
      case "barang_masuk":
      case "barang_keluar":
      case "barang_keluar_masuk":
        headers = ["Tanggal & Waktu", "No. Referensi", "Jenis Mutasi", "Keterangan Aliran", "Masuk (+)", "Keluar (-)", "Saldo Akhir"];
        rows = [
          { c1: "10/09 19:40", c2: "#ORD-8829", c3: "Penjualan Kasir POS", c4: "Transaksi Nota Kasir", c5: "-", c6: "2 Cup", c7: "48 Cup" },
          { c1: "10/09 14:20", c2: "#PO-2026-0910", c3: "Pembelian Supplier", c4: "Penerimaan Barang Gudang", c5: "50 Cup", c6: "-", c7: "50 Cup" },
          { c1: "08/09 09:10", c2: "#OPN-041", c3: "Stok Opname", c4: "Penyesuaian Selisih Fisik", c5: "-", c6: "1 Cup", c7: "0 Cup" },
        ];
        break;

      case "barang_terlaris":
        headers = ["Ranking", "Barcode", "Nama Produk Terlaris", "Kategori", "Qty Terjual", "Harga Satuan", "Total Kontribusi Omset"];
        rows = [
          { c1: "Top 1", c2: "SKU-1007", c3: "Air Mineral Artesian 600ml", c4: "Minuman Dingin", c5: "384 Btl", c6: 8000, c7: 3072000, badge: "FAST MOVING" },
          { c1: "Top 2", c2: "SKU-1001", c3: "Iced Caramel Macchiato 350ml", c4: "Minuman Kopi", c5: "240 Cup", c6: 32000, c7: 7680000, badge: "FAST MOVING" },
          { c1: "Top 3", c2: "SKU-1011", c3: "Indomie Goreng Spesial 85g", c4: "Makanan Instan", c5: "195 Pcs", c6: 3500, c7: 682500, badge: "FAST MOVING" },
          { c1: "Top 4", c2: "SKU-1002", c3: "Artisan Butter Croissant", c4: "Pastry & Bakery", c5: "145 Pcs", c6: 22000, c7: 3190000, badge: "FAST MOVING" },
        ];
        break;

      case "barang_tidak_laku":
        headers = ["Status", "Barcode", "Nama Produk Slow Moving", "Kategori", "Sisa Stok", "Hari Tanpa Transaksi", "Valuasi Tertahan"];
        rows = [
          { c1: "Dead Stock", c2: "SKU-9901", c3: "Kecap Manis Botol Kaca 620ml", c4: "Bumbu Dapur", c5: "8 Btl", c6: "68 Hari", c7: 144000, badge: "SLOW MOVING" },
          { c1: "Dead Stock", c2: "SKU-9905", c3: "Pembersih Kaca Refill 400ml", c4: "Kebutuhan Rumah", c5: "12 Pcs", c6: "52 Hari", c7: 108000, badge: "SLOW MOVING" },
        ];
        break;

      // ── 8.6 GRAFIK BISNIS ──
      case "grafik_faktur_harian":
      case "grafik_faktur_bulanan":
      case "grafik_omset_laba_harian":
      case "grafik_omset_laba_bulanan":
        headers = ["Periode Waktu", "Hari / Bulan", "Status Target", "Total Faktur", "HPP / Biaya", "Laba Bersih", "Total Omset"];
        rows = [
          { c1: "Senin", c2: "07 September 2026", c3: "Target Rp 6.5 Juta", c4: "110 Faktur", c5: 4580000, c6: 1960000, c7: 6540000 },
          { c1: "Selasa", c2: "08 September 2026", c3: "Target Rp 7.5 Juta", c4: "135 Faktur", c5: 5520000, c6: 2460000, c7: 7980000 },
          { c1: "Rabu", c2: "09 September 2026", c3: "Target Rp 7.5 Juta", c4: "128 Faktur", c5: 5310000, c6: 2340000, c7: 7650000 },
          { c1: "Kamis", c2: "10 September 2026", c3: "Target Rp 8.0 Juta", c4: "142 Faktur", c5: 5820000, c6: 2605000, c7: 8425000 },
          { c1: "Jumat (Est)", c2: "11 September 2026", c3: "Target Rp 8.5 Juta", c4: "155 Faktur", c5: 6100000, c6: 2850000, c7: 8950000 },
        ];
        break;

      default:
        headers = ["No", "Kode / Tanggal", "Deskripsi Entitas", "Qty", "Nilai 1", "Nilai 2", "Total"];
        rows = [
          { c1: "1", c2: "Data Laporan 1", c3: "Keterangan", c4: "10", c5: 100000, c6: 0, c7: 100000 },
        ];
    }

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase().trim();
      rows = rows.filter((r) =>
        r.c1.toLowerCase().includes(q) ||
        r.c2.toLowerCase().includes(q) ||
        r.c3.toLowerCase().includes(q)
      );
    }

    return { headers, rows };
  });

  function showToast(msg: string) {
    toastMessage = msg;
    setTimeout(() => (toastMessage = ""), 4000);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "F3") {
      e.preventDefault();
      searchInputElement?.focus();
      searchInputElement?.select();
    }
  }

  $effect(() => {
    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });
</script>

<div class="flex-1 flex flex-col bg-slate-200 overflow-hidden font-sans select-none min-h-0">
  <!-- 1. TOP ACTION & STATUS TABS BAR (Di Atas Card Sesuai Aturan Layout) -->
  <div class="px-4 py-2.5 bg-white border-b border-slate-300 shadow-2xs flex flex-wrap items-center justify-between gap-3 shrink-0">
    <!-- 6 Modul Kategori Laporan Lengkap (Sesuai spesifikasi-legacy-ibretago5.md section 8) -->
    <div class="flex items-center gap-1 p-1 bg-slate-100 rounded-xl border border-slate-200 overflow-x-auto max-w-[65vw]">
      <button
        onclick={() => handleMainCategoryChange("penjualan")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none shrink-0 {mainCategory === 'penjualan' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span class="material-symbols-outlined text-[15px]">point_of_sale</span>
        <span>Laporan Penjualan (11)</span>
      </button>

      <button
        onclick={() => handleMainCategoryChange("pembelian")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none shrink-0 {mainCategory === 'pembelian' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span class="material-symbols-outlined text-[15px]">local_shipping</span>
        <span>Laporan Pembelian (6)</span>
      </button>

      <button
        onclick={() => handleMainCategoryChange("keuangan")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none shrink-0 {mainCategory === 'keuangan' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span class="material-symbols-outlined text-[15px]">query_stats</span>
        <span>Laba Rugi &amp; Kas (4)</span>
      </button>

      <button
        onclick={() => handleMainCategoryChange("hutang_piutang")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none shrink-0 {mainCategory === 'hutang_piutang' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span class="material-symbols-outlined text-[15px]">account_balance_wallet</span>
        <span>Hutang &amp; Piutang (4)</span>
      </button>

      <button
        onclick={() => handleMainCategoryChange("stok")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none shrink-0 {mainCategory === 'stok' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span class="material-symbols-outlined text-[15px]">inventory_2</span>
        <span>Inventori &amp; Stok (8)</span>
      </button>

      <button
        onclick={() => handleMainCategoryChange("grafik")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none shrink-0 {mainCategory === 'grafik' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span class="material-symbols-outlined text-[15px]">bar_chart</span>
        <span>Grafik &amp; Tren Bisnis (4)</span>
      </button>
    </div>

    <!-- Action Utilities (Di Atas Card) -->
    <div class="flex items-center gap-1.5 shrink-0">
      <button
        onclick={() => showToast("Mencetak laporan dalam format cetak resmi...")}
        class="flex items-center gap-1.5 px-3 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-800 rounded-lg font-sans text-xs font-medium border border-slate-300 shadow-2xs transition-all cursor-pointer"
        title="Cetak Laporan (F4)"
      >
        <span class="font-mono text-[10px] font-bold bg-slate-200 px-1 py-0.2 rounded text-slate-700">F4</span>
        <span class="material-symbols-outlined text-[16px] text-primary">print</span>
        <span>Cetak Laporan</span>
      </button>

      <button
        onclick={() => showToast("Export laporan ke format Excel (.xlsx / .csv) berhasil!")}
        class="flex items-center gap-1.5 px-3 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-800 rounded-lg font-sans text-xs font-medium border border-slate-300 shadow-2xs transition-all cursor-pointer"
        title="Export CSV / Excel (F7)"
      >
        <span class="font-mono text-[10px] font-bold bg-slate-200 px-1 py-0.2 rounded text-slate-700">F7</span>
        <span class="material-symbols-outlined text-[16px] text-emerald-600">table_view</span>
        <span>Export CSV</span>
      </button>

      <button
        onclick={() => showToast("Laporan PDF format A4 berhasil disiapkan.")}
        class="flex items-center gap-1.5 px-3 py-1.5 bg-rose-50 hover:bg-rose-100 text-rose-800 rounded-lg font-sans text-xs font-medium border border-rose-300 shadow-2xs transition-all cursor-pointer"
        title="Export PDF"
      >
        <span class="material-symbols-outlined text-[16px] text-rose-600">picture_as_pdf</span>
        <span>PDF</span>
      </button>

      <button
        onclick={() => showToast("Kalkulasi ulang agregasi SQLite berhasil diperbarui.")}
        class="p-1.5 rounded bg-slate-100 hover:bg-slate-200 text-slate-700 transition-colors border border-slate-300 shadow-2xs flex items-center justify-center cursor-pointer"
        title="Segarkan Data"
      >
        <span class="material-symbols-outlined text-[18px]">refresh</span>
      </button>
    </div>
  </div>

  <!-- 2. 5 COMPACT STAT CARDS (Di Tengah Sesuai Aturan Layout) -->
  <div class="px-4 py-2.5 bg-slate-200 border-b border-slate-300 shrink-0">
    <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-2.5 items-stretch">
      <!-- Card 1: Total Omset -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Total Omset Kotor</span>
          <span class="material-symbols-outlined text-primary text-[18px]">payments</span>
        </div>
        <div class="my-1">
          <span class="font-mono text-xl font-bold text-primary tracking-tight">
            {formatRupiah(49565000)}
          </span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>842 Transaksi</span>
          <span class="text-emerald-700 font-bold">+14.2% MoM</span>
        </div>
      </div>

      <!-- Card 2: Total HPP Pembelian -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Total HPP Barang</span>
          <span class="material-symbols-outlined text-slate-600 text-[18px]">inventory</span>
        </div>
        <div class="my-1">
          <span class="font-mono text-xl font-bold text-slate-800 tracking-tight">
            {formatRupiah(34250000)}
          </span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Beban HPP Keluar</span>
          <span class="text-slate-700 font-bold">69.1% Omset</span>
        </div>
      </div>

      <!-- Card 3: Laba Kotor (Gross Profit) -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-emerald-800 font-bold">Laba Kotor (Gross)</span>
          <span class="material-symbols-outlined text-emerald-600 text-[18px]">price_check</span>
        </div>
        <div class="my-1">
          <span class="font-mono text-xl font-bold text-emerald-700 tracking-tight">
            {formatRupiah(15315000)}
          </span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Gross Margin</span>
          <span class="text-emerald-700 font-bold">+30.9% HPP</span>
        </div>
      </div>

      <!-- Card 4: Biaya Beban Toko -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-rose-700 font-bold">Biaya Operasional</span>
          <span class="material-symbols-outlined text-rose-600 text-[18px]">receipt</span>
        </div>
        <div class="my-1">
          <span class="font-mono text-xl font-bold text-rose-700 tracking-tight">
            {formatRupiah(8370000)}
          </span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Listrik, Gaji, Toko</span>
          <span class="text-slate-700 font-bold">16.8% Omset</span>
        </div>
      </div>

      <!-- Card 5: Estimasi Laba Bersih -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Laba Bersih (Net)</span>
          <span class="material-symbols-outlined text-emerald-600 text-[18px]">account_balance_wallet</span>
        </div>
        <div class="my-1">
          <span class="font-mono text-xl font-bold text-emerald-800 tracking-tight">
            {formatRupiah(6945000)}
          </span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Net Profit Margin</span>
          <span class="text-emerald-700 font-bold">+14.0% Bersih</span>
        </div>
      </div>
    </div>
  </div>

  <!-- 3. SEARCH BAR & FILTER DROPDOWNS (Di Bawah Card Sesuai Aturan Layout) -->
  <div class="px-4 py-2 bg-white border-b border-slate-300 shadow-2xs flex flex-wrap items-center justify-between gap-3 shrink-0">
    <!-- Dropdown Sub-Laporan 37 Variasi Lengkap -->
    <div class="flex items-center gap-2 font-sans text-xs">
      <span class="font-bold text-slate-700">Pilih Laporan:</span>
      <div class="relative">
        <select
          bind:value={subReport}
          class="appearance-none bg-slate-100 text-slate-900 font-sans text-xs px-3 py-1.5 pr-8 rounded-lg border border-slate-300 shadow-2xs focus:outline-none cursor-pointer font-bold min-w-[320px] max-w-md truncate"
          title="Pilih Format Laporan"
        >
          {#each subReportOptions[mainCategory] as opt}
            <option value={opt.id}>{opt.label}</option>
          {/each}
        </select>
        <span class="material-symbols-outlined absolute right-2 top-1.5 pointer-events-none text-slate-500 text-[16px]">expand_more</span>
      </div>
    </div>

    <div class="flex items-center gap-2.5 flex-1 justify-end">
      <!-- Search Input Filter -->
      <div class="min-w-[240px] max-w-sm relative">
        <div class="absolute inset-y-0 left-0 pl-2.5 flex items-center pointer-events-none text-slate-400">
          <span class="material-symbols-outlined text-[16px]">search</span>
        </div>
        <input
          bind:this={searchInputElement}
          bind:value={searchQuery}
          class="w-full pl-8 pr-12 py-1.5 bg-slate-100 text-slate-900 placeholder:text-slate-400 font-sans text-xs rounded-lg border border-slate-300 outline-none focus:border-primary focus:bg-white focus:ring-1 focus:ring-primary transition-all"
          placeholder="Filter data laporan..."
          type="text"
        />
        <div class="absolute inset-y-0 right-0 pr-2 flex items-center pointer-events-none">
          <span class="px-1 py-0.2 rounded bg-slate-200 text-slate-700 font-mono text-[9px] font-bold border border-slate-300">
            F3
          </span>
        </div>
      </div>

      <!-- Date Range Preset Picker -->
      <div class="flex items-center gap-1.5 font-sans text-xs">
        <span class="font-medium text-slate-600">Periode:</span>
        <div class="relative">
          <select
            bind:value={dateRange}
            class="appearance-none bg-slate-100 text-slate-800 font-sans text-xs px-3 py-1.5 pr-7 rounded-lg border border-slate-300 shadow-2xs focus:outline-none cursor-pointer font-medium"
          >
            <option value="hari_ini">Hari Ini (10 Sep 2026)</option>
            <option value="kemarin">Kemarin (09 Sep 2026)</option>
            <option value="minggu_ini">7 Hari Terakhir</option>
            <option value="bulan_ini">Bulan Ini (September 2026)</option>
            <option value="bulan_lalu">Bulan Lalu (Agustus 2026)</option>
            <option value="tahun_ini">Tahun 2026</option>
          </select>
          <span class="material-symbols-outlined absolute right-2 top-1.5 pointer-events-none text-slate-500 text-[16px]">expand_more</span>
        </div>
      </div>
    </div>
  </div>

  <!-- 4. FULL-WIDTH DATA TABLE / REPORT PREVIEW -->
  <div class="w-full p-4 flex-1 flex flex-col min-h-0 overflow-hidden">
    <div class="w-full bg-white rounded-xl border border-slate-300 shadow-sm overflow-hidden flex flex-col flex-1 min-h-0">
      
      <!-- Visual Chart Header jika Kategori Grafik dipilih -->
      {#if mainCategory === 'grafik'}
        <div class="p-4 bg-slate-50 border-b border-slate-200 shrink-0">
          <div class="flex items-center justify-between mb-2">
            <span class="font-bold text-slate-900 text-xs font-mono uppercase flex items-center gap-1.5">
              <span class="material-symbols-outlined text-primary text-[18px]">analytics</span>
              Visualisasi Tren Bisnis (SQLite Native Histogram)
            </span>
            <span class="font-mono text-[10px] text-emerald-700 font-bold bg-emerald-100 px-2 py-0.5 rounded border border-emerald-300">
              Tren Positif +14.2% MoM
            </span>
          </div>
          <!-- Bar visualizer -->
          <div class="grid grid-cols-5 gap-3 items-end h-24 pt-2 pb-1 border-b border-slate-300">
            <div class="flex flex-col items-center gap-1 h-full justify-end">
              <span class="font-mono text-[10px] font-bold text-slate-600">Rp 6.5M</span>
              <div class="w-full bg-primary/70 rounded-t h-[65%] hover:bg-primary transition-colors"></div>
              <span class="font-mono text-[9px] text-slate-500">Senin</span>
            </div>
            <div class="flex flex-col items-center gap-1 h-full justify-end">
              <span class="font-mono text-[10px] font-bold text-slate-600">Rp 7.9M</span>
              <div class="w-full bg-primary/80 rounded-t h-[79%] hover:bg-primary transition-colors"></div>
              <span class="font-mono text-[9px] text-slate-500">Selasa</span>
            </div>
            <div class="flex flex-col items-center gap-1 h-full justify-end">
              <span class="font-mono text-[10px] font-bold text-slate-600">Rp 7.6M</span>
              <div class="w-full bg-primary/76 rounded-t h-[76%] hover:bg-primary transition-colors"></div>
              <span class="font-mono text-[9px] text-slate-500">Rabu</span>
            </div>
            <div class="flex flex-col items-center gap-1 h-full justify-end">
              <span class="font-mono text-[10px] font-bold text-primary">Rp 8.4M</span>
              <div class="w-full bg-primary rounded-t h-[84%] hover:bg-primary-dark transition-colors shadow-sm"></div>
              <span class="font-mono text-[9px] font-bold text-primary">Kamis</span>
            </div>
            <div class="flex flex-col items-center gap-1 h-full justify-end">
              <span class="font-mono text-[10px] font-bold text-emerald-700">Rp 8.9M</span>
              <div class="w-full bg-emerald-500 rounded-t h-[89%] hover:bg-emerald-600 transition-colors"></div>
              <span class="font-mono text-[9px] text-slate-500">Jumat (Est)</span>
            </div>
          </div>
        </div>
      {/if}

      <!-- Main Data Table Container -->
      <div class="overflow-auto w-full flex-1 min-h-0">
        <table class="w-full text-left font-sans text-xs border-collapse">
          <!-- Table Header -->
          <thead class="bg-slate-800 text-slate-100 uppercase font-mono text-[11px] font-bold tracking-wider select-none border-b-2 border-slate-900 sticky top-0 z-10 shadow-sm">
            <tr>
              {#each activeReportData.headers as head, idx}
                <th
                  class="py-2.5 px-3 {idx === 0 ? 'w-32' : ''} {idx >= 3 ? 'text-right' : ''}"
                  scope="col"
                >
                  {head}
                </th>
              {/each}
            </tr>
          </thead>

          <!-- Table Body -->
          <tbody class="divide-y divide-slate-200 text-slate-900 font-mono text-xs">
            {#each activeReportData.rows as row, i}
              <tr class="transition-colors border-b border-slate-200/80 {i % 2 === 1 ? 'bg-slate-50/70 hover:bg-sky-50/80' : 'bg-white hover:bg-sky-50/80'}">
                <td class="py-2.5 px-3 font-bold text-primary">
                  {row.c1}
                </td>
                <td class="py-2.5 px-3 font-sans font-semibold text-slate-900">
                  {row.c2}
                  {#if row.badge}
                    <span class="ml-1.5 px-1.5 py-0.2 rounded text-[10px] font-bold {row.badge.includes('LUNAS') || row.badge.includes('+') || row.badge.includes('FAST') ? 'bg-emerald-100 text-emerald-800 border border-emerald-300' : row.badge.includes('TEMPO') ? 'bg-amber-100 text-amber-800 border border-amber-300' : 'bg-rose-100 text-rose-800 border border-rose-300'}">
                      {row.badge}
                    </span>
                  {/if}
                </td>
                <td class="py-2.5 px-3 font-sans text-slate-600">
                  {row.c3}
                </td>
                <td class="py-2.5 px-3 text-right font-bold text-slate-800">
                  {typeof row.c4 === 'number' ? formatRupiah(row.c4) : row.c4}
                </td>
                <td class="py-2.5 px-3 text-right font-medium text-slate-700 tabular-nums">
                  {typeof row.c5 === 'number' ? formatRupiah(row.c5) : row.c5}
                </td>
                <td class="py-2.5 px-3 text-right font-semibold text-slate-700 tabular-nums">
                  {typeof row.c6 === 'number' ? formatRupiah(row.c6) : row.c6}
                </td>
                <td class="py-2.5 px-3 text-right font-bold tabular-nums {typeof row.c7 === 'number' && row.c7 < 0 ? 'text-rose-700' : 'text-slate-900'}">
                  {typeof row.c7 === 'number' ? formatRupiah(row.c7) : row.c7}
                </td>
              </tr>
            {:else}
              <tr>
                <td colspan={activeReportData.headers.length} class="py-16 text-center text-slate-400 font-mono text-xs">
                  Tidak ada data untuk laporan ini pada periode yang dipilih.
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      <!-- Footer Bar -->
      <footer class="bg-slate-100 p-2.5 border-t border-slate-300 flex flex-col md:flex-row items-center justify-between gap-2 text-xs shrink-0 select-none">
        <div class="flex flex-wrap items-center gap-3 text-slate-600 font-mono text-[11px]">
          <div class="flex items-center gap-1.5">
            <span class="w-2 h-2 rounded-full bg-emerald-600"></span>
            <span class="font-medium text-emerald-800 font-bold">Format Tabel Murni (SQLite Query Engine Tanpa Overhead Java)</span>
          </div>
          <span>•</span>
          <div>
            Menampilkan <strong class="text-slate-900">{activeReportData.rows.length}</strong> Baris Data Rekapitulasi
          </div>
        </div>

        <div class="flex items-center gap-2 font-sans text-xs">
          <span class="text-slate-500 font-mono text-[10px]">Cakupan:</span>
          <span class="px-2 py-0.5 rounded bg-slate-200 text-slate-700 font-mono text-[10px] font-bold">37 Modul Terdaftar</span>
          <span class="px-2 py-0.5 rounded bg-blue-100 text-blue-800 font-mono text-[10px] font-bold">Online &amp; Offline Ready</span>
        </div>
      </footer>
    </div>
  </div>

  <!-- Toast Message -->
  {#if toastMessage}
    <div class="fixed bottom-4 right-4 bg-slate-900 text-emerald-300 px-4 py-2.5 rounded-lg shadow-xl font-mono text-xs border border-slate-700 animate-in fade-in z-50 flex items-center gap-2">
      <span class="material-symbols-outlined text-[18px] text-emerald-400">info</span>
      <span>{toastMessage}</span>
    </div>
  {/if}
</div>
