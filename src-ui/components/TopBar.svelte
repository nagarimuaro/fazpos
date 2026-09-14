<script lang="ts">
  import { api, canOperatorAccess, type OperatorDTO, type StatusInfoDTO } from "../lib/api";

  export type ViewType =
    | "kasir"
    | "produk"
    | "penjualan"
    | "pembelian"
    | "member"
    | "supplier"
    | "hutang_piutang"
    | "laporan"
    | "pengaturan";

  let {
    status,
    currentUser = null,
    currentView = "penjualan",
    onNavigate,
    onOpenChangePassword,
    onLogout,
  }: {
    status: StatusInfoDTO | null;
    currentUser?: OperatorDTO | null;
    currentView?: ViewType;
    onNavigate?: (view: ViewType) => void;
    onOpenChangePassword?: () => void;
    onLogout?: () => void;
  } = $props();
</script>

<header class="h-10 bg-slate-900 text-slate-200 flex items-center justify-between px-3 shrink-0 border-b border-slate-800 select-none gap-2" data-tauri-drag-region>
  <!-- Window Controls & System Identity -->
  <div class="flex items-center gap-2.5 shrink-0">
    <!-- Mac Window Controls -->
    <div class="flex items-center gap-1.5 pr-2 border-r border-slate-700">
      <button
        class="w-2.5 h-2.5 rounded-full bg-red-500 hover:opacity-80 cursor-pointer border-none p-0 inline-block"
        onclick={() => api.closeWindow()}
        title="Tutup Window"
        aria-label="Tutup Window"
      ></button>
      <button
        class="w-2.5 h-2.5 rounded-full bg-amber-400 hover:opacity-80 cursor-pointer border-none p-0 inline-block"
        onclick={() => api.minimizeWindow()}
        title="Minimize"
        aria-label="Minimize"
      ></button>
      <button
        class="w-2.5 h-2.5 rounded-full bg-emerald-500 hover:opacity-80 cursor-pointer border-none p-0 inline-block"
        onclick={() => api.toggleMaximizeWindow()}
        title="Maximize"
        aria-label="Maximize"
      ></button>
    </div>
    <div class="flex items-center gap-1.5">
      <span class="font-black text-slate-100 text-xs tracking-tight">FazPos</span>
    </div>
  </div>

  {#if currentView !== "kasir"}
    <!-- Main POS Menu Tabs (Responsif jika layar mengecil, scrollable tanpa scrollbar kasar) -->
    <nav class="flex items-center p-0.5 bg-slate-800/80 rounded-xl gap-0.5 border border-slate-700/60 font-sans overflow-x-auto min-w-0 max-w-full [scrollbar-width:none] [&::-webkit-scrollbar]:hidden flex-1 justify-start lg:justify-center mx-1">
      <!-- 1. Kasir POS -->
      {#if canOperatorAccess(currentUser, 'kasir')}
        <button
          class="px-1.5 sm:px-2 py-1 rounded-lg text-xs font-medium transition-all flex items-center gap-1 cursor-pointer border-none shrink-0 {currentView === 'kasir' ? 'bg-primary text-white font-bold shadow-xs' : 'text-slate-300 hover:bg-slate-700/60 hover:text-white bg-transparent'}"
          onclick={() => onNavigate?.('kasir')}
          title="Layar Transaksi Kasir [F1]"
        >
          <span class="font-mono text-[9px] uppercase px-1 py-0.2 rounded font-bold {currentView === 'kasir' ? 'bg-blue-700 text-blue-100' : 'bg-slate-700 text-slate-200 border border-slate-600'}">F1</span>
          <span class="hidden xl:inline">Kasir (POS)</span>
          <span class="xl:hidden hidden sm:inline">Kasir</span>
        </button>
      {/if}

      <!-- 2. Menu Produk -->
      {#if canOperatorAccess(currentUser, 'produk')}
        <button
          class="px-1.5 sm:px-2 py-1 rounded-lg text-xs font-medium transition-all flex items-center gap-1 cursor-pointer border-none shrink-0 {currentView === 'produk' ? 'bg-primary text-white font-bold shadow-xs' : 'text-slate-300 hover:bg-slate-700/60 hover:text-white bg-transparent'}"
          onclick={() => onNavigate?.('produk')}
          title="Master Produk & Stok [F2]"
        >
          <span class="font-mono text-[9px] uppercase px-1 py-0.2 rounded font-bold {currentView === 'produk' ? 'bg-blue-700 text-blue-100' : 'bg-slate-700 text-slate-200 border border-slate-600'}">F2</span>
          <span class="hidden xl:inline">Menu Produk</span>
          <span class="xl:hidden hidden sm:inline">Produk</span>
        </button>
      {/if}

      <!-- 3. Riwayat Penjualan -->
      {#if canOperatorAccess(currentUser, 'penjualan')}
        <button
          class="px-1.5 sm:px-2 py-1 rounded-lg text-xs font-medium transition-all flex items-center gap-1 cursor-pointer border-none shrink-0 {currentView === 'penjualan' ? 'bg-primary text-white font-bold shadow-xs' : 'text-slate-300 hover:bg-slate-700/60 hover:text-white bg-transparent'}"
          onclick={() => onNavigate?.('penjualan')}
          title="Riwayat Faktur Penjualan Kasir [F3]"
        >
          <span class="font-mono text-[9px] uppercase px-1 py-0.2 rounded font-bold {currentView === 'penjualan' ? 'bg-blue-700 text-blue-100' : 'bg-slate-700 text-slate-200 border border-slate-600'}">F3</span>
          <span class="hidden xl:inline">Riwayat Penjualan</span>
          <span class="xl:hidden hidden sm:inline">Penjualan</span>
        </button>
      {/if}

      <!-- 4. Riwayat Pembelian -->
      {#if canOperatorAccess(currentUser, 'pembelian')}
        <button
          class="px-1.5 sm:px-2 py-1 rounded-lg text-xs font-medium transition-all flex items-center gap-1 cursor-pointer border-none shrink-0 {currentView === 'pembelian' ? 'bg-primary text-white font-bold shadow-xs' : 'text-slate-300 hover:bg-slate-700/60 hover:text-white bg-transparent'}"
          onclick={() => onNavigate?.('pembelian')}
          title="Riwayat Pembelian & Pengadaan Barang Masuk"
        >
          <span class="material-symbols-outlined text-[14px]">local_shipping</span>
          <span class="hidden xl:inline">Riwayat Pembelian</span>
          <span class="xl:hidden hidden md:inline">Pembelian</span>
        </button>
      {/if}

      <!-- 5. Member -->
      {#if canOperatorAccess(currentUser, 'member')}
        <button
          class="px-1.5 sm:px-2 py-1 rounded-lg text-xs font-medium transition-all flex items-center gap-1 cursor-pointer border-none shrink-0 {currentView === 'member' ? 'bg-primary text-white font-bold shadow-xs' : 'text-slate-300 hover:bg-slate-700/60 hover:text-white bg-transparent'}"
          onclick={() => onNavigate?.('member')}
          title="Master Pelanggan & Loyalty Poin Member"
        >
          <span class="material-symbols-outlined text-[14px]">loyalty</span>
          <span class="hidden md:inline">Member</span>
        </button>
      {/if}

      <!-- 6. Supplier -->
      {#if canOperatorAccess(currentUser, 'supplier')}
        <button
          class="px-1.5 sm:px-2 py-1 rounded-lg text-xs font-medium transition-all flex items-center gap-1 cursor-pointer border-none shrink-0 {currentView === 'supplier' ? 'bg-primary text-white font-bold shadow-xs' : 'text-slate-300 hover:bg-slate-700/60 hover:text-white bg-transparent'}"
          onclick={() => onNavigate?.('supplier')}
          title="Master Rekanan Vendor Supplier"
        >
          <span class="material-symbols-outlined text-[14px]">factory</span>
          <span class="hidden md:inline">Supplier</span>
        </button>
      {/if}

      <!-- 7. Hutang & Piutang -->
      {#if canOperatorAccess(currentUser, 'hutang_piutang')}
        <button
          class="px-1.5 sm:px-2 py-1 rounded-lg text-xs font-medium transition-all flex items-center gap-1 cursor-pointer border-none shrink-0 {currentView === 'hutang_piutang' ? 'bg-primary text-white font-bold shadow-xs' : 'text-slate-300 hover:bg-slate-700/60 hover:text-white bg-transparent'}"
          onclick={() => onNavigate?.('hutang_piutang')}
          title="Buku Hutang Dagang & Piutang Usaha"
        >
          <span class="material-symbols-outlined text-[14px]">account_balance_wallet</span>
          <span class="hidden xl:inline">Hutang &amp; Piutang</span>
          <span class="xl:hidden hidden lg:inline">Hutang</span>
        </button>
      {/if}

      <!-- 8. Laporan -->
      {#if canOperatorAccess(currentUser, 'laporan')}
        <button
          class="px-1.5 sm:px-2 py-1 rounded-lg text-xs font-medium transition-all flex items-center gap-1 cursor-pointer border-none shrink-0 {currentView === 'laporan' ? 'bg-primary text-white font-bold shadow-xs' : 'text-slate-300 hover:bg-slate-700/60 hover:text-white bg-transparent'}"
          onclick={() => onNavigate?.('laporan')}
          title="Pusat Analisis & Laporan Lengkap (JasperReports)"
        >
          <span class="material-symbols-outlined text-[14px]">monitoring</span>
          <span class="hidden md:inline">Laporan</span>
        </button>
      {/if}

      <!-- 9. Pengaturan -->
      {#if canOperatorAccess(currentUser, 'pengaturan')}
        <button
          class="px-1.5 sm:px-2 py-1 rounded-lg text-xs font-medium transition-all flex items-center gap-1 cursor-pointer border-none shrink-0 {currentView === 'pengaturan' ? 'bg-primary text-white font-bold shadow-xs' : 'text-slate-300 hover:bg-slate-700/60 hover:text-white bg-transparent'}"
          onclick={() => onNavigate?.('pengaturan')}
          title="Konfigurasi Profil Toko, Hardware ESC/POS, Kasir & Database"
        >
          <span class="material-symbols-outlined text-[14px]">settings</span>
          <span class="hidden xl:inline">Pengaturan</span>
          <span class="xl:hidden hidden md:inline">Setting</span>
        </button>
      {/if}
    </nav>
  {:else}
    <!-- Khusus Halaman Kasir (POS): Bersih tanpa menu tab -->
    <div class="flex items-center gap-2.5 font-sans">
      <button
        onclick={() => onNavigate?.('penjualan')}
        class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 hover:text-white border border-slate-700 text-xs font-semibold cursor-pointer transition-colors shadow-2xs"
        title="Kembali ke Riwayat Penjualan [ESC]"
      >
        <span class="font-mono text-[9px] bg-slate-700 px-1 py-0.2 rounded font-bold text-slate-200 border border-slate-600">ESC</span>
        <span class="material-symbols-outlined text-[15px]">arrow_back</span>
        <span>Keluar Kasir</span>
      </button>
      <div class="flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-emerald-950/60 border border-emerald-500/30 text-emerald-400 font-mono text-[10px] font-bold">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
        <span>SESI KASIR AKTIF</span>
      </div>
    </div>
  {/if}

  <!-- Operational State & Live Hardware Badges -->
  <div class="flex items-center gap-1.5 sm:gap-2 font-mono text-[11px] shrink-0">
    {#if currentView === "kasir"}
      <div class="flex items-center gap-1 px-1.5 sm:px-2 py-0.5 rounded bg-emerald-950/80 border border-emerald-500/30 text-emerald-300 text-[10px]">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
        <span class="hidden lg:inline">ESC/POS: READY</span>
        <span class="lg:hidden text-[9px]">POS</span>
      </div>
    {/if}

    <!-- Operator & Auth Badges -->
    <div class="flex items-center gap-1 pl-1 border-l border-slate-700">
      <div class="flex items-center gap-1 text-slate-300 text-[11px] font-sans font-medium px-1.5 py-0.5 rounded bg-slate-800/80 border border-slate-700">
        <span class="material-symbols-outlined text-[14px] text-blue-400">account_circle</span>
        <span class="max-w-[70px] sm:max-w-[110px] truncate">{currentUser ? `${currentUser.nama}` : (status?.operator_nama ?? "Kasir")}</span>
        {#if currentUser?.is_admin}
          <span class="text-[9px] font-bold px-1 rounded bg-amber-500/20 text-amber-300 border border-amber-500/40 uppercase hidden sm:inline">Admin</span>
        {/if}
      </div>

      <!-- Tombol Ubah Password -->
      <button
        onclick={onOpenChangePassword}
        class="flex items-center gap-1 px-1.5 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white cursor-pointer border border-slate-700 transition-colors text-[10px]"
        title="Ubah Password Operator / Admin"
      >
        <span class="material-symbols-outlined text-[12px]">key</span>
        <span class="hidden lg:inline font-sans">Password</span>
      </button>

      <!-- Tombol Logout -->
      <button
        onclick={onLogout}
        class="flex items-center gap-1 px-1.5 py-0.5 rounded bg-red-950/60 hover:bg-red-900/80 text-red-300 hover:text-white cursor-pointer border border-red-800/50 transition-colors text-[10px]"
        title="Keluar / Logout"
      >
        <span class="material-symbols-outlined text-[12px]">logout</span>
        <span class="hidden lg:inline font-sans">Logout</span>
      </button>
    </div>

    <div class="text-amber-400 font-semibold tabular-nums text-[11px] hidden md:block" id="live-clock">
      {status?.clock ?? "19:43:50 WIB"}
    </div>
    <button
      onclick={() => alert('Pintasan Keyboard:\nF1: Buka Kasir (POS)\nF2: Menu Produk & Stok\nF3: Riwayat Penjualan\nF4: Cetak Rekap Struk\nF5: Sinkron Data SQLite\nF6: Tambah Stok Masuk / Faktur\nF7: Export CSV\nESC: Tutup Kasir / Kembali')}
      class="hidden xl:flex items-center gap-1 px-1.5 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 hover:text-white cursor-pointer border border-slate-700 transition-colors text-[10px]"
      title="Bantuan Pintasan Keyboard (F12)"
    >
      <span class="font-mono text-[9px] text-slate-400 uppercase">F12</span>
      <span class="font-sans text-[10px]">Bantuan</span>
    </button>
  </div>
</header>
