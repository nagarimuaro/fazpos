<script lang="ts">
  import { formatRupiah } from "../lib/api";

  interface CashflowItem {
    id: string;
    no_bukti: string;
    tanggal: string;
    waktu: string;
    jenis: "KELUAR" | "MASUK"; // KELUAR = Beban/Pengeluaran, MASUK = Kas Masuk/Pemasukan
    kategori: string;
    kategori_label: string;
    keterangan: string;
    pic: string;
    operator: string;
    nominal: number;
    metode: "TUNAI" | "TRANSFER";
  }

  // State
  let activeTab = $state<"SEMUA" | "KELUAR" | "MASUK">("SEMUA");
  let filterKategori = $state<string>("SEMUA");
  let filterPeriode = $state<"HARI_INI" | "7_HARI" | "BULAN_INI">("HARI_INI");
  let searchQuery = $state("");
  let isAddModalOpen = $state(false);

  // Form Tambah Biaya/Kas Baru
  let newJenis = $state<"KELUAR" | "MASUK">("KELUAR");
  let newKategori = $state("LISTRIK_AIR_WIFI");
  let newNominal = $state<number | string>("");
  let newKeterangan = $state("");
  let newPic = $state("");
  let newMetode = $state<"TUNAI" | "TRANSFER">("TUNAI");

  const posKategoriList = [
    { value: "LISTRIK_AIR_WIFI", label: "Listrik, Air & Internet", icon: "bolt", defaultJenis: "KELUAR" },
    { value: "GAJI", label: "Gaji, Upah & Lembur", icon: "badge", defaultJenis: "KELUAR" },
    { value: "KONSUMSI", label: "Konsumsi & Pantry", icon: "restaurant", defaultJenis: "KELUAR" },
    { value: "ATK_PERLENGKAPAN", label: "ATK & Kertas Struk", icon: "inventory", defaultJenis: "KELUAR" },
    { value: "TRANSPORTASI", label: "Bensin & Transportasi", icon: "local_shipping", defaultJenis: "KELUAR" },
    { value: "MAINTENANCE", label: "Perbaikan & Kebersihan", icon: "build", defaultJenis: "KELUAR" },
    { value: "SEWA_TEMPAT", label: "Sewa Tempat & Ruko", icon: "store", defaultJenis: "KELUAR" },
    { value: "MODAL_KASIR", label: "Modal Kasir / Tambahan Kas", icon: "savings", defaultJenis: "MASUK" },
    { value: "PENDAPATAN_LAIN", label: "Pendapatan Lain-lain", icon: "add_circle", defaultJenis: "MASUK" },
    { value: "LAINNYA", label: "Beban Operasional Lain", icon: "more_horiz", defaultJenis: "KELUAR" },
  ];

  let items = $state<CashflowItem[]>([
    {
      id: "cf-1",
      no_bukti: "#BOP-2026-0914-01",
      tanggal: "14/09/2026",
      waktu: "08:15:00",
      jenis: "MASUK",
      kategori: "MODAL_KASIR",
      kategori_label: "Modal Kasir / Tambahan Kas",
      keterangan: "Penyediaan uang kembalian modal kasir Shift 01",
      pic: "Alexander P.",
      operator: "Administrator",
      nominal: 500000,
      metode: "TUNAI",
    },
    {
      id: "cf-2",
      no_bukti: "#BOP-2026-0914-02",
      tanggal: "14/09/2026",
      waktu: "10:30:20",
      jenis: "KELUAR",
      kategori: "KONSUMSI",
      kategori_label: "Konsumsi & Pantry",
      keterangan: "Beli air galon 2 pcs & konsumsi makan siang shift pagi",
      pic: "Siti Rahma",
      operator: "Alexander P.",
      nominal: 68000,
      metode: "TUNAI",
    },
    {
      id: "cf-3",
      no_bukti: "#BOP-2026-0914-03",
      tanggal: "14/09/2026",
      waktu: "13:45:10",
      jenis: "KELUAR",
      kategori: "ATK_PERLENGKAPAN",
      kategori_label: "ATK & Kertas Struk",
      keterangan: "Isi ulang kertas thermal 80mm (1 pack / 10 roll) + lakban",
      pic: "Rudi Hartono",
      operator: "Alexander P.",
      nominal: 125000,
      metode: "TUNAI",
    },
    {
      id: "cf-4",
      no_bukti: "#BOP-2026-0914-04",
      tanggal: "14/09/2026",
      waktu: "15:20:00",
      jenis: "MASUK",
      kategori: "PENDAPATAN_LAIN",
      kategori_label: "Pendapatan Lain-lain",
      keterangan: "Bagi hasil komisi top-up e-money & jasa titip",
      pic: "Alexander P.",
      operator: "Alexander P.",
      nominal: 185000,
      metode: "TUNAI",
    },
    {
      id: "cf-5",
      no_bukti: "#BOP-2026-0913-01",
      tanggal: "13/09/2026",
      waktu: "11:10:00",
      jenis: "KELUAR",
      kategori: "LISTRIK_AIR_WIFI",
      kategori_label: "Listrik, Air & Internet",
      keterangan: "Beli token listrik PLN 100k toko depan & kasir",
      pic: "Bambang W.",
      operator: "Administrator",
      nominal: 202500,
      metode: "TRANSFER",
    },
    {
      id: "cf-6",
      no_bukti: "#BOP-2026-0912-01",
      tanggal: "12/09/2026",
      waktu: "16:40:00",
      jenis: "KELUAR",
      kategori: "MAINTENANCE",
      kategori_label: "Perbaikan & Kebersihan",
      keterangan: "Cuci filter AC split kasir & beli cairan pembersih lantai",
      pic: "Joko AC",
      operator: "Administrator",
      nominal: 150000,
      metode: "TUNAI",
    },
    {
      id: "cf-7",
      no_bukti: "#BOP-2026-0910-01",
      tanggal: "10/09/2026",
      waktu: "14:00:00",
      jenis: "KELUAR",
      kategori: "TRANSPORTASI",
      kategori_label: "Bensin & Transportasi",
      keterangan: "Penggantian uang bensin motor kirim pesanan grosir langganan",
      pic: "Andi Saputra",
      operator: "Alexander P.",
      nominal: 45000,
      metode: "TUNAI",
    },
  ]);

  // Statistik Dinamis
  let totalMasuk = $derived(
    items.filter((i) => i.jenis === "MASUK").reduce((acc, i) => acc + i.nominal, 0)
  );
  let totalKeluar = $derived(
    items.filter((i) => i.jenis === "KELUAR").reduce((acc, i) => acc + i.nominal, 0)
  );
  let saldoBersih = $derived(totalMasuk - totalKeluar);

  // Filter Data
  let filteredItems = $derived(
    items.filter((item) => {
      // Filter Tab Jenis
      if (activeTab === "KELUAR" && item.jenis !== "KELUAR") return false;
      if (activeTab === "MASUK" && item.jenis !== "MASUK") return false;

      // Filter Pos Beban
      if (filterKategori !== "SEMUA" && item.kategori !== filterKategori) return false;

      // Search
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase();
        const matchKet = item.keterangan.toLowerCase().includes(q);
        const matchBukti = item.no_bukti.toLowerCase().includes(q);
        const matchKat = item.kategori_label.toLowerCase().includes(q);
        const matchPic = item.pic.toLowerCase().includes(q);
        if (!matchKet && !matchBukti && !matchKat && !matchPic) return false;
      }

      return true;
    })
  );

  function handleSimpan() {
    const nom = Number(newNominal);
    if (!nom || nom <= 0) {
      alert("Masukkan nominal biaya/kas yang valid!");
      return;
    }
    if (!newKeterangan.trim()) {
      alert("Masukkan keterangan pengeluaran/pemasukan!");
      return;
    }

    const katObj = posKategoriList.find((k) => k.value === newKategori);
    const now = new Date();
    const tgl = `${String(now.getDate()).padStart(2, "0")}/${String(now.getMonth() + 1).padStart(2, "0")}/${now.getFullYear()}`;
    const jam = `${String(now.getHours()).padStart(2, "0")}:${String(now.getMinutes()).padStart(2, "0")}:${String(now.getSeconds()).padStart(2, "0")}`;

    const newItem: CashflowItem = {
      id: `cf-${Date.now()}`,
      no_bukti: `#BOP-${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, "0")}${String(now.getDate()).padStart(2, "0")}-${String(items.length + 1).padStart(2, "0")}`,
      tanggal: tgl,
      waktu: jam,
      jenis: newJenis,
      kategori: newKategori,
      kategori_label: katObj ? katObj.label : "Lain-lain",
      keterangan: newKeterangan.trim(),
      pic: newPic.trim() || "Staff Kasir",
      operator: "Alexander P.",
      nominal: nom,
      metode: newMetode,
    };

    items = [newItem, ...items];

    // Reset
    newNominal = "";
    newKeterangan = "";
    newPic = "";
    isAddModalOpen = false;
  }
