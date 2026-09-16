<script lang="ts">
  import { api, type StatusInfoDTO } from "../lib/api";

  let {
    status = null,
    onActivated,
  }: {
    status: StatusInfoDTO | null;
    onActivated: () => void;
  } = $props();

  let tokenInput = $state("");
  let isSubmitting = $state(false);
  let errorMessage = $state("");
  let successMessage = $state("");
  let isCopied = $state(false);

  const machineId = $derived(status?.machine_id || "");

  function copyMachineId() {
    if (!machineId) return;
    navigator.clipboard?.writeText(machineId);
    isCopied = true;
    setTimeout(() => {
      isCopied = false;
    }, 2500);
  }

  function handleTokenInput(e: Event) {
    const target = e.target as HTMLInputElement;
    let val = target.value.toUpperCase().replace(/[^A-Z0-9]/g, "");
    if (val.length > 16) {
      val = val.slice(0, 16);
    }
    // Format XXXX-XXXX-XXXX-XXXX
    const parts: string[] = [];
    for (let i = 0; i < val.length; i += 4) {
      parts.push(val.slice(i, i + 4));
    }
    tokenInput = parts.join("-");
    errorMessage = "";
  }

  async function handleActivate(e?: Event) {
    if (e) e.preventDefault();
    const cleanToken = tokenInput.replace(/-/g, "").trim();
    if (cleanToken.length !== 16) {
      errorMessage = "Serial Token Lisensi harus terdiri dari 16 karakter (format: XXXX-XXXX-XXXX-XXXX)";
      return;
    }

    isSubmitting = true;
    errorMessage = "";
    successMessage = "";

    try {
      await api.aktivasiLisensi(tokenInput);
      successMessage = "Aktivasi Berhasil! Membuka sistem kasir...";
      setTimeout(() => {
        onActivated();
      }, 900);
    } catch (err: any) {
      errorMessage = typeof err === "string" ? err : err?.message || "Aktivasi gagal. Pastikan token sesuai dengan Machine ID perangkat ini.";
    } finally {
      isSubmitting = false;
    }
  }
</script>

