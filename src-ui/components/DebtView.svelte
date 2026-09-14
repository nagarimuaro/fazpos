<script lang="ts">
  import { formatRupiah, type DebtItemDTO, type DebtStatsDTO } from "../lib/api";

  let activeTab = $state<"HUTANG" | "PIUTANG">("HUTANG");

  let debts = $state<DebtItemDTO[]>([
    {
      id: "d-01",
      jenis: "HUTANG",
      faktur_ref: "#PO-2026-0909",
      tanggal: "09/09/2026",
      pihak: "PT Kopi Mandiri",
      kontak: "0812-9876-1122",
      jatuh_tempo: "23/09/2026",
      tagihan_awal: 3840000,
      telah_dibayar: 0,
      sisa: 3840000,
      status: "TEMPO",
    },
    {
      id: "d-02",
      jenis: "HUTANG",
      faktur_ref: "#PO-2026-0906",
      tanggal: "06/09/2026",
      pihak: "Danone Tirta",
      kontak: "0813-5566-7788",
      jatuh_tempo: "14/09/2026",
      tagihan_awal: 2360000,
      telah_dibayar: 1000000,
      sisa: 1360000,
      status: "TEMPO",
    },
    {
      id: "d-03",
      jenis: "HUTANG",
      faktur_ref: "#PO-2026-0828",
      tanggal: "28/08/2026",
      pihak: "Unilever Distribusi",
      kontak: "0811-8877-6655",
      jatuh_tempo: "05/09/2026",
      tagihan_awal: 4950000,
      telah_dibayar: 3950000,
      sisa: 1000000,
      status: "LEWAT_TEMPO",
    },
    {
      id: "d-04",
      jenis: "HUTANG",
      faktur_ref: "#PO-2026-0815",
      tanggal: "15/08/2026",
      pihak: "PT Sumber Makmur",
      kontak: "0811-3456-7890",
      jatuh_tempo: "29/08/2026",
      tagihan_awal: 8500000,
      telah_dibayar: 8500000,
      sisa: 0,
      status: "LUNAS",
    },
    {
      id: "d-05",
      jenis: "PIUTANG",
      faktur_ref: "#ORD-8812",
      tanggal: "08/09/2026",
      pihak: "dr. Hendra Kurnia (VIP)",
      kontak: "0811-2233-4455",
      jatuh_tempo: "22/09/2026",
      tagihan_awal: 1250000,
      telah_dibayar: 500000,
      sisa: 750000,
      status: "TEMPO",
    },
    {
      id: "d-06",
      jenis: "PIUTANG",
      faktur_ref: "#ORD-8790",
      tanggal: "02/09/2026",
      pihak: "Siti Rahmawati (Gold)",
      kontak: "0821-9876-5432",
      jatuh_tempo: "16/09/2026",
      tagihan_awal: 850000,
      telah_dibayar: 0,
      sisa: 850000,
      status: "TEMPO",
    },
    {
      id: "d-07",
      jenis: "PIUTANG",
      faktur_ref: "#ORD-8745",
      tanggal: "25/08/2026",
      pihak: "Ahmad Fauzi (Regular)",
      kontak: "0813-8899-7766",
      jatuh_tempo: "04/09/2026",
      tagihan_awal: 650000,
      telah_dibayar: 200000,
      sisa: 450000,
      status: "LEWAT_TEMPO",
    },
    {
      id: "d-08",
      jenis: "PIUTANG",
      faktur_ref: "#ORD-8711",
      tanggal: "18/08/2026",
      pihak: "Dewi Lestari (Silver)",
      kontak: "0856-1122-3344",
      jatuh_tempo: "01/09/2026",
      tagihan_awal: 800000,
      telah_dibayar: 800000,
      sisa: 0,
      status: "LUNAS",
    },
  ]);

  let stats = $state<DebtStatsDTO>({
    total_hutang: 6200000,
    total_piutang: 2050000,
    jatuh_tempo_minggu_ini: 2360000,
    terbayar_bulan_ini: 14250000,
    rasio_lancar: 94.2,
  });

  let searchQuery = $state("");
  let statusFilter = $state("all");
  let searchInputElement: HTMLInputElement | null = $state(null);
  let selectedDebtForPayment = $state<DebtItemDTO | null>(null);
  let paymentAmount = $state(0);
  let toastMessage = $state("");

  let filteredDebts = $derived(
    debts.filter((d) => {
      if (d.jenis !== activeTab) return false;
      if (statusFilter === "belum_lunas" && d.status === "LUNAS") return false;
      if (statusFilter === "lunas" && d.status !== "LUNAS") return false;
      if (statusFilter === "lewat_tempo" && d.status !== "LEWAT_TEMPO") return false;

      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase().trim();
        return (
          d.faktur_ref.toLowerCase().includes(q) ||
          d.pihak.toLowerCase().includes(q) ||
          d.kontak.toLowerCase().includes(q)
        );
      }
      return true;
    })
  );

  function showToast(msg: string) {
    toastMessage = msg;
    setTimeout(() => (toastMessage = ""), 4000);
  }

  function openPaymentModal(d: DebtItemDTO) {
    selectedDebtForPayment = d;
    paymentAmount = d.sisa;
  }

  function processPayment() {
    if (!selectedDebtForPayment || paymentAmount <= 0) return;
    const target = debts.find((x) => x.id === selectedDebtForPayment!.id);
    if (target) {
      target.telah_dibayar += paymentAmount;
      target.sisa = Math.max(0, target.tagihan_awal - target.telah_dibayar);
      if (target.sisa === 0) {
        target.status = "LUNAS";
      }
      if (target.jenis === "HUTANG") {
        stats.total_hutang -= paymentAmount;
      } else {
        stats.total_piutang -= paymentAmount;
      }
      stats.terbayar_bulan_ini += paymentAmount;
      showToast(`Pembayaran ${formatRupiah(paymentAmount)} untuk ${target.faktur_ref} berhasil dicatat.`);
    }
    selectedDebtForPayment = null;
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
  <!-- 1. TOP ACTION & STATUS TABS BAR (Di Atas Card) -->
  <div class="px-4 py-2.5 bg-white border-b border-slate-300 shadow-2xs flex flex-wrap items-center justify-between gap-3 shrink-0">
    <!-- Dual Mode Tabs: Hutang vs Piutang -->
    <div class="flex items-center gap-1 p-1 bg-slate-100 rounded-xl border border-slate-200">
      <button
        onclick={() => (activeTab = "HUTANG")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {activeTab === 'HUTANG' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span class="material-symbols-outlined text-[15px]">outbox</span>
        <span>Hutang Dagang (Ke Supplier)</span>
        <span class="px-1.5 py-0.5 rounded {activeTab === 'HUTANG' ? 'bg-white/20 text-white' : 'bg-slate-200 text-slate-700'} font-mono text-[10px] font-bold">
          {debts.filter((d) => d.jenis === 'HUTANG' && d.status !== 'LUNAS').length}
        </span>
      </button>

      <button
        onclick={() => (activeTab = "PIUTANG")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {activeTab === 'PIUTANG' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span class="material-symbols-outlined text-[15px]">move_to_inbox</span>
        <span>Piutang Usaha (Dari Pelanggan)</span>
        <span class="px-1.5 py-0.5 rounded {activeTab === 'PIUTANG' ? 'bg-white/20 text-white' : 'bg-slate-200 text-slate-700'} font-mono text-[10px] font-bold">
          {debts.filter((d) => d.jenis === 'PIUTANG' && d.status !== 'LUNAS').length}
        </span>
      </button>
    </div>

    <!-- Top Action Buttons (Di Atas Card) -->
    <div class="flex items-center gap-1.5">
      <button
        onclick={() => (statusFilter = statusFilter === "lewat_tempo" ? "all" : "lewat_tempo")}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-sans text-xs font-bold border transition-all cursor-pointer {statusFilter === 'lewat_tempo' ? 'bg-rose-600 text-white border-rose-700 shadow-sm' : 'bg-rose-50 text-rose-700 border-rose-300 hover:bg-rose-100 shadow-2xs'}"
      >
        <span class="material-symbols-outlined text-[16px]">warning</span>
        <span>Lewat Jatuh Tempo</span>
      </button>

      <button
        onclick={() => showToast("Mencetak rekap buku hutang piutang...")}
        class="flex items-center gap-1.5 px-3 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-800 rounded-lg font-sans text-xs font-medium border border-slate-300 shadow-2xs transition-all cursor-pointer"
        title="Cetak Rekap (F4)"
      >
        <span class="font-mono text-[10px] font-bold bg-slate-200 px-1 py-0.2 rounded text-slate-700">F4</span>
        <span class="material-symbols-outlined text-[16px] text-primary">receipt_long</span>
        <span>Cetak Rekap</span>
      </button>

      <button
        onclick={() => showToast("Export data hutang piutang ke CSV berhasil!")}
        class="flex items-center gap-1.5 px-3 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-800 rounded-lg font-sans text-xs font-medium border border-slate-300 shadow-2xs transition-all cursor-pointer"
        title="Export CSV (F7)"
      >
        <span class="font-mono text-[10px] font-bold bg-slate-200 px-1 py-0.2 rounded text-slate-700">F7</span>
        <span class="material-symbols-outlined text-[16px] text-emerald-600">table_view</span>
        <span>Export CSV</span>
      </button>

      <button
        onclick={() => showToast("Database hutang piutang tersinkronisasi offline SQLite.")}
        class="p-1.5 rounded bg-slate-100 hover:bg-slate-200 text-slate-700 transition-colors border border-slate-300 shadow-2xs flex items-center justify-center cursor-pointer"
        title="Segarkan Data (F5)"
      >
        <span class="material-symbols-outlined text-[18px]">refresh</span>
      </button>
    </div>
  </div>

  <!-- 2. 5 COMPACT STAT CARDS (Di Tengah) -->
  <div class="px-4 py-2.5 bg-slate-200 border-b border-slate-300 shrink-0">
    <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-2.5 items-stretch">
      <!-- Card 1: Total Hutang Dagang -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Total Hutang Dagang</span>
          <span class="material-symbols-outlined text-amber-600 text-[18px]">hourglass_empty</span>
        </div>
        <div class="my-1">
          <span class="font-mono text-xl font-bold text-amber-700 tracking-tight">
            {formatRupiah(stats.total_hutang)}
          </span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Kewajiban Supplier</span>
          <span class="text-amber-800 font-bold">3 Faktur Tempo</span>
        </div>
      </div>

      <!-- Card 2: Total Piutang Pelanggan -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Total Piutang Usaha</span>
          <span class="material-symbols-outlined text-primary text-[18px]">account_balance</span>
        </div>
        <div class="my-1">
          <span class="font-mono text-xl font-bold text-primary tracking-tight">
            {formatRupiah(stats.total_piutang)}
          </span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Tagihan Member</span>
          <span class="text-emerald-700 font-bold">Lancar Terkendali</span>
        </div>
      </div>

      <!-- Card 3: Jatuh Tempo Minggu Ini -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-rose-700 font-bold">Tempo &lt; 7 Hari</span>
          <span class="material-symbols-outlined text-rose-600 text-[18px]">alarm</span>
        </div>
        <div class="my-1">
          <span class="font-mono text-xl font-bold text-rose-700 tracking-tight">
            {formatRupiah(stats.jatuh_tempo_minggu_ini)}
          </span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Segera Diangsur</span>
          <span class="text-rose-700 font-bold">Prioritas Kas</span>
        </div>
      </div>

      <!-- Card 4: Pelunasan Bulan Ini -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Terbayar Bulan Ini</span>
          <span class="material-symbols-outlined text-emerald-600 text-[18px]">check_circle</span>
        </div>
        <div class="my-1">
          <span class="font-mono text-xl font-bold text-emerald-700 tracking-tight">
            {formatRupiah(stats.terbayar_bulan_ini)}
          </span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Mutasi Kas Lunas</span>
          <span class="text-emerald-700 font-bold">100% Realisasi</span>
        </div>
      </div>

      <!-- Card 5: Tingkat Kelancaran -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Tingkat Kolektibilitas</span>
          <span class="material-symbols-outlined text-primary text-[18px]">trending_up</span>
        </div>
        <div class="my-1 flex items-baseline gap-1.5">
          <span class="font-mono text-xl font-bold text-slate-900 tracking-tight">{stats.rasio_lancar}%</span>
          <span class="font-mono text-xs text-emerald-700 font-bold">Sangat Sehat</span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Arus Kas Aman</span>
          <span>Cashflow Positif</span>
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
        placeholder="Cari No. Faktur / Nama Rekanan / No. Telepon..."
        type="text"
      />
      <div class="absolute inset-y-0 right-0 pr-2 flex items-center pointer-events-none">
        <span class="px-1.5 py-0.5 rounded bg-slate-200 text-slate-700 font-mono text-[10px] font-bold uppercase border border-slate-300">
          F3
        </span>
      </div>
    </div>

    <!-- Dropdown Filter Status -->
    <div class="flex items-center gap-2 font-sans text-xs">
      <div class="relative">
        <select
          bind:value={statusFilter}
          class="appearance-none bg-slate-100 text-slate-800 font-sans text-xs px-3 py-1.5 pr-7 rounded-lg border border-slate-300 shadow-2xs focus:outline-none cursor-pointer font-medium"
          title="Filter Status Pembayaran"
        >
          <option value="all">Semua Status Tagihan</option>
          <option value="belum_lunas">Belum Lunas (Saldo > 0)</option>
          <option value="lunas">Lunas</option>
          <option value="lewat_tempo">Lewat Jatuh Tempo</option>
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
              <th class="py-2.5 px-3 w-36" scope="col">No. Faktur Ref</th>
              <th class="py-2.5 px-3 w-32" scope="col">Tanggal Faktur</th>
              <th class="py-2.5 px-3 min-w-[200px]" scope="col">
                {activeTab === 'HUTANG' ? 'Supplier Rekanan' : 'Pelanggan / Member'}
              </th>
              <th class="py-2.5 px-3 w-36" scope="col">Kontak</th>
              <th class="py-2.5 px-3 text-center w-32" scope="col">Jatuh Tempo</th>
              <th class="py-2.5 px-3 text-right w-32" scope="col">Tagihan Awal</th>
              <th class="py-2.5 px-3 text-right w-32" scope="col">Telah Dibayar</th>
              <th class="py-2.5 px-3 text-right w-32" scope="col">Sisa Saldo</th>
              <th class="py-2.5 px-3 text-center w-28" scope="col">Status</th>
              <th class="py-2.5 px-3 text-center w-28" scope="col">Aksi</th>
            </tr>
          </thead>

          <!-- Table Body -->
          <tbody class="divide-y divide-slate-200 text-slate-900">
            {#each filteredDebts as d, i}
              <tr
                class="transition-colors border-b border-slate-200/80 cursor-pointer {d.status === 'LEWAT_TEMPO' ? 'bg-rose-50/70 hover:bg-rose-100 border-l-4 border-l-rose-600' : d.status === 'TEMPO' ? 'bg-amber-50/50 hover:bg-amber-100 border-l-4 border-l-amber-500' : i % 2 === 1 ? 'bg-slate-50/70 hover:bg-sky-50/80' : 'bg-white hover:bg-sky-50/80'}"
              >
                <td class="py-2.5 px-3 text-center font-mono text-xs font-bold text-slate-500">
                  {i + 1}
                </td>
                <td class="py-2.5 px-3 font-mono text-xs font-bold text-primary">
                  {d.faktur_ref}
                </td>
                <td class="py-2.5 px-3 font-mono text-xs text-slate-600">
                  {d.tanggal}
                </td>
                <td class="py-2.5 px-3 font-semibold text-slate-900">
                  {d.pihak}
                </td>
                <td class="py-2.5 px-3 font-mono text-xs text-slate-600">
                  {d.kontak}
                </td>
                <td class="py-2.5 px-3 text-center font-mono text-xs font-bold {d.status === 'LEWAT_TEMPO' ? 'text-rose-700' : 'text-slate-700'}">
                  {d.jatuh_tempo}
                </td>
                <td class="py-2.5 px-3 text-right font-mono text-xs text-slate-600 tabular-nums">
                  {formatRupiah(d.tagihan_awal)}
                </td>
                <td class="py-2.5 px-3 text-right font-mono text-xs text-emerald-700 font-semibold tabular-nums">
                  {formatRupiah(d.telah_dibayar)}
                </td>
                <td class="py-2.5 px-3 text-right font-mono text-xs font-bold tabular-nums {d.sisa > 0 ? (d.status === 'LEWAT_TEMPO' ? 'text-rose-700' : 'text-amber-700') : 'text-slate-500'}">
                  {formatRupiah(d.sisa)}
                </td>
                <td class="py-2.5 px-3 text-center">
                  <span class="inline-flex items-center px-2 py-0.5 rounded font-mono text-[10px] font-bold {d.status === 'LUNAS' ? 'bg-emerald-100 text-emerald-800 border border-emerald-300' : d.status === 'LEWAT_TEMPO' ? 'bg-rose-100 text-rose-800 border border-rose-300 animate-pulse' : 'bg-amber-100 text-amber-800 border border-amber-300'}">
                    {d.status === 'LEWAT_TEMPO' ? 'LEWAT TEMPO' : d.status}
                  </span>
                </td>
                <td class="py-2.5 px-3 text-center" onclick={(e) => e.stopPropagation()}>
                  {#if d.sisa > 0}
                    <button
                      onclick={() => openPaymentModal(d)}
                      class="px-2.5 py-1 rounded bg-emerald-600 hover:bg-emerald-700 text-white font-sans text-xs font-bold shadow-2xs border-none cursor-pointer flex items-center justify-center gap-1 mx-auto"
                      title="Bayar / Catat Cicilan"
                    >
                      <span class="material-symbols-outlined text-[14px]">payments</span>
                      <span>Bayar</span>
                    </button>
                  {:else}
                    <span class="font-mono text-xs text-slate-400 font-bold">LUNAS</span>
                  {/if}
                </td>
              </tr>
            {:else}
              <tr>
                <td colspan="11" class="py-16 text-center text-slate-400 font-mono text-xs">
                  Tidak ada catatan {activeTab === 'HUTANG' ? 'hutang supplier' : 'piutang pelanggan'} yang cocok.
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
            <span class="font-medium text-emerald-800 font-bold">Buku kas &amp; hutang-piutang tersinkronisasi offline SQLite</span>
          </div>
          <span>•</span>
          <div>
            Menampilkan <strong class="text-slate-900">{filteredDebts.length}</strong> Catatan Aktif
          </div>
        </div>

        <div class="flex items-center gap-1 font-mono text-xs">
          <button class="px-2.5 py-1 rounded bg-white border border-slate-300 text-slate-400 cursor-not-allowed font-medium" disabled>
            Sebelumnya
          </button>
          <button class="w-7 h-7 rounded bg-primary text-white font-bold flex items-center justify-center shadow-2xs border-none cursor-pointer">
            1
          </button>
          <button class="px-2.5 py-1 rounded bg-white hover:bg-slate-200 text-slate-800 font-medium transition-colors shadow-2xs border border-slate-300 cursor-pointer">
            Selanjutnya
          </button>
        </div>
      </footer>
    </div>
  </div>

  <!-- Modal Pembayaran Cicilan / Pelunasan -->
  {#if selectedDebtForPayment}
    <div
      class="fixed inset-0 bg-slate-900/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none"
      onclick={() => (selectedDebtForPayment = null)}
      role="button"
      tabindex="0"
      onkeydown={(e) => e.key === "Escape" && (selectedDebtForPayment = null)}
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
            <span class="material-symbols-outlined text-emerald-600 text-[20px]">payments</span>
            <span class="font-bold text-slate-900 text-sm font-sans">
              Pembayaran: {selectedDebtForPayment.faktur_ref}
            </span>
          </div>
          <button
            onclick={() => (selectedDebtForPayment = null)}
            class="text-slate-400 hover:text-slate-700 p-1 hover:bg-slate-200 rounded border-none bg-transparent cursor-pointer"
          >
            <span class="material-symbols-outlined text-[18px]">close</span>
          </button>
        </div>

        <div class="p-5 font-sans text-xs flex flex-col gap-3">
          <div class="p-3 bg-slate-50 border border-slate-200 rounded-lg font-mono text-xs flex flex-col gap-1.5">
            <div class="flex justify-between">
              <span class="text-slate-500">Pihak Terkait:</span>
              <strong class="text-slate-900">{selectedDebtForPayment.pihak}</strong>
            </div>
            <div class="flex justify-between">
              <span class="text-slate-500">Total Tagihan Awal:</span>
              <strong class="text-slate-900">{formatRupiah(selectedDebtForPayment.tagihan_awal)}</strong>
            </div>
            <div class="flex justify-between">
              <span class="text-slate-500">Telah Dibayar:</span>
              <strong class="text-emerald-700">{formatRupiah(selectedDebtForPayment.telah_dibayar)}</strong>
            </div>
            <div class="flex justify-between pt-1 border-t border-slate-200">
              <span class="text-slate-700 font-bold">Sisa Tagihan:</span>
              <strong class="text-rose-700 text-sm">{formatRupiah(selectedDebtForPayment.sisa)}</strong>
            </div>
          </div>

          <div>
            <label for="form-nominal-bayar" class="block font-semibold text-slate-700 mb-1">Nominal Pembayaran (Rp)</label>
            <input
              id="form-nominal-bayar"
              type="number"
              bind:value={paymentAmount}
              max={selectedDebtForPayment.sisa}
              class="w-full px-3 py-2 bg-slate-50 border-2 border-emerald-600 rounded font-mono text-base focus:outline-none text-right font-bold text-emerald-800"
            />
          </div>

          <div class="flex items-center gap-2">
            <button
              type="button"
              onclick={() => (paymentAmount = selectedDebtForPayment!.sisa)}
              class="flex-1 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-800 rounded font-mono text-xs font-bold border border-slate-300 cursor-pointer"
            >
              Bayar Lunas (100%)
            </button>
            <button
              type="button"
              onclick={() => (paymentAmount = Math.round(selectedDebtForPayment!.sisa / 2))}
              class="flex-1 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-800 rounded font-mono text-xs font-bold border border-slate-300 cursor-pointer"
            >
              Bayar 50%
            </button>
          </div>
        </div>

        <div class="px-5 py-3 bg-slate-50 border-t border-slate-300 flex items-center justify-end gap-2">
          <button
            type="button"
            onclick={() => (selectedDebtForPayment = null)}
            class="px-4 py-1.5 bg-white hover:bg-slate-100 text-slate-700 border border-slate-300 rounded-lg text-xs font-bold shadow-2xs cursor-pointer"
          >
            Batal
          </button>
          <button
            type="button"
            onclick={processPayment}
            class="px-4 py-1.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg text-xs font-bold shadow-sm cursor-pointer border-none flex items-center gap-1"
          >
            <span class="material-symbols-outlined text-[16px]">check</span>
            Konfirmasi Pembayaran
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
