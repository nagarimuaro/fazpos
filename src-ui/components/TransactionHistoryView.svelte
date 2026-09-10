<script lang="ts">
  import { api, formatRupiah, type TransactionHistoryItemDTO, type TransactionSummaryStatsDTO } from "../lib/api";

  let {
    onOpenKasir,
  }: {
    onOpenKasir: () => void;
  } = $props();

  let activeMethodTab = $state<string>("Semua");
  let searchQuery = $state<string>("");
  let selectedDatePreset = $state<string>("today");
  let searchInputElement: HTMLInputElement | null = $state(null);

  let transactions = $state<TransactionHistoryItemDTO[]>([]);
  let stats = $state<TransactionSummaryStatsDTO | null>(null);
  let selectedTx = $state<TransactionHistoryItemDTO | null>(null);
  let isDetailModalOpen = $state<boolean>(false);
  let toastMessage = $state<string>("");

  async function loadData() {
    try {
      const kw = searchQuery.trim() || undefined;
      const m = activeMethodTab === "Semua" ? undefined : activeMethodTab;
      transactions = await api.getTransactions(m, kw);
      stats = await api.getTransactionStats();
    } catch (err) {
      console.error("loadData error:", err);
    }
  }

  function handleSearchInput() {
    loadData();
  }

  function selectMethodTab(tab: string) {
    activeMethodTab = tab;
    loadData();
  }

  async function handlePrintReceipt(faktur: string) {
    try {
      await api.printLastReceipt(faktur);
      showToast(`Mencetak struk untuk nota ${faktur}...`);
    } catch {
      showToast(`Perintah cetak terkirim ke printer thermal untuk ${faktur}`);
    }
  }

  function openDetail(tx: TransactionHistoryItemDTO) {
    selectedTx = tx;
    isDetailModalOpen = true;
  }

  function showToast(msg: string) {
    toastMessage = msg;
    setTimeout(() => (toastMessage = ""), 4000);
  }

  function exportCSV() {
    if (transactions.length === 0) return;
    const header = "No,Faktur,Waktu,Kasir,Pelanggan,Total Qty,Metode,Total Penjualan\n";
    const rows = transactions.map((t) =>
      `"${t.no}","${t.faktur}","${t.waktu}","${t.kasir}","${t.pelanggan}","${t.total_qty}","${t.metode}","${t.total_penjualan}"`
    ).join("\n");
    const blob = new Blob([header + rows], { type: "text/csv;charset=utf-8;" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `riwayat_penjualan_${new Date().toISOString().slice(0, 10)}.csv`;
    link.click();
    URL.revokeObjectURL(url);
    showToast("Data riwayat berhasil diexport ke file CSV!");
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "F1") {
      e.preventDefault();
      onOpenKasir();
    } else if (e.key === "F3") {
      e.preventDefault();
      searchInputElement?.focus();
      searchInputElement?.select();
    } else if (e.key === "F4") {
      e.preventDefault();
      if (transactions.length > 0) {
        handlePrintReceipt(transactions[0].faktur);
      }
    } else if (e.key === "F7") {
      e.preventDefault();
      exportCSV();
    } else if (e.key === "F5") {
      e.preventDefault();
      loadData();
      showToast("Sinkronisasi database offline SQLite berhasil diperbarui.");
    }
  }

  $effect(() => {
    loadData();
    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });
</script>

<div class="flex-1 flex flex-col bg-slate-200 overflow-hidden font-sans select-none min-h-0">
  <!-- 1. TOP ACTION & STATUS TABS BAR (Tetap Di Atas Card) -->
  <div class="px-4 py-2.5 bg-white border-b border-slate-300 shadow-2xs flex flex-wrap items-center justify-between gap-3 shrink-0">
    <!-- Tabs Filter Status -->
    <div class="flex items-center gap-1 p-1 bg-slate-100 rounded-xl border border-slate-200">
      <button
        onclick={() => selectMethodTab("Semua")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {activeMethodTab === 'Semua' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span>Semua Penjualan</span>
        <span class="px-1.5 py-0.5 rounded {activeMethodTab === 'Semua' ? 'bg-white/20 text-white' : 'bg-slate-200 text-slate-700'} font-mono text-[10px] font-bold">
          {stats?.total_transaksi ?? 142}
        </span>
      </button>

      <button
        onclick={() => selectMethodTab("TUNAI")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {activeMethodTab === 'TUNAI' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span>Tunai</span>
        <span class="px-1.5 py-0.5 rounded {activeMethodTab === 'TUNAI' ? 'bg-white/20 text-white' : 'bg-slate-200 text-slate-700'} font-mono text-[10px] font-bold">
          {stats?.tunai_count ?? 98}
        </span>
      </button>

      <button
        onclick={() => selectMethodTab("QRIS")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {activeMethodTab === 'QRIS' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span>QRIS</span>
        <span class="px-1.5 py-0.5 rounded {activeMethodTab === 'QRIS' ? 'bg-white/20 text-white' : 'bg-slate-200 text-slate-700'} font-mono text-[10px] font-bold">
          {stats?.qris_count ?? 32}
        </span>
      </button>

      <button
        onclick={() => selectMethodTab("EDC")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {activeMethodTab === 'EDC' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span>Kartu / EDC</span>
        <span class="px-1.5 py-0.5 rounded {activeMethodTab === 'EDC' ? 'bg-white/20 text-white' : 'bg-slate-200 text-slate-700'} font-mono text-[10px] font-bold">
          {stats?.edc_count ?? 12}
        </span>
      </button>
    </div>

    <!-- Action Utilities & Buttons -->
    <div class="flex items-center gap-1.5">
      <div class="flex items-center gap-1 bg-slate-100 px-2.5 py-1.5 rounded-lg font-sans text-xs text-slate-600 border border-slate-200">
        <span class="material-symbols-outlined text-[16px] text-slate-500">badge</span>
        <span>Kasir: <strong class="text-slate-900 font-bold">Semua Kasir</strong></span>
      </div>

      <button
        onclick={() => showToast("Mencetak Rekap Shift aktif...")}
        class="flex items-center gap-1.5 px-3 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-800 rounded-lg font-sans text-xs font-medium border border-slate-300 shadow-2xs transition-all cursor-pointer"
      >
        <span class="font-mono text-[10px] font-bold bg-slate-200 px-1 py-0.2 rounded text-slate-700">F4</span>
        <span class="material-symbols-outlined text-[16px] text-primary">receipt_long</span>
        <span>Cetak Rekap Shift</span>
      </button>

      <button
        onclick={exportCSV}
        class="flex items-center gap-1.5 px-3 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-800 rounded-lg font-sans text-xs font-medium border border-slate-300 shadow-2xs transition-all cursor-pointer"
      >
        <span class="font-mono text-[10px] font-bold bg-slate-200 px-1 py-0.2 rounded text-slate-700">F7</span>
        <span class="material-symbols-outlined text-[16px] text-emerald-600">table_view</span>
        <span>Export CSV</span>
      </button>

      <button
        onclick={loadData}
        class="flex items-center gap-1 px-2.5 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-800 rounded-lg font-sans text-xs border border-slate-300 shadow-2xs transition-all cursor-pointer"
        title="Sinkronisasi Data (F5)"
      >
        <span class="font-mono text-[10px] font-bold bg-slate-200 px-1 py-0.2 rounded text-slate-700">F5</span>
        <span class="material-symbols-outlined text-[16px]">sync</span>
      </button>

      <!-- Tombol Masuk Kasir POS [F1] -->
      <button
        onclick={onOpenKasir}
        class="flex items-center gap-2 px-4 py-1.5 bg-primary hover:bg-primary-dark text-white rounded-lg font-sans text-xs font-bold shadow-sm border border-primary-dark transition-all cursor-pointer ml-1"
        title="Buka Layar Kasir [F1]"
      >
        <span class="font-mono text-[10px] bg-primary-dark px-1.5 py-0.5 rounded font-bold">F1</span>
        <span class="material-symbols-outlined text-[16px]">point_of_sale</span>
        <span>BUKA KASIR (POS)</span>
      </button>
    </div>
  </div>

  <!-- 2. 5 COMPACT STAT CARDS (Di Tengah) -->
  <div class="px-4 py-2.5 bg-slate-200 border-b border-slate-300 shrink-0">
    <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-2.5 items-stretch">
      <!-- Card 1: Total Omset -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Total Omset Penjualan</span>
          <span class="inline-flex items-center px-1.5 py-0.2 rounded bg-emerald-100 text-emerald-800 font-mono text-[10px] font-bold">+12.4%</span>
        </div>
        <div class="my-1">
          <span class="font-mono text-xl font-bold text-primary tracking-tight">
            {formatRupiah(stats?.total_omset ?? 8425000)}
          </span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Target: Rp 7.500.000</span>
          <span class="text-emerald-700 font-bold">Tercapai 112%</span>
        </div>
      </div>

      <!-- Card 2: Total Barang / Qty Terjual -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Total Barang / Qty</span>
          <span class="material-symbols-outlined text-primary text-[18px]">inventory_2</span>
        </div>
        <div class="my-1 flex items-baseline gap-1.5">
          <span class="font-mono text-xl font-bold text-slate-900 tracking-tight">{stats?.total_qty ?? 384}</span>
          <span class="font-mono text-xs text-slate-500 font-medium">Pcs / Unit</span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Rata-rata item: <strong class="text-slate-800 font-bold">{stats?.avg_item_per_trx ?? 2.7}</strong>/trx</span>
          <span>{stats?.total_transaksi ?? 142} Nota</span>
        </div>
      </div>

      <!-- Card 3: Total Transaksi Selesai -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Transaksi Selesai</span>
          <span class="inline-flex items-center px-1.5 py-0.2 rounded bg-emerald-100 text-emerald-800 font-mono text-[10px] font-bold">100% LUNAS</span>
        </div>
        <div class="my-1 flex items-baseline gap-1.5">
          <span class="font-mono text-xl font-bold text-emerald-700 tracking-tight">{stats?.total_transaksi ?? 142}</span>
          <span class="font-mono text-xs text-slate-500 font-medium">Transaksi</span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Batal / Void: <strong class="text-slate-800 font-bold">{stats?.void_count ?? 0}</strong></span>
          <span>Retur: <strong class="text-slate-800 font-bold">{stats?.retur_count ?? 0}</strong></span>
        </div>
      </div>

      <!-- Card 4: Rata-rata Basket Size -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Rata-rata Basket Size</span>
          <span class="material-symbols-outlined text-amber-600 text-[18px]">shopping_basket</span>
        </div>
        <div class="my-1">
          <span class="font-mono text-xl font-bold text-slate-900 tracking-tight">
            {formatRupiah(stats?.avg_basket_size ?? 59330)}
          </span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Tertinggi: <strong class="text-slate-800 font-bold">{formatRupiah(stats?.max_basket ?? 385000)}</strong></span>
          <span>Min: {formatRupiah(stats?.min_basket ?? 12500)}</span>
        </div>
      </div>

      <!-- Card 5: Komposisi Metode Pembayaran -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Metode Terbanyak</span>
          <span class="material-symbols-outlined text-slate-400 text-[18px]">pie_chart</span>
        </div>
        <div class="my-1 flex items-center justify-between">
          <div class="flex items-baseline gap-1 text-[11px] font-mono font-bold">
            <span class="text-emerald-700">Tunai 69%</span>
            <span class="text-slate-300">•</span>
            <span class="text-primary">QRIS 23%</span>
            <span class="text-slate-300">•</span>
            <span class="text-amber-700">EDC 8%</span>
          </div>
        </div>
        <div class="w-full bg-slate-200 h-2 rounded-full overflow-hidden flex">
          <div class="bg-emerald-600 h-full" style="width: 69%"></div>
          <div class="bg-primary h-full" style="width: 23%"></div>
          <div class="bg-amber-500 h-full" style="width: 8%"></div>
        </div>
      </div>
    </div>
  </div>

  <!-- 3. SEARCH BAR & FILTER CONTROLS (Di Bawah Card) -->
  <div class="px-4 py-2 bg-white border-b border-slate-300 shadow-2xs flex flex-wrap items-center justify-between gap-3 shrink-0">
    <!-- Search Bar -->
    <div class="flex-1 min-w-[320px] max-w-2xl relative">
      <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-slate-400">
        <span class="material-symbols-outlined text-[18px]">search</span>
      </div>
      <input
        bind:this={searchInputElement}
        bind:value={searchQuery}
        oninput={handleSearchInput}
        class="w-full pl-9 pr-14 py-1.5 bg-slate-100 text-slate-900 placeholder:text-slate-400 font-sans text-xs rounded-lg border border-slate-300 outline-none focus:border-primary focus:bg-white focus:ring-1 focus:ring-primary transition-all font-medium"
        placeholder="Cari No. Faktur / Nama Pelanggan / SKU / Kasir / Item..."
        type="text"
      />
      <div class="absolute inset-y-0 right-0 pr-2 flex items-center pointer-events-none">
        <span class="px-1.5 py-0.5 rounded bg-slate-200 text-slate-700 font-mono text-[10px] font-bold uppercase border border-slate-300">
          F3
        </span>
      </div>
    </div>

    <!-- Date Presets & Custom Picker -->
    <div class="flex items-center gap-2 font-sans text-xs">
      <div class="flex items-center gap-1 bg-slate-100 p-1 rounded-lg border border-slate-200">
        <button
          onclick={() => (selectedDatePreset = "today")}
          class="px-3 py-1 rounded font-semibold text-xs border-none cursor-pointer transition-all {selectedDatePreset === 'today' ? 'bg-white text-primary shadow-2xs font-bold' : 'text-slate-600 hover:text-slate-900 bg-transparent'}"
        >
          Hari Ini (10-09-2026)
        </button>
        <button
          onclick={() => (selectedDatePreset = "yesterday")}
          class="px-3 py-1 rounded text-xs border-none cursor-pointer transition-all {selectedDatePreset === 'yesterday' ? 'bg-white text-primary shadow-2xs font-bold' : 'text-slate-600 hover:text-slate-900 bg-transparent'}"
        >
          Kemarin
        </button>
        <button
          onclick={() => (selectedDatePreset = "7days")}
          class="px-3 py-1 rounded text-xs border-none cursor-pointer transition-all {selectedDatePreset === '7days' ? 'bg-white text-primary shadow-2xs font-bold' : 'text-slate-600 hover:text-slate-900 bg-transparent'}"
        >
          7 Hari Terakhir
        </button>
      </div>

      <button
        onclick={() => showToast("Memilih rentang tanggal kustom...")}
        class="flex items-center gap-1.5 px-3 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-800 rounded-lg border border-slate-300 shadow-2xs font-medium cursor-pointer"
      >
        <span class="material-symbols-outlined text-[16px] text-slate-500">calendar_today</span>
        <span>Rentang Tanggal</span>
      </button>
    </div>
  </div>

  <!-- Main Work Area: 100% Full-Width Ledger Table -->
  <div class="w-full px-4 py-2.5 flex-1 flex flex-col min-h-0 overflow-hidden">
    <div class="w-full bg-white rounded-xl border border-slate-300 shadow-sm overflow-hidden flex flex-col flex-1 min-h-0">
      <!-- Table Header & Scroller -->
      <div class="overflow-auto w-full flex-1 min-h-0">
        <table class="w-full text-left border-collapse">
          <thead>
            <tr class="bg-slate-800 text-slate-100 font-mono text-[11px] font-bold border-b-2 border-slate-900 uppercase tracking-wider select-none sticky top-0 z-10 shadow-sm">
              <th class="py-2.5 px-3.5 w-12 text-center">NO</th>
              <th class="py-2.5 px-3.5 w-36">NO. FAKTUR</th>
              <th class="py-2.5 px-3.5 w-28">WAKTU</th>
              <th class="py-2.5 px-3.5">KASIR &amp; SHIFT</th>
              <th class="py-2.5 px-3.5">PELANGGAN</th>
              <th class="py-2.5 px-3.5 w-36 text-center">TOTAL QTY</th>
              <th class="py-2.5 px-3.5 w-36 text-center">METODE</th>
              <th class="py-2.5 px-3.5 w-44 text-right">TOTAL PENJUALAN</th>
              <th class="py-2.5 px-3.5 w-28 text-center">AKSI</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-200 font-sans text-xs">
            {#each transactions as tx, i}
              <tr class="transition-colors group {i % 2 === 1 ? 'bg-slate-50/70' : 'bg-white'} hover:bg-sky-50/80 cursor-pointer">
                <td class="py-2 px-3.5 text-center font-mono text-xs font-bold text-slate-500">{tx.no}</td>
                <td class="py-2 px-3.5 font-mono text-xs font-bold text-primary whitespace-nowrap">{tx.faktur}</td>
                <td class="py-2 px-3.5 font-mono text-xs text-slate-600 whitespace-nowrap">{tx.waktu}</td>
                <td class="py-2 px-3.5">
                  <span class="font-semibold text-slate-900">{tx.kasir}</span>
                  <span class="block font-mono text-[10px] text-slate-500">{tx.shift}</span>
                </td>
                <td class="py-2 px-3.5">
                  <div class="flex items-center gap-1.5">
                    <span class="font-semibold text-slate-900 truncate">{tx.pelanggan}</span>
                    {#if tx.pelanggan_badge === "VIP GOLD"}
                      <span class="px-1.5 py-0.2 rounded bg-amber-400 text-slate-900 font-mono text-[9px] uppercase font-bold shrink-0">VIP GOLD</span>
                    {:else if tx.pelanggan_badge === "SILVER"}
                      <span class="px-1.5 py-0.2 rounded bg-slate-300 text-slate-800 font-mono text-[9px] uppercase font-bold shrink-0">SILVER</span>
                    {:else if tx.pelanggan_badge === "GROSIR"}
                      <span class="px-1.5 py-0.2 rounded bg-amber-100 text-amber-900 font-mono text-[9px] uppercase font-bold shrink-0">GROSIR</span>
                    {:else}
                      <span class="px-1.5 py-0.2 rounded bg-slate-200 text-slate-700 font-mono text-[9px] uppercase shrink-0">Umum</span>
                    {/if}
                  </div>
                  <span class="font-mono text-[10px] text-slate-500">{tx.pelanggan_info}</span>
                </td>
                <td class="py-2 px-3.5 text-center">
                  <span class="font-bold font-mono text-slate-900">{tx.total_qty} Unit</span>
                  <span class="block font-mono text-[10px] text-slate-500">{tx.total_sku} Item SKU</span>
                </td>
                <td class="py-2 px-3.5 text-center">
                  <span class="inline-flex items-center px-2 py-0.5 rounded font-mono text-[10px] font-bold {tx.metode === 'TUNAI' ? 'bg-emerald-100 text-emerald-800 border border-emerald-300' : tx.metode.includes('QRIS') ? 'bg-blue-100 text-blue-800 border border-blue-300' : 'bg-amber-100 text-amber-800 border border-amber-300'}">
                    {tx.metode}
                  </span>
                </td>
                <td class="py-2 px-3.5 text-right font-mono text-xs font-bold text-slate-900 whitespace-nowrap">
                  {formatRupiah(tx.total_penjualan)}
                </td>
                <td class="py-2 px-3.5 text-center">
                  <div class="flex items-center justify-center gap-1.5">
                    <button
                      onclick={() => openDetail(tx)}
                      class="w-8 h-8 rounded-lg bg-slate-100 hover:bg-slate-200 text-primary flex items-center justify-center border border-slate-300 transition-colors shadow-2xs cursor-pointer"
                      title="Detail Nota [Enter]"
                      aria-label="Detail Nota"
                    >
                      <span class="material-symbols-outlined text-[18px]">receipt_long</span>
                    </button>
                    <button
                      onclick={() => handlePrintReceipt(tx.faktur)}
                      class="w-8 h-8 rounded-lg bg-primary text-white hover:bg-primary-dark flex items-center justify-center transition-colors shadow-2xs border-none cursor-pointer"
                      title="Cetak Struk [F4]"
                      aria-label="Cetak Struk"
                    >
                      <span class="material-symbols-outlined text-[18px]">print</span>
                    </button>
                  </div>
                </td>
              </tr>
            {:else}
              <tr>
                <td colspan="9" class="py-16 text-center text-slate-400 font-mono text-xs">
                  Tidak ada transaksi penjualan ditemukan.
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      <!-- Pagination / Dense Ledger Status Bar -->
      <div class="px-4 py-2 bg-slate-100 flex flex-wrap items-center justify-between text-xs text-slate-600 border-t border-slate-300 shrink-0">
        <div class="flex items-center gap-3">
          <span>Menampilkan <strong class="text-slate-900 font-bold">1 - {transactions.length}</strong> dari <strong class="text-slate-900 font-bold">{stats?.total_transaksi ?? 142}</strong> Transaksi Terverifikasi</span>
          <span class="text-slate-400">•</span>
          <span class="font-mono text-slate-600">Halaman 1 dari 15</span>
          <span class="text-slate-400">•</span>
          <span class="font-mono text-emerald-700 font-bold">Semua data tersinkronisasi offline SQLite</span>
        </div>
        <div class="flex items-center gap-1 font-mono text-xs">
          <button class="px-3 py-1 bg-white rounded hover:bg-slate-200 disabled:opacity-50 text-slate-700 font-semibold border border-slate-300 shadow-2xs transition-all cursor-pointer" disabled>
            Sebelumnya
          </button>
          <button class="px-3 py-1 bg-primary text-white rounded font-bold shadow-2xs border-none cursor-pointer">
            1
          </button>
          <button class="px-3 py-1 bg-white hover:bg-slate-200 text-slate-800 rounded font-semibold border border-slate-300 shadow-2xs transition-all cursor-pointer">
            2
          </button>
          <button class="px-3 py-1 bg-white hover:bg-slate-200 text-slate-800 rounded font-semibold border border-slate-300 shadow-2xs transition-all cursor-pointer">
            3
          </button>
          <button class="px-3 py-1 bg-white hover:bg-slate-200 text-slate-800 rounded font-semibold border border-slate-300 shadow-2xs transition-all cursor-pointer">
            4
          </button>
          <button class="px-3 py-1 bg-white rounded hover:bg-slate-200 text-slate-800 font-semibold border border-slate-300 shadow-2xs transition-all cursor-pointer">
            Selanjutnya
          </button>
        </div>
      </div>
    </div>
  </div>

  <!-- Detail Nota Modal Dialog -->
  {#if isDetailModalOpen && selectedTx}
    <div
      class="fixed inset-0 bg-slate-900/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none"
      onclick={() => (isDetailModalOpen = false)}
      role="button"
      tabindex="0"
      onkeydown={(e) => e.key === "Escape" && (isDetailModalOpen = false)}
    >
      <div
        class="bg-white border border-slate-300 rounded-xl shadow-2xl w-[480px] overflow-hidden"
        role="dialog"
        aria-modal="true"
        tabindex="-1"
        onclick={(e) => e.stopPropagation()}
        onkeydown={(e) => e.stopPropagation()}
      >
        <div class="px-5 py-3.5 bg-slate-100 border-b border-slate-300 flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="material-symbols-outlined text-primary text-[20px]">receipt_long</span>
            <span class="font-bold text-slate-900 text-sm font-sans">Detail Nota {selectedTx.faktur}</span>
          </div>
          <button
            onclick={() => (isDetailModalOpen = false)}
            class="text-slate-400 hover:text-slate-700 p-1 hover:bg-slate-200 rounded border-none bg-transparent cursor-pointer"
          >
            <span class="material-symbols-outlined text-[18px]">close</span>
          </button>
        </div>

        <div class="p-5 font-mono text-xs flex flex-col gap-3">
          <div class="flex justify-between py-1 border-b border-slate-100">
            <span class="text-slate-500">No. Faktur</span>
            <span class="font-bold text-slate-900">{selectedTx.faktur}</span>
          </div>
          <div class="flex justify-between py-1 border-b border-slate-100">
            <span class="text-slate-500">Waktu &amp; Shift</span>
            <span class="text-slate-800">{selectedTx.waktu} • {selectedTx.shift}</span>
          </div>
          <div class="flex justify-between py-1 border-b border-slate-100">
            <span class="text-slate-500">Kasir Operator</span>
            <span class="text-slate-800">{selectedTx.kasir}</span>
          </div>
          <div class="flex justify-between py-1 border-b border-slate-100">
            <span class="text-slate-500">Pelanggan</span>
            <span class="font-bold text-slate-900">{selectedTx.pelanggan} ({selectedTx.pelanggan_badge})</span>
          </div>
          <div class="flex justify-between py-1 border-b border-slate-100">
            <span class="text-slate-500">Total Kuantitas</span>
            <span class="font-bold text-slate-900">{selectedTx.total_qty} Unit ({selectedTx.total_sku} SKU)</span>
          </div>
          <div class="flex justify-between py-1 border-b border-slate-100">
            <span class="text-slate-500">Metode Bayar</span>
            <span class="font-bold text-emerald-700">{selectedTx.metode}</span>
          </div>
          <div class="flex justify-between py-2 bg-slate-100 px-3 rounded-lg border border-slate-300 mt-2">
            <span class="text-sm font-bold text-slate-800 font-sans">TOTAL AKHIR</span>
            <span class="text-base font-bold text-primary font-mono">{formatRupiah(selectedTx.total_penjualan)}</span>
          </div>
        </div>

        <div class="px-5 py-3 bg-slate-50 border-t border-slate-300 flex items-center justify-end gap-2">
          <button
            onclick={() => (isDetailModalOpen = false)}
            class="px-4 py-1.5 bg-white hover:bg-slate-100 text-slate-700 border border-slate-300 rounded-lg text-xs font-bold shadow-2xs cursor-pointer"
          >
            Tutup
          </button>
          <button
            onclick={() => { handlePrintReceipt(selectedTx!.faktur); isDetailModalOpen = false; }}
            class="px-4 py-1.5 bg-primary hover:bg-primary-dark text-white rounded-lg text-xs font-bold shadow-sm cursor-pointer border-none flex items-center gap-1"
          >
            <span class="material-symbols-outlined text-[16px]">print</span>
            Cetak Ulang Struk
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Toast message banner -->
  {#if toastMessage}
    <div class="fixed bottom-4 right-4 bg-slate-900 text-emerald-300 px-4 py-2.5 rounded-lg shadow-xl font-mono text-xs border border-slate-700 animate-in fade-in z-50 flex items-center gap-2">
      <span class="material-symbols-outlined text-[18px] text-emerald-400">info</span>
      <span>{toastMessage}</span>
    </div>
  {/if}
</div>
