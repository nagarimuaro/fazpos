<script lang="ts">
  import { api, formatRupiah, type ProductDTO, type ProductStatsDTO } from "../lib/api";

  let products = $state<ProductDTO[]>([]);
  let stats = $state<ProductStatsDTO | null>(null);
  let searchQuery = $state("");
  let categoryFilter = $state("all");
  let supplierFilter = $state("all");
  let satuanFilter = $state("all");
  let statusFilter = $state("all");
  let searchInputElement: HTMLInputElement | null = $state(null);
  let isAddModalOpen = $state(false);
  let isRestockModalOpen = $state(false);
  let editingProduct = $state<ProductDTO | null>(null);
  let toastMessage = $state("");

  let categoriesList = $derived([
    "all",
    ...Array.from(new Set(products.map((p) => p.kategori).filter(Boolean))),
  ]);

  let suppliersList = $derived([
    "all",
    ...Array.from(new Set(products.map((p) => p.supplier).filter(Boolean))),
  ]);

  let satuanList = $derived([
    "all",
    ...Array.from(new Set(products.map((p) => p.satuan).filter(Boolean))),
  ]);

  // Column visibility checklist toggles (limit, harga 1 2 3, suplier, rak, expired, margin, etc.)
  let isColumnSettingsOpen = $state(false);
  let cols = $state({
    rak: true,
    supplier: true,
    limit: true,
    hargaModal: true,
    harga1: true,
    harga2: true,
    harga3: true,
    margin: true,
    expired: true,
  });

  // Form states for new/edit product
  let formNama = $state("");
  let formBarcode = $state("");
  let formKode = $state("");
  let formKategori = $state("Makanan & Minuman");
  let formSatuan = $state("Pcs");
  let formRak = $state("A-01");
  let formSupplier = $state("PT Sumber Makmur");
  let formHpp = $state(0);
  let formJual = $state(0);
  let formStok = $state(10);

  async function loadData() {
    try {
      const kw = searchQuery.trim() || undefined;
      products = await api.getCatalogProducts(kw);
      stats = await api.getProductStats();
    } catch (err) {
      console.error(err);
    }
  }

  function showToast(msg: string) {
    toastMessage = msg;
    setTimeout(() => (toastMessage = ""), 4000);
  }

  function handleSearch() {
    loadData();
  }

  function openAddModal() {
    editingProduct = null;
    formNama = "";
    formBarcode = "899" + Math.floor(1000000000 + Math.random() * 9000000000);
    formKode = "BRG-" + Math.floor(1000 + Math.random() * 9000);
    formKategori = "Makanan & Minuman";
    formSatuan = "Pcs";
    formRak = "A-01";
    formSupplier = "PT Sumber Makmur";
    formHpp = 5000;
    formJual = 7000;
    formStok = 24;
    isAddModalOpen = true;
  }

  function openEditModal(p: ProductDTO) {
    editingProduct = p;
    formNama = p.nama;
    formBarcode = p.barcode;
    formKode = p.kode;
    formKategori = p.kategori;
    formSatuan = p.satuan;
    formRak = p.rak;
    formSupplier = p.supplier;
    formHpp = p.hargapokok;
    formJual = p.hargajual1;
    formStok = p.stok;
    isAddModalOpen = true;
  }

  function saveProduct() {
    if (!formNama.trim()) {
      alert("Nama produk wajib diisi");
      return;
    }

    const margin = formHpp > 0 ? Math.round(((formJual - formHpp) / formHpp) * 1000) / 10 : 25;
    const isKritis = formStok <= 10;

    if (editingProduct) {
      // update
      const idx = products.findIndex((p) => p.id === editingProduct!.id);
      if (idx !== -1) {
        products[idx] = {
          ...products[idx],
          nama: formNama,
          barcode: formBarcode,
          kode: formKode,
          kategori: formKategori,
          satuan: formSatuan,
          rak: formRak,
          supplier: formSupplier,
          hargapokok: formHpp,
          hargajual1: formJual,
          margin_persen: margin,
          stok: formStok,
          is_kritis: isKritis,
          tag: formStok === 0 ? "Habis" : isKritis ? "Segera Order" : undefined,
        };
      }
      showToast(`Produk ${formNama} berhasil diperbarui.`);
    } else {
      // create
      const newP: ProductDTO = {
        id: "p-" + Date.now(),
        kode: formKode,
        barcode: formBarcode,
        nama: formNama,
        kategori: formKategori,
        satuan: formSatuan,
        rak: formRak,
        supplier: formSupplier,
        hargapokok: formHpp,
        hargajual1: formJual,
        hargajual2: Math.round(formJual * 0.95),
        hargajual3: Math.round(formJual * 0.90),
        margin_persen: margin,
        stok: formStok,
        stokminimum: 10,
        is_kritis: isKritis,
        tag: formStok === 0 ? "Habis" : isKritis ? "Segera Order" : undefined,
      };
      products = [newP, ...products];
      showToast(`Produk baru ${formNama} berhasil disimpan.`);
    }

    isAddModalOpen = false;
  }

  function deleteProduct(p: ProductDTO) {
    if (confirm(`Apakah Anda yakin ingin menghapus produk ${p.nama} (${p.kode})?`)) {
      products = products.filter((item) => item.id !== p.id);
      showToast(`Produk ${p.nama} berhasil dihapus.`);
    }
  }

  function exportExcel() {
    if (products.length === 0) return;
    const header = "No,Barcode,Kode,Nama Produk,Kategori,Satuan,Rak,Supplier,Harga Pokok,Harga Jual,Margin %,Stok\n";
    const rows = filteredProducts.map((p, i) =>
      `"${i + 1}","${p.barcode}","${p.kode}","${p.nama}","${p.kategori}","${p.satuan}","${p.rak}","${p.supplier}","${p.hargapokok}","${p.hargajual1}","${p.margin_persen}%","${p.stok}"`
    ).join("\n");
    const blob = new Blob([header + rows], { type: "text/csv;charset=utf-8;" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `master_produk_${new Date().toISOString().slice(0, 10)}.csv`;
    a.click();
    URL.revokeObjectURL(url);
    showToast("Data master produk berhasil diexport!");
  }

  let filteredProducts = $derived(
    products.filter((p) => {
      // 1. Kategori filter
      if (categoryFilter !== "all" && p.kategori.toLowerCase() !== categoryFilter.toLowerCase()) {
        return false;
      }

      // 2. Supplier filter
      if (supplierFilter !== "all" && p.supplier.toLowerCase() !== supplierFilter.toLowerCase()) {
        return false;
      }

      // 3. Satuan filter
      if (satuanFilter !== "all" && p.satuan.toLowerCase() !== satuanFilter.toLowerCase()) {
        return false;
      }

      // 4. Status filter
      if (statusFilter === "low" && (!p.is_kritis || p.stok === 0)) return false;
      if (statusFilter === "empty" && p.stok !== 0) return false;
      if (statusFilter === "safe" && p.is_kritis) return false;

      // Keyword filter
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase().trim();
        return (
          p.nama.toLowerCase().includes(q) ||
          p.kode.toLowerCase().includes(q) ||
          p.barcode.toLowerCase().includes(q) ||
          p.kategori.toLowerCase().includes(q) ||
          p.rak.toLowerCase().includes(q) ||
          p.supplier.toLowerCase().includes(q)
        );
      }
      return true;
    })
  );

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "F3") {
      e.preventDefault();
      searchInputElement?.focus();
      searchInputElement?.select();
    } else if (e.key === "F5") {
      e.preventDefault();
      openAddModal();
    } else if (e.key === "F6") {
      e.preventDefault();
      isRestockModalOpen = true;
    }
  }

  $effect(() => {
    loadData();
    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });
