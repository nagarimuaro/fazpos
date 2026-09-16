<script lang="ts">
  import { api, canOperatorAccess, type OperatorDTO, type StatusInfoDTO } from "../lib/api";

  export type ViewType =
    | "beranda"
    | "kasir"
    | "produk"
    | "penjualan"
    | "pembelian"
    | "operasional"
    | "member"
    | "supplier"
    | "hutang_piutang"
    | "laporan"
    | "pengaturan";

  let {
    status,
    currentUser = null,
    currentView = "beranda",
    cloudStatus = "offline",
    cloudPendingCount = 0,
    lanDeviceCount = 0,
    onNavigate,
    onOpenChangePassword,
    onLogout,
  }: {
    status: StatusInfoDTO | null;
    currentUser?: OperatorDTO | null;
    currentView?: ViewType;
    cloudStatus?: "synced" | "pending" | "offline";
    cloudPendingCount?: number;
    lanDeviceCount?: number;
    onNavigate?: (view: ViewType) => void;
    onOpenChangePassword?: () => void;
    onLogout?: () => void;
  } = $props();

  let isMobileDrawerOpen = $state(false);
  let isProfileMenuOpen = $state(false);
  let isActivationModalOpen = $state(false);
  let activationKeyInput = $state("");
  let isCopied = $state(false);
  let activationToast = $state("");
  let isSubmitting = $state(false);

  const currentMachineId = $derived(status?.machine_id || "");
  const isAlreadyRegistered = $derived(
    Boolean(status?.license_status && status.license_status !== "BELUM AKTIVASI" && status.license_status.startsWith("TERAKTIVASI"))
  );
  const currentLicenseStatus = $derived(status?.license_status || "BELUM AKTIVASI");

  function copyMachineId() {
    if (!currentMachineId) return;
    navigator.clipboard?.writeText(currentMachineId);
    isCopied = true;
    setTimeout(() => (isCopied = false), 2500);
  }

  async function handleActivate() {
    const token = activationKeyInput.trim();
    if (!token) {
      activationToast = "Mohon masukkan Serial Token Lisensi!";
      return;
    }

    isSubmitting = true;
    try {
      // Verifikasi kriptografi & penyimpanan lisensi dieksekusi aman di binary Rust
      const res = await api.aktivasiLisensi(token);
      if (status) {
        status.license_status = res.license_status;
      }
      activationToast = "Lisensi berhasil diverifikasi & aktif permanen!";
      setTimeout(() => {
        activationToast = "";
        isActivationModalOpen = false;
        activationKeyInput = "";
      }, 1500);
    } catch (err: any) {
      activationToast = typeof err === "string" ? err : (err?.message || "Token serial lisensi tidak valid!");
    } finally {
      isSubmitting = false;
    }
  }

  interface NavItem {
    id: ViewType;
    label: string;
    shortcut?: string;
    icon: string;
    description: string;
    permission: string;
  }

  // Label dibuat ringkas dan padat agar 100% muat di layar 13 inch tanpa terpotong
  const navItems: NavItem[] = [
    {
      id: "beranda",
      label: "Beranda",
      icon: "dashboard",
      description: "Ringkasan performa toko, omset & stok",
      permission: "beranda",
    },
    {
      id: "kasir",
      label: "Kasir",
      shortcut: "F1",
      icon: "point_of_sale",
      description: "Transaksi kasir ritel & scan barcode",
      permission: "kasir",
    },
    {
      id: "produk",
      label: "Produk",
      shortcut: "F2",
      icon: "inventory_2",
      description: "Master barang, kategori, harga & stok",
      permission: "produk",
    },
    {
      id: "penjualan",
      label: "Penjualan",
      shortcut: "F3",
      icon: "receipt_long",
      description: "Faktur kasir, retur & rekap penjualan",
      permission: "penjualan",
    },
    {
      id: "pembelian",
      label: "Pembelian",
      icon: "local_shipping",
      description: "Faktur pembelian & stok masuk vendor",
      permission: "pembelian",
    },
    {
      id: "operasional",
      label: "Operasional",
      icon: "payments",
      description: "Buku kas operasional, beban toko & cashflow",
      permission: "operasional",
    },
    {
      id: "member",
      label: "Member",
      icon: "loyalty",
      description: "Data pelanggan & loyalty reward poin",
      permission: "member",
    },
    {
      id: "supplier",
      label: "Supplier",
      icon: "factory",
      description: "Data rekanan vendor & distributor",
      permission: "supplier",
    },
    {
      id: "hutang_piutang",
      label: "Hutang & Piutang",
      icon: "account_balance_wallet",
      description: "Buku hutang dagang & piutang langganan",
      permission: "hutang_piutang",
    },
    {
      id: "laporan",
      label: "Laporan",
      icon: "monitoring",
      description: "Laporan laba kotor, omset & analitik",
      permission: "laporan",
    },
    {
      id: "pengaturan",
      label: "Setting",
      icon: "settings",
      description: "Profil toko, printer ESC/POS & database",
      permission: "pengaturan",
    },
  ];

  function handleNavClick(viewId: ViewType) {
    onNavigate?.(viewId);
    isMobileDrawerOpen = false;
  }

  function getCurrentViewTitle(v: ViewType): string {
    const found = navItems.find((i) => i.id === v);
    return found ? found.label : "Menu";
  }

  function handleWindowClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest("#profile-menu-container")) {
      isProfileMenuOpen = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      isMobileDrawerOpen = false;
      isProfileMenuOpen = false;
      isActivationModalOpen = false;
    }
  }

  $effect(() => {
    window.addEventListener("click", handleWindowClick);
    window.addEventListener("keydown", handleKeydown);
    return () => {
      window.removeEventListener("click", handleWindowClick);
      window.removeEventListener("keydown", handleKeydown);
    };
  });