<div class="h-screen w-screen flex flex-col bg-slate-900 text-slate-100 select-none overflow-hidden font-sans">
  <!-- Top Bar Jendela -->
  <header class="h-12 px-5 flex items-center justify-between border-b border-slate-800 bg-slate-950/80 shrink-0">
    <div class="flex items-center gap-2.5">
      <div class="w-7 h-7 rounded-lg bg-amber-500/10 border border-amber-500/20 text-amber-400 flex items-center justify-center">
        <span class="material-symbols-outlined text-[18px]">lock</span>
      </div>
      <span class="text-xs font-bold tracking-wider uppercase text-slate-300">FAZPOS • Sistem Kasir Ritel</span>
    </div>

    <!-- Window Controls -->
    <div class="flex items-center gap-1.5">
      <button
        type="button"
        onclick={() => api.minimizeWindow()}
        class="w-8 h-8 rounded-lg text-slate-400 hover:text-slate-200 hover:bg-slate-800 flex items-center justify-center transition-colors"
        title="Minimize"
      >
        <span class="material-symbols-outlined text-[16px]">remove</span>
      </button>
      <button
        type="button"
        onclick={() => api.closeWindow()}
        class="w-8 h-8 rounded-lg text-slate-400 hover:text-red-300 hover:bg-red-500/20 flex items-center justify-center transition-colors"
        title="Tutup Aplikasi"
      >
        <span class="material-symbols-outlined text-[16px]">close</span>
      </button>
    </div>
  </header>

  <!-- Content Kunci Aktivasi -->
  <main class="flex-1 flex items-center justify-center p-6 bg-slate-900 overflow-y-auto">
    <div class="w-full max-w-lg bg-slate-950 border border-slate-800 rounded-2xl p-7 shadow-2xl relative">
      
      <!-- Icon & Header -->
      <div class="text-center mb-6">
        <div class="w-14 h-14 rounded-2xl bg-amber-500/10 border border-amber-500/30 text-amber-400 mx-auto flex items-center justify-center mb-3.5 shadow-inner">
          <span class="material-symbols-outlined text-[32px]">vpn_key</span>
        </div>
        <h1 class="text-xl font-black tracking-tight text-white">Aktivasi Lisensi Diperlukan</h1>
        <p class="text-xs text-slate-400 mt-1.5 max-w-sm mx-auto leading-relaxed">
          Perangkat kasir ini belum teraktivasi. Masukkan Serial Token Lisensi untuk membuka dan menggunakan seluruh fitur transaksi FAZPOS.
        </p>
      </div>

      <!-- Kartu Machine ID -->
      <div class="bg-slate-900 border border-slate-800 rounded-xl p-4 mb-5">
        <div class="flex items-center justify-between mb-1.5">
          <span class="text-[11px] font-bold text-slate-400 uppercase tracking-wider">Machine ID Hardware Anda</span>
          <span class="text-[10px] text-slate-500 font-mono">Terkunci di Chip Disk</span>
        </div>
        
        <div class="flex items-center justify-between gap-3 bg-slate-950 px-3.5 py-2.5 rounded-lg border border-slate-800">
          <span class="font-mono text-base font-bold text-amber-400 tracking-wider">
            {machineId || "Memuat ID Hardware..."}
          </span>
          <button
            type="button"
            onclick={copyMachineId}
            class="px-2.5 py-1 rounded text-xs font-semibold flex items-center gap-1.5 transition-colors {isCopied ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/30' : 'bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700'}"
          >
            <span class="material-symbols-outlined text-[14px]">{isCopied ? 'check' : 'content_copy'}</span>
            <span>{isCopied ? 'Tersalin' : 'Salin ID'}</span>
          </button>
        </div>

        <p class="text-[11px] text-slate-500 mt-2 flex items-center gap-1.5">
          <span class="material-symbols-outlined text-[14px] text-slate-400">info</span>
          <span>Salin dan kirimkan Machine ID di atas ke pihak Vendor untuk mendapatkan Serial Token Aktivasi.</span>
        </p>
      </div>

      <!-- Form Token Aktivasi -->
      <form onsubmit={handleActivate} class="space-y-4">
        <div>
          <label for="token-input" class="block text-[11px] font-bold text-slate-300 uppercase tracking-wider mb-1.5">
            Serial Token Lisensi (16 Karakter)
          </label>
          <input
            id="token-input"
            type="text"
            value={tokenInput}
            oninput={handleTokenInput}
            placeholder="XXXX-XXXX-XXXX-XXXX"
            maxlength="19"
            class="w-full bg-slate-900 border border-slate-700 focus:border-amber-400 focus:ring-1 focus:ring-amber-400 rounded-xl px-4 py-3 font-mono text-center text-lg font-bold tracking-widest text-white placeholder-slate-600 transition-all uppercase outline-none"
            autocomplete="off"
            spellcheck="false"
          />
        </div>

        <!-- Feedback Alert -->
        {#if errorMessage}
          <div class="p-3 rounded-xl bg-red-500/10 border border-red-500/30 text-red-300 text-xs flex items-center gap-2">
            <span class="material-symbols-outlined text-[18px] text-red-400 shrink-0">error</span>
            <span>{errorMessage}</span>
          </div>
        {/if}

        {#if successMessage}
          <div class="p-3 rounded-xl bg-emerald-500/10 border border-emerald-500/30 text-emerald-300 text-xs flex items-center gap-2">
            <span class="material-symbols-outlined text-[18px] text-emerald-400 shrink-0">check_circle</span>
            <span>{successMessage}</span>
          </div>
        {/if}

        <!-- Tombol Aktivasi -->
        <button
          type="submit"
          disabled={isSubmitting || !tokenInput.trim()}
          class="w-full py-3 px-4 rounded-xl font-bold text-sm flex items-center justify-center gap-2 transition-all cursor-pointer shadow-lg {isSubmitting || !tokenInput.trim() ? 'bg-slate-800 text-slate-500 border border-slate-700 cursor-not-allowed' : 'bg-amber-500 hover:bg-amber-400 text-slate-950 active:scale-[0.99]'}"
        >
          {#if isSubmitting}
            <span class="material-symbols-outlined text-[18px] animate-spin">progress_activity</span>
            <span>Memverifikasi Hardware & Token...</span>
          {:else}
            <span class="material-symbols-outlined text-[18px]">verified</span>
            <span>Aktivasi Lisensi Sekarang</span>
          {/if}
        </button>
      </form>

      <!-- Footer Info -->
      <div class="mt-5 pt-4 border-t border-slate-800/80 text-center flex items-center justify-between text-[11px] text-slate-500">
        <span>Lisensi Permanen (Hardware-Locked)</span>
        <span>Keamanan Tingkat Binary</span>
      </div>

    </div>
  </main>
</div>
