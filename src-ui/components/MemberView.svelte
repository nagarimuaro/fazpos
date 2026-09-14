<script lang="ts">
  import { formatRupiah, type MemberDTO, type MemberStatsDTO } from "../lib/api";

  let members = $state<MemberDTO[]>([
    {
      id: "mbr-01",
      kode: "MBR-0421",
      nama: "Budi Santoso",
      telepon: "0812-3456-7890",
      alamat: "Jl. Sudirman No. 45, Jakarta",
      tier: "VIP",
      poin: 128,
      total_belanja: 4850000,
      kunjungan_terakhir: "Hari ini 19:40",
      is_aktif: true,
    },
    {
      id: "mbr-02",
      kode: "MBR-0112",
      nama: "Siti Rahmawati",
      telepon: "0821-9876-5432",
      alamat: "Jl. Melati Blok C-12, Padang",
      tier: "GOLD",
      poin: 450,
      total_belanja: 3200000,
      kunjungan_terakhir: "Kemarin 14:15",
      is_aktif: true,
    },
    {
      id: "mbr-03",
      kode: "MBR-0782",
      nama: "Dewi Lestari",
      telepon: "0856-1122-3344",
      alamat: "Jl. Gatot Subroto Kav. 9, Jakarta",
      tier: "SILVER",
      poin: 640,
      total_belanja: 2150000,
      kunjungan_terakhir: "10/09/2026",
      is_aktif: true,
    },
    {
      id: "mbr-04",
      kode: "MBR-0309",
      nama: "Ahmad Fauzi",
      telepon: "0813-8899-7766",
      alamat: "Komplek Griya Indah D-04, Bandung",
      tier: "REGULAR",
      poin: 85,
      total_belanja: 950000,
      kunjungan_terakhir: "08/09/2026",
      is_aktif: true,
    },
    {
      id: "mbr-05",
      kode: "MBR-0941",
      nama: "Indah Permatasari",
      telepon: "0877-6655-4433",
      alamat: "Jl. Riau No. 18, Bandung",
      tier: "GOLD",
      poin: 320,
      total_belanja: 2890000,
      kunjungan_terakhir: "05/09/2026",
      is_aktif: true,
    },
    {
      id: "mbr-06",
      kode: "MBR-0055",
      nama: "dr. Hendra Kurnia",
      telepon: "0811-2233-4455",
      alamat: "Jl. Diponegoro No. 88, Surabaya",
      tier: "VIP",
      poin: 890,
      total_belanja: 6420000,
      kunjungan_terakhir: "02/09/2026",
      is_aktif: true,
    },
    {
      id: "mbr-07",
      kode: "MBR-1024",
      nama: "Rizky Pratama",
      telepon: "0898-7766-5544",
      alamat: "Jl. Khatib Sulaiman No. 20, Padang",
      tier: "REGULAR",
      poin: 40,
      total_belanja: 480000,
      kunjungan_terakhir: "29/08/2026",
      is_aktif: true,
    },
  ]);

  let stats = $state<MemberStatsDTO>({
    total_member: 542,
    member_aktif: 218,
    total_poin: 38450,
    vip_gold_count: 64,
    member_baru_minggu_ini: 14,
  });

  let searchQuery = $state("");
  let tierFilter = $state("all");
  let searchInputElement: HTMLInputElement | null = $state(null);
  let isAddModalOpen = $state(false);
  let editingMember = $state<MemberDTO | null>(null);
  let toastMessage = $state("");

  // Form states
  let formKode = $state("");
  let formNama = $state("");
  let formTelepon = $state("");
  let formAlamat = $state("");
  let formTier = $state<"VIP" | "GOLD" | "SILVER" | "REGULAR">("REGULAR");

  let filteredMembers = $derived(
    members.filter((m) => {
      if (tierFilter !== "all" && m.tier !== tierFilter) {
        return false;
      }
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase().trim();
        return (
          m.nama.toLowerCase().includes(q) ||
          m.kode.toLowerCase().includes(q) ||
          m.telepon.toLowerCase().includes(q) ||
          m.alamat.toLowerCase().includes(q)
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
    editingMember = null;
    formKode = "MBR-" + Math.floor(1000 + Math.random() * 9000);
    formNama = "";
    formTelepon = "08";
    formAlamat = "";
    formTier = "REGULAR";
    isAddModalOpen = true;
  }

  function openEditModal(m: MemberDTO) {
    editingMember = m;
    formKode = m.kode;
    formNama = m.nama;
    formTelepon = m.telepon;
    formAlamat = m.alamat;
    formTier = m.tier;
    isAddModalOpen = true;
  }

  function saveMember() {
    if (!formNama.trim()) {
      alert("Nama member wajib diisi!");
      return;
    }

    if (editingMember) {
      const idx = members.findIndex((item) => item.id === editingMember!.id);
      if (idx !== -1) {
        members[idx] = {
          ...members[idx],
          nama: formNama,
          telepon: formTelepon,
          alamat: formAlamat,
          tier: formTier,
        };
      }
      showToast(`Data member ${formNama} berhasil diperbarui.`);
    } else {
      const newM: MemberDTO = {
        id: "mbr-" + Date.now(),
        kode: formKode,
        nama: formNama,
        telepon: formTelepon,
        alamat: formAlamat,
        tier: formTier,
        poin: 10,
        total_belanja: 0,
        kunjungan_terakhir: "Baru Terdaftar",
        is_aktif: true,
      };
      members = [newM, ...members];
      stats.total_member += 1;
      showToast(`Member baru ${formNama} berhasil terdaftar.`);
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
        onclick={() => (tierFilter = "all")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {tierFilter === 'all' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span>Semua Member</span>
        <span class="px-1.5 py-0.5 rounded {tierFilter === 'all' ? 'bg-white/20 text-white' : 'bg-slate-200 text-slate-700'} font-mono text-[10px] font-bold">
          {members.length}
        </span>
      </button>

      <button
        onclick={() => (tierFilter = "VIP")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {tierFilter === 'VIP' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span>VIP</span>
        <span class="px-1.5 py-0.5 rounded {tierFilter === 'VIP' ? 'bg-white/20 text-white' : 'bg-slate-200 text-slate-700'} font-mono text-[10px] font-bold">
          {members.filter((m) => m.tier === 'VIP').length}
        </span>
      </button>

      <button
        onclick={() => (tierFilter = "GOLD")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {tierFilter === 'GOLD' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span>Gold</span>
        <span class="px-1.5 py-0.5 rounded {tierFilter === 'GOLD' ? 'bg-white/20 text-white' : 'bg-slate-200 text-slate-700'} font-mono text-[10px] font-bold">
          {members.filter((m) => m.tier === 'GOLD').length}
        </span>
      </button>

      <button
        onclick={() => (tierFilter = "SILVER")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {tierFilter === 'SILVER' ? 'bg-primary text-white shadow-sm' : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span>Silver</span>
        <span class="px-1.5 py-0.5 rounded {tierFilter === 'SILVER' ? 'bg-white/20 text-white' : 'bg-slate-200 text-slate-700'} font-mono text-[10px] font-bold">
          {members.filter((m) => m.tier === 'SILVER').length}
        </span>
      </button>
    </div>

    <!-- Action Utilities (Di Atas Card) -->
    <div class="flex items-center gap-1.5">
      <button
        onclick={() => showToast("Fitur Import Member dari Excel siap digunakan.")}
        class="flex items-center gap-1 px-2.5 py-1.5 rounded bg-slate-100 hover:bg-slate-200 text-slate-800 font-sans text-xs font-medium transition-colors border border-slate-300 shadow-2xs cursor-pointer"
        title="Import Data Pelanggan"
      >
        <span class="material-symbols-outlined text-[16px] text-primary">file_upload</span>
        <span>Import Excel</span>
      </button>

      <button
        onclick={() => showToast("Export database member ke CSV berhasil!")}
        class="flex items-center gap-1 px-2.5 py-1.5 rounded bg-slate-100 hover:bg-slate-200 text-slate-800 font-sans text-xs font-medium transition-colors border border-slate-300 shadow-2xs cursor-pointer"
        title="Export CSV"
      >
        <span class="material-symbols-outlined text-[16px] text-emerald-700">file_download</span>
        <span>Export CSV</span>
      </button>

      <button
        onclick={() => showToast("Sinkronisasi data loyalty poin member berhasil.")}
        class="p-1.5 rounded bg-slate-100 hover:bg-slate-200 text-slate-700 transition-colors border border-slate-300 shadow-2xs flex items-center justify-center cursor-pointer"
        title="Segarkan Data"
      >
        <span class="material-symbols-outlined text-[18px]">refresh</span>
      </button>

      <!-- CTA Tambah Member Baru [F5] -->
      <button
        onclick={openAddModal}
        class="flex items-center gap-1.5 px-3.5 py-1.5 rounded bg-primary hover:bg-primary-dark text-white font-sans text-xs font-bold shadow-sm transition-all border border-primary-dark cursor-pointer ml-1"
        title="Registrasi Member Baru [F5]"
      >
        <span class="font-mono text-[10px] bg-primary-dark px-1.5 py-0.5 rounded font-bold">F5</span>
        <span class="material-symbols-outlined text-[16px]">person_add</span>
        <span>+ Tambah Member Baru</span>
      </button>
    </div>
  </div>

  <!-- 2. 5 COMPACT STAT CARDS (Di Tengah) -->
  <div class="px-4 py-2.5 bg-slate-200 border-b border-slate-300 shrink-0">
    <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-2.5 items-stretch">
      <!-- Card 1: Total Member -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Total Member</span>
          <span class="material-symbols-outlined text-primary text-[18px]">group</span>
        </div>
        <div class="my-1 flex items-baseline gap-1.5">
          <span class="font-mono text-xl font-bold text-slate-900 tracking-tight">{stats.total_member}</span>
          <span class="font-mono text-xs text-slate-500 font-medium">Pelanggan</span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span class="text-emerald-700 font-bold">100% Terverifikasi</span>
          <span>Database Lokal</span>
        </div>
      </div>

      <!-- Card 2: Member Aktif Belanja -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Member Aktif</span>
          <span class="material-symbols-outlined text-emerald-600 text-[18px]">verified</span>
        </div>
        <div class="my-1 flex items-baseline gap-1.5">
          <span class="font-mono text-xl font-bold text-emerald-700 tracking-tight">{stats.member_aktif}</span>
          <span class="font-mono text-xs text-slate-500 font-medium">Orang</span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Bulan Berjalan</span>
          <span class="text-emerald-700 font-bold">40.2% Retensi</span>
        </div>
      </div>

      <!-- Card 3: Akumulasi Poin -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Total Poin Beredar</span>
          <span class="material-symbols-outlined text-amber-500 text-[18px]">stars</span>
        </div>
        <div class="my-1 flex items-baseline gap-1.5">
          <span class="font-mono text-xl font-bold text-amber-600 tracking-tight">{stats.total_poin.toLocaleString()}</span>
          <span class="font-mono text-xs text-slate-500 font-medium">Pts</span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Valuasi Reward</span>
          <span class="text-slate-800 font-bold">{formatRupiah(stats.total_poin * 100)}</span>
        </div>
      </div>

      <!-- Card 4: Member VIP & Gold -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Tier VIP &amp; Gold</span>
          <span class="material-symbols-outlined text-purple-600 text-[18px]">workspace_premium</span>
        </div>
        <div class="my-1 flex items-baseline gap-1.5">
          <span class="font-mono text-xl font-bold text-purple-700 tracking-tight">{stats.vip_gold_count}</span>
          <span class="font-mono text-xs text-slate-500 font-medium">Pelanggan</span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Kontribusi Omset</span>
          <span class="text-purple-800 font-bold">42.8% Penjualan</span>
        </div>
      </div>

      <!-- Card 5: Member Baru Minggu Ini -->
      <div class="flex flex-col justify-between p-3 bg-white rounded-lg border border-slate-300 shadow-2xs">
        <div class="flex items-center justify-between">
          <span class="font-mono text-[10px] uppercase tracking-wider text-slate-500 font-bold">Pendaftaran Baru</span>
          <span class="material-symbols-outlined text-emerald-600 text-[18px]">how_to_reg</span>
        </div>
        <div class="my-1 flex items-baseline gap-1.5">
          <span class="font-mono text-xl font-bold text-emerald-700 tracking-tight">+{stats.member_baru_minggu_ini}</span>
          <span class="font-mono text-xs text-slate-500 font-medium">Minggu Ini</span>
        </div>
        <div class="flex items-center justify-between text-[10px] font-mono text-slate-500">
          <span>Pertumbuhan</span>
          <span class="text-emerald-700 font-bold">+18.5% MoM</span>
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
        placeholder="Cari Nama Pelanggan / No. HP / Kode Member / Alamat..."
        type="text"
      />
      <div class="absolute inset-y-0 right-0 pr-2 flex items-center pointer-events-none">
        <span class="px-1.5 py-0.5 rounded bg-slate-200 text-slate-700 font-mono text-[10px] font-bold uppercase border border-slate-300">
          F3
        </span>
      </div>
    </div>

    <!-- Dropdown Filter Tier -->
    <div class="flex items-center gap-2 font-sans text-xs">
      <div class="relative">
        <select
          bind:value={tierFilter}
          class="appearance-none bg-slate-100 text-slate-800 font-sans text-xs px-3 py-1.5 pr-7 rounded-lg border border-slate-300 shadow-2xs focus:outline-none cursor-pointer font-medium"
          title="Filter Tingkat Member"
        >
          <option value="all">Semua Kategori Tier</option>
          <option value="VIP">VIP (Diskon 10%)</option>
          <option value="GOLD">Gold (Diskon 5%)</option>
          <option value="SILVER">Silver (Diskon 2.5%)</option>
          <option value="REGULAR">Regular</option>
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
              <th class="py-2.5 px-3 w-32" scope="col">Kode Member</th>
              <th class="py-2.5 px-3 min-w-[180px]" scope="col">Nama Lengkap</th>
              <th class="py-2.5 px-3 w-36" scope="col">No. WhatsApp / HP</th>
              <th class="py-2.5 px-3 min-w-[200px]" scope="col">Alamat Domisili</th>
              <th class="py-2.5 px-3 text-center w-28" scope="col">Tingkat Tier</th>
              <th class="py-2.5 px-3 text-right w-28" scope="col">Saldo Poin</th>
              <th class="py-2.5 px-3 text-right w-32" scope="col">Total Belanja</th>
              <th class="py-2.5 px-3 w-36" scope="col">Kunjungan Terakhir</th>
              <th class="py-2.5 px-3 text-center w-28" scope="col">Aksi</th>
            </tr>
          </thead>

          <!-- Table Body -->
          <tbody class="divide-y divide-slate-200 text-slate-900">
            {#each filteredMembers as m, i}
              <tr
                class="transition-colors border-b border-slate-200/80 cursor-pointer {i % 2 === 1 ? 'bg-slate-50/70 hover:bg-sky-50/80' : 'bg-white hover:bg-sky-50/80'}"
              >
                <td class="py-2.5 px-3 text-center font-mono text-xs font-bold text-slate-500">
                  {i + 1}
                </td>
                <td class="py-2.5 px-3 font-mono text-xs font-bold text-primary">
                  {m.kode}
                </td>
                <td class="py-2.5 px-3 font-semibold text-slate-900">
                  {m.nama}
                </td>
                <td class="py-2.5 px-3 font-mono text-xs text-slate-700">
                  {m.telepon}
                </td>
                <td class="py-2.5 px-3 text-slate-600 truncate max-w-[220px]" title={m.alamat}>
                  {m.alamat}
                </td>
                <td class="py-2.5 px-3 text-center">
                  <span class="inline-flex items-center px-2 py-0.5 rounded font-mono text-[10px] font-bold {m.tier === 'VIP' ? 'bg-purple-100 text-purple-900 border border-purple-300' : m.tier === 'GOLD' ? 'bg-amber-100 text-amber-900 border border-amber-300' : m.tier === 'SILVER' ? 'bg-slate-200 text-slate-800' : 'bg-slate-100 text-slate-600'}">
                    {m.tier}
                  </span>
                </td>
                <td class="py-2.5 px-3 text-right font-mono text-xs font-bold text-amber-600 tabular-nums">
                  {m.poin} Pts
                </td>
                <td class="py-2.5 px-3 text-right font-mono text-xs font-bold text-slate-900 tabular-nums">
                  {formatRupiah(m.total_belanja)}
                </td>
                <td class="py-2.5 px-3 font-mono text-xs text-slate-600">
                  {m.kunjungan_terakhir}
                </td>
                <td class="py-2.5 px-3 text-center" onclick={(e) => e.stopPropagation()}>
                  <div class="inline-flex items-center gap-1">
                    <button
                      onclick={() => openEditModal(m)}
                      class="p-1 rounded hover:bg-slate-200 text-slate-600 hover:text-primary transition-colors border-none bg-transparent cursor-pointer"
                      title="Edit Data Member"
                    >
                      <span class="material-symbols-outlined text-[16px]">edit</span>
                    </button>
                    <button
                      onclick={() => showToast(`Mencetak kartu loyalty barcode untuk ${m.nama}...`)}
                      class="p-1 rounded hover:bg-slate-200 text-slate-600 hover:text-primary transition-colors border-none bg-transparent cursor-pointer"
                      title="Cetak Kartu Member"
                    >
                      <span class="material-symbols-outlined text-[16px]">badge</span>
                    </button>
                  </div>
                </td>
              </tr>
            {:else}
              <tr>
                <td colspan="10" class="py-16 text-center text-slate-400 font-mono text-xs">
                  Tidak ada data member yang sesuai filter / pencarian.
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
            <span class="font-medium text-emerald-800 font-bold">Database member tersinkronisasi offline SQLite</span>
          </div>
          <span>•</span>
          <div>
            Menampilkan <strong class="text-slate-900">{filteredMembers.length}</strong> dari <strong class="text-slate-900">{stats.total_member}</strong> Member Terdaftar
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

  <!-- Modal Tambah / Edit Member -->
  {#if isAddModalOpen}
    <div
      class="fixed inset-0 bg-slate-900/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none"
      onclick={() => (isAddModalOpen = false)}
      role="button"
      tabindex="0"
      onkeydown={(e) => e.key === "Escape" && (isAddModalOpen = false)}
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
            <span class="material-symbols-outlined text-primary text-[20px]">
              {editingMember ? "edit" : "person_add"}
            </span>
            <span class="font-bold text-slate-900 text-sm font-sans">
              {editingMember ? `Edit Member: ${editingMember.nama}` : "Registrasi Member Baru [F5]"}
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
              <label for="member-kode" class="block font-semibold text-slate-700 mb-1">Kode Member</label>
              <input
                id="member-kode"
                type="text"
                bind:value={formKode}
                class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded font-mono text-xs focus:outline-none"
              />
            </div>
            <div>
              <label for="member-tier" class="block font-semibold text-slate-700 mb-1">Tingkat Tier</label>
              <select
                id="member-tier"
                bind:value={formTier}
                class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded text-xs focus:outline-none font-bold"
              >
                <option value="REGULAR">Regular (Umum)</option>
                <option value="SILVER">Silver (Diskon 2.5%)</option>
                <option value="GOLD">Gold (Diskon 5%)</option>
                <option value="VIP">VIP (Diskon 10%)</option>
              </select>
            </div>
          </div>

          <div>
            <label for="member-nama" class="block font-semibold text-slate-700 mb-1">Nama Lengkap</label>
            <input
              id="member-nama"
              type="text"
              bind:value={formNama}
              placeholder="Contoh: Budi Santoso"
              class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded text-xs focus:outline-none font-medium"
            />
          </div>

          <div>
            <label for="member-telp" class="block font-semibold text-slate-700 mb-1">No. WhatsApp / HP</label>
            <input
              id="member-telp"
              type="text"
              bind:value={formTelepon}
              placeholder="Contoh: 0812-3456-7890"
              class="w-full px-2.5 py-1.5 bg-slate-50 border border-slate-300 rounded font-mono text-xs focus:outline-none"
            />
          </div>

          <div>
            <label for="member-alamat" class="block font-semibold text-slate-700 mb-1">Alamat Domisili</label>
            <textarea
              id="member-alamat"
              bind:value={formAlamat}
              rows="2"
              placeholder="Jl. Sudirman No. 45..."
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
            onclick={saveMember}
            class="px-4 py-1.5 bg-primary hover:bg-primary-dark text-white rounded-lg text-xs font-bold shadow-sm cursor-pointer border-none flex items-center gap-1"
          >
            <span class="material-symbols-outlined text-[16px]">save</span>
            Simpan Data Member
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