</script>

<header
  class="h-11 bg-slate-900 text-slate-200 flex items-center justify-between px-2 sm:px-3 shrink-0 border-b border-slate-800 select-none gap-1 sm:gap-2 relative z-30"
  data-tauri-drag-region
>
  <!-- ========================================================================= -->
  <!-- BAGIAN KIRI: Hamburger Mobile / Window Controls & Identitas Brand         -->
  <!-- ========================================================================= -->
  <div class="flex items-center gap-1.5 sm:gap-2 shrink-0">
    <!-- Tombol Hamburger untuk Layar Tablet & Mobile (< 1100px) -->
    {#if currentView !== "kasir"}
      <button
        onclick={() => (isMobileDrawerOpen = !isMobileDrawerOpen)}
        class="xl:hidden w-8 h-8 rounded-lg bg-slate-800 hover:bg-slate-700 active:bg-slate-600 text-slate-200 border border-slate-700 flex items-center justify-center cursor-pointer transition-colors shrink-0 shadow-xs"
        title="Buka Menu Navigasi Lengkap"
        aria-label="Menu Navigasi"
      >
        <span class="material-symbols-outlined text-[20px]">
          {isMobileDrawerOpen ? "close" : "menu"}
        </span>
      </button>
    {/if}

    <!-- Mac Window Controls (Disembunyikan di layar < 640px agar muat di layar 4 inch) -->
    <div class="hidden sm:flex items-center gap-1.5 pr-1.5 sm:pr-2 border-r border-slate-700/80">
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
        title="Toggle Layar Penuh (F11)"
        aria-label="Toggle Fullscreen"
      ></button>
    </div>

    <!-- Indikator Halaman Aktif di Layar Tablet/Mobile (< 1100px) -->
    {#if currentView !== "kasir"}
      <div class="flex items-center gap-1 xl:hidden">
        <span class="text-[10px] font-bold text-sky-300 bg-sky-950/90 px-1.5 py-0.5 rounded border border-sky-500/40 uppercase tracking-wider truncate max-w-[120px] sm:max-w-none">
          {getCurrentViewTitle(currentView)}
        </span>
      </div>
    {/if}
  </div>

  <!-- ========================================================================= -->
  <!-- BAGIAN TENGAH: Menu Navigasi Desktop & Laptop 13 Inch (>= 1100px)        -->
  <!-- ========================================================================= -->
  {#if currentView !== "kasir"}
    <div class="hidden xl:flex flex-1 justify-center min-w-0 overflow-x-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden px-1">
      <nav
        class="flex items-center p-0.5 bg-slate-800/90 rounded-xl gap-0.5 border border-slate-700/60 font-sans shrink-0 shadow-xs"
      >
        {#each navItems as item}
          {#if canOperatorAccess(currentUser, item.permission)}
            <button
              class="px-1.5 2xl:px-2 py-1 rounded-lg text-xs font-medium transition-all flex items-center gap-1 cursor-pointer border-none shrink-0 {currentView === item.id ? 'bg-primary text-white font-bold shadow-xs' : 'text-slate-300 hover:bg-slate-700/80 hover:text-white bg-transparent'}"
              onclick={() => handleNavClick(item.id)}
              title="{item.label}: {item.description}"
            >
              {#if item.shortcut}
                <span
                  class="font-mono text-[9px] uppercase px-1 py-0.2 rounded font-bold {currentView === item.id ? 'bg-blue-700 text-blue-100' : 'bg-slate-700 text-slate-200 border border-slate-600'}"
                >
                  {item.shortcut}
                </span>
              {:else}
                <span class="material-symbols-outlined text-[13px] opacity-85">{item.icon}</span>
              {/if}
              <span>{item.label}</span>
            </button>
          {/if}
        {/each}
      </nav>
    </div>
  {:else}
    <!-- Khusus Mode Kasir (POS): Desain Bersih & Sangat Responsif Sampai 4 Inch -->
    <div class="flex items-center gap-1.5 sm:gap-2.5 font-sans min-w-0">
      <button
        onclick={() => handleNavClick('beranda')}
        class="flex items-center gap-1 px-2 sm:px-2.5 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 hover:text-white border border-slate-700 text-xs font-semibold cursor-pointer transition-colors shadow-2xs shrink-0"
        title="Kembali ke Beranda [ESC]"
      >
        <span class="font-mono text-[9px] bg-slate-700 px-1 py-0.2 rounded font-bold text-slate-200 border border-slate-600">ESC</span>
        <span class="material-symbols-outlined text-[14px]">arrow_back</span>
        <span class="hidden sm:inline">Keluar Kasir</span>
        <span class="sm:hidden text-[10px]">Keluar</span>
      </button>
      <div class="flex items-center gap-1 px-1.5 sm:px-2 py-0.5 rounded-full bg-emerald-950/70 border border-emerald-500/30 text-emerald-400 font-mono text-[9px] sm:text-[10px] font-bold shrink-0">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
        <span class="hidden sm:inline">SESI KASIR AKTIF</span>
        <span class="sm:hidden">KASIR</span>
      </div>
    </div>
  {/if}

  <!-- ========================================================================= -->
  <!-- BAGIAN KANAN: Menu Profil Operator Terpadu, Fullscreen & Jam Live         -->
  <!-- ========================================================================= -->
  <div class="flex items-center gap-1 sm:gap-1.5 font-mono text-[11px] shrink-0">
    <!-- ESC/POS Status (Khusus Layar Kasir) -->
    {#if currentView === "kasir"}
      <div class="hidden sm:flex items-center gap-1 px-1.5 py-0.5 rounded bg-emerald-950/80 border border-emerald-500/30 text-emerald-300 text-[10px]">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
        <span class="hidden xl:inline">ESC/POS: READY</span>
        <span class="xl:hidden text-[9px]">POS</span>
      </div>
    {/if}

    <!-- Indikator Cloud Supabase -->
    {#if cloudStatus === "synced"}
      <div class="hidden md:flex items-center gap-1 px-2 py-0.5 rounded bg-sky-950/80 border border-sky-500/30 text-sky-300 text-[10px]" title="Supabase Cloud: Terhubung &amp; Tersinkron">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
        <span class="material-symbols-outlined text-[13px] text-sky-400">cloud_done</span>
        <span class="hidden lg:inline font-sans">Cloud</span>
      </div>
    {:else if cloudStatus === "pending"}
      <div class="hidden md:flex items-center gap-1 px-2 py-0.5 rounded bg-amber-950/80 border border-amber-500/30 text-amber-300 text-[10px]" title="Supabase Cloud: {cloudPendingCount} record pending sync">
        <span class="w-1.5 h-1.5 rounded-full bg-amber-400 animate-pulse"></span>
        <span class="material-symbols-outlined text-[13px] text-amber-400">cloud_sync</span>
        <span class="hidden lg:inline font-sans">Sync ({cloudPendingCount})</span>
      </div>
    {:else}
      <div class="hidden md:flex items-center gap-1 px-1.5 py-0.5 rounded bg-slate-800 border border-slate-700 text-slate-400 text-[10px]" title="Supabase Cloud belum ditautkan">
        <span class="material-symbols-outlined text-[13px]">cloud_off</span>
      </div>
    {/if}

    <!-- Indikator LAN Discovery -->
    {#if lanDeviceCount > 0}
      <div class="hidden sm:flex items-center gap-1 px-2 py-0.5 rounded bg-emerald-950/60 border border-emerald-500/30 text-emerald-300 text-[10px]" title="{lanDeviceCount} perangkat kasir terdeteksi di LAN">
        <span class="material-symbols-outlined text-[13px] text-emerald-400">sensors</span>
        <span>LAN: {lanDeviceCount}</span>
      </div>
    {/if}

    <!-- Tombol Aktivasi Lisensi: Menampilkan Status Real-Time -->
    <button
      onclick={() => (isActivationModalOpen = true)}
      class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 active:bg-slate-600 text-slate-300 hover:text-white border border-slate-700 cursor-pointer transition-colors shadow-2xs shrink-0"
      title="Status Lisensi Aplikasi"
      aria-label="Aktivasi Lisensi"
    >
      {#if isAlreadyRegistered}
        <span class="material-symbols-outlined text-[15px] text-emerald-400">check_circle</span>
        <span class="text-xs font-semibold">Aktivasi</span>
        <span class="text-[9px] font-mono px-1 py-0.2 rounded bg-emerald-950/80 border border-emerald-500/30 text-emerald-400">
          PRO
        </span>
      {:else}
        <span class="material-symbols-outlined text-[15px] text-amber-400">key</span>
        <span class="text-xs font-semibold text-amber-200">Aktivasi</span>
        <span class="text-[9px] font-mono px-1 py-0.2 rounded bg-amber-950/80 border border-amber-500/40 text-amber-300">
          BELUM AKTIF
        </span>
      {/if}
    </button>

    <!-- Tombol Cepat Fullscreen Langsung di Navbar -->
    <button
      onclick={() => api.toggleMaximizeWindow()}
      class="w-7 h-7 rounded-lg bg-slate-800 hover:bg-slate-700 active:bg-slate-600 text-slate-300 hover:text-white cursor-pointer border border-slate-700 transition-colors flex items-center justify-center shrink-0"
      title="Toggle Layar Penuh (F11)"
      aria-label="Toggle Fullscreen"
    >
      <span class="material-symbols-outlined text-[15px]">fullscreen</span>
    </button>

    <!-- Menu Dropdown Operator (Menghemat 300px ruang horizontal!) -->
    <div class="relative" id="profile-menu-container">
      <button
        onclick={(e) => {
          e.stopPropagation();
          isProfileMenuOpen = !isProfileMenuOpen;
        }}
        class="flex items-center gap-1 text-slate-200 text-[11px] font-sans font-medium px-2 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 active:bg-slate-600 border border-slate-700 cursor-pointer transition-colors shadow-2xs"
        title="Buka Menu Pengguna"
        aria-label="Menu Pengguna"
      >
        <span class="material-symbols-outlined text-[15px] text-blue-400 shrink-0">account_circle</span>
        <span class="truncate max-w-[80px] sm:max-w-[110px] text-[11px] font-semibold">
          {currentUser ? `${currentUser.nama}` : (status?.operator_nama ?? "Kasir")}
        </span>
        {#if currentUser?.is_admin}
          <span class="text-[8px] font-bold px-1 rounded bg-amber-500/20 text-amber-300 border border-amber-500/40 uppercase hidden sm:inline shrink-0">
            Admin
          </span>
        {/if}
        <span class="material-symbols-outlined text-[12px] text-slate-400 shrink-0">expand_more</span>
      </button>

      <!-- Floating Popover Menu Operator -->
      {#if isProfileMenuOpen}
        <div
          class="absolute right-0 top-full mt-1.5 w-56 bg-slate-900 border border-slate-700/80 rounded-xl shadow-2xl p-1.5 z-50 text-slate-200 font-sans animate-in fade-in zoom-in-95 duration-150"
        >
          <!-- Info Operator -->
          <div class="p-2 bg-slate-800/60 rounded-lg mb-1 border border-slate-700/50">
            <div class="text-xs font-bold text-white truncate">
              {currentUser?.nama ?? status?.operator_nama ?? "Operator"}
            </div>
            <div class="text-[10px] text-slate-400 flex items-center justify-between mt-0.5 font-mono">
              <span>Role: <strong class="text-slate-200 uppercase">{currentUser?.role ?? "Kasir"}</strong></span>
              {#if currentUser?.is_admin}
                <span class="text-[8px] font-bold px-1 rounded bg-amber-500/20 text-amber-300 border border-amber-500/40 uppercase">
                  Admin
                </span>
              {/if}
            </div>
          </div>

          <!-- Aksi Menu -->
          <button
            onclick={() => {
              isProfileMenuOpen = false;
              api.toggleMaximizeWindow();
            }}
            class="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg text-xs hover:bg-slate-800 text-slate-300 hover:text-white cursor-pointer border-none text-left transition-colors"
          >
            <span class="material-symbols-outlined text-[15px] text-sky-400">fullscreen</span>
            <span>Mode Layar Penuh (F11)</span>
          </button>

          <button
            onclick={() => {
              isProfileMenuOpen = false;
              isActivationModalOpen = true;
            }}
            class="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg text-xs hover:bg-slate-800 text-slate-300 hover:text-white cursor-pointer border-none text-left transition-colors"
          >
            <span class="material-symbols-outlined text-[15px] text-emerald-400">check_circle</span>
            <span>Lisensi &amp; Aktivasi</span>
          </button>

          <button
            onclick={() => {
              isProfileMenuOpen = false;
              onOpenChangePassword?.();
            }}
            class="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg text-xs hover:bg-slate-800 text-slate-300 hover:text-white cursor-pointer border-none text-left transition-colors"
          >
            <span class="material-symbols-outlined text-[15px] text-amber-400">key</span>
            <span>Ubah Password Akun</span>
          </button>

          <div class="my-1 border-t border-slate-800"></div>

          <button
            onclick={() => {
              isProfileMenuOpen = false;
              onLogout?.();
            }}
            class="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg text-xs hover:bg-red-950/80 text-red-300 hover:text-red-200 cursor-pointer border-none text-left transition-colors font-medium"
          >
            <span class="material-symbols-outlined text-[15px] text-red-400">logout</span>
            <span>Keluar Sesi (Logout)</span>
          </button>
        </div>
      {/if}
    </div>

    <!-- Live Clock (Ditampilkan mulai layar tablet md ke atas) -->
    <div class="text-amber-400 font-semibold tabular-nums text-[10px] sm:text-[11px] hidden md:block pl-1" id="live-clock">
      {status?.clock ?? "19:43:50 WIB"}
    </div>
  </div>
</header>

<!-- ========================================================================= -->
<!-- SLIDE-OVER MOBILE & TABLET DRAWER NAVIGATION (< 1100px / Layar Sentuh)   -->
<!-- ========================================================================= -->
{#if isMobileDrawerOpen}
  <!-- Backdrop Blur -->
  <div
    class="fixed inset-0 bg-black/70 backdrop-blur-xs z-40 xl:hidden transition-opacity"
    onclick={() => (isMobileDrawerOpen = false)}
    role="button"
    tabindex="0"
    onkeydown={(e) => e.key === "Escape" && (isMobileDrawerOpen = false)}
  ></div>

  <!-- Drawer Container -->
  <aside
    class="fixed top-0 left-0 bottom-0 w-[290px] sm:w-[330px] max-w-[85vw] bg-slate-900 border-r border-slate-800 z-50 flex flex-col shadow-2xl text-slate-100 font-sans xl:hidden overflow-hidden animate-in slide-in-from-left duration-200"
  >
    <!-- Drawer Header -->
    <div class="p-3.5 bg-slate-950 border-b border-slate-800 flex items-center justify-between">
      <div class="flex items-center gap-2.5">
        <div class="w-8 h-8 rounded-lg bg-sky-600 text-white flex items-center justify-center font-black text-sm shadow-xs">
          FP
        </div>
        <div>
          <div class="font-black text-sm tracking-tight text-white leading-none">FAZPOS</div>
          <div class="text-[10px] text-slate-400 font-mono mt-0.5">Point of Sale &amp; Retail</div>
        </div>
      </div>
      <button
        onclick={() => (isMobileDrawerOpen = false)}
        class="w-7 h-7 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-400 hover:text-white flex items-center justify-center border border-slate-700 cursor-pointer"
        aria-label="Tutup Menu"
      >
        <span class="material-symbols-outlined text-[16px]">close</span>
      </button>
    </div>

    <!-- Operator Profile Card di Drawer -->
    <div class="p-3 bg-slate-800/60 border-b border-slate-800 flex items-center justify-between gap-2">
      <div class="flex items-center gap-2 min-w-0">
        <span class="material-symbols-outlined text-[24px] text-sky-400 shrink-0">account_circle</span>
        <div class="min-w-0">
          <div class="text-xs font-bold text-white truncate">
            {currentUser?.nama ?? status?.operator_nama ?? "Operator"}
          </div>
          <div class="flex items-center gap-1 mt-0.5">
            <span class="text-[9px] font-mono px-1 rounded bg-slate-700 text-slate-300 uppercase">
              {currentUser?.role ?? "Kasir"}
            </span>
            {#if currentUser?.is_admin}
              <span class="text-[9px] font-mono px-1 rounded bg-amber-500/20 text-amber-300 border border-amber-500/40 uppercase">
                Admin
              </span>
            {/if}
          </div>
        </div>
      </div>
      <div class="flex items-center gap-1 shrink-0">
        <button
          onclick={() => {
            isMobileDrawerOpen = false;
            onOpenChangePassword?.();
          }}
          class="w-7 h-7 rounded bg-slate-700 hover:bg-slate-600 text-slate-200 flex items-center justify-center cursor-pointer border border-slate-600"
          title="Ubah Password"
        >
          <span class="material-symbols-outlined text-[14px]">key</span>
        </button>
        <button
          onclick={() => {
            isMobileDrawerOpen = false;
            onLogout?.();
          }}
          class="w-7 h-7 rounded bg-red-950/80 hover:bg-red-900 text-red-300 flex items-center justify-center cursor-pointer border border-red-800/50"
          title="Logout"
        >
          <span class="material-symbols-outlined text-[14px]">logout</span>
        </button>
      </div>
    </div>

    <!-- Status Lisensi di Drawer -->
    <div class="mx-3 mt-2 p-2.5 bg-slate-800/50 border border-slate-800 rounded-xl flex items-center justify-between">
      <div class="flex items-center gap-2 min-w-0">
        <span class="material-symbols-outlined text-[18px] {isAlreadyRegistered ? 'text-emerald-400' : 'text-amber-400'} shrink-0">
          {isAlreadyRegistered ? 'check_circle' : 'key'}
        </span>
        <div class="min-w-0">
          <div class="text-xs font-semibold text-white">Lisensi FAZPOS</div>
          <div class="text-[10px] text-slate-400">
            {isAlreadyRegistered ? 'Aktif • Bebas Kadaluarsa' : 'Status: Belum Aktivasi'}
          </div>
        </div>
      </div>
      <button
        onclick={() => {
          isMobileDrawerOpen = false;
          isActivationModalOpen = true;
        }}
        class="px-2.5 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium cursor-pointer border border-slate-700 transition-colors"
      >
        {isAlreadyRegistered ? 'Lihat' : 'Aktivasi'}
      </button>
    </div>

    <!-- Daftar Navigasi Utama (Touch-Friendly, min height 44px) -->
    <div class="flex-1 overflow-y-auto p-2 space-y-1 [scrollbar-width:thin]">
      <div class="text-[10px] font-bold text-slate-400 uppercase tracking-wider px-2 py-1 font-mono">
        Modul Sistem
      </div>
      {#each navItems as item}
        {#if canOperatorAccess(currentUser, item.permission)}
          <button
            onclick={() => handleNavClick(item.id)}
            class="w-full flex items-center justify-between p-2.5 rounded-xl text-left cursor-pointer transition-all border {currentView === item.id ? 'bg-primary text-white border-sky-400/50 shadow-md font-bold' : 'bg-slate-800/40 hover:bg-slate-800 text-slate-300 hover:text-white border-transparent'}"
          >
            <div class="flex items-center gap-2.5 min-w-0">
              <span class="material-symbols-outlined text-[20px] {currentView === item.id ? 'text-white' : 'text-sky-400'} shrink-0">
                {item.icon}
              </span>
              <div class="min-w-0">
                <div class="text-xs tracking-tight truncate">{item.label}</div>
                <div class="text-[10px] {currentView === item.id ? 'text-sky-100' : 'text-slate-400'} font-normal truncate">
                  {item.description}
                </div>
              </div>
            </div>
            {#if item.shortcut}
              <span
                class="font-mono text-[9px] uppercase px-1.5 py-0.5 rounded font-bold shrink-0 {currentView === item.id ? 'bg-blue-800 text-blue-100' : 'bg-slate-700 text-slate-300 border border-slate-600'}"
              >
                {item.shortcut}
              </span>
            {/if}
          </button>
        {/if}
      {/each}
    </div>

    <!-- Drawer Footer: Hardware & Fullscreen Quick Toggle -->
    <div class="p-3 bg-slate-950 border-t border-slate-800 space-y-2">
      <button
        onclick={() => {
          api.toggleMaximizeWindow();
          isMobileDrawerOpen = false;
        }}
        class="w-full py-2 px-3 rounded-lg bg-slate-800 hover:bg-slate-700 active:bg-slate-600 text-slate-200 border border-slate-700 flex items-center justify-center gap-2 text-xs font-semibold cursor-pointer transition-colors shadow-2xs"
      >
        <span class="material-symbols-outlined text-[16px] text-sky-400">fullscreen</span>
        <span>Toggle Mode Layar Penuh (F11)</span>
      </button>

      <div class="flex items-center justify-between text-[10px] font-mono text-slate-400 pt-1 border-t border-slate-800/80">
        <div class="flex items-center gap-1">
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
          <span>SQLite OK</span>
        </div>
        <div class="text-amber-400 font-semibold tabular-nums">
          {status?.clock ?? "19:43:50 WIB"}
        </div>
      </div>
    </div>
  </aside>
{/if}

<!-- ========================================================================= -->
<!-- MODAL DIALOG: LISENSI & AKTIVASI (Clean, Simple, Professional)            -->
<!-- ========================================================================= -->
{#if isActivationModalOpen}
  <div
    class="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-50 p-4 animate-in fade-in duration-150"
    role="dialog"
    aria-modal="true"
    aria-labelledby="activation-modal-title"
    tabindex="-1"
    onkeydown={(e) => e.key === "Escape" && (isActivationModalOpen = false)}
  >
    <!-- Backdrop Click -->
    <div
      class="fixed inset-0 -z-10"
      onclick={() => (isActivationModalOpen = false)}
      role="button"
      tabindex="-1"
      aria-label="Tutup Dialog"
      onkeydown={(e) => e.key === "Escape" && (isActivationModalOpen = false)}
    ></div>

    <div
      class="w-full max-w-md bg-slate-900 border border-slate-800 rounded-2xl shadow-xl overflow-hidden text-slate-100 font-sans"
    >
      <!-- Modal Header -->
      <div class="px-5 py-4 border-b border-slate-800 flex items-center justify-between">
        <div>
          <h2 id="activation-modal-title" class="text-sm font-bold text-white">
            Aktivasi Lisensi
          </h2>
          <p class="text-[11px] text-slate-400 mt-0.5">
            Status lisensi dan identitas perangkat FAZPOS
          </p>
        </div>
        <button
          onclick={() => (isActivationModalOpen = false)}
          class="w-7 h-7 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-400 hover:text-white flex items-center justify-center border border-slate-700/80 cursor-pointer transition-colors"
          aria-label="Tutup"
        >
          <span class="material-symbols-outlined text-[16px]">close</span>
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-5 space-y-4">
        <!-- Toast Status -->
        {#if activationToast}
          <div class="p-2.5 rounded-lg bg-emerald-950/70 border border-emerald-500/30 text-emerald-300 text-xs flex items-center gap-2">
            <span class="material-symbols-outlined text-[16px] text-emerald-400">check_circle</span>
            <span>{activationToast}</span>
          </div>
        {/if}

        <!-- Status Lisensi Card -->
        <div class="p-3 bg-slate-800/40 border border-slate-800 rounded-xl space-y-1.5">
          <div class="flex items-center justify-between text-xs">
            <span class="text-slate-400">Status Perangkat:</span>
            {#if isAlreadyRegistered}
              <span class="font-medium text-emerald-400 flex items-center gap-1">
                <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                Aktif (Lifetime)
              </span>
            {:else}
              <span class="font-medium text-amber-400 flex items-center gap-1">
                <span class="w-1.5 h-1.5 rounded-full bg-amber-400"></span>
                Belum Aktivasi
              </span>
            {/if}
          </div>
          <div class="flex items-center justify-between text-xs pt-1 border-t border-slate-800/80">
            <span class="text-slate-400">Toko Terdaftar:</span>
            <span class="font-medium text-slate-200">{status?.toko_nama || "Toko Utama"}</span>
          </div>
        </div>

        <!-- Hardware Machine ID -->
        <div>
          <div class="flex items-center justify-between mb-1.5">
            <label for="machine-id-display" class="text-xs font-medium text-slate-300">
              Hardware Machine ID
            </label>
            {#if isCopied}
              <span class="text-[11px] text-emerald-400 font-medium">Tersalin ke clipboard</span>
            {/if}
          </div>
          <div class="flex items-center gap-2">
            <input
              id="machine-id-display"
              type="text"
              readonly
              value={currentMachineId}
              class="flex-1 bg-slate-950 border border-slate-800 text-slate-200 font-mono text-xs px-3 py-2 rounded-lg select-all focus:outline-hidden"
            />
            <button
              type="button"
              onclick={copyMachineId}
              class="px-3 py-2 bg-slate-800 hover:bg-slate-700 active:bg-slate-600 text-slate-200 border border-slate-700 rounded-lg text-xs font-medium flex items-center gap-1 cursor-pointer transition-colors"
            >
              <span class="material-symbols-outlined text-[14px]">content_copy</span>
              <span>Salin</span>
            </button>
          </div>
          <p class="text-[11px] text-slate-500 mt-1">
            Gunakan ID ini jika memerlukan registrasi atau pembaruan token lisensi.
          </p>
        </div>

        <!-- Input Token Serial Lisensi: Hanya tampil jika BELUM terdaftar -->
        {#if !isAlreadyRegistered}
          <div>
            <label for="activation-token-input" class="block text-xs font-medium text-slate-300 mb-1.5">
              Token Serial Lisensi
            </label>
            <textarea
              id="activation-token-input"
              rows="2"
              bind:value={activationKeyInput}
              placeholder="Tempelkan token serial lisensi..."
              class="w-full bg-slate-950 border border-slate-800 focus:border-sky-500 text-slate-200 font-mono text-xs p-2.5 rounded-lg transition-colors resize-none placeholder:text-slate-600 focus:outline-hidden"
            ></textarea>
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="px-5 py-3 bg-slate-950/60 border-t border-slate-800 flex items-center justify-end gap-2">
        <button
          type="button"
          onclick={() => (isActivationModalOpen = false)}
          class="px-3.5 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-medium cursor-pointer border border-slate-700 transition-colors"
        >
          Tutup
        </button>
        {#if !isAlreadyRegistered}
          <button
            type="button"
            onclick={handleActivate}
            disabled={isSubmitting}
            class="px-4 py-1.5 rounded-lg bg-primary hover:bg-primary-dark disabled:opacity-50 text-white text-xs font-medium cursor-pointer transition-colors shadow-xs"
          >
            {isSubmitting ? "Memverifikasi..." : "Aktifkan"}
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}

