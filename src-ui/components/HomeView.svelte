<script lang="ts">
  import { formatRupiah, type OperatorDTO, type SettingsDTO, type StatusInfoDTO } from "../lib/api";
  import type { ViewType } from "./TopBar.svelte";

  let {
    status = null,
    currentUser = null,
    appSettings = null,
    onNavigate,
  }: {
    status?: StatusInfoDTO | null;
    currentUser?: OperatorDTO | null;
    appSettings?: SettingsDTO | null;
    onNavigate?: (view: ViewType) => void;
  } = $props();

  // Deteksi Role Akun yang Login
  const userRole = $derived.by(() => {
    if (!currentUser) return "kasir";
    if (currentUser.is_admin || currentUser.role === "admin" || currentUser.role.startsWith("admin")) return "admin";
    const r = currentUser.role.toLowerCase();
    if (r === "supervisor") return "supervisor";
    if (r === "gudang") return "gudang";
    return "kasir";
  });

  // DATA KEUANGAN TERINTEGRASI (SALING TERKAIT):
  // 1. Penjualan Hari Ini
  const penjualan = {
    total: 4850000,
    tunai: 2450000,
    nonTunai: 2400000,
    qris: 1150000,
    edc: 1250000,
    totalFaktur: 48,
    persenKenaikan: 12.5,
  };

  // 2. Operasional & Beban Toko
  const operasional = {
    totalBeban: 385000,
    modalAwalKasir: 500000,
    rincian: [
      { nama: "Token Listrik PLN Kasir", nominal: 150000, jam: "09:30" },
      { nama: "Kertas Struk Thermal 80mm", nominal: 110000, jam: "11:15" },
      { nama: "Air Galon & Konsumsi Pantry", nominal: 75000, jam: "13:40" },
      { nama: "Bensin Motor Antar Barang", nominal: 50000, jam: "15:20" },
    ],
  };

  // 3. Pembelian & Hutang/Piutang Terkait
  const hutangPiutang = {
    piutangTerbayar: 150000, // member melunasi cicilan -> masuk kas
    pembelianTunai: 180000,  // beli stok supplier tunai -> kas keluar
  };

  // 4. PERHITUNGAN OTOMATIS SALDO LACI KASIR & DIGITAL (SALING TERKAIT):
  // Rumus Saldo Kas Fisik di Laci:
  // Modal Awal + Penjualan Tunai + Piutang Terbayar - Beban Operasional Tunai - Pembelian Tunai
  const saldoDiLaci = $derived(
    operasional.modalAwalKasir +
    penjualan.tunai +
    hutangPiutang.piutangTerbayar -
    operasional.totalBeban -
    hutangPiutang.pembelianTunai
  ); // 500.000 + 2.450.000 + 150.000 - 385.000 - 180.000 = Rp 2.535.000

  // Saldo Non-Tunai Masuk Rekening Toko
  const saldoNonTunai = $derived(penjualan.qris + penjualan.edc); // Rp 2.400.000

  // Total Kas Toko Masuk Bersih Hari Ini (Uang Fisik + Uang Rekening)
  const totalKasTokoBersih = $derived(saldoDiLaci + saldoNonTunai);

  // Estimasi Laba Bersih Toko (Omset - HPP - Beban Operasional)
  const hppBarangTerjual = 3230000;
  const labaKotor = $derived(penjualan.total - hppBarangTerjual); // 1.620.000
  const labaBersih = $derived(labaKotor - operasional.totalBeban); // 1.235.000

  // Riwayat Transaksi Terkini
  const transaksiTerbaru = [
    { no: "#ORD-8829", jam: "19:43:50", kasir: "Alexander P.", metode: "TUNAI", total: 147520, member: "Budi Santoso" },
    { no: "#ORD-8828", jam: "19:38:12", kasir: "Alexander P.", metode: "QRIS", total: 85000, member: "Umum" },
    { no: "#ORD-8827", jam: "19:20:05", kasir: "Alexander P.", metode: "EDC BCA", total: 230000, member: "Siti Rahma" },
    { no: "#ORD-8826", jam: "19:05:44", kasir: "Alexander P.", metode: "TUNAI", total: 38000, member: "Umum" },
    { no: "#ORD-8825", jam: "18:48:19", kasir: "Alexander P.", metode: "QRIS", total: 95500, member: "dr. Hendra K." },
  ];

  // Data Khusus Gudang
  const barangMenipisGudang = [
    { sku: "BRG001", nama: "Aqua Galon 19L", sisa: 4, minimum: 10, satuan: "Galon", supplier: "Danone Tirta" },
    { sku: "SKU-1005", nama: "Club Sandwich Triple", sisa: 2, minimum: 5, satuan: "Porsi", supplier: "Dapur Fresh" },
    { sku: "SKU-1007", nama: "Air Mineral Artesian 600ml", sisa: 8, minimum: 12, satuan: "Botol", supplier: "PT Sumber Air" },
    { sku: "SKU-1012", nama: "Biji Kopi Arabika Gayo 250g", sisa: 3, minimum: 8, satuan: "Pack", supplier: "PT Kopi Mandiri" },
  ];
