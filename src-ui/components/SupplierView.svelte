<script lang="ts">
  import { formatRupiah, type SupplierDTO, type SupplierStatsDTO } from "../lib/api";

  let suppliers = $state<SupplierDTO[]>([
    {
      id: "sup-01",
      kode: "SUP-001",
      nama: "PT Sumber Makmur",
      pic: "Bambang W.",
      telepon: "0811-3456-7890",
      alamat: "Kawasan Industri Pulo Gadung Blok A-12, Jakarta",
      rekening: "BCA 882-0192-331 a.n PT Sumber Makmur",
      termin_hari: 14,
      total_transaksi: 42500000,
      saldo_hutang: 0,
      is_aktif: true,
    },
    {
      id: "sup-02",
      kode: "SUP-002",
      nama: "PT Kopi Mandiri",
      pic: "Andi Wijaya",
      telepon: "0812-9876-1122",
      alamat: "Jl. Raya Lembang No. 40, Bandung",
      rekening: "Mandiri 131-00-99882-1 a.n PT Kopi Mandiri",
      termin_hari: 21,
      total_transaksi: 28400000,
      saldo_hutang: 3840000,
      is_aktif: true,
    },
    {
      id: "sup-03",
      kode: "SUP-003",
      nama: "Prima Bakery",
      pic: "Ibu Ratna",
      telepon: "0857-4433-2211",
      alamat: "Jl. Riau No. 58, Bandung",
      rekening: "BCA 440-1234-567 a.n Ratna Juwita",
      termin_hari: 7,
      total_transaksi: 18900000,
      saldo_hutang: 0,
      is_aktif: true,
    },
    {
      id: "sup-04",
      kode: "SUP-004",
      nama: "Danone Tirta",
      pic: "Suryono",
      telepon: "0813-5566-7788",
      alamat: "Jl. TB Simatupang Kav. 15, Jakarta Selatan",
      rekening: "BCA 008-8822-119 a.n PT Tirta Investama",
      termin_hari: 30,
      total_transaksi: 34500000,
      saldo_hutang: 2360000,
      is_aktif: true,
    },
    {
      id: "sup-05",
      kode: "SUP-005",
      nama: "Indofood Sukses Makmur",
      pic: "Haryanto S.",
      telepon: "0812-1122-8899",
      alamat: "Sudirman Plaza Lt. 24, Jakarta",
      rekening: "Mandiri 102-00-44991-0 a.n PT Indofood CBP",
      termin_hari: 14,
      total_transaksi: 58900000,
      saldo_hutang: 0,
      is_aktif: true,
    },
    {
      id: "sup-06",
      kode: "SUP-006",
      nama: "Mayora Retail Group",
      pic: "Denny K.",
      telepon: "0818-9900-1122",
      alamat: "Jl. Tomang Raya No. 21, Jakarta Barat",
      rekening: "BCA 522-0199-881 a.n Mayora Distribusi",
      termin_hari: 14,
      total_transaksi: 24700000,
      saldo_hutang: 0,
      is_aktif: true,
    },
    {
      id: "sup-07",
      kode: "SUP-007",
      nama: "Unilever Distribusi",
      pic: "Budi Gunawan",
      telepon: "0811-8877-6655",
      alamat: "BSD Green Office Park, Tangerang",
      rekening: "BNI 099-2233-441 a.n PT Unilever Indonesia",
      termin_hari: 30,
      total_transaksi: 41200000,
      saldo_hutang: 0,
      is_aktif: true,
    },
  ]);

  let stats = $state<SupplierStatsDTO>({
    total_supplier: 28,
    total_belanja: 28450000,
    total_hutang: 6200000,
    jatuh_tempo_segera: 1,
    avg_termin: 18,
  });

  let searchQuery = $state("");
  let hutangFilter = $state("all");
  let searchInputElement: HTMLInputElement | null = $state(null);
  let isAddModalOpen = $state(false);
  let editingSupplier = $state<SupplierDTO | null>(null);
  let toastMessage = $state("");

  // Form states
  let formKode = $state("");
  let formNama = $state("");
  let formPic = $state("");
  let formTelepon = $state("");
  let formAlamat = $state("");
  let formRekening = $state("");
  let formTermin = $state(14);

  let filteredSuppliers = $derived(
    suppliers.filter((s) => {
      if (hutangFilter === "ada_hutang" && s.saldo_hutang === 0) return false;
      if (hutangFilter === "lunas" && s.saldo_hutang > 0) return false;

      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase().trim();
        return (
          s.nama.toLowerCase().includes(q) ||
          s.kode.toLowerCase().includes(q) ||
          s.pic.toLowerCase().includes(q) ||
          s.telepon.toLowerCase().includes(q) ||
          s.alamat.toLowerCase().includes(q)
        );
      }
      return true;
    })
  );

  function showToast(msg: string) {
    toastMessage = msg;
    setTimeout(() => (toastMessage = ""), 4000);
  }

  function openAddModal() {
    editingSupplier = null;
    formKode = "SUP-" + Math.floor(100 + Math.random() * 900);
    formNama = "";
    formPic = "";
    formTelepon = "08";
    formAlamat = "";
    formRekening = "BCA - a.n ";
    formTermin = 14;
    isAddModalOpen = true;
  }

  function openEditModal(s: SupplierDTO) {
    editingSupplier = s;
    formKode = s.kode;
    formNama = s.nama;
    formPic = s.pic;
    formTelepon = s.telepon;
    formAlamat = s.alamat;
    formRekening = s.rekening;
    formTermin = s.termin_hari;
    isAddModalOpen = true;
  }

  function saveSupplier() {
    if (!formNama.trim()) {
      alert("Nama supplier wajib diisi!");
      return;
    }

    if (editingSupplier) {
      const idx = suppliers.findIndex((s) => s.id === editingSupplier!.id);
      if (idx !== -1) {
        suppliers[idx] = {
          ...suppliers[idx],
          nama: formNama,
          pic: formPic,
          telepon: formTelepon,
          alamat: formAlamat,
          rekening: formRekening,
          termin_hari: Number(formTermin),
        };
      }
      showToast(`Supplier ${formNama} berhasil diperbarui.`);
    } else {
      const newS: SupplierDTO = {
        id: "sup-" + Date.now(),
        kode: formKode,
        nama: formNama,
        pic: formPic,
        telepon: formTelepon,
        alamat: formAlamat,
        rekening: formRekening,
        termin_hari: Number(formTermin),
        total_transaksi: 0,
        saldo_hutang: 0,
        is_aktif: true,
      };
      suppliers = [newS, ...suppliers];
      stats.total_supplier += 1;
      showToast(`Supplier ${formNama} berhasil didaftarkan.`);
    }
    isAddModalOpen = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "F3") {
      e.preventDefault();
      searchInputElement?.focus();
      searchInputElement?.select();
    } else if (e.key === "F5") {
      e.preventDefault();
      openAddModal();
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
    <!-- Status Tabs -->
    <div class="flex items-center gap-1 p-1 bg-slate-100 rounded-xl border border-slate-200">
      <button
        onclick={() => (hutangFilter = "all")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {hutangFilter === 'all' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span>Semua Supplier</span>
        <span class="px-1.5 py-0.5 rounded {hutangFilter === 'all' ? 'bg-white/20 text-white' : 'bg-slate-200 text-slate-700'} font-mono text-[10px] font-bold">
          {suppliers.length}
        </span>
      </button>

      <button
        onclick={() => (hutangFilter = "ada_hutang")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {hutangFilter === 'ada_hutang' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span>Ada Tagihan Hutang</span>
        <span class="px-1.5 py-0.5 rounded {hutangFilter === 'ada_hutang' ? 'bg-white/20 text-white' : 'bg-slate-200 text-slate-700'} font-mono text-[10px] font-bold">
          {suppliers.filter((s) => s.saldo_hutang > 0).length}
        </span>
      </button>

      <button
        onclick={() => (hutangFilter = "lunas")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {hutangFilter === 'lunas' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span>Lunas</span>
        <span class="px-1.5 py-0.5 rounded {hutangFilter === 'lunas' ? 'bg-white/20 text-white' : 'bg-slate-200 text-slate-700'} font-mono text-[10px] font-bold">
          {suppliers.filter((s) => s.saldo_hutang === 0).length}
        </span>
      </button>
    </div>

    <!-- Action Buttons (Di Atas Card) -->
    <div class="flex items-center gap-1.5">
      <button
        onclick={() => showToast("Fitur Import Supplier dari Excel siap digunakan.")}
        class="flex items-center gap-1 px-2.5 py-1.5 rounded bg-slate-100 hover:bg-slate-200 text-slate-800 font-sans text-xs font-medium transition-colors border border-slate-300 shadow-2xs cursor-pointer"
        title="Import Data Rekanan"
      >
        <span class="material-symbols-outlined text-[16px] text-primary">file_upload</span>
        <span>Import Excel</span>
      </button>

      <button
        onclick={() => showToast("Export daftar supplier ke CSV berhasil!")}
        class="flex items-center gap-1 px-2.5 py-1.5 rounded bg-slate-100 hover:bg-slate-200 text-slate-800 font-sans text-xs font-medium transition-colors border border-slate-300 shadow-2xs cursor-pointer"
        title="Export CSV"
      >
        <span class="material-symbols-outlined text-[16px] text-emerald-700">file_download</span>
        <span>Export CSV</span>
      </button>

      <button
        onclick={() => showToast("Data supplier tersinkronisasi offline SQLite.")}
        class="p-1.5 rounded bg-slate-100 hover:bg-slate-200 text-slate-700 transition-colors border border-slate-300 shadow-2xs flex items-center justify-center cursor-pointer"
        title="Segarkan Data"
      >
        <span class="material-symbols-outlined text-[18px]">refresh</span>
      </button>

      <!-- CTA Tambah Supplier Baru [F5] -->
      <button
        onclick={openAddModal}
        class="flex items-center gap-1.5 px-3.5 py-1.5 rounded bg-primary hover:bg-primary-dark text-white font-sans text-xs font-bold shadow-sm transition-all border border-primary-dark cursor-pointer ml-1"
        title="Registrasi Supplier Baru [F5]"
      >
        <span class="font-mono text-[10px] bg-primary-dark px-1.5 py-0.5 rounded font-bold">F5</span>
        <span class="material-symbols-outlined text-[16px]">add_business</span>
        <span>+ Tambah Supplier Baru</span>
      </button>
    </div>
  </div>

  <!-- 2. 5 COMPACT STAT CARDS (Di Tengah) -->
  <div class="px-4 py-2.5 bg-slate-200 border-b border-slate-300 shrink-0">
    <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-2.5 items-stretch">
      <!-- Card 1: Total Rekanan -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Total Rekanan Vendor</span>
          <span class="material-symbols-outlined text-primary text-[18px]">domain</span>
        </div>
        <div class="my-1 flex items-baseline gap-1.5">
          <span class="font-mono text-xl font-bold text-slate-900 tracking-tight">{stats.total_supplier}</span>
          <span class="font-mono text-xs text-slate-500 font-medium">Perusahaan</span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span class="text-emerald-700 font-bold">100% Aktif</span>
          <span>Database Lokal</span>
        </div>
      </div>

      <!-- Card 2: Total Belanja Bulan Ini -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Total Belanja Bulan Ini</span>
          <span class="material-symbols-outlined text-emerald-600 text-[18px]">shopping_cart</span>
        </div>
        <div class="my-1">
          <span class="font-mono text-xl font-bold text-slate-900 tracking-tight">
            {formatRupiah(stats.total_belanja)}
          </span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>24 Faktur PO</span>
          <span class="text-slate-700 font-bold">Bulan Berjalan</span>
        </div>
      </div>

      <!-- Card 3: Hutang Usaha Berjalan -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-amber-700 font-bold">Hutang Dagang Aktif</span>
          <span class="material-symbols-outlined text-amber-600 text-[18px]">hourglass_top</span>
        </div>
        <div class="my-1">
          <span class="font-mono text-xl font-bold text-amber-700 tracking-tight">
            {formatRupiah(stats.total_hutang)}
          </span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>2 Supplier Tempo</span>
          <span class="text-amber-800 font-bold">Belum Jatuh Tempo</span>
        </div>
      </div>

      <!-- Card 4: Jatuh Tempo < 7 Hari -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-rose-700 font-bold">Tempo &lt; 7 Hari</span>
          <span class="material-symbols-outlined text-rose-600 text-[18px]">priority_high</span>
        </div>
        <div class="my-1 flex items-baseline gap-1.5">
          <span class="font-mono text-xl font-bold text-rose-700 tracking-tight">{stats.jatuh_tempo_segera}</span>
          <span class="font-mono text-xs text-slate-500 font-medium">Tagihan</span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span class="text-rose-700 font-bold">Rp 2.360.000</span>
          <span>Danone Tirta</span>
        </div>
      </div>

      <!-- Card 5: Rata-rata Termin Tempo -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Rata-rata Termin</span>
          <span class="material-symbols-outlined text-primary text-[18px]">calendar_month</span>
        </div>
        <div class="my-1 flex items-baseline gap-1.5">
          <span class="font-mono text-xl font-bold text-slate-900 tracking-tight">{stats.avg_termin}</span>
          <span class="font-mono text-xs text-slate-500 font-medium">Hari Kerja</span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Rentang</span>
          <span class="text-emerald-700 font-bold">7 s/d 30 Hari</span>
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
        placeholder="Cari Nama Supplier / Sales PIC / No. Telepon / Alamat Gudang..."
        type="text"
      />
      <div class="absolute inset-y-0 right-0 pr-2 flex items-center pointer-events-none">
        <span class="px-1.5 py-0.5 rounded bg-slate-200 text-slate-700 font-mono text-[10px] font-bold uppercase border border-slate-300">
          F3
        </span>
      </div>
    </div>

    <!-- Filter Dropdown -->
    <div class="flex items-center gap-2 font-sans text-xs">
      <div class="relative">
        <select
          bind:value={hutangFilter}
          class="appearance-none bg-slate-100 text-slate-800 font-sans text-xs px-3 py-1.5 pr-7 rounded-lg border border-slate-300 shadow-2xs focus:outline-none cursor-pointer font-medium"
          title="Filter Status Tagihan"
        >
          <option value="all">Semua Status Hutang</option>
          <option value="ada_hutang">Ada Saldo Hutang</option>
          <option value="lunas">Lunas (Saldo 0)</option>
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
              <th class="py-2.5 px-3 w-28" scope="col">Kode Supplier</th>
              <th class="py-2.5 px-3 min-w-[180px]" scope="col">Nama Perusahaan Vendor</th>
              <th class="py-2.5 px-3 w-36" scope="col">Sales / PIC</th>
              <th class="py-2.5 px-3 w-36" scope="col">No. Telepon / WA</th>
              <th class="py-2.5 px-3 min-w-[200px]" scope="col">Alamat Gudang / Kantor</th>
              <th class="py-2.5 px-3 min-w-[180px]" scope="col">Rekening Bank</th>
              <th class="py-2.5 px-3 text-center w-24" scope="col">Termin Tempo</th>
              <th class="py-2.5 px-3 text-right w-32" scope="col">Total Transaksi</th>
              <th class="py-2.5 px-3 text-right w-32" scope="col">Saldo Hutang</th>
              <th class="py-2.5 px-3 text-center w-24" scope="col">Aksi</th>
            </tr>
          </thead>

          <!-- Table Body -->
          <tbody class="divide-y divide-slate-200 text-slate-900">
            {#each filteredSuppliers as s, i}
              <tr
                class="transition-colors border-b border-slate-200/80 cursor-pointer {s.saldo_hutang > 0 ? 'bg-amber-50/60 hover:bg-amber-100/70 border-l-4 border-l-amber-500' : i % 2 === 1 ? 'bg-slate-50/70 hover:bg-sky-50/80' : 'bg-white hover:bg-sky-50/80'}"
              >
                <td class="py-2.5 px-3 text-center font-mono text-xs font-bold text-slate-500">
                  {i + 1}
                </td>
                <td class="py-2.5 px-3 font-mono text-xs font-bold text-primary">
                  {s.kode}
                </td>
                <td class="py-2.5 px-3 font-semibold text-slate-900">
                  {s.nama}
                </td>
                <td class="py-2.5 px-3 font-medium text-slate-700">
                  {s.pic}
                </td>
                <td class="py-2.5 px-3 font-mono text-xs text-slate-700">
                  {s.telepon}
                </td>
                <td class="py-2.5 px-3 text-slate-600 truncate max-w-[200px]" title={s.alamat}>
                  {s.alamat}
                </td>
                <td class="py-2.5 px-3 font-mono text-[11px] text-slate-600 truncate max-w-[180px]" title={s.rekening}>
                  {s.rekening}
                </td>
                <td class="py-2.5 px-3 text-center font-mono text-xs font-bold text-slate-700">
                  {s.termin_hari} Hari
                </td>
                <td class="py-2.5 px-3 text-right font-mono text-xs font-bold text-slate-900 tabular-nums">
                  {formatRupiah(s.total_transaksi)}
                </td>
                <td class="py-2.5 px-3 text-right font-mono text-xs font-bold tabular-nums {s.saldo_hutang > 0 ? 'text-amber-700' : 'text-slate-500'}">
                  {formatRupiah(s.saldo_hutang)}
                </td>
                <td class="py-2.5 px-3 text-center" onclick={(e) => e.stopPropagation()}>
                  <div class="inline-flex items-center gap-1">
                    <button
                      onclick={() => openEditModal(s)}
                      class="p-1 rounded hover:bg-slate-200 text-slate-600 hover:text-primary transition-colors border-none bg-transparent cursor-pointer"
                      title="Edit Data Supplier"
                    >
                      <span class="material-symbols-outlined text-[16px]">edit</span>
                    </button>
                    <button
                      onclick={() => showToast(`Riwayat order pembelian dengan ${s.nama}...`)}
                      class="p-1 rounded hover:bg-slate-200 text-slate-600 hover:text-primary transition-colors border-none bg-transparent cursor-pointer"
                      title="Histori Faktur PO"
                    >
                      <span class="material-symbols-outlined text-[16px]">history</span>
                    </button>
                  </div>
                </td>
              </tr>
            {:else}
              <tr>
                <td colspan="11" class="py-16 text-center text-slate-400 font-mono text-xs">
                  Tidak ada data supplier yang cocok dengan filter / pencarian.
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
            <span class="font-medium text-emerald-800 font-bold">Database vendor supplier tersinkronisasi offline SQLite</span>
          </div>
          <span>•</span>
          <div>
            Menampilkan <strong class="text-slate-900">{filteredSuppliers.length}</strong> dari <strong class="text-slate-900">{stats.total_supplier}</strong> Rekanan Vendor
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

  <!-- Modal Tambah / Edit Supplier -->
  {#if isAddModalOpen}
    <div
      class="fixed inset-0 bg-slate-900/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none"
      onclick={() => (isAddModalOpen = false)}
      role="button"
      tabindex="0"
      onkeydown={(e) => e.key === "Escape" && (isAddModalOpen = false)}
    >
      <div
        class="bg-white border border-slate-300 rounded-xl shadow-2xl w-[500px] overflow-hidden"
        role="dialog"
        aria-modal="true"
        tabindex="-1"
        onclick={(e) => e.stopPropagation()}
        onkeydown={(e) => e.stopPropagation()}
      >
        <div class="px-5 py-3.5 bg-slate-100 border-b border-slate-300 flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="material-symbols-outlined text-primary text-[20px]">
              {editingSupplier ? "edit" : "add_business"}
            </span>
            <span class="font-bold text-slate-900 text-sm font-sans">
              {editingSupplier ? `Edit Supplier: ${editingSupplier.nama}` : "Registrasi Supplier Baru [F5]"}
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
              <label for="supplier-kode" class="block font-semibold text-slate-700 mb-1">Kode Supplier</label>
              <input
                id="supplier-kode"
                type="text"
                bind:value={formKode}
                class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded font-mono text-xs focus:outline-none"
              />
            </div>
            <div>
              <label for="supplier-termin" class="block font-semibold text-slate-700 mb-1">Termin Tempo (Hari)</label>
              <input
                id="supplier-termin"
                type="number"
                bind:value={formTermin}
                class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded font-mono text-xs focus:outline-none text-right font-bold"
              />
            </div>
          </div>

          <div>
            <label for="supplier-nama" class="block font-semibold text-slate-700 mb-1">Nama Perusahaan / Distributor</label>
            <input
              id="supplier-nama"
              type="text"
              bind:value={formNama}
              placeholder="Contoh: PT Sumber Makmur"
              class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded text-xs focus:outline-none font-medium"
            />
          </div>

          <div class="grid grid-cols-2 gap-3">
            <div>
              <label for="supplier-pic" class="block font-semibold text-slate-700 mb-1">Sales / PIC Kontak</label>
              <input
                id="supplier-pic"
                type="text"
                bind:value={formPic}
                placeholder="Contoh: Bambang W."
                class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded text-xs focus:outline-none"
              />
            </div>
            <div>
              <label for="supplier-telepon" class="block font-semibold text-slate-700 mb-1">No. Telepon / WA</label>
              <input
                id="supplier-telepon"
                type="text"
                bind:value={formTelepon}
                placeholder="0812-..."
                class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded font-mono text-xs focus:outline-none"
              />
            </div>
          </div>

          <div>
            <label for="supplier-rekening" class="block font-semibold text-slate-700 mb-1">Nomor Rekening Pembayaran</label>
            <input
              id="supplier-rekening"
              type="text"
              bind:value={formRekening}
              placeholder="Contoh: BCA 882-0192-331 a.n PT Sumber Makmur"
              class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded font-mono text-xs focus:outline-none"
            />
          </div>

          <div>
            <label for="supplier-alamat" class="block font-semibold text-slate-700 mb-1">Alamat Gudang / Kantor</label>
            <textarea
              id="supplier-alamat"
              bind:value={formAlamat}
              rows="2"
              placeholder="Kawasan Industri..."
              class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded text-xs focus:outline-none"
            ></textarea>
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
            onclick={saveSupplier}
            class="px-4 py-1.5 bg-primary hover:bg-primary-dark text-white rounded-lg text-xs font-bold shadow-sm cursor-pointer border-none flex items-center gap-1"
          >
            <span class="material-symbols-outlined text-[16px]">save</span>
            Simpan Supplier
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