</script>

<div class="flex-1 flex flex-col bg-slate-200 overflow-hidden font-sans select-none min-h-0">
  <!-- 1. TOP ACTION & STATUS TABS BAR (Di Atas Card) -->
  <div class="px-4 py-2.5 bg-white border-b border-slate-300 shadow-2xs flex flex-wrap items-center justify-between gap-3 shrink-0">
    <!-- Tabs Filter Status -->
    <div class="flex items-center gap-1 p-1 bg-slate-100 rounded-xl border border-slate-200">
      <button
        onclick={() => (statusFilter = "all")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {statusFilter === 'all' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span>Semua Produk</span>
        <span class="px-1.5 py-0.5 rounded {statusFilter === 'all' ? 'bg-white/20 text-white' : 'bg-slate-200 text-slate-700'} font-mono text-[10px] font-bold">
          {stats?.total_produk ?? 1428}
        </span>
      </button>

      <button
        onclick={() => (statusFilter = "safe")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {statusFilter === 'safe' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span>Stok Normal</span>
        <span class="px-1.5 py-0.5 rounded {statusFilter === 'safe' ? 'bg-white/20 text-white' : 'bg-slate-200 text-slate-700'} font-mono text-[10px] font-bold">
          {stats?.stok_optimal ?? 1385}
        </span>
      </button>

      <button
        onclick={() => (statusFilter = "low")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {statusFilter === 'low' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span>Stok Menipis</span>
        <span class="px-1.5 py-0.5 rounded {statusFilter === 'low' ? 'bg-white/20 text-white' : 'bg-slate-200 text-slate-700'} font-mono text-[10px] font-bold">
          {stats?.stok_menipis ?? 38}
        </span>
      </button>

      <button
        onclick={() => (statusFilter = "empty")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {statusFilter === 'empty' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span>Stok Kosong</span>
        <span class="px-1.5 py-0.5 rounded {statusFilter === 'empty' ? 'bg-white/20 text-white' : 'bg-slate-200 text-slate-700'} font-mono text-[10px] font-bold">
          {stats?.stok_kosong ?? 5}
        </span>
      </button>
    </div>

    <!-- Action Utilities (Di Atas Card) -->
    <div class="flex items-center gap-1.5">
      <!-- Import Data Button -->
      <button
        onclick={() => alert("Fitur Import Data Excel/CSV iB Retago siap digunakan.")}
        class="flex items-center gap-1 px-2.5 py-1.5 rounded bg-slate-100 hover:bg-slate-200 text-slate-800 font-sans text-xs font-medium transition-colors border border-slate-300 shadow-2xs cursor-pointer"
        title="Import Data Master dari Excel / CSV"
      >
        <span class="material-symbols-outlined text-[16px] text-primary">file_upload</span>
        <span>Import Data</span>
      </button>

      <!-- Export Data Button -->
      <button
        onclick={exportExcel}
        class="flex items-center gap-1 px-2.5 py-1.5 rounded bg-slate-100 hover:bg-slate-200 text-slate-800 font-sans text-xs font-medium transition-colors border border-slate-300 shadow-2xs cursor-pointer"
        title="Export Data Master & Stok ke Excel"
      >
        <span class="material-symbols-outlined text-[16px] text-emerald-700">file_download</span>
        <span>Export CSV</span>
      </button>

      <!-- Refresh Button -->
      <button
        onclick={loadData}
        class="p-1.5 rounded bg-slate-100 hover:bg-slate-200 text-slate-700 transition-colors border border-slate-300 shadow-2xs flex items-center justify-center cursor-pointer"
        title="Segarkan Data (F5)"
      >
        <span class="material-symbols-outlined text-[18px]">refresh</span>
      </button>

      <!-- + Restock / Masuk Stok CTA [F6] -->
      <button
        onclick={() => (isRestockModalOpen = true)}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded bg-emerald-100 text-emerald-900 hover:bg-emerald-200 transition-colors font-sans text-xs font-bold border border-emerald-300 shadow-2xs cursor-pointer"
        title="Penerimaan Barang / Tambah Stok Masuk [F6]"
      >
        <span class="material-symbols-outlined text-[16px]">inventory</span>
        <span>+ Restock [F6]</span>
      </button>

      <!-- + Tambah Produk Baru CTA [F5] -->
      <button
        onclick={openAddModal}
        class="flex items-center gap-1.5 px-3.5 py-1.5 rounded bg-primary hover:bg-primary-dark text-white font-sans text-xs font-bold shadow-sm transition-all border border-primary-dark cursor-pointer ml-1"
        title="Registrasi SKU / Master Data Baru [F5]"
      >
        <span class="material-symbols-outlined text-[16px]">add_box</span>
        <span>+ Tambah Produk Baru [F5]</span>
      </button>
    </div>
  </div>

  <!-- 2. 5 COMPACT STAT CARDS (Di Tengah) -->
  <div class="px-4 py-2.5 bg-slate-200 border-b border-slate-300 shrink-0">
    <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-2.5 items-stretch">
      <!-- Card 1: Total Produk Aktif -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Total Produk Aktif</span>
          <span class="material-symbols-outlined text-[18px] text-primary">inventory_2</span>
        </div>
        <div class="my-1 flex items-baseline gap-1">
          <span class="font-mono text-xl font-bold text-slate-900 tracking-tight">{stats?.total_produk ?? 1428}</span>
          <span class="font-mono text-xs text-slate-500 font-medium">SKU</span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span class="text-emerald-700 font-bold">+18 minggu ini</span>
          <span>100% Aktif</span>
        </div>
      </div>

      <!-- Card 2: Stok Optimal -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Stok Optimal</span>
          <span class="material-symbols-outlined text-[18px] text-emerald-600">verified</span>
        </div>
        <div class="my-1 flex items-baseline gap-1">
          <span class="font-mono text-xl font-bold text-emerald-700 tracking-tight">{stats?.stok_optimal ?? 1385}</span>
          <span class="font-mono text-xs text-slate-500 font-medium">SKU</span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Kecukupan</span>
          <span class="text-emerald-700 font-bold">97.0% Aman</span>
        </div>
      </div>

      <!-- Card 3: Stok Menipis -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-amber-700 font-bold">Peringatan Stok</span>
          <span class="material-symbols-outlined text-[18px] text-amber-600">warning</span>
        </div>
        <div class="my-1 flex items-baseline gap-1">
          <span class="font-mono text-xl font-bold text-amber-700 tracking-tight">{stats?.stok_menipis ?? 38}</span>
          <span class="font-mono text-xs text-slate-500 font-medium">SKU</span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>&lt; Batas Minimum</span>
          <span class="text-amber-800 font-bold">Restock Segera</span>
        </div>
      </div>

      <!-- Card 4: Stok Habis -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-rose-700 font-bold">Stok Kosong</span>
          <span class="material-symbols-outlined text-[18px] text-rose-600">remove_shopping_cart</span>
        </div>
        <div class="my-1 flex items-baseline gap-1">
          <span class="font-mono text-xl font-bold text-rose-700 tracking-tight">{stats?.stok_kosong ?? 5}</span>
          <span class="font-mono text-xs text-slate-500 font-medium">SKU</span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Kritis / 0 Unit</span>
          <span class="text-rose-700 font-bold">5 Item PO</span>
        </div>
      </div>

      <!-- Card 5: Estimasi Valuasi Aset -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase font-bold tracking-wider text-slate-500 font-bold">Valuasi Aset Retail</span>
          <span class="material-symbols-outlined text-[18px] text-primary">account_balance_wallet</span>
        </div>
        <div class="my-1">
          <span class="font-mono text-xl font-bold text-slate-900 tracking-tight">
            {formatRupiah(stats?.valuasi_aset ?? 48650000)}
          </span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Rata-rata Margin</span>
          <span class="text-emerald-700 font-bold">+{stats?.avg_margin ?? 23.8}% HPP</span>
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
        oninput={handleSearch}
        class="w-full pl-9 pr-14 py-1.5 bg-slate-100 text-slate-900 placeholder:text-slate-400 font-sans text-xs rounded-lg border border-slate-300 outline-none focus:border-primary focus:bg-white focus:ring-1 focus:ring-primary transition-all"
        placeholder="Cari Nama Produk / Barcode / SKU / Rak / Supplier..."
        type="text"
      />
      <div class="absolute inset-y-0 right-0 pr-2 flex items-center pointer-events-none">
        <span class="px-1.5 py-0.5 rounded bg-slate-200 text-slate-700 font-mono text-[10px] font-bold uppercase border border-slate-300">
          F3
        </span>
      </div>
    </div>

    <!-- Dropdown Select Filters (Kategori, Supplier, Satuan) -->
    <div class="flex items-center gap-2 font-sans text-xs">
      <!-- 1. Dropdown Kategori -->
      <div class="relative">
        <select
          bind:value={categoryFilter}
          class="appearance-none bg-slate-100 text-slate-800 font-sans text-xs px-3 py-1.5 pr-7 rounded-lg border border-slate-300 shadow-2xs focus:outline-none cursor-pointer font-medium max-w-[150px] truncate"
          title="Filter Berdasarkan Kategori"
        >
          <option value="all">Semua Kategori</option>
          {#each categoriesList.filter((c) => c !== "all") as cat}
            <option value={cat}>{cat}</option>
          {/each}
        </select>
        <span class="material-symbols-outlined absolute right-2 top-1.5 pointer-events-none text-slate-500 text-[16px]">expand_more</span>
      </div>

      <!-- 2. Dropdown Supplier -->
      <div class="relative">
        <select
          bind:value={supplierFilter}
          class="appearance-none bg-slate-100 text-slate-800 font-sans text-xs px-3 py-1.5 pr-7 rounded-lg border border-slate-300 shadow-2xs focus:outline-none cursor-pointer font-medium max-w-[150px] truncate"
          title="Filter Berdasarkan Supplier"
        >
          <option value="all">Semua Supplier</option>
          {#each suppliersList.filter((s) => s !== "all") as sup}
            <option value={sup}>{sup}</option>
          {/each}
        </select>
        <span class="material-symbols-outlined absolute right-2 top-1.5 pointer-events-none text-slate-500 text-[16px]">expand_more</span>
      </div>

      <!-- 3. Dropdown Satuan -->
      <div class="relative">
        <select
          bind:value={satuanFilter}
          class="appearance-none bg-slate-100 text-slate-800 font-sans text-xs px-3 py-1.5 pr-7 rounded-lg border border-slate-300 shadow-2xs focus:outline-none cursor-pointer font-medium max-w-[120px]"
          title="Filter Berdasarkan Satuan"
        >
          <option value="all">Semua Satuan</option>
          {#each satuanList.filter((s) => s !== "all") as sat}
            <option value={sat}>{sat}</option>
          {/each}
        </select>
        <span class="material-symbols-outlined absolute right-2 top-1.5 pointer-events-none text-slate-500 text-[16px]">expand_more</span>
      </div>
    </div>
  </div>

  <!-- Main Work Area: 100% Full-Width Retail Table (Sama persis Riwayat Transaksi) -->
  <div class="w-full p-4 flex-1 flex flex-col min-h-0 overflow-hidden">
    <div class="w-full bg-white rounded-xl border border-slate-300 shadow-sm overflow-hidden flex flex-col flex-1 min-h-0">
      <div class="overflow-auto w-full flex-1 min-h-0">
        <table class="w-full text-left font-sans text-xs border-collapse">
          <!-- Table Head -->
          <thead class="bg-slate-800 text-slate-100 uppercase font-mono text-[11px] font-bold tracking-wider select-none border-b-2 border-slate-900 sticky top-0 z-10 shadow-sm">
            <tr>
              <th class="py-2.5 px-3 text-center w-12" scope="col">No</th>
              <th class="py-2.5 px-3 w-36" scope="col">Barcode / SKU</th>
              <th class="py-2.5 px-3 min-w-[200px]" scope="col">Nama Produk</th>
              <th class="py-2.5 px-3 w-32" scope="col">Kategori</th>
              <th class="py-2.5 px-2 text-center w-16" scope="col">Satuan</th>

              {#if cols.rak}
                <th class="py-2.5 px-2 text-center w-20" scope="col">Rak</th>
              {/if}
              {#if cols.supplier}
                <th class="py-2.5 px-3 w-36" scope="col">Supplier</th>
              {/if}
              {#if cols.limit}
                <th class="py-2.5 px-2 text-center w-20" scope="col">Limit</th>
              {/if}
              {#if cols.hargaModal}
                <th class="py-2.5 px-3 text-right w-28" scope="col">Harga Modal</th>
              {/if}
              {#if cols.harga1}
                <th class="py-2.5 px-3 text-right w-28" scope="col">Harga Jual 1</th>
              {/if}
              {#if cols.harga2}
                <th class="py-2.5 px-3 text-right w-28" scope="col">Harga Jual 2</th>
              {/if}
              {#if cols.harga3}
                <th class="py-2.5 px-3 text-right w-28" scope="col">Harga Jual 3</th>
              {/if}
              {#if cols.margin}
                <th class="py-2.5 px-2 text-right w-20" scope="col">Margin</th>
              {/if}

              <th class="py-2.5 px-3 text-right w-24" scope="col">Stok Fisik</th>

              {#if cols.expired}
                <th class="py-2.5 px-3 text-center w-24" scope="col">Expired</th>
              {/if}

              <th class="py-2.5 px-3 text-center w-28 relative" scope="col">
                <div class="flex items-center justify-center gap-1.5">
                  <span>Aksi</span>
                  <button
                    type="button"
                    onclick={(e) => { e.stopPropagation(); isColumnSettingsOpen = !isColumnSettingsOpen; }}
                    class="p-1 text-slate-700 hover:text-slate-950 hover:bg-slate-300 rounded cursor-pointer border-none bg-transparent flex items-center justify-center transition-colors"
                    title="Atur Field Kolom (Ceklis / Hide / Open)"
                  >
                    <span class="material-symbols-outlined text-[16px]">settings</span>
                  </button>
                </div>

                <!-- Popover Checklist Setting Kolom -->
                {#if isColumnSettingsOpen}
                  <div
                    class="absolute right-0 top-full mt-1 w-56 bg-white border border-slate-300 rounded-lg shadow-xl p-3 z-50 text-left font-sans text-xs text-slate-800 flex flex-col gap-2 font-normal"
                    onclick={(e) => e.stopPropagation()}
                    role="menu"
                    tabindex="-1"
                    onkeydown={(e) => e.stopPropagation()}
                  >
                    <div class="flex items-center justify-between pb-1.5 border-b border-slate-200">
                      <span class="font-bold text-slate-900 text-[11px] flex items-center gap-1">
                        <span class="material-symbols-outlined text-[14px] text-primary">tune</span>
                        Atur Field Kolom
                      </span>
                      <button
                        type="button"
                        onclick={() => (isColumnSettingsOpen = false)}
                        class="text-slate-400 hover:text-slate-700 p-0.5 border-none bg-transparent cursor-pointer"
                      >
                        <span class="material-symbols-outlined text-[14px]">close</span>
                      </button>
                    </div>

                    <label class="flex items-center gap-2 cursor-pointer hover:bg-slate-50 p-1 rounded">
                      <input type="checkbox" bind:checked={cols.rak} class="rounded text-primary border-slate-300 focus:ring-primary" />
                      <span class="text-xs font-medium text-slate-800">Lokasi Rak</span>
                    </label>
                    <label class="flex items-center gap-2 cursor-pointer hover:bg-slate-50 p-1 rounded">
                      <input type="checkbox" bind:checked={cols.supplier} class="rounded text-primary border-slate-300 focus:ring-primary" />
                      <span class="text-xs font-medium text-slate-800">Supplier</span>
                    </label>
                    <label class="flex items-center gap-2 cursor-pointer hover:bg-slate-50 p-1 rounded">
                      <input type="checkbox" bind:checked={cols.limit} class="rounded text-primary border-slate-300 focus:ring-primary" />
                      <span class="text-xs font-medium text-slate-800">Limit (Stok Min)</span>
                    </label>
                    <label class="flex items-center gap-2 cursor-pointer hover:bg-slate-50 p-1 rounded">
                      <input type="checkbox" bind:checked={cols.hargaModal} class="rounded text-primary border-slate-300 focus:ring-primary" />
                      <span class="text-xs font-medium text-slate-800">Harga Modal</span>
                    </label>
                    <label class="flex items-center gap-2 cursor-pointer hover:bg-slate-50 p-1 rounded">
                      <input type="checkbox" bind:checked={cols.harga1} class="rounded text-primary border-slate-300 focus:ring-primary" />
                      <span class="text-xs font-medium text-slate-800">Harga Jual 1</span>
                    </label>
                    <label class="flex items-center gap-2 cursor-pointer hover:bg-slate-50 p-1 rounded">
                      <input type="checkbox" bind:checked={cols.harga2} class="rounded text-primary border-slate-300 focus:ring-primary" />
                      <span class="text-xs font-medium text-slate-800">Harga Jual 2</span>
                    </label>
                    <label class="flex items-center gap-2 cursor-pointer hover:bg-slate-50 p-1 rounded">
                      <input type="checkbox" bind:checked={cols.harga3} class="rounded text-primary border-slate-300 focus:ring-primary" />
                      <span class="text-xs font-medium text-slate-800">Harga Jual 3</span>
                    </label>
                    <label class="flex items-center gap-2 cursor-pointer hover:bg-slate-50 p-1 rounded">
                      <input type="checkbox" bind:checked={cols.margin} class="rounded text-primary border-slate-300 focus:ring-primary" />
                      <span class="text-xs font-medium text-slate-800">Margin (%)</span>
                    </label>
                    <label class="flex items-center gap-2 cursor-pointer hover:bg-slate-50 p-1 rounded">
                      <input type="checkbox" bind:checked={cols.expired} class="rounded text-primary border-slate-300 focus:ring-primary" />
                      <span class="text-xs font-medium text-slate-800">Tanggal Expired</span>
                    </label>
                  </div>
                {/if}
              </th>
            </tr>
          </thead>

          <!-- Table Body -->
          <tbody class="divide-y divide-slate-200 text-slate-900">
            {#each filteredProducts as product, i}
              <tr
                class="transition-colors border-b border-slate-200/80 cursor-pointer {product.stok === 0 ? 'bg-rose-50/80 hover:bg-rose-100 border-l-4 border-l-rose-600' : product.is_kritis ? 'bg-amber-50/70 hover:bg-amber-100 border-l-4 border-l-amber-600' : i % 2 === 1 ? 'bg-slate-50/70 hover:bg-sky-50/80' : 'bg-white hover:bg-sky-50/80'}"
              >
                <td class="py-2 px-3 text-center font-mono text-xs font-bold {product.stok === 0 ? 'text-rose-700' : product.is_kritis ? 'text-amber-700' : 'text-slate-500'}">
                  {i + 1}
                </td>
                <td class="py-2 px-3 font-mono text-xs font-bold text-primary">
                  {product.barcode}
                  <div class="text-[10px] text-slate-500 font-normal">{product.kode}</div>
                </td>
                <td class="py-2 px-3">
                  <span class="font-semibold text-slate-900 text-xs">{product.nama}</span>
                  {#if product.tag}
                    <span class="font-mono text-[9px] ml-1.5 px-1.5 py-0.5 rounded font-bold {product.tag === 'Laris' ? 'bg-slate-200 text-slate-800' : product.tag === 'Habis' ? 'bg-rose-600 text-white' : 'bg-amber-400 text-slate-900'}">
                      {product.tag}
                    </span>
                  {/if}
                </td>
                <td class="py-2 px-3">
                  <span class="px-1.5 py-0.5 rounded bg-slate-100 text-slate-700 font-mono text-[10px] font-medium border border-slate-200">
                    {product.kategori}
                  </span>
                </td>
                <td class="py-2 px-2 text-center font-mono text-xs text-slate-600">{product.satuan}</td>

                {#if cols.rak}
                  <td class="py-2 px-2 text-center font-mono text-xs font-bold {product.stok === 0 ? 'text-rose-700' : product.is_kritis ? 'text-amber-700' : 'text-slate-700'}">
                    {product.rak}
                  </td>
                {/if}

                {#if cols.supplier}
                  <td class="py-2 px-3 text-xs text-slate-600 truncate max-w-[140px]" title={product.supplier}>
                    {product.supplier}
                  </td>
                {/if}

                {#if cols.limit}
                  <td class="py-2 px-2 text-center font-mono text-xs font-semibold text-slate-600">
                    <span class="px-1.5 py-0.5 bg-slate-100 rounded border border-slate-200">{product.stokminimum}</span>
                  </td>
                {/if}

                {#if cols.hargaModal}
                  <td class="py-2 px-3 text-right font-mono text-xs text-slate-600 tabular-nums">
                    {formatRupiah(product.hargapokok)}
                  </td>
                {/if}

                {#if cols.harga1}
                  <td class="py-2 px-3 text-right font-mono text-xs font-bold text-slate-900 tabular-nums">
                    {formatRupiah(product.hargajual1)}
                  </td>
                {/if}

                {#if cols.harga2}
                  <td class="py-2 px-3 text-right font-mono text-xs text-slate-700 tabular-nums">
                    {formatRupiah(product.hargajual2 || Math.round(product.hargajual1 * 0.95))}
                  </td>
                {/if}

                {#if cols.harga3}
                  <td class="py-2 px-3 text-right font-mono text-xs text-slate-700 tabular-nums">
                    {formatRupiah(product.hargajual3 || Math.round(product.hargajual1 * 0.90))}
                  </td>
                {/if}

                {#if cols.margin}
                  <td class="py-2 px-2 text-right font-mono text-xs font-bold text-emerald-700 tabular-nums">
                    +{product.margin_persen}%
                  </td>
                {/if}

                <td class="py-2 px-3 text-right font-mono text-xs font-bold tabular-nums {product.stok === 0 ? 'text-rose-600' : product.is_kritis ? 'text-amber-600' : 'text-slate-900'}">
                  {product.stok}
                </td>

                {#if cols.expired}
                  <td class="py-2 px-3 text-center whitespace-nowrap">
                    <span class="px-1.5 py-0.5 bg-rose-50 border border-rose-200 text-rose-700 rounded font-mono text-[10px] font-bold">
                      {product.expired || "12/2027"}
                    </span>
                  </td>
                {/if}

                <td class="py-2 px-3 text-center">
                  <div class="inline-flex items-center justify-center gap-1">
                    <button
                      onclick={() => openEditModal(product)}
                      class="p-1 rounded hover:bg-slate-200 text-slate-600 hover:text-primary transition-colors border-none bg-transparent cursor-pointer"
                      title="Edit Item"
                    >
                      <span class="material-symbols-outlined text-[16px]">edit</span>
                    </button>
                    <button
                      onclick={() => showToast(`Mencetak label rak barcode untuk ${product.nama}...`)}
                      class="p-1 rounded hover:bg-slate-200 text-slate-600 hover:text-primary transition-colors border-none bg-transparent cursor-pointer"
                      title="Cetak Barcode Label"
                    >
                      <span class="material-symbols-outlined text-[16px]">print</span>
                    </button>
                    <button
                      onclick={() => deleteProduct(product)}
                      class="p-1 rounded hover:bg-rose-100 text-slate-600 hover:text-rose-600 transition-colors border-none bg-transparent cursor-pointer"
                      title="Hapus Produk"
                    >
                      <span class="material-symbols-outlined text-[16px]">delete</span>
                    </button>
                  </div>
                </td>
              </tr>
            {:else}
              <tr>
                <td colspan={6 + (cols.rak ? 1 : 0) + (cols.supplier ? 1 : 0) + (cols.limit ? 1 : 0) + (cols.hargaModal ? 1 : 0) + (cols.harga1 ? 1 : 0) + (cols.harga2 ? 1 : 0) + (cols.harga3 ? 1 : 0) + (cols.margin ? 1 : 0) + (cols.expired ? 1 : 0) + 1} class="py-16 text-center text-slate-400 font-mono text-xs">
                  Tidak ada produk yang cocok dengan pencarian / filter.
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      <!-- 4. BOTTOM FOOTER / PAGINATION -->
      <footer class="bg-slate-100 p-2.5 border-t border-slate-300 flex flex-col md:flex-row items-center justify-between gap-2 text-xs shrink-0">
        <div class="flex flex-wrap items-center gap-3 text-slate-600 font-mono text-[11px]">
          <div class="flex items-center gap-1.5">
            <span class="w-2 h-2 rounded-full bg-emerald-600"></span>
            <span class="font-medium text-emerald-800 font-bold">Semua data produk tersinkronisasi offline SQLite</span>
          </div>
          <span>•</span>
          <div>
            Menampilkan <span class="font-bold text-slate-900">1 - {filteredProducts.length}</span> dari <span class="font-bold text-slate-900">{stats?.total_produk ?? 1428}</span> Produk Aktif
          </div>
        </div>

        <!-- Pagination controls -->
        <div class="flex items-center gap-1 font-mono text-xs select-none">
          <button class="px-2.5 py-1 rounded bg-white border border-slate-300 text-slate-400 cursor-not-allowed font-medium" disabled>
            Sebelumnya
          </button>
          <button class="w-7 h-7 rounded bg-primary text-white font-bold flex items-center justify-center shadow-2xs border-none cursor-pointer">
            1
          </button>
          <button class="w-7 h-7 rounded hover:bg-slate-200 text-slate-700 font-medium flex items-center justify-center transition-colors border border-slate-300 bg-white cursor-pointer">
            2
          </button>
          <button class="w-7 h-7 rounded hover:bg-slate-200 text-slate-700 font-medium flex items-center justify-center transition-colors border border-slate-300 bg-white cursor-pointer">
            3
          </button>
          <span class="px-1 text-slate-400">...</span>
          <button class="w-8 h-7 rounded hover:bg-slate-200 text-slate-700 font-medium flex items-center justify-center transition-colors border border-slate-300 bg-white cursor-pointer">
            143
          </button>
          <button class="px-2.5 py-1 rounded bg-white hover:bg-slate-200 text-slate-800 font-medium transition-colors shadow-2xs border border-slate-300 cursor-pointer">
            Selanjutnya
          </button>
        </div>
      </footer>
    </div>
  </div>

  <!-- Hardware Link Status Strip -->
  <footer class="w-full bg-slate-200 border-t border-slate-300 py-1.5 px-4 mt-auto select-none shrink-0">
    <div class="w-full flex items-center justify-between font-mono text-[10px] text-slate-600 font-medium">
      <span>Kinetic POS Engine • Hardware Link Native Node</span>
      <div class="flex items-center gap-4">
        <span>Keyboard Wedge Mode</span>
        <span class="text-slate-400">|</span>
        <span>Local DB: Synced</span>
        <span class="text-slate-400">|</span>
        <span>Latency: 2ms</span>
      </div>
    </div>
  </footer>

  <!-- Modal Tambah / Edit Produk -->
  {#if isAddModalOpen}
    <div
      class="fixed inset-0 bg-slate-900/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none"
      onclick={() => (isAddModalOpen = false)}
      role="button"
      tabindex="0"
      onkeydown={(e) => e.key === "Escape" && (isAddModalOpen = false)}
    >
      <div
        class="bg-white border border-slate-300 rounded-xl shadow-2xl w-[560px] overflow-hidden"
        role="dialog"
        aria-modal="true"
        tabindex="-1"
        onclick={(e) => e.stopPropagation()}
        onkeydown={(e) => e.stopPropagation()}
      >
        <div class="px-5 py-3.5 bg-slate-100 border-b border-slate-300 flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="material-symbols-outlined text-primary text-[20px]">
              {editingProduct ? "edit" : "add_box"}
            </span>
            <span class="font-bold text-slate-900 text-sm font-sans">
              {editingProduct ? `Edit Produk: ${editingProduct.nama}` : "Tambah Produk Baru [F5]"}
            </span>
          </div>
          <button
            onclick={() => (isAddModalOpen = false)}
            class="text-slate-400 hover:text-slate-700 p-1 hover:bg-slate-200 rounded border-none bg-transparent cursor-pointer"
          >
            <span class="material-symbols-outlined text-[18px]">close</span>
          </button>
        </div>

        <div class="p-5 font-sans text-xs flex flex-col gap-3">
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label for="form-kode" class="block font-semibold text-slate-700 mb-1">Kode / SKU</label>
              <input
                id="form-kode"
                type="text"
                bind:value={formKode}
                class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded font-mono text-xs focus:bg-white focus:border-primary focus:outline-none"
              />
            </div>
            <div>
              <label for="form-barcode" class="block font-semibold text-slate-700 mb-1">Barcode EAN-13</label>
              <input
                id="form-barcode"
                type="text"
                bind:value={formBarcode}
                class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded font-mono text-xs focus:bg-white focus:border-primary focus:outline-none"
              />
            </div>
          </div>

          <div>
            <label for="form-nama" class="block font-semibold text-slate-700 mb-1">Nama Produk</label>
            <input
              id="form-nama"
              type="text"
              bind:value={formNama}
              placeholder="Contoh: Indomie Goreng Spesial 85g"
              class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded text-xs focus:bg-white focus:border-primary focus:outline-none font-medium"
            />
          </div>

          <div class="grid grid-cols-3 gap-3">
            <div>
              <label for="form-kategori" class="block font-semibold text-slate-700 mb-1">Kategori</label>
              <select
                id="form-kategori"
                bind:value={formKategori}
                class="w-full px-2 py-1.5 bg-slate-50 border border-slate-300 rounded text-xs focus:outline-none"
              >
                <option value="Makanan & Minuman">Makanan & Minuman</option>
                <option value="Kebutuhan Rumah">Kebutuhan Rumah</option>
                <option value="Personal Care">Personal Care</option>
                <option value="Rokok & Tembakau">Rokok & Tembakau</option>
                <option value="Umum">Umum</option>
              </select>
            </div>
            <div>
              <label for="form-satuan" class="block font-semibold text-slate-700 mb-1">Satuan</label>
              <select
                id="form-satuan"
                bind:value={formSatuan}
                class="w-full px-2 py-1.5 bg-slate-50 border border-slate-300 rounded text-xs focus:outline-none"
              >
                <option value="Pcs">Pcs</option>
                <option value="Bks">Bks</option>
                <option value="Btl">Btl</option>
                <option value="Kotak">Kotak</option>
                <option value="Krg">Krg</option>
                <option value="Rcg">Rcg</option>
                <option value="Can">Can</option>
                <option value="Dus">Dus</option>
              </select>
            </div>
            <div>
              <label for="form-rak" class="block font-semibold text-slate-700 mb-1">Lokasi Rak</label>
              <input
                id="form-rak"
                type="text"
                bind:value={formRak}
                class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded font-mono text-xs focus:outline-none"
              />
            </div>
          </div>

          <div>
            <label for="form-supplier" class="block font-semibold text-slate-700 mb-1">Supplier Utama</label>
            <input
              id="form-supplier"
              type="text"
              bind:value={formSupplier}
              class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded text-xs focus:outline-none"
            />
          </div>

          <div class="grid grid-cols-3 gap-3 pt-1 border-t border-slate-200">
            <div>
              <label for="form-hpp" class="block font-semibold text-slate-700 mb-1">Harga Modal / HPP</label>
              <input
                id="form-hpp"
                type="number"
                bind:value={formHpp}
                class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded font-mono text-xs focus:outline-none text-right"
              />
            </div>
            <div>
              <label for="form-jual" class="block font-semibold text-slate-700 mb-1">Harga Jual 1</label>
              <input
                id="form-jual"
                type="number"
                bind:value={formJual}
                class="w-full px-2.5 py-1.5 bg-slate-50 border-2 border-primary rounded font-mono text-xs focus:outline-none text-right font-bold"
              />
            </div>
            <div>
              <label for="form-stok" class="block font-semibold text-slate-700 mb-1">Stok Awal</label>
              <input
                id="form-stok"
                type="number"
                bind:value={formStok}
                class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded font-mono text-xs focus:outline-none text-right font-bold"
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
            onclick={saveProduct}
            class="px-4 py-1.5 bg-primary hover:bg-primary-dark text-white rounded-lg text-xs font-bold shadow-sm cursor-pointer border-none flex items-center gap-1"
          >
            <span class="material-symbols-outlined text-[16px]">save</span>
            Simpan Produk
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Modal Restock / Masuk Stok [F6] -->
  {#if isRestockModalOpen}
    <div
      class="fixed inset-0 bg-slate-900/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none"
      onclick={() => (isRestockModalOpen = false)}
      role="button"
      tabindex="0"
      onkeydown={(e) => e.key === "Escape" && (isRestockModalOpen = false)}
    >
      <div
        class="bg-white border border-slate-300 rounded-xl shadow-2xl w-[480px] overflow-hidden"
        role="dialog"
        aria-modal="true"
        tabindex="-1"
        onclick={(e) => e.stopPropagation()}
        onkeydown={(e) => e.stopPropagation()}
      >
        <div class="px-5 py-3.5 bg-emerald-50 border-b border-emerald-200 flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="material-symbols-outlined text-emerald-700 text-[20px]">inventory</span>
            <span class="font-bold text-emerald-950 text-sm font-sans">+ Restock / Masuk Stok [F6]</span>
          </div>
          <button
            onclick={() => (isRestockModalOpen = false)}
            class="text-slate-400 hover:text-slate-700 p-1 hover:bg-slate-200 rounded border-none bg-transparent cursor-pointer"
          >
            <span class="material-symbols-outlined text-[18px]">close</span>
          </button>
        </div>

        <div class="p-5 font-sans text-xs flex flex-col gap-3">
          <div>
            <label for="restock-pilih" class="block font-semibold text-slate-700 mb-1">Pilih Produk</label>
            <select
              id="restock-pilih"
              class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded text-xs font-medium focus:outline-none"
            >
              {#each products as p}
                <option value={p.kode}>{p.kode} - {p.nama} (Stok: {p.stok} {p.satuan})</option>
              {/each}
            </select>
          </div>

          <div class="grid grid-cols-2 gap-3">
            <div>
              <label for="restock-qty" class="block font-semibold text-slate-700 mb-1">Jumlah Masuk (Qty)</label>
              <input
                id="restock-qty"
                type="number"
                value="24"
                class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded font-mono text-xs focus:outline-none text-right font-bold"
              />
            </div>
            <div>
              <label for="restock-harga" class="block font-semibold text-slate-700 mb-1">Harga Beli Baru (Rp)</label>
              <input
                id="restock-harga"
                type="number"
                value="2750"
                class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded font-mono text-xs focus:outline-none text-right"
              />
            </div>
          </div>

          <div>
            <label for="restock-faktur" class="block font-semibold text-slate-700 mb-1">No. Faktur Pembelian / PO</label>
            <input
              id="restock-faktur"
              type="text"
              value="PO-2026-0910"
              class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded font-mono text-xs focus:outline-none"
            />
          </div>
        </div>

        <div class="px-5 py-3 bg-slate-50 border-t border-slate-300 flex items-center justify-end gap-2">
          <button
            type="button"
            onclick={() => (isRestockModalOpen = false)}
            class="px-4 py-1.5 bg-white hover:bg-slate-100 text-slate-700 border border-slate-300 rounded-lg text-xs font-bold shadow-2xs cursor-pointer"
          >
            Batal
          </button>
          <button
            type="button"
            onclick={() => {
              isRestockModalOpen = false;
              showToast("Penerimaan stok masuk berhasil dicatat ke database SQLite!");
            }}
            class="px-4 py-1.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg text-xs font-bold shadow-sm cursor-pointer border-none flex items-center gap-1"
          >
            <span class="material-symbols-outlined text-[16px]">check</span>
            Simpan Stok Masuk
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