</script>

<div class="flex-1 flex flex-col bg-slate-200 overflow-hidden font-sans select-none">
  <!-- SUB-HEADER TITLE & ACTION BAR -->
  <div class="bg-slate-100 border-b border-slate-300 px-3 sm:px-5 py-2.5 sm:py-3 flex flex-col sm:flex-row sm:items-center justify-between gap-2 shadow-xs shrink-0">
    <div class="flex items-center gap-2.5">
      <div class="w-9 h-9 sm:w-10 sm:h-10 rounded-xl bg-sky-600 text-white flex items-center justify-center shadow-xs shrink-0">
        <span class="material-symbols-outlined text-[22px] sm:text-[24px]">payments</span>
      </div>
      <div>
        <div class="flex items-center gap-2">
          <h1 class="text-base sm:text-lg font-black text-slate-900 uppercase tracking-tight leading-tight">
            Buku Kas &amp; Biaya Operasional
          </h1>
          <span class="text-[10px] font-mono font-bold px-1.5 py-0.5 rounded bg-sky-100 text-sky-800 border border-sky-300">
            CASHFLOW
          </span>
        </div>
        <div class="text-[11px] text-slate-500 font-medium">
          Pencatatan pos beban operasional toko, kas masuk/keluar non-penjualan &amp; modal shift
        </div>
      </div>
    </div>

    <!-- Tombol Catat Pengeluaran / Kas Masuk -->
    <div class="flex items-center gap-2">
      <button
        onclick={() => {
          newJenis = "KELUAR";
          newKategori = "LISTRIK_AIR_WIFI";
          isAddModalOpen = true;
        }}
        class="flex-1 sm:flex-none px-3.5 py-2 rounded-xl bg-red-600 hover:bg-red-700 active:bg-red-800 text-white font-bold text-xs flex items-center justify-center gap-1.5 cursor-pointer transition-colors shadow-xs"
      >
        <span class="material-symbols-outlined text-[16px]">remove_circle</span>
        <span>Catat Beban Biaya</span>
      </button>

      <button
        onclick={() => {
          newJenis = "MASUK";
          newKategori = "MODAL_KASIR";
          isAddModalOpen = true;
        }}
        class="flex-1 sm:flex-none px-3.5 py-2 rounded-xl bg-emerald-600 hover:bg-emerald-700 active:bg-emerald-800 text-white font-bold text-xs flex items-center justify-center gap-1.5 cursor-pointer transition-colors shadow-xs"
      >
        <span class="material-symbols-outlined text-[16px]">add_circle</span>
        <span>Kas Masuk / Modal</span>
      </button>
    </div>
  </div>

  <!-- SUMMARY STATS WIDGETS -->
  <div class="p-2 sm:p-3 pb-0 grid grid-cols-2 lg:grid-cols-4 gap-2 shrink-0">
    <!-- Card 1: Total Pengeluaran Beban -->
    <div class="bg-white p-2.5 sm:p-3 rounded-xl border border-slate-300 shadow-2xs flex items-center gap-3">
      <div class="w-9 h-9 rounded-lg bg-red-100 text-red-600 flex items-center justify-center shrink-0">
        <span class="material-symbols-outlined text-[20px]">trending_down</span>
      </div>
      <div class="min-w-0">
        <div class="text-[10px] text-slate-500 font-bold uppercase tracking-wider">Total Beban Keluar</div>
        <div class="text-sm sm:text-base font-black text-red-600 font-mono truncate">
          {formatRupiah(totalKeluar)}
        </div>
      </div>
    </div>

    <!-- Card 2: Total Kas Masuk -->
    <div class="bg-white p-2.5 sm:p-3 rounded-xl border border-slate-300 shadow-2xs flex items-center gap-3">
      <div class="w-9 h-9 rounded-lg bg-emerald-100 text-emerald-600 flex items-center justify-center shrink-0">
        <span class="material-symbols-outlined text-[20px]">trending_up</span>
      </div>
      <div class="min-w-0">
        <div class="text-[10px] text-slate-500 font-bold uppercase tracking-wider">Kas Masuk Lainnya</div>
        <div class="text-sm sm:text-base font-black text-emerald-600 font-mono truncate">
          {formatRupiah(totalMasuk)}
        </div>
      </div>
    </div>

    <!-- Card 3: Saldo Kas Operasional Bersih -->
    <div class="bg-white p-2.5 sm:p-3 rounded-xl border border-slate-300 shadow-2xs flex items-center gap-3">
      <div class="w-9 h-9 rounded-lg bg-sky-100 text-sky-600 flex items-center justify-center shrink-0">
        <span class="material-symbols-outlined text-[20px]">account_balance_wallet</span>
      </div>
      <div class="min-w-0">
        <div class="text-[10px] text-slate-500 font-bold uppercase tracking-wider">Arus Kas Bersih</div>
        <div class="text-sm sm:text-base font-black {saldoBersih >= 0 ? 'text-sky-700' : 'text-red-600'} font-mono truncate">
          {formatRupiah(saldoBersih)}
        </div>
      </div>
    </div>

    <!-- Card 4: Total Record Transaksi -->
    <div class="bg-white p-2.5 sm:p-3 rounded-xl border border-slate-300 shadow-2xs flex items-center gap-3">
      <div class="w-9 h-9 rounded-lg bg-slate-100 text-slate-700 flex items-center justify-center shrink-0">
        <span class="material-symbols-outlined text-[20px]">receipt</span>
      </div>
      <div class="min-w-0">
        <div class="text-[10px] text-slate-500 font-bold uppercase tracking-wider">Total Transaksi</div>
        <div class="text-sm sm:text-base font-black text-slate-800 font-mono truncate">
          {filteredItems.length} <span class="text-xs font-normal text-slate-500">Record</span>
        </div>
      </div>
    </div>
  </div>

  <!-- FILTER & SEARCH CONTROLS -->
  <div class="p-2 sm:p-3 flex flex-wrap items-center justify-between gap-2 shrink-0">
    <!-- Filter Tabs (Semua / Pengeluaran / Pemasukan) -->
    <div class="flex items-center gap-1 bg-slate-300/80 p-1 rounded-xl border border-slate-400/50">
      <button
        onclick={() => (activeTab = "SEMUA")}
        class="px-2.5 sm:px-3 py-1 rounded-lg text-xs font-bold transition-all cursor-pointer border-none {activeTab === 'SEMUA' ? 'bg-white text-slate-900 shadow-xs' : 'text-slate-600 hover:text-slate-900 bg-transparent'}"
      >
        Semua Arus Kas
      </button>
      <button
        onclick={() => (activeTab = "KELUAR")}
        class="px-2.5 sm:px-3 py-1 rounded-lg text-xs font-bold transition-all cursor-pointer border-none flex items-center gap-1 {activeTab === 'KELUAR' ? 'bg-red-600 text-white shadow-xs' : 'text-slate-600 hover:text-slate-900 bg-transparent'}"
      >
        <span class="w-1.5 h-1.5 rounded-full bg-red-400"></span>
        <span>Beban Toko</span>
      </button>
      <button
        onclick={() => (activeTab = "MASUK")}
        class="px-2.5 sm:px-3 py-1 rounded-lg text-xs font-bold transition-all cursor-pointer border-none flex items-center gap-1 {activeTab === 'MASUK' ? 'bg-emerald-600 text-white shadow-xs' : 'text-slate-600 hover:text-slate-900 bg-transparent'}"
      >
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
        <span>Kas Masuk</span>
      </button>
    </div>

    <!-- Filter Pos Beban & Search Input -->
    <div class="flex items-center gap-2 flex-1 max-w-xl justify-end">
      <select
        bind:value={filterKategori}
        class="px-2.5 py-1.5 bg-white border border-slate-300 rounded-xl text-xs font-medium text-slate-700 outline-none focus:border-sky-500 shadow-2xs"
      >
        <option value="SEMUA">-- Semua Pos Beban --</option>
        {#each posKategoriList as kat}
          <option value={kat.value}>{kat.label}</option>
        {/each}
      </select>

      <div class="relative flex-1 max-w-xs">
        <span class="material-symbols-outlined absolute left-2.5 top-1/2 -translate-y-1/2 text-slate-400 text-[18px]">
          search
        </span>
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Cari keterangan, bukti..."
          class="w-full pl-8 pr-3 py-1.5 bg-white border border-slate-300 rounded-xl text-xs text-slate-800 placeholder-slate-400 outline-none focus:border-sky-500 shadow-2xs"
        />
        {#if searchQuery}
          <button
            onclick={() => (searchQuery = "")}
            class="absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600 border-none bg-transparent cursor-pointer p-0"
          >
            <span class="material-symbols-outlined text-[14px]">close</span>
          </button>
        {/if}
      </div>
    </div>
  </div>

  <!-- TABLE CONTAINER (FULL HEIGHT SCROLLABLE) -->
  <div class="flex-1 overflow-hidden px-2 sm:px-3 pb-2 sm:pb-3">
    <div class="h-full bg-white rounded-xl border border-slate-300 shadow-xs flex flex-col overflow-hidden">
      <div class="flex-1 overflow-auto">
        <table class="w-full text-left border-collapse">
          <thead class="bg-slate-100 text-slate-600 uppercase font-mono text-[10px] sm:text-[11px] font-bold border-b border-slate-300 sticky top-0 z-10 select-none">
            <tr>
              <th class="py-2.5 px-3">No. Bukti / Waktu</th>
              <th class="py-2.5 px-3">Pos Kategori</th>
              <th class="py-2.5 px-3">Keterangan Biaya / Kas</th>
              <th class="py-2.5 px-3">PIC / Penerima</th>
              <th class="py-2.5 px-3">Metode</th>
              <th class="py-2.5 px-3 text-right">Nominal (Rp)</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-200 text-xs text-slate-700">
            {#if filteredItems.length === 0}
              <tr>
                <td colspan="6" class="py-12 text-center text-slate-400">
                  <div class="flex flex-col items-center gap-1.5">
                    <span class="material-symbols-outlined text-[36px] text-slate-300">receipt_long</span>
                    <span class="font-medium text-xs">Tidak ada catatan kas operasional yang cocok</span>
                  </div>
                </td>
              </tr>
            {:else}
              {#each filteredItems as item}
                <tr class="hover:bg-slate-50 transition-colors">
                  <!-- No Bukti & Waktu -->
                  <td class="py-2.5 px-3 font-mono">
                    <div class="font-bold text-slate-900 text-xs">{item.no_bukti}</div>
                    <div class="text-[10px] text-slate-400">{item.tanggal} • {item.waktu}</div>
                  </td>

                  <!-- Kategori & Jenis Badge -->
                  <td class="py-2.5 px-3">
                    <div class="flex items-center gap-1.5">
                      <span
                        class="px-1.5 py-0.5 rounded font-mono text-[9px] font-bold uppercase {item.jenis === 'MASUK' ? 'bg-emerald-100 text-emerald-800 border border-emerald-300' : 'bg-red-100 text-red-800 border border-red-300'}"
                      >
                        {item.jenis === 'MASUK' ? 'Kas Masuk' : 'Beban'}
                      </span>
                      <span class="font-semibold text-slate-800 text-xs">{item.kategori_label}</span>
                    </div>
                  </td>

                  <!-- Keterangan -->
                  <td class="py-2.5 px-3 max-w-xs truncate font-medium text-slate-800" title={item.keterangan}>
                    {item.keterangan}
                  </td>

                  <!-- PIC & Operator -->
                  <td class="py-2.5 px-3">
                    <div class="font-medium text-slate-800">{item.pic}</div>
                    <div class="text-[10px] text-slate-400">Opr: {item.operator}</div>
                  </td>

                  <!-- Metode Pembayaran -->
                  <td class="py-2.5 px-3 font-mono">
                    <span class="px-1.5 py-0.5 rounded bg-slate-100 text-slate-600 border border-slate-300 text-[10px] font-semibold">
                      {item.metode}
                    </span>
                  </td>

                  <!-- Nominal -->
                  <td class="py-2.5 px-3 text-right font-mono font-bold text-xs sm:text-sm {item.jenis === 'MASUK' ? 'text-emerald-600' : 'text-red-600'}">
                    {item.jenis === 'MASUK' ? '+' : '-'} {formatRupiah(item.nominal)}
                  </td>
                </tr>
              {/each}
            {/if}
          </tbody>
        </table>
      </div>

      <!-- FOOTER TABLE SUMMARY -->
      <div class="bg-slate-50 border-t border-slate-300 px-3 py-2 flex items-center justify-between text-xs text-slate-600 font-mono shrink-0">
        <div>
          Menampilkan <strong>{filteredItems.length}</strong> dari <strong>{items.length}</strong> transaksi kas operasional
        </div>
        <div class="flex items-center gap-3">
          <span>Total Beban: <strong class="text-red-600">{formatRupiah(totalKeluar)}</strong></span>
          <span>•</span>
          <span>Kas Masuk: <strong class="text-emerald-600">{formatRupiah(totalMasuk)}</strong></span>
        </div>
      </div>
    </div>
  </div>
</div>

<!-- ========================================================================= -->
<!-- MODAL: CATAT BIAYA BEBAN / KAS MASUK BARU                                 -->
<!-- ========================================================================= -->
{#if isAddModalOpen}
  <div
    class="fixed inset-0 bg-black/60 backdrop-blur-xs z-50 flex items-center justify-center p-3 animate-in fade-in duration-150"
    onclick={(e) => {
      if (e.target === e.currentTarget) isAddModalOpen = false;
    }}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onkeydown={(e) => e.key === "Escape" && (isAddModalOpen = false)}
  >
    <div class="bg-white rounded-2xl shadow-2xl border border-slate-300 w-full max-w-lg overflow-hidden flex flex-col animate-in zoom-in-95 duration-150 font-sans">
      <!-- Modal Header -->
      <div class="bg-slate-900 text-slate-100 px-4 py-3 flex items-center justify-between border-b border-slate-800">
        <div class="flex items-center gap-2">
          <span class="material-symbols-outlined text-[20px] text-sky-400">payments</span>
          <h2 class="font-bold text-sm tracking-tight">
            {newJenis === "KELUAR" ? "Catat Beban Biaya Operasional Toko" : "Catat Penerimaan Kas Masuk / Modal"}
          </h2>
        </div>
        <button
          onclick={() => (isAddModalOpen = false)}
          class="w-7 h-7 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-400 hover:text-white flex items-center justify-center border-none cursor-pointer"
        >
          <span class="material-symbols-outlined text-[18px]">close</span>
        </button>
      </div>

      <!-- Modal Body Form -->
      <div class="p-4 space-y-3.5 overflow-y-auto max-h-[80vh]">
        <!-- Pilihan Jenis: Beban vs Kas Masuk -->
        <div>
          <span class="block text-xs font-bold text-slate-700 mb-1">Jenis Transaksi Arus Kas</span>
          <div class="grid grid-cols-2 gap-2">
            <button
              type="button"
              onclick={() => (newJenis = "KELUAR")}
              class="p-2.5 rounded-xl border text-xs font-bold flex items-center justify-center gap-1.5 cursor-pointer transition-all {newJenis === 'KELUAR' ? 'bg-red-50 border-red-500 text-red-700 shadow-xs' : 'bg-slate-50 border-slate-300 text-slate-600 hover:bg-slate-100'}"
            >
              <span class="material-symbols-outlined text-[18px] text-red-500">remove_circle</span>
              <span>Pengeluaran (Beban Toko)</span>
            </button>
            <button
              type="button"
              onclick={() => (newJenis = "MASUK")}
              class="p-2.5 rounded-xl border text-xs font-bold flex items-center justify-center gap-1.5 cursor-pointer transition-all {newJenis === 'MASUK' ? 'bg-emerald-50 border-emerald-500 text-emerald-700 shadow-xs' : 'bg-slate-50 border-slate-300 text-slate-600 hover:bg-slate-100'}"
            >
              <span class="material-symbols-outlined text-[18px] text-emerald-500">add_circle</span>
              <span>Penerimaan (Kas Masuk)</span>
            </button>
          </div>
        </div>

        <!-- Pilihan Pos Kategori -->
        <div>
          <label for="pos-kategori-select" class="block text-xs font-bold text-slate-700 mb-1">Pos Kategori Beban / Kas</label>
          <select
            id="pos-kategori-select"
            bind:value={newKategori}
            class="w-full px-3 py-2 bg-slate-50 border border-slate-300 rounded-xl text-xs font-medium text-slate-800 outline-none focus:border-sky-500 focus:bg-white"
          >
            {#each posKategoriList as kat}
              <option value={kat.value}>{kat.label}</option>
            {/each}
          </select>
        </div>

        <!-- Input Nominal -->
        <div>
          <label for="nominal-input" class="block text-xs font-bold text-slate-700 mb-1">Nominal Jumlah (Rp)</label>
          <div class="relative">
            <span class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400 font-mono font-bold text-sm">Rp</span>
            <input
              id="nominal-input"
              type="number"
              bind:value={newNominal}
              placeholder="0"
              min="0"
              step="1000"
              class="w-full pl-10 pr-3 py-2 bg-slate-50 border border-slate-300 rounded-xl text-base font-bold font-mono text-slate-900 outline-none focus:border-sky-500 focus:bg-white"
            />
          </div>
        </div>

        <!-- Input Keterangan -->
        <div>
          <label for="keterangan-input" class="block text-xs font-bold text-slate-700 mb-1">Keterangan / Deskripsi Pengeluaran</label>
          <textarea
            id="keterangan-input"
            bind:value={newKeterangan}
            placeholder="Contoh: Beli pulsa token PLN 100rb, bensin kirim barang, beli air galon pantry..."
            rows="2"
            class="w-full px-3 py-2 bg-slate-50 border border-slate-300 rounded-xl text-xs font-medium text-slate-800 outline-none focus:border-sky-500 focus:bg-white resize-none"
          ></textarea>
        </div>

        <!-- PIC / Penerima & Metode -->
        <div class="grid grid-cols-2 gap-2">
          <div>
            <label for="pic-input" class="block text-xs font-bold text-slate-700 mb-1">PIC / Penerima Uang</label>
            <input
              id="pic-input"
              type="text"
              bind:value={newPic}
              placeholder="Nama staff / penerima"
              class="w-full px-3 py-2 bg-slate-50 border border-slate-300 rounded-xl text-xs font-medium text-slate-800 outline-none focus:border-sky-500 focus:bg-white"
            />
          </div>
          <div>
            <label for="metode-select" class="block text-xs font-bold text-slate-700 mb-1">Metode Kas</label>
            <select
              id="metode-select"
              bind:value={newMetode}
              class="w-full px-3 py-2 bg-slate-50 border border-slate-300 rounded-xl text-xs font-medium text-slate-800 outline-none focus:border-sky-500 focus:bg-white"
            >
              <option value="TUNAI">Kas Tunai (Cash Laci)</option>
              <option value="TRANSFER">Transfer Bank</option>
            </select>
          </div>
        </div>
      </div>

      <!-- Modal Footer Action -->
      <div class="bg-slate-100 px-4 py-3 border-t border-slate-200 flex items-center justify-end gap-2">
        <button
          type="button"
          onclick={() => (isAddModalOpen = false)}
          class="px-3.5 py-1.5 rounded-xl border border-slate-300 bg-white hover:bg-slate-50 text-slate-700 text-xs font-bold cursor-pointer"
        >
          Batal
        </button>
        <button
          type="button"
          onclick={handleSimpan}
          class="px-4 py-1.5 rounded-xl bg-sky-600 hover:bg-sky-700 active:bg-sky-800 text-white text-xs font-bold cursor-pointer shadow-xs"
        >
          Simpan Transaksi
        </button>
      </div>
    </div>
  </div>
{/if}
