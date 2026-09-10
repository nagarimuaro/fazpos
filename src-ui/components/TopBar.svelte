<script lang="ts">
  import { api, type StatusInfoDTO } from "../lib/api";

  let {
    status,
    currentView = "transaksi",
    onNavigate,
  }: {
    status: StatusInfoDTO | null;
    currentView?: "transaksi" | "produk" | "kasir";
    onNavigate?: (view: "transaksi" | "produk" | "kasir") => void;
  } = $props();
</script>

<header class="h-10 bg-slate-900 text-slate-200 flex items-center justify-between px-3 shrink-0 border-b border-slate-800 select-none" data-tauri-drag-region>
  <!-- Window Controls & System Identity -->
  <div class="flex items-center gap-3">
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
    <div class="flex items-center gap-2">
      <span class="font-bold text-slate-100 text-xs tracking-tight">FazPos</span>
    </div>
  </div>

  <!-- Main POS Menu Tabs -->
  <nav class="flex items-center p-0.5 bg-slate-800/60 rounded-xl gap-0.5 border border-slate-700/60 font-sans">
    <button
      class="px-2.5 py-1 rounded-lg text-xs font-medium transition-all flex items-center gap-1.5 cursor-pointer border-none {currentView === 'kasir' ? 'bg-primary text-white font-semibold shadow-sm' : 'text-slate-300 hover:bg-slate-700/60 hover:text-white bg-transparent'}"
      onclick={() => onNavigate?.('kasir')}
    >
      <span class="font-mono text-[9px] uppercase px-1 py-0.5 rounded font-bold {currentView === 'kasir' ? 'bg-blue-700/70 text-blue-100' : 'bg-slate-700 text-slate-200 border border-slate-600'}">F1</span>
      <span>Kasir (POS)</span>
    </button>
    <button
      class="px-2.5 py-1 rounded-lg text-xs font-medium transition-all flex items-center gap-1.5 cursor-pointer border-none {currentView === 'produk' ? 'bg-primary text-white font-semibold shadow-sm' : 'text-slate-300 hover:bg-slate-700/60 hover:text-white bg-transparent'}"
      onclick={() => onNavigate?.('produk')}
    >
      <span class="font-mono text-[9px] uppercase px-1 py-0.5 rounded font-bold {currentView === 'produk' ? 'bg-blue-700/70 text-blue-100' : 'bg-slate-700 text-slate-200 border border-slate-600'}">F2</span>
      <span>Menu Produk</span>
    </button>
    <button
      class="px-2.5 py-1 rounded-lg text-xs font-medium transition-all flex items-center gap-1.5 cursor-pointer border-none {currentView === 'transaksi' ? 'bg-primary text-white font-semibold shadow-sm' : 'text-slate-300 hover:bg-slate-700/60 hover:text-white bg-transparent'}"
      onclick={() => onNavigate?.('transaksi')}
    >
      <span class="font-mono text-[9px] uppercase px-1 py-0.5 rounded font-bold {currentView === 'transaksi' ? 'bg-blue-700/70 text-blue-100' : 'bg-slate-700 text-slate-200 border border-slate-600'}">F3</span>
      <span>Riwayat Transaksi</span>
    </button>
    <button
      class="px-2.5 py-1 rounded-lg text-xs text-slate-300 hover:bg-slate-700/60 hover:text-white transition-all flex items-center gap-1.5 cursor-pointer border-none bg-transparent"
      onclick={() => alert('Fitur Hardware & Setup akan segera hadir')}
    >
      <span class="font-mono text-[9px] uppercase px-1 py-0.5 rounded bg-slate-700 text-slate-200 border border-slate-600">F4</span>
      <span>Hardware &amp; Setup</span>
    </button>
  </nav>

  <!-- Operational State & Live Hardware Badges -->
  <div class="flex items-center gap-3 font-mono text-[11px]">
    <div class="flex items-center gap-1 px-2 py-0.5 rounded bg-emerald-950/80 border border-emerald-500/30 text-emerald-300">
      <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
      <span>ESC/POS 80mm: READY</span>
    </div>
    <div class="flex items-center gap-1 text-slate-400">
      <span class="material-symbols-outlined text-[14px]">account_circle</span>
      <span>{status?.operator_nama ?? "Alexander P."} ({status?.shift_status ?? "Shift 1"})</span>
    </div>
    <div class="text-amber-400 font-semibold tabular-nums" id="live-clock">
      {status?.clock ?? "19:43:50 WIB"}
    </div>
    <button
      onclick={() => alert('Pintasan Keyboard:\nF1: Buka Kasir\nF3: Cari Riwayat\nF4: Cetak Struk\nF6: List Pending\nF7: Export CSV\nESC: Tutup Kasir / Kembali')}
      class="flex items-center gap-1 px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 hover:text-white cursor-pointer border border-slate-700 transition-colors"
      title="Bantuan Pintasan Keyboard (F12)"
    >
      <span class="font-mono text-[9px] text-slate-400 uppercase">F12</span>
      <span class="font-sans text-[11px]">Bantuan</span>
    </button>
  </div>
</header>
