<script lang="ts">
  import { formatRupiah, type PurchaseHistoryItemDTO, type PurchaseStatsDTO } from "../lib/api";

  let purchases = $state<PurchaseHistoryItemDTO[]>([
    {
      id: "po-01",
      no: 1,
      faktur: "#PO-2026-0910",
      faktur_supplier: "INV/SM/8821",
      tanggal: "10/09/2026 14:20",
      suplier: "PT Sumber Makmur",
      total_item: 8,
      total_qty: 240,
      total_beli: 4250000,
      metode: "TUNAI",
      jatuh_tempo: "-",
      status: "LUNAS",
      penerima: "Gudang Utama (Hendra)",
    },
    {
      id: "po-02",
      no: 2,
      faktur: "#PO-2026-0909",
      faktur_supplier: "FP-KM-1042",
      tanggal: "09/09/2026 10:15",
      suplier: "PT Kopi Mandiri",
      total_item: 4,
      total_qty: 120,
      total_beli: 3840000,
      metode: "KREDIT / TEMPO",
      jatuh_tempo: "23/09/2026",
      status: "TEMPO",
      penerima: "Gudang Chiller (Roni)",
    },
    {
      id: "po-03",
      no: 3,
      faktur: "#PO-2026-0908",
      faktur_supplier: "INV-PB-993",
      tanggal: "08/09/2026 08:30",
      suplier: "Prima Bakery",
      total_item: 6,
      total_qty: 180,
      total_beli: 2750000,
      metode: "TUNAI",
      jatuh_tempo: "-",
      status: "LUNAS",
      penerima: "Pantry Kasir (Siti)",
    },
    {
      id: "po-04",
      no: 4,
      faktur: "#PO-2026-0906",
      faktur_supplier: "DT/SJ/7721",
      tanggal: "06/09/2026 16:45",
      suplier: "Danone Tirta",
      total_item: 5,
      total_qty: 350,
      total_beli: 2360000,
      metode: "KREDIT / TEMPO",
      jatuh_tempo: "20/09/2026",
      status: "TEMPO",
      penerima: "Gudang Belakang (Hendra)",
    },
    {
      id: "po-05",
      no: 5,
      faktur: "#PO-2026-0904",
      faktur_supplier: "IND-094-JKT",
      tanggal: "04/09/2026 11:20",
      suplier: "Indofood Sukses Makmur",
      total_item: 12,
      total_qty: 480,
      total_beli: 6850000,
      metode: "TUNAI",
      jatuh_tempo: "-",
      status: "LUNAS",
      penerima: "Gudang Utama (Roni)",
    },
    {
      id: "po-06",
      no: 6,
      faktur: "#PO-2026-0901",
      faktur_supplier: "MAY-998-BDG",
      tanggal: "01/09/2026 09:10",
      suplier: "Mayora Retail Group",
      total_item: 7,
      total_qty: 210,
      total_beli: 3450000,
      metode: "TUNAI",
      jatuh_tempo: "-",
      status: "LUNAS",
      penerima: "Gudang Utama (Hendra)",
    },
    {
      id: "po-07",
      no: 7,
      faktur: "#PO-2026-0828",
      faktur_supplier: "UNL-REC-441",
      tanggal: "28/08/2026 15:40",
      suplier: "Unilever Distribusi",
      total_item: 9,
      total_qty: 260,
      total_beli: 4950000,
      metode: "KREDIT / TEMPO",
      jatuh_tempo: "11/09/2026",
      status: "TEMPO",
      penerima: "Gudang Utama (Hendra)",
    },
  ]);

  let stats = $state<PurchaseStatsDTO>({
    total_belanja_bulan_ini: 28450000,
    total_qty_masuk: 1840,
    total_faktur: 24,
    total_hutang_tempo: 6200000,
    supplier_teraktif: "PT Sumber Makmur",
  });

  let searchQuery = $state("");
  let supplierFilter = $state("all");
  let statusFilter = $state("all");
  let periodeFilter = $state("bulan_ini");
  let searchInputElement: HTMLInputElement | null = $state(null);
  let selectedDetailPurchase = $state<PurchaseHistoryItemDTO | null>(null);
  let isAddModalOpen = $state(false);
  let toastMessage = $state("");

  // Form states faktur baru
  let formSupplier = $state("PT Sumber Makmur");
  let formFakturSupplier = $state("");
  let formMetode = $state("TUNAI");
  let formTempoHari = $state(14);
  let formTotalBeli = $state(2500000);
  let formTotalQty = $state(120);

  let suppliersList = $derived([
    "all",
    ...Array.from(new Set(purchases.map((p) => p.suplier).filter(Boolean))),
  ]);

  let filteredPurchases = $derived(
    purchases.filter((p) => {
      if (supplierFilter !== "all" && p.suplier.toLowerCase() !== supplierFilter.toLowerCase()) {
        return false;
      }
      if (statusFilter === "tunai" && p.status !== "LUNAS") return false;
      if (statusFilter === "tempo" && p.status !== "TEMPO") return false;

      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase().trim();
        return (
          p.faktur.toLowerCase().includes(q) ||
          p.faktur_supplier.toLowerCase().includes(q) ||
          p.suplier.toLowerCase().includes(q) ||
          p.penerima.toLowerCase().includes(q)
        );
      }
      return true;
    })
  );

  function showToast(msg: string) {
    toastMessage = msg;
    setTimeout(() => (toastMessage = ""), 4000);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "F3") {
      e.preventDefault();
      searchInputElement?.focus();
      searchInputElement?.select();
    } else if (e.key === "F6") {
      e.preventDefault();
      isAddModalOpen = true;
    }
  }

  function simpanFakturPembelian() {
    const today = new Date();
    const newPo: PurchaseHistoryItemDTO = {
      id: "po-" + Date.now(),
      no: purchases.length + 1,
      faktur: "#PO-" + today.toISOString().slice(0, 10).replace(/-/g, "") + "-" + Math.floor(100 + Math.random() * 900),
      faktur_supplier: formFakturSupplier.trim() || `INV/${Math.floor(1000 + Math.random() * 9000)}`,
      tanggal: today.toLocaleDateString("id-ID") + " " + today.toLocaleTimeString("id-ID", { hour: "2-digit", minute: "2-digit" }),
      suplier: formSupplier,
      total_item: 4,
      total_qty: Number(formTotalQty),
      total_beli: Number(formTotalBeli),
      metode: formMetode,
      jatuh_tempo: formMetode === "TUNAI" ? "-" : `${formTempoHari} Hari`,
      status: formMetode === "TUNAI" ? "LUNAS" : "TEMPO",
      penerima: "Gudang Utama (Kasir)",
    };

    purchases = [newPo, ...purchases];
    stats.total_belanja_bulan_ini += Number(formTotalBeli);
    stats.total_qty_masuk += Number(formTotalQty);
    stats.total_faktur += 1;
    if (formMetode !== "TUNAI") {
      stats.total_hutang_tempo += Number(formTotalBeli);
    }
    isAddModalOpen = false;
    showToast(`Faktur Pembelian ${newPo.faktur} berhasil dicatat ke stok & buku pembelian.`);
  }

  $effect(() => {
    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });
</script>

<div class="flex-1 flex flex-col bg-slate-200 overflow-hidden font-sans select-none min-h-0">
  <!-- 1. TOP ACTION & STATUS TABS BAR (Di Atas Card) -->
  <div class="px-4 py-2.5 bg-white border-b border-slate-300 shadow-2xs flex flex-wrap items-center justify-between gap-3 shrink-0">
    <!-- Status Tabs -->
    <div class="flex items-center gap-1 p-1 bg-slate-100 rounded-xl border border-slate-200">
      <button
        onclick={() => (statusFilter = "all")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {statusFilter === 'all' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span>Semua Pembelian</span>
        <span class="px-1.5 py-0.5 rounded {statusFilter === 'all' ? 'bg-white/20 text-white' : 'bg-slate-200 text-slate-700'} font-mono text-[10px] font-bold">
          {purchases.length}
        </span>
      </button>

      <button
        onclick={() => (statusFilter = "tunai")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {statusFilter === 'tunai' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span>Tunai / Lunas</span>
        <span class="px-1.5 py-0.5 rounded {statusFilter === 'tunai' ? 'bg-white/20 text-white' : 'bg-slate-200 text-slate-700'} font-mono text-[10px] font-bold">
          {purchases.filter((p) => p.status === 'LUNAS').length}
        </span>
      </button>

      <button
        onclick={() => (statusFilter = "tempo")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {statusFilter === 'tempo' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span>Tempo / Hutang Dagang</span>
        <span class="px-1.5 py-0.5 rounded {statusFilter === 'tempo' ? 'bg-white/20 text-white' : 'bg-slate-200 text-slate-700'} font-mono text-[10px] font-bold">
          {purchases.filter((p) => p.status === 'TEMPO').length}
        </span>
      </button>
    </div>

    <!-- Top Action Buttons (Di Atas Card) -->
    <div class="flex items-center gap-1.5">
      <button
        onclick={() => showToast("Mencetak rekap pembelian stok...")}
        class="flex items-center gap-1.5 px-3 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-800 rounded-lg font-sans text-xs font-medium border border-slate-300 shadow-2xs transition-all cursor-pointer"
        title="Cetak Rekap Pembelian (F4)"
      >
        <span class="font-mono text-[10px] font-bold bg-slate-200 px-1 py-0.2 rounded text-slate-700">F4</span>
        <span class="material-symbols-outlined text-[16px] text-primary">receipt_long</span>
        <span>Cetak Rekap</span>
      </button>

      <button
        onclick={() => showToast("Export data pembelian ke CSV berhasil!")}
        class="flex items-center gap-1.5 px-3 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-800 rounded-lg font-sans text-xs font-medium border border-slate-300 shadow-2xs transition-all cursor-pointer"
        title="Export CSV (F7)"
      >
        <span class="font-mono text-[10px] font-bold bg-slate-200 px-1 py-0.2 rounded text-slate-700">F7</span>
        <span class="material-symbols-outlined text-[16px] text-emerald-600">table_view</span>
        <span>Export CSV</span>
      </button>

      <button
        onclick={() => showToast("Database pembelian offline SQLite tersinkronisasi.")}
        class="flex items-center gap-1 px-2.5 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-800 rounded-lg font-sans text-xs border border-slate-300 shadow-2xs transition-all cursor-pointer"
        title="Sinkronisasi Data (F5)"
      >
        <span class="font-mono text-[10px] font-bold bg-slate-200 px-1 py-0.2 rounded text-slate-700">F5</span>
        <span class="material-symbols-outlined text-[16px]">sync</span>
      </button>

      <!-- + Faktur Pembelian Baru CTA [F6] -->
      <button
        onclick={() => (isAddModalOpen = true)}
        class="flex items-center gap-1.5 px-3.5 py-1.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg font-sans text-xs font-bold shadow-sm border border-emerald-700 transition-all cursor-pointer ml-1"
        title="Input Faktur Penerimaan Barang / Pembelian [F6]"
      >
        <span class="font-mono text-[10px] bg-emerald-800 px-1.5 py-0.5 rounded font-bold">F6</span>
        <span class="material-symbols-outlined text-[16px]">post_add</span>
        <span>+ Faktur Pembelian Baru</span>
      </button>
    </div>
  </div>

  <!-- 2. 5 COMPACT STAT CARDS (Di Tengah) -->
  <div class="px-4 py-2.5 bg-slate-200 border-b border-slate-300 shrink-0">
    <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-2.5 items-stretch">
      <!-- Card 1: Total Belanja Stok -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Total Pembelian Stok</span>
          <span class="inline-flex items-center px-1.5 py-0.2 rounded bg-emerald-100 text-emerald-800 font-mono text-[10px] font-bold">+8.5%</span>
        </div>
        <div class="my-1">
          <span class="font-mono text-xl font-bold text-primary tracking-tight">
            {formatRupiah(stats.total_belanja_bulan_ini)}
          </span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Bulan Berjalan</span>
          <span class="text-slate-700 font-bold">HPP Masuk</span>
        </div>
      </div>

      <!-- Card 2: Total Qty Masuk -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Total Qty Masuk</span>
          <span class="material-symbols-outlined text-emerald-600 text-[18px]">inventory</span>
        </div>
        <div class="my-1 flex items-baseline gap-1.5">
          <span class="font-mono text-xl font-bold text-slate-900 tracking-tight">{stats.total_qty_masuk}</span>
          <span class="font-mono text-xs text-slate-500 font-medium">Pcs / Unit</span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Rata-rata/PO: <strong class="text-slate-800 font-bold">{Math.round(stats.total_qty_masuk / stats.total_faktur)}</strong> pcs</span>
          <span>42 SKU</span>
        </div>
      </div>

      <!-- Card 3: Total Faktur Pembelian -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Faktur Pembelian</span>
          <span class="inline-flex items-center px-1.5 py-0.2 rounded bg-blue-100 text-blue-800 font-mono text-[10px] font-bold">100% Sah</span>
        </div>
        <div class="my-1 flex items-baseline gap-1.5">
          <span class="font-mono text-xl font-bold text-slate-900 tracking-tight">{stats.total_faktur}</span>
          <span class="font-mono text-xs text-slate-500 font-medium">Nota PO</span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Retur: <strong class="text-rose-600 font-bold">0 Nota</strong></span>
          <span>Gudang Siap</span>
        </div>
      </div>

      <!-- Card 4: Hutang Pembelian Tempo -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-amber-700 font-bold">Hutang Belum Lunas</span>
          <span class="material-symbols-outlined text-amber-600 text-[18px]">pending_actions</span>
        </div>
        <div class="my-1">
          <span class="font-mono text-xl font-bold text-amber-700 tracking-tight">
            {formatRupiah(stats.total_hutang_tempo)}
          </span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>3 Faktur Tempo</span>
          <span class="text-amber-800 font-bold">Tempo 14 Hari</span>
        </div>
      </div>

      <!-- Card 5: Rekanan Utama -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Supplier Teraktif</span>
          <span class="material-symbols-outlined text-primary text-[18px]">factory</span>
        </div>
        <div class="my-1 truncate">
          <span class="font-sans text-sm font-bold text-slate-900 truncate block">
            {stats.supplier_teraktif}
          </span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Total 8 Pengiriman</span>
          <span class="text-emerald-700 font-bold">Top Vendor</span>
        </div>
      </div>
    </div>
  </div>

  <!-- 3. SEARCH BAR & FILTER DROPDOWNS (Di Bawah Card) -->
  <div class="px-4 py-2 bg-white border-b border-slate-300 shadow-2xs flex flex-wrap items-center justify-between gap-3 shrink-0">
    <!-- Search Bar -->
    <div class="flex-1 min-w-[320px] max-w-2xl relative">
      <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-slate-400">
        <span class="material-symbols-outlined text-[18px]">search</span>
      </div>
      <input
        bind:this={searchInputElement}
        bind:value={searchQuery}
        class="w-full pl-9 pr-14 py-1.5 bg-slate-100 text-slate-900 placeholder:text-slate-400 font-sans text-xs rounded-lg border border-slate-300 outline-none focus:border-primary focus:bg-white focus:ring-1 focus:ring-primary transition-all"
        placeholder="Cari No. Faktur PO / No. Faktur Supplier / Nama Supplier / Penerima..."
        type="text"
      />
      <div class="absolute inset-y-0 right-0 pr-2 flex items-center pointer-events-none">
        <span class="px-1.5 py-0.5 rounded bg-slate-200 text-slate-700 font-mono text-[10px] font-bold uppercase border border-slate-300">
          F3
        </span>
      </div>
    </div>

    <!-- Dropdown Filters (Supplier, Periode) -->
    <div class="flex items-center gap-2 font-sans text-xs">
      <!-- 1. Dropdown Supplier -->
      <div class="relative">
        <select
          bind:value={supplierFilter}
          class="appearance-none bg-slate-100 text-slate-800 font-sans text-xs px-3 py-1.5 pr-7 rounded-lg border border-slate-300 shadow-2xs focus:outline-none cursor-pointer font-medium max-w-[160px] truncate"
          title="Filter Supplier"
        >
          <option value="all">Semua Supplier</option>
          {#each suppliersList.filter((s) => s !== 'all') as sup}
            <option value={sup}>{sup}</option>
          {/each}
        </select>
        <span class="material-symbols-outlined absolute right-2 top-1.5 pointer-events-none text-slate-500 text-[16px]">expand_more</span>
      </div>

      <!-- 2. Dropdown Periode -->
      <div class="relative">
        <select
          bind:value={periodeFilter}
          class="appearance-none bg-slate-100 text-slate-800 font-sans text-xs px-3 py-1.5 pr-7 rounded-lg border border-slate-300 shadow-2xs focus:outline-none cursor-pointer font-medium"
          title="Filter Periode Pembelian"
        >
          <option value="hari_ini">Hari Ini</option>
          <option value="minggu_ini">7 Hari Terakhir</option>
          <option value="bulan_ini">Bulan Ini</option>
          <option value="semua">Semua Periode</option>
        </select>
        <span class="material-symbols-outlined absolute right-2 top-1.5 pointer-events-none text-slate-500 text-[16px]">expand_more</span>
      </div>
    </div>
  </div>

  <!-- 4. FULL-WIDTH DATA TABLE -->
  <div class="w-full p-4 flex-1 flex flex-col min-h-0 overflow-hidden">
    <div class="w-full bg-white rounded-xl border border-slate-300 shadow-sm overflow-hidden flex flex-col flex-1 min-h-0">
      <div class="overflow-auto w-full flex-1 min-h-0">
        <table class="w-full text-left font-sans text-xs border-collapse">
          <!-- Table Header -->
          <thead class="bg-slate-800 text-slate-100 uppercase font-mono text-[11px] font-bold tracking-wider select-none border-b-2 border-slate-900 sticky top-0 z-10 shadow-sm">
            <tr>
              <th class="py-2.5 px-3 text-center w-12" scope="col">No</th>
              <th class="py-2.5 px-3 w-36" scope="col">No. Faktur PO</th>
              <th class="py-2.5 px-3 w-32" scope="col">Faktur Vendor</th>
              <th class="py-2.5 px-3 w-36" scope="col">Tanggal &amp; Jam</th>
              <th class="py-2.5 px-3 min-w-[180px]" scope="col">Nama Supplier</th>
              <th class="py-2.5 px-3 text-center w-24" scope="col">Total Item</th>
              <th class="py-2.5 px-3 text-center w-24" scope="col">Total Qty</th>
              <th class="py-2.5 px-3 text-right w-32" scope="col">Total Belanja (HPP)</th>
              <th class="py-2.5 px-3 text-center w-28" scope="col">Status Bayar</th>
              <th class="py-2.5 px-3 text-center w-28" scope="col">Jatuh Tempo</th>
              <th class="py-2.5 px-3 w-40" scope="col">Penerima Gudang</th>
              <th class="py-2.5 px-3 text-center w-24" scope="col">Aksi</th>
            </tr>
          </thead>

          <!-- Table Body -->
          <tbody class="divide-y divide-slate-200 text-slate-900">
            {#each filteredPurchases as item, i}
              <tr
                class="transition-colors border-b border-slate-200/80 cursor-pointer {item.status === 'TEMPO' ? 'bg-amber-50/50 hover:bg-amber-100/70 border-l-4 border-l-amber-500' : i % 2 === 1 ? 'bg-slate-50/70 hover:bg-sky-50/80' : 'bg-white hover:bg-sky-50/80'}"
                onclick={() => (selectedDetailPurchase = item)}
              >
                <td class="py-2.5 px-3 text-center font-mono text-xs font-bold text-slate-500">
                  {i + 1}
                </td>
                <td class="py-2.5 px-3 font-mono text-xs font-bold text-primary">
                  {item.faktur}
                </td>
                <td class="py-2.5 px-3 font-mono text-xs text-slate-600">
                  {item.faktur_supplier}
                </td>
                <td class="py-2.5 px-3 font-mono text-xs text-slate-600">
                  {item.tanggal}
                </td>
                <td class="py-2.5 px-3 font-semibold text-slate-900">
                  {item.suplier}
                </td>
                <td class="py-2.5 px-3 text-center font-mono text-xs font-bold text-slate-700">
                  {item.total_item} SKU
                </td>
                <td class="py-2.5 px-3 text-center font-mono text-xs font-bold text-slate-900">
                  {item.total_qty} Pcs
                </td>
                <td class="py-2.5 px-3 text-right font-mono text-xs font-bold text-primary tabular-nums">
                  {formatRupiah(item.total_beli)}
                </td>
                <td class="py-2.5 px-3 text-center">
                  <span class="inline-flex items-center px-2 py-0.5 rounded font-mono text-[10px] font-bold {item.status === 'LUNAS' ? 'bg-emerald-100 text-emerald-800 border border-emerald-300' : 'bg-amber-100 text-amber-800 border border-amber-300'}">
                    {item.status}
                  </span>
                </td>
                <td class="py-2.5 px-3 text-center font-mono text-xs {item.status === 'TEMPO' ? 'text-amber-700 font-bold' : 'text-slate-500'}">
                  {item.jatuh_tempo}
                </td>
                <td class="py-2.5 px-3 text-xs text-slate-600 truncate max-w-[150px]">
                  {item.penerima}
                </td>
                <td class="py-2.5 px-3 text-center" onclick={(e) => e.stopPropagation()}>
                  <div class="inline-flex items-center gap-1">
                    <button
                      onclick={() => (selectedDetailPurchase = item)}
                      class="p-1 rounded hover:bg-slate-200 text-slate-600 hover:text-primary transition-colors border-none bg-transparent cursor-pointer"
                      title="Lihat Detail Faktur"
                    >
                      <span class="material-symbols-outlined text-[16px]">visibility</span>
                    </button>
                    <button
                      onclick={() => showToast(`Mencetak nota pembelian ${item.faktur}...`)}
                      class="p-1 rounded hover:bg-slate-200 text-slate-600 hover:text-primary transition-colors border-none bg-transparent cursor-pointer"
                      title="Cetak Bukti Pembelian"
                    >
                      <span class="material-symbols-outlined text-[16px]">print</span>
                    </button>
                  </div>
                </td>
              </tr>
            {:else}
              <tr>
                <td colspan="12" class="py-16 text-center text-slate-400 font-mono text-xs">
                  Tidak ada faktur pembelian yang cocok dengan filter / pencarian.
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      <!-- Footer Bar / Pagination -->
      <footer class="bg-slate-100 p-2.5 border-t border-slate-300 flex flex-col md:flex-row items-center justify-between gap-2 text-xs shrink-0 select-none">
        <div class="flex flex-wrap items-center gap-3 text-slate-600 font-mono text-[11px]">
          <div class="flex items-center gap-1.5">
            <span class="w-2 h-2 rounded-full bg-emerald-600"></span>
            <span class="font-medium text-emerald-800 font-bold">Data pembelian tersinkronisasi offline SQLite</span>
          </div>
          <span>•</span>
          <div>
            Menampilkan <strong class="text-slate-900">{filteredPurchases.length}</strong> dari <strong class="text-slate-900">{stats.total_faktur}</strong> Faktur PO
          </div>
        </div>

        <div class="flex items-center gap-1 font-mono text-xs">
          <button class="px-2.5 py-1 rounded bg-white border border-slate-300 text-slate-400 cursor-not-allowed font-medium" disabled>
            Sebelumnya
          </button>
          <button class="w-7 h-7 rounded bg-primary text-white font-bold flex items-center justify-center shadow-2xs border-none cursor-pointer">
            1
          </button>
          <button class="w-7 h-7 rounded hover:bg-slate-200 text-slate-700 font-medium flex items-center justify-center transition-colors border border-slate-300 bg-white cursor-pointer">
            2
          </button>
          <button class="px-2.5 py-1 rounded bg-white hover:bg-slate-200 text-slate-800 font-medium transition-colors shadow-2xs border border-slate-300 cursor-pointer">
            Selanjutnya
          </button>
        </div>
      </footer>
    </div>
  </div>

  <!-- Modal Detail Faktur Pembelian -->
  {#if selectedDetailPurchase}
    <div
      class="fixed inset-0 bg-slate-900/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none"
      onclick={() => (selectedDetailPurchase = null)}
      role="button"
      tabindex="0"
      onkeydown={(e) => e.key === "Escape" && (selectedDetailPurchase = null)}
    >
      <div
        class="bg-white border border-slate-300 rounded-xl shadow-2xl w-[600px] overflow-hidden"
        role="dialog"
        aria-modal="true"
        tabindex="-1"
        onclick={(e) => e.stopPropagation()}
        onkeydown={(e) => e.stopPropagation()}
      >
        <div class="px-5 py-3.5 bg-slate-100 border-b border-slate-300 flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="material-symbols-outlined text-primary text-[20px]">receipt</span>
            <span class="font-bold text-slate-900 text-sm font-sans">
              Detail Faktur Pembelian: {selectedDetailPurchase.faktur}
            </span>
          </div>
          <button
            onclick={() => (selectedDetailPurchase = null)}
            class="text-slate-400 hover:text-slate-700 p-1 hover:bg-slate-200 rounded border-none bg-transparent cursor-pointer"
          >
            <span class="material-symbols-outlined text-[18px]">close</span>
          </button>
        </div>

        <div class="p-5 font-sans text-xs flex flex-col gap-3">
          <div class="grid grid-cols-2 gap-3 p-3 bg-slate-50 rounded-lg border border-slate-200 font-mono text-xs">
            <div>
              <span class="text-slate-500 block">Supplier:</span>
              <strong class="text-slate-900 text-sm">{selectedDetailPurchase.suplier}</strong>
            </div>
            <div>
              <span class="text-slate-500 block">Faktur Supplier:</span>
              <strong class="text-slate-900">{selectedDetailPurchase.faktur_supplier}</strong>
            </div>
            <div>
              <span class="text-slate-500 block">Tanggal Terima:</span>
              <strong class="text-slate-900">{selectedDetailPurchase.tanggal}</strong>
            </div>
            <div>
              <span class="text-slate-500 block">Status Pembayaran:</span>
              <span class="inline-flex px-1.5 py-0.2 rounded font-bold {selectedDetailPurchase.status === 'LUNAS' ? 'bg-emerald-100 text-emerald-800' : 'bg-amber-100 text-amber-800'}">
                {selectedDetailPurchase.status} ({selectedDetailPurchase.metode})
              </span>
            </div>
          </div>

          <div class="border border-slate-200 rounded-lg overflow-hidden">
            <table class="w-full text-left font-sans text-xs">
              <thead class="bg-slate-100 text-slate-700 font-mono text-[11px] border-b border-slate-200">
                <tr>
                  <th class="p-2">Item Barang</th>
                  <th class="p-2 text-center">Qty</th>
                  <th class="p-2 text-right">Harga Beli</th>
                  <th class="p-2 text-right">Subtotal</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-slate-100 font-mono text-xs">
                <tr>
                  <td class="p-2 font-sans font-medium text-slate-900">Indomie Goreng Spesial 85g (Karton)</td>
                  <td class="p-2 text-center font-bold">50 Dus</td>
                  <td class="p-2 text-right">{formatRupiah(108000)}</td>
                  <td class="p-2 text-right font-bold text-slate-900">{formatRupiah(5400000)}</td>
                </tr>
                <tr>
                  <td class="p-2 font-sans font-medium text-slate-900">Minyak Goreng SunCo 2L Pouch</td>
                  <td class="p-2 text-center font-bold">30 Pcs</td>
                  <td class="p-2 text-right">{formatRupiah(33500)}</td>
                  <td class="p-2 text-right font-bold text-slate-900">{formatRupiah(1005000)}</td>
                </tr>
                <tr>
                  <td class="p-2 font-sans font-medium text-slate-900">Gula Pasir Gulaku Premium 1kg</td>
                  <td class="p-2 text-center font-bold">40 Pcs</td>
                  <td class="p-2 text-right">{formatRupiah(15500)}</td>
                  <td class="p-2 text-right font-bold text-slate-900">{formatRupiah(620000)}</td>
                </tr>
              </tbody>
            </table>
          </div>

          <div class="flex items-center justify-between p-3 bg-slate-100 rounded-lg font-mono">
            <span class="text-slate-600 font-bold">TOTAL PEMBELIAN:</span>
            <span class="text-primary font-bold text-base">{formatRupiah(selectedDetailPurchase.total_beli)}</span>
          </div>
        </div>

        <div class="px-5 py-3 bg-slate-50 border-t border-slate-300 flex items-center justify-end gap-2">
          <button
            type="button"
            onclick={() => showToast("Mencetak faktur pembelian...")}
            class="px-4 py-1.5 bg-white hover:bg-slate-100 text-slate-700 border border-slate-300 rounded-lg text-xs font-bold shadow-2xs cursor-pointer flex items-center gap-1"
          >
            <span class="material-symbols-outlined text-[16px]">print</span>
            Cetak Nota PO
          </button>
          <button
            type="button"
            onclick={() => (selectedDetailPurchase = null)}
            class="px-4 py-1.5 bg-primary hover:bg-primary-dark text-white rounded-lg text-xs font-bold shadow-sm cursor-pointer border-none"
          >
            Tutup
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Modal Faktur Pembelian Baru -->
  {#if isAddModalOpen}
    <div
      class="fixed inset-0 bg-slate-900/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none"
      onclick={() => (isAddModalOpen = false)}
      role="button"
      tabindex="0"
      onkeydown={(e) => e.key === "Escape" && (isAddModalOpen = false)}
    >
      <div
        class="bg-white border border-slate-300 rounded-xl shadow-2xl w-[520px] overflow-hidden"
        role="dialog"
        aria-modal="true"
        tabindex="-1"
        onclick={(e) => e.stopPropagation()}
        onkeydown={(e) => e.stopPropagation()}
      >
        <div class="px-5 py-3.5 bg-emerald-50 border-b border-emerald-200 flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="material-symbols-outlined text-emerald-700 text-[20px]">post_add</span>
            <span class="font-bold text-emerald-950 text-sm font-sans">Input Faktur Pembelian Baru [F6]</span>
          </div>
          <button
            onclick={() => (isAddModalOpen = false)}
            class="text-slate-400 hover:text-slate-700 p-1 hover:bg-slate-200 rounded border-none bg-transparent cursor-pointer"
          >
            <span class="material-symbols-outlined text-[18px]">close</span>
          </button>
        </div>

        <div class="p-5 font-sans text-xs flex flex-col gap-3">
          <div>
            <label for="form-pilih-supplier" class="block font-semibold text-slate-700 mb-1">Pilih Supplier Rekanan</label>
            <select
              id="form-pilih-supplier"
              bind:value={formSupplier}
              class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded text-xs font-medium focus:outline-none"
            >
              <option value="PT Sumber Makmur">PT Sumber Makmur (Distributor Sembako)</option>
              <option value="PT Kopi Mandiri">PT Kopi Mandiri (Distributor Kopi & Sirup)</option>
              <option value="Prima Bakery">Prima Bakery (Pastry & Roti)</option>
              <option value="Danone Tirta">Danone Tirta (Air Mineral & Minuman)</option>
              <option value="Indofood Sukses Makmur">Indofood Sukses Makmur</option>
              <option value="Mayora Retail Group">Mayora Retail Group</option>
            </select>
          </div>

          <div class="grid grid-cols-2 gap-3">
            <div>
              <label for="form-faktur-supp" class="block font-semibold text-slate-700 mb-1">No. Faktur Vendor</label>
              <input
                id="form-faktur-supp"
                type="text"
                bind:value={formFakturSupplier}
                placeholder="Contoh: INV/SM/8892"
                class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded font-mono text-xs focus:outline-none"
              />
            </div>
            <div>
              <label for="form-metode-bayar" class="block font-semibold text-slate-700 mb-1">Sistem Pembayaran</label>
              <select
                id="form-metode-bayar"
                bind:value={formMetode}
                class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded text-xs focus:outline-none font-bold"
              >
                <option value="TUNAI">TUNAI (Kas Toko)</option>
                <option value="KREDIT / TEMPO">KREDIT / TEMPO (Hutang Dagang)</option>
              </select>
            </div>
          </div>

          {#if formMetode === "KREDIT / TEMPO"}
            <div class="p-2.5 bg-amber-50 border border-amber-200 rounded-lg flex items-center justify-between">
              <span class="text-amber-900 font-medium">Termin Jatuh Tempo:</span>
              <div class="flex items-center gap-1 font-mono">
                <input
                  type="number"
                  bind:value={formTempoHari}
                  class="w-16 px-2 py-1 bg-white border border-amber-300 rounded text-right font-bold"
                />
                <span class="text-amber-800">Hari</span>
              </div>
            </div>
          {/if}

          <div class="grid grid-cols-2 gap-3 pt-1 border-t border-slate-200">
            <div>
              <label for="form-total-qty" class="block font-semibold text-slate-700 mb-1">Total Kuantiti Masuk</label>
              <input
                id="form-total-qty"
                type="number"
                bind:value={formTotalQty}
                class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded font-mono text-xs focus:outline-none text-right font-bold"
              />
            </div>
            <div>
              <label for="form-total-beli" class="block font-semibold text-slate-700 mb-1">Total Tagihan Beli (Rp)</label>
              <input
                id="form-total-beli"
                type="number"
                bind:value={formTotalBeli}
                class="w-full px-2.5 py-1.5 bg-slate-50 border-2 border-emerald-600 rounded font-mono text-xs focus:outline-none text-right font-bold text-emerald-800"
              />
            </div>
          </div>
        </div>

        <div class="px-5 py-3 bg-slate-50 border-t border-slate-300 flex items-center justify-end gap-2">
          <button
            type="button"
            onclick={() => (isAddModalOpen = false)}
            class="px-4 py-1.5 bg-white hover:bg-slate-100 text-slate-700 border border-slate-300 rounded-lg text-xs font-bold shadow-2xs cursor-pointer"
          >
            Batal
          </button>
          <button
            type="button"
            onclick={simpanFakturPembelian}
            class="px-4 py-1.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg text-xs font-bold shadow-sm cursor-pointer border-none flex items-center gap-1"
          >
            <span class="material-symbols-outlined text-[16px]">check</span>
            Simpan Faktur Pembelian
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Toast Message -->
  {#if toastMessage}
    <div class="fixed bottom-4 right-4 bg-slate-900 text-emerald-300 px-4 py-2.5 rounded-lg shadow-xl font-mono text-xs border border-slate-700 animate-in fade-in z-50 flex items-center gap-2">
      <span class="material-symbols-outlined text-[18px] text-emerald-400">info</span>
      <span>{toastMessage}</span>
    </div>
  {/if}
</div>