</script>

<div class="flex-1 flex flex-col bg-slate-200 overflow-y-auto font-sans select-none [scrollbar-width:thin]">
  <!-- ========================================================================= -->
  <!-- 1. HERO HEADER: IDENTITAS TOKO, OPERATOR & TOMBOL PINTASAN CEPAT          -->
  <!-- ========================================================================= -->
  <div class="bg-gradient-to-r from-slate-900 via-slate-800 to-slate-900 text-slate-100 px-3.5 sm:px-6 py-3 sm:py-4 border-b border-slate-800 shrink-0 shadow-sm">
    <div class="flex flex-col lg:flex-row lg:items-center justify-between gap-3">
      <!-- Info Akun & Sesi -->
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-2xl {userRole === 'admin' ? 'bg-amber-500' : userRole === 'gudang' ? 'bg-emerald-600' : 'bg-sky-600'} text-white flex items-center justify-center font-black text-lg shadow-md shrink-0">
          <span class="material-symbols-outlined text-[22px]">
            {userRole === 'admin' ? 'admin_panel_settings' : userRole === 'gudang' ? 'warehouse' : 'point_of_sale'}
          </span>
        </div>
        <div>
          <div class="flex items-center gap-2 flex-wrap">
            <h1 class="text-base sm:text-lg font-black tracking-tight leading-tight">
              Ringkasan Keuangan &amp; Operasional Toko
            </h1>
            <span class="text-[9px] font-mono font-bold px-2 py-0.5 rounded-full uppercase {userRole === 'admin' ? 'bg-amber-500/20 text-amber-300 border border-amber-500/40' : 'bg-sky-500/20 text-sky-300 border border-sky-500/40'}">
              {currentUser?.nama ?? "Alexander P."} ({userRole.toUpperCase()})
            </span>
          </div>
          <div class="text-[11px] text-slate-400 flex items-center gap-2 mt-0.5 font-medium">
            <span>{appSettings?.toko_nama ?? status?.toko_nama ?? "MUEEZA STORE"}</span>
            <span>•</span>
            <span class="text-emerald-400 flex items-center gap-1 font-mono">
              <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
              Shift 01 Aktif
            </span>
            <span>•</span>
            <span class="font-mono text-amber-300">{status?.clock ?? "19:43:50 WIB"}</span>
          </div>
        </div>
      </div>

      <!-- Tombol Aksi Cepat -->
      <div class="flex items-center gap-1.5 flex-wrap">
        <button
          onclick={() => onNavigate?.("kasir")}
          class="px-3 py-1.5 rounded-xl bg-sky-500 hover:bg-sky-400 active:bg-sky-600 text-white font-bold text-xs flex items-center gap-1.5 cursor-pointer transition-all shadow-xs"
          title="Buka Layar Transaksi Kasir [F1]"
        >
          <span class="font-mono text-[9px] bg-sky-700 px-1 py-0.2 rounded font-bold">F1</span>
          <span class="material-symbols-outlined text-[15px]">point_of_sale</span>
          <span>Buka Kasir</span>
        </button>

        <button
          onclick={() => onNavigate?.("operasional")}
          class="px-3 py-1.5 rounded-xl bg-red-600 hover:bg-red-500 text-white font-bold text-xs flex items-center gap-1.5 cursor-pointer transition-colors shadow-xs"
          title="Catat Biaya Operasional Toko"
        >
          <span class="material-symbols-outlined text-[15px]">payments</span>
          <span>Beban Toko</span>
        </button>

        <button
          onclick={() => onNavigate?.("penjualan")}
          class="px-3 py-1.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 hover:text-white font-semibold text-xs border border-slate-700 flex items-center gap-1.5 cursor-pointer transition-colors"
          title="Riwayat Faktur Penjualan [F3]"
        >
          <span class="font-mono text-[9px] bg-slate-700 px-1 py-0.2 rounded font-bold">F3</span>
          <span class="material-symbols-outlined text-[15px]">receipt_long</span>
          <span>Faktur</span>
        </button>

        <button
          onclick={() => onNavigate?.("laporan")}
          class="px-3 py-1.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 hover:text-white font-semibold text-xs border border-slate-700 flex items-center gap-1.5 cursor-pointer transition-colors"
          title="Laporan Laba Rugi Komprehensif"
        >
          <span class="material-symbols-outlined text-[15px] text-amber-400">monitoring</span>
          <span>Laporan</span>
        </button>
      </div>
    </div>
  </div>

  <!-- ========================================================================= -->
  <!-- 2. KARTU UTAMA POSISI KEUANGAN (JUALAN, OPERASIONAL, LACI & NON-TUNAI)    -->
  <!-- ========================================================================= -->
  <div class="p-2.5 sm:p-5 space-y-3.5 sm:space-y-4 max-w-7xl mx-auto w-full">
    <!-- GRID 4 KARTU KEUANGAN TERPADU -->
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-2.5 sm:gap-3">
      <!-- 1. TOTAL JUALAN (OMSET PENJUALAN HARI INI) -->
      <div class="bg-white p-3 sm:p-4 rounded-2xl border border-slate-300 shadow-2xs relative overflow-hidden">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-1.5">
            <div class="w-7 h-7 rounded-lg bg-sky-100 text-sky-600 flex items-center justify-center">
              <span class="material-symbols-outlined text-[18px]">shopping_cart</span>
            </div>
            <span class="text-[11px] font-bold text-slate-600 uppercase tracking-wider">Total Jualan</span>
          </div>
          <span class="text-[9px] font-mono font-bold px-1.5 py-0.5 rounded bg-sky-50 text-sky-700 border border-sky-200">
            {penjualan.totalFaktur} Struk
          </span>
        </div>
        <div class="text-base sm:text-2xl font-black text-slate-900 font-mono mt-1.5">
          {formatRupiah(penjualan.total)}
        </div>
        <div class="text-[10px] text-slate-500 mt-1 flex items-center justify-between border-t border-slate-100 pt-1">
          <span>Tunai: <strong class="text-slate-800 font-mono">{formatRupiah(penjualan.tunai)}</strong></span>
          <span>Non-Tunai: <strong class="text-indigo-600 font-mono">{formatRupiah(penjualan.nonTunai)}</strong></span>
        </div>
      </div>

      <!-- 2. TOTAL OPERASIONAL (BEBAN PENGELUARAN TOKO) -->
      <div class="bg-white p-3 sm:p-4 rounded-2xl border border-slate-300 shadow-2xs relative overflow-hidden">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-1.5">
            <div class="w-7 h-7 rounded-lg bg-red-100 text-red-600 flex items-center justify-center">
              <span class="material-symbols-outlined text-[18px]">payments</span>
            </div>
            <span class="text-[11px] font-bold text-slate-600 uppercase tracking-wider">Total Operasional</span>
          </div>
          <span class="text-[9px] font-mono font-bold px-1.5 py-0.5 rounded bg-red-50 text-red-700 border border-red-200">
            {operasional.rincian.length} Beban
          </span>
        </div>
        <div class="text-base sm:text-2xl font-black text-red-600 font-mono mt-1.5">
          -{formatRupiah(operasional.totalBeban)}
        </div>
        <div class="text-[10px] text-slate-500 mt-1 flex items-center justify-between border-t border-slate-100 pt-1">
          <span>Listrik, Kertas, Galon, Bensin</span>
          <button
            onclick={() => onNavigate?.("operasional")}
            class="text-[10px] text-red-600 hover:text-red-800 font-bold border-none bg-transparent cursor-pointer p-0"
          >
            Rincian &rarr;
          </button>
        </div>
      </div>

      <!-- 3. SALDO KAS DI LACI (UANG FISIK CASH ON HAND KASIR) -->
      <div class="bg-white p-3 sm:p-4 rounded-2xl border-2 border-emerald-500/60 shadow-xs relative overflow-hidden bg-gradient-to-b from-white to-emerald-50/30">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-1.5">
            <div class="w-7 h-7 rounded-lg bg-emerald-600 text-white flex items-center justify-center shadow-2xs">
              <span class="material-symbols-outlined text-[18px]">point_of_sale</span>
            </div>
            <span class="text-[11px] font-black text-emerald-950 uppercase tracking-wider">Saldo di Laci Kasir</span>
          </div>
          <span class="text-[9px] font-mono font-bold px-1.5 py-0.5 rounded bg-emerald-100 text-emerald-800 border border-emerald-300">
            FISIK LACI
          </span>
        </div>
        <div class="text-base sm:text-2xl font-black text-emerald-700 font-mono mt-1.5">
          {formatRupiah(saldoDiLaci)}
        </div>
        <div class="text-[10px] text-slate-600 mt-1 flex items-center justify-between border-t border-emerald-200/60 pt-1">
          <span title="Modal Awal (500k) + Tunai (2.450k) + Piutang (150k) - Beban (385k) - Beli (180k)">
            Modal: <strong>500rb</strong> • Masuk: <strong>2.6jt</strong> • Keluar: <strong>565rb</strong>
          </span>
        </div>
      </div>

      <!-- 4. SALDO NON-TUNAI (QRIS & EDC BANK) -->
      <div class="bg-white p-3 sm:p-4 rounded-2xl border border-slate-300 shadow-2xs relative overflow-hidden">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-1.5">
            <div class="w-7 h-7 rounded-lg bg-indigo-100 text-indigo-600 flex items-center justify-center">
              <span class="material-symbols-outlined text-[18px]">contactless</span>
            </div>
            <span class="text-[11px] font-bold text-slate-600 uppercase tracking-wider">Saldo Non-Tunai</span>
          </div>
          <span class="text-[9px] font-mono font-bold px-1.5 py-0.5 rounded bg-indigo-50 text-indigo-700 border border-indigo-200">
            REKENING
          </span>
        </div>
        <div class="text-base sm:text-2xl font-black text-indigo-600 font-mono mt-1.5">
          {formatRupiah(saldoNonTunai)}
        </div>
        <div class="text-[10px] text-slate-500 mt-1 flex items-center justify-between border-t border-slate-100 pt-1">
          <span>QRIS: <strong class="text-slate-800 font-mono">{formatRupiah(penjualan.qris)}</strong></span>
          <span>EDC: <strong class="text-slate-800 font-mono">{formatRupiah(penjualan.edc)}</strong></span>
        </div>
      </div>
    </div>

    <!-- ======================================================================= -->
    <!-- 3. PANEL REKONSILIASI KEUANGAN TERPADU (MEMUDAHKAN HITUNG REAL-TIME)    -->
    <!-- ======================================================================= -->
    <div class="bg-white rounded-2xl border border-slate-300 p-3.5 sm:p-5 shadow-xs space-y-3">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-1 border-b border-slate-200 pb-2.5">
        <div>
          <div class="flex items-center gap-2">
            <h2 class="text-xs sm:text-sm font-black text-slate-900 uppercase tracking-tight">
              Pusat Rekonsiliasi &amp; Posisi Kas Toko Hari Ini
            </h2>
            <span class="text-[9px] font-mono font-bold px-1.5 py-0.5 rounded bg-emerald-100 text-emerald-800 border border-emerald-300">
              SINKRON OTOMATIS
            </span>
          </div>
          <div class="text-[11px] text-slate-500">
            Arus uang masuk dan keluar saling terhubung otomatis dari Kasir, Operasional, Pembelian &amp; Hutang-Piutang
          </div>
        </div>

        <div class="flex items-center gap-2 text-xs font-mono">
          <span class="text-slate-500">Total Kas Toko Hari Ini:</span>
          <span class="font-black text-slate-900 bg-slate-100 px-2.5 py-1 rounded-lg border border-slate-300 text-sm">
            {formatRupiah(totalKasTokoBersih)}
          </span>
        </div>
      </div>

      <!-- GRID DUA KOLOM: ARUS MASUK VS ARUS KELUAR -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3 sm:gap-4 text-xs">
        <!-- KOLOM KIRI: ARUS UANG MASUK (PENERIMAAN) -->
        <div class="bg-slate-50/80 rounded-xl p-3 border border-slate-200 space-y-2">
          <div class="flex items-center justify-between font-bold text-slate-900 pb-1.5 border-b border-slate-200">
            <span class="flex items-center gap-1.5 text-emerald-700">
              <span class="material-symbols-outlined text-[16px]">arrow_circle_down</span>
              <span>1. Arus Penerimaan (Uang Masuk)</span>
            </span>
            <span class="font-mono text-emerald-700">+{formatRupiah(penjualan.total + operasional.modalAwalKasir + hutangPiutang.piutangTerbayar)}</span>
          </div>

          <div class="space-y-1.5 font-mono text-[11px]">
            <div class="flex items-center justify-between text-slate-700">
              <span class="font-sans">Penjualan Tunai di Kasir</span>
              <strong class="text-emerald-700">+{formatRupiah(penjualan.tunai)}</strong>
            </div>
            <div class="flex items-center justify-between text-slate-700">
              <span class="font-sans">Penjualan QRIS (Bank Toko)</span>
              <strong class="text-indigo-700">+{formatRupiah(penjualan.qris)}</strong>
            </div>
            <div class="flex items-center justify-between text-slate-700">
              <span class="font-sans">Penjualan EDC BCA/Mandiri (Bank)</span>
              <strong class="text-indigo-700">+{formatRupiah(penjualan.edc)}</strong>
            </div>
            <div class="flex items-center justify-between text-slate-700">
              <span class="font-sans">Modal Awal Kasir (Buka Shift 01)</span>
              <strong class="text-slate-900">+{formatRupiah(operasional.modalAwalKasir)}</strong>
            </div>
            <div class="flex items-center justify-between text-slate-700">
              <span class="font-sans">Pelunasan Piutang Member (#ORD-8812)</span>
              <strong class="text-emerald-700">+{formatRupiah(hutangPiutang.piutangTerbayar)}</strong>
            </div>
          </div>
        </div>

        <!-- KOLOM KANAN: ARUS UANG KELUAR (PENGELUARAN) -->
        <div class="bg-slate-50/80 rounded-xl p-3 border border-slate-200 space-y-2">
          <div class="flex items-center justify-between font-bold text-slate-900 pb-1.5 border-b border-slate-200">
            <span class="flex items-center gap-1.5 text-red-700">
              <span class="material-symbols-outlined text-[16px]">arrow_circle_up</span>
              <span>2. Arus Pengeluaran (Uang Keluar)</span>
            </span>
            <span class="font-mono text-red-700">-{formatRupiah(operasional.totalBeban + hutangPiutang.pembelianTunai)}</span>
          </div>

          <div class="space-y-1.5 font-mono text-[11px]">
            <div class="flex items-center justify-between text-slate-700">
              <span class="font-sans">Beban Operasional Toko (4 Item Biaya)</span>
              <strong class="text-red-600">-{formatRupiah(operasional.totalBeban)}</strong>
            </div>
            <div class="flex items-center justify-between text-slate-700">
              <span class="font-sans">Pembelian Stok Supplier Tunai</span>
              <strong class="text-red-600">-{formatRupiah(hutangPiutang.pembelianTunai)}</strong>
            </div>
            <div class="flex items-center justify-between text-slate-700">
              <span class="font-sans">Pembayaran Hutang Supplier</span>
              <strong class="text-slate-500">Rp 0 (Belum Ada)</strong>
            </div>
            <div class="flex items-center justify-between text-slate-700">
              <span class="font-sans">Retur Penjualan Kembalian Tunai</span>
              <strong class="text-slate-500">Rp 0</strong>
            </div>
            <div class="flex items-center justify-between text-slate-700">
              <span class="font-sans">Pengeluaran Lainnya</span>
              <strong class="text-slate-500">Rp 0</strong>
            </div>
          </div>
        </div>
      </div>

      <!-- KOTAK RANGKUMAN HASIL AKHIR (SANGAT MUDAH DIHITUNG) -->
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-2.5 pt-1">
        <!-- Kotak 1: Saldo Laci Kasir -->
        <div class="p-3 rounded-xl bg-emerald-900 text-white border border-emerald-700 flex flex-col justify-between">
          <div>
            <div class="text-[10px] font-mono text-emerald-300 uppercase font-bold">Uang Fisik Kasir (Laci)</div>
            <div class="text-lg font-black font-mono mt-0.5">{formatRupiah(saldoDiLaci)}</div>
          </div>
          <div class="text-[9px] text-emerald-200 mt-1 font-sans">
            Wajib cocok dengan hitungan fisik lembaran uang di meja kasir saat tutup shift.
          </div>
        </div>

        <!-- Kotak 2: Saldo Non-Tunai Bank -->
        <div class="p-3 rounded-xl bg-slate-900 text-white border border-slate-700 flex flex-col justify-between">
          <div>
            <div class="text-[10px] font-mono text-indigo-300 uppercase font-bold">Saldo Digital / Rekening (QRIS &amp; EDC)</div>
            <div class="text-lg font-black font-mono text-indigo-300 mt-0.5">{formatRupiah(saldoNonTunai)}</div>
          </div>
          <div class="text-[9px] text-slate-300 mt-1 font-sans">
            Otomatis masuk ke settlement rekening bank toko tanpa risiko uang palsu.
          </div>
        </div>

        <!-- Kotak 3: Laba Bersih Usaha -->
        <div class="p-3 rounded-xl bg-amber-950 text-white border border-amber-700 flex flex-col justify-between">
          <div>
            <div class="text-[10px] font-mono text-amber-300 uppercase font-bold">Estimasi Laba Bersih Hari Ini</div>
            <div class="text-lg font-black font-mono text-amber-400 mt-0.5">{formatRupiah(labaBersih)}</div>
          </div>
          <div class="text-[9px] text-amber-200 mt-1 font-sans">
            Laba Kotor ({formatRupiah(labaKotor)}) dikurangi Beban Operasional ({formatRupiah(operasional.totalBeban)}).
          </div>
        </div>
      </div>
    </div>

    <!-- ======================================================================= -->
    <!-- 4. AKTIVITAS TRANSAKSI & BEBAN OPERASIONAL HARI INI                     -->
    <!-- ======================================================================= -->
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-3 sm:gap-4">
      <!-- Kolom Kiri: 5 Transaksi Kasir Terakhir (7 Cols) -->
      <div class="lg:col-span-7 bg-white rounded-2xl border border-slate-300 p-3 sm:p-4 shadow-2xs">
        <div class="flex items-center justify-between mb-2.5">
          <div class="flex items-center gap-2">
            <span class="material-symbols-outlined text-[18px] text-sky-600">receipt_long</span>
            <h2 class="text-xs sm:text-sm font-bold text-slate-900 uppercase tracking-tight">
              Faktur Penjualan Terkini
            </h2>
          </div>
          <button
            onclick={() => onNavigate?.("penjualan")}
            class="text-[11px] text-sky-600 hover:text-sky-800 font-bold flex items-center gap-0.5 cursor-pointer border-none bg-transparent"
          >
            <span>Semua Faktur</span>
            <span class="material-symbols-outlined text-[14px]">arrow_forward</span>
          </button>
        </div>

        <div class="divide-y divide-slate-100 text-xs">
          {#each transaksiTerbaru as tx}
            <div class="py-2 flex items-center justify-between gap-2">
              <div class="min-w-0">
                <div class="flex items-center gap-1.5">
                  <span class="font-bold text-slate-900 font-mono">{tx.no}</span>
                  <span class="text-[10px] text-slate-400 font-mono">• {tx.jam}</span>
                  <span class="text-[10px] text-slate-500">Opr: {tx.kasir}</span>
                </div>
                <div class="text-[11px] text-slate-500">
                  Pelanggan: <strong class="text-slate-700">{tx.member}</strong>
                </div>
              </div>
              <div class="text-right shrink-0">
                <div class="font-black text-slate-900 font-mono">
                  {formatRupiah(tx.total)}
                </div>
                <span class="text-[9px] font-mono px-1.5 py-0.2 rounded font-bold {tx.metode === 'TUNAI' ? 'bg-emerald-100 text-emerald-800 border border-emerald-300' : 'bg-indigo-100 text-indigo-800 border border-indigo-300'}">
                  {tx.metode}
                </span>
              </div>
            </div>
          {/each}
        </div>
      </div>

      <!-- Kolom Kanan: Rincian 4 Biaya Operasional Hari Ini (5 Cols) -->
      <div class="lg:col-span-5 bg-white rounded-2xl border border-slate-300 p-3 sm:p-4 shadow-2xs space-y-2.5">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="material-symbols-outlined text-[18px] text-red-600">payments</span>
            <h2 class="text-xs sm:text-sm font-bold text-slate-900 uppercase tracking-tight">
              Beban Operasional Hari Ini
            </h2>
          </div>
          <button
            onclick={() => onNavigate?.("operasional")}
            class="text-[11px] text-red-600 hover:text-red-800 font-bold flex items-center gap-0.5 cursor-pointer border-none bg-transparent"
          >
            <span>Buku Kas</span>
            <span class="material-symbols-outlined text-[14px]">arrow_forward</span>
          </button>
        </div>

        <div class="space-y-2 text-xs">
          {#each operasional.rincian as b}
            <div class="p-2 rounded-xl bg-red-50/60 border border-red-200/80 flex items-center justify-between">
              <div class="min-w-0 pr-2">
                <div class="font-bold text-slate-900 truncate">{b.nama}</div>
                <div class="text-[10px] text-slate-400 font-mono">Jam: {b.jam} • Keluar dari Kasir</div>
              </div>
              <div class="font-mono font-bold text-red-600 shrink-0">
                -{formatRupiah(b.nominal)}
              </div>
            </div>
          {/each}
        </div>

        <button
          onclick={() => onNavigate?.("operasional")}
          class="w-full py-2 rounded-xl bg-slate-100 hover:bg-slate-200 text-slate-700 text-xs font-bold flex items-center justify-center gap-1.5 cursor-pointer transition-colors border border-slate-300"
        >
          <span class="material-symbols-outlined text-[16px] text-red-600">add_circle</span>
          <span>+ Catat Biaya Pengeluaran Toko</span>
        </button>
      </div>
    </div>
  </div>
</div>
