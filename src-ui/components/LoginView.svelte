<script lang="ts">
  import { api, type OperatorDTO, type StatusInfoDTO } from "../lib/api";

  let {
    tokoNama = "",
    logoUrl = "",
    logoIcon = "storefront",
    onLoginSuccess,
  }: {
    tokoNama?: string;
    logoUrl?: string;
    logoIcon?: string;
    onLoginSuccess: (user: OperatorDTO) => void;
  } = $props();

  let activeTokoNama = $state("");
  let activeLogoUrl = $state("");
  let activeLogoIcon = $state("storefront");
  let operators = $state<OperatorDTO[]>([]);
  let kode = $state("admin");
  let password = $state("admin");
  let showPassword = $state(false);
  let isLoading = $state(false);
  let errorMessage = $state("");
  let status = $state<StatusInfoDTO | null>(null);
  let liveClock = $state("");
  let passwordInput: HTMLInputElement | null = $state(null);

  let selectedOp = $derived(
    operators.find((o) => o.kode === kode) ??
      (kode === "admin"
        ? {
            id: "admin",
            cabang_id: "",
            kode: "admin",
            nama: "Administrator",
            role: "admin",
            is_admin: true,
          }
        : null)
  );

  function handleOperatorChange(newKode: string) {
    kode = newKode;
    errorMessage = "";
    // User dasar (admin) langsung diisikan password default dan tersembunyi
    if (newKode === "admin") {
      password = "admin";
    } else {
      password = "";
    }
    passwordInput?.focus();
  }

  async function loadData() {
    try {
      const [st, ops, sett] = await Promise.all([
        api.getStatusInfo().catch(() => null),
        api.getOperators().catch(() => []),
        api.getSettings().catch(() => null),
      ]);
      if (st) {
        status = st;
        if (st.clock) liveClock = st.clock;
      }
      if (sett) {
        activeTokoNama = sett.toko_nama || tokoNama || status?.toko_nama || "MUEEZA STORE";
        activeLogoUrl = sett.logo_url || logoUrl || "";
        activeLogoIcon = sett.logo_icon || logoIcon || "storefront";
      } else {
        activeTokoNama = tokoNama || status?.toko_nama || "MUEEZA STORE";
        activeLogoUrl = logoUrl;
        activeLogoIcon = logoIcon;
      }
      if (ops && ops.length > 0) {
        operators = ops;
        if (!kode || !ops.some((o) => o.kode === kode)) {
          kode = ops[0].kode;
        }
        if (kode === "admin" && !password) {
          password = "admin";
        }
      }
    } catch (err) {
      console.error("loadData error:", err);
    }
  }

  function updateClock() {
    const now = new Date();
    const h = String(now.getHours()).padStart(2, "0");
    const m = String(now.getMinutes()).padStart(2, "0");
    const s = String(now.getSeconds()).padStart(2, "0");
    liveClock = `${h}:${m}:${s} WIB`;
  }

  async function handleSubmit(e?: Event) {
    if (e) e.preventDefault();
    if (!kode.trim()) {
      errorMessage = "Silakan pilih operator kasir";
      return;
    }
    if (!password) {
      errorMessage = "Silakan masukkan password operator";
      passwordInput?.focus();
      return;
    }

    isLoading = true;
    errorMessage = "";

    try {
      const user = await api.login(kode.trim(), password);
      onLoginSuccess(user);
    } catch (err: any) {
      errorMessage = typeof err === "string" ? err : err?.message || "Login gagal. Periksa kembali password Anda.";
    } finally {
      isLoading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      handleSubmit();
    } else if (e.key === "Escape") {
      e.preventDefault();
      api.closeWindow();
    }
  }

  $effect(() => {
    loadData();
    updateClock();
    const interval = setInterval(updateClock, 1000);
    window.addEventListener("keydown", handleKeydown);
    passwordInput?.focus();

    return () => {
      clearInterval(interval);
      window.removeEventListener("keydown", handleKeydown);
    };
  });
</script>

<div class="h-screen w-screen flex flex-col bg-slate-200 font-sans select-none overflow-hidden">
  <!-- Top Bar Window Controls & Identity (Satu Konsep dengan TopBar Aplikasi Utama) -->
  <header class="h-10 bg-slate-900 text-slate-200 flex items-center justify-between px-3 border-b border-slate-800 shrink-0 gap-2" data-tauri-drag-region>
    <div class="flex items-center gap-2.5 shrink-0">
      <!-- Mac Window Controls -->
      <div class="flex items-center gap-1.5 pr-2 border-r border-slate-700">
        <button
          class="w-2.5 h-2.5 rounded-full bg-red-500 hover:opacity-80 cursor-pointer border-none p-0 inline-block"
          onclick={() => api.closeWindow()}
          title="Tutup Aplikasi"
          aria-label="Tutup Aplikasi"
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
        <span class="font-black text-slate-100 text-xs tracking-tight">FazPos</span>
        <span class="text-slate-600 text-xs">|</span>
        <span class="text-xs text-slate-400 font-medium">Autentikasi &amp; Sesi Kasir</span>
      </div>
    </div>

    <!-- Status Hardware & Info Kanan -->
    <div class="flex items-center gap-2 font-mono text-[11px] shrink-0">
      <div class="flex items-center gap-1.5 px-2 py-0.5 rounded bg-emerald-950/80 border border-emerald-500/30 text-emerald-300 text-[10px]">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
        <span>SQLITE: READY</span>
      </div>

      <div class="flex items-center gap-1 text-slate-300 text-[10px] px-2 py-0.5 rounded bg-slate-800/80 border border-slate-700 font-sans">
        <span class="material-symbols-outlined text-[13px] text-blue-400">computer</span>
        <span class="font-mono">{status?.terminal_id ? status.terminal_id.replace("Terminal: ", "") : "REG-01"}</span>
      </div>

      <div class="text-amber-400 font-semibold tabular-nums text-[11px] px-1 font-mono">
        {liveClock || "00:00:00 WIB"}
      </div>
    </div>
  </header>

  <!-- SubHeader (Satu Konsep dengan SubHeader Layar Utama) -->
  <div class="bg-slate-100 border-b border-slate-300 px-5 py-2.5 flex items-center justify-between shadow-2xs shrink-0 select-none">
    <div class="flex items-center gap-3">
      <div class="w-10 h-10 rounded-xl {activeLogoUrl ? 'bg-white border border-slate-300 p-1' : 'bg-primary text-white'} flex items-center justify-center shadow-xs shrink-0 overflow-hidden">
        {#if activeLogoUrl}
          <img src={activeLogoUrl} alt="Logo" class="w-full h-full object-contain" />
        {:else}
          <span class="material-symbols-outlined text-[24px]">{activeLogoIcon || 'storefront'}</span>
        {/if}
      </div>
      <div>
        <div class="text-xl font-black text-slate-900 uppercase tracking-tight leading-tight">
          {activeTokoNama || status?.toko_nama || "MUEEZA STORE"}
        </div>
        <div class="text-[11px] text-slate-500 font-medium">
          Sistem Kasir Ritel &amp; Database Terintegrasi
        </div>
      </div>
    </div>

    <div class="flex items-center gap-2 font-mono text-[11px]">
      <span class="px-2.5 py-1 rounded bg-slate-200 text-slate-700 font-semibold border border-slate-300 flex items-center gap-1.5 shadow-2xs">
        <span class="material-symbols-outlined text-[14px] text-slate-600">dns</span>
        <span>Local Database Standalone</span>
      </span>
      <span class="px-2.5 py-1 rounded bg-emerald-100 text-emerald-800 font-semibold border border-emerald-300 flex items-center gap-1 shadow-2xs">
        <span class="w-2 h-2 rounded-full bg-emerald-500"></span> Sistem Siap
      </span>
    </div>
  </div>

  <!-- Main Login Card Area (Konsep Bersih Desktop POS) -->
  <main class="flex-1 flex items-center justify-center p-6 bg-slate-200 overflow-y-auto">
    <div class="w-full max-w-md bg-white border border-slate-300 rounded-2xl shadow-xs overflow-hidden">
      
      <!-- Card Header -->
      <div class="px-6 pt-6 pb-4 border-b border-slate-100 bg-slate-50/60 text-center">
        <div class="inline-flex items-center justify-center w-12 h-12 rounded-xl bg-blue-50 text-primary border border-blue-200 mb-2.5 shadow-2xs">
          <span class="material-symbols-outlined text-[28px]">badge</span>
        </div>
        <h1 class="text-xl font-black text-slate-900 tracking-tight">Masuk Sistem Kasir</h1>
        <p class="text-xs text-slate-500 mt-1">Pilih operator dan masukkan password untuk membuka sesi transaksi</p>
      </div>

      <div class="p-6">
        {#if errorMessage}
          <div class="mb-4 p-3 rounded-xl bg-red-50 border border-red-200 text-red-700 text-xs flex items-center gap-2.5">
            <span class="material-symbols-outlined text-[18px] shrink-0 text-red-600">error</span>
            <span class="font-medium">{errorMessage}</span>
          </div>
        {/if}

        <!-- Form -->
        <form onsubmit={handleSubmit} class="space-y-4">
          <!-- Dropdown Operator dari DB -->
          <div>
            <div class="flex items-center justify-between mb-1.5">
              <label for="select-operator" class="block text-xs font-bold text-slate-700 uppercase tracking-wider">
                Operator Kasir
              </label>
              {#if selectedOp}
                <div class="flex items-center gap-1">
                  <span class="text-[10px] font-bold px-1.5 py-0.2 rounded {selectedOp.is_admin ? 'bg-amber-100 text-amber-800 border border-amber-300' : 'bg-blue-100 text-blue-800 border border-blue-200'} font-sans uppercase">
                    {selectedOp.is_admin ? "ADMIN" : selectedOp.role}
                  </span>
                </div>
              {/if}
            </div>
            <div class="relative">
              <span class="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-slate-400 text-[18px] pointer-events-none">
                person
              </span>
              <select
                id="select-operator"
                bind:value={kode}
                onchange={(e) => handleOperatorChange((e.target as HTMLSelectElement).value)}
                class="w-full bg-slate-50 hover:bg-white focus:bg-white border border-slate-300 focus:border-primary focus:ring-1 focus:ring-primary rounded-xl pl-10 pr-9 py-2.5 text-sm font-medium text-slate-800 transition-all font-sans cursor-pointer appearance-none"
              >
                {#if operators.length === 0}
                  <option value="admin">Administrator (admin) • [Admin]</option>
                {:else}
                  {#each operators as op (op.id)}
                    <option value={op.kode}>
                      {op.nama} ({op.kode}) {op.is_admin ? "• [Admin]" : `• [${op.role}]`}
                    </option>
                  {/each}
                {/if}
              </select>
              <span class="material-symbols-outlined absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 pointer-events-none text-[20px]">
                unfold_more
              </span>
            </div>

            <!-- Quick Operator Switch Chips jika lebih dari 1 operator -->
            {#if operators.length > 1}
              <div class="flex items-center gap-1.5 mt-2 flex-wrap">
                <span class="text-[10px] text-slate-400 font-medium">Pilih cepat:</span>
                {#each operators.slice(0, 4) as op (op.id)}
                  <button
                    type="button"
                    onclick={() => handleOperatorChange(op.kode)}
                    class="text-[10px] font-mono font-bold px-2 py-0.5 rounded transition-colors cursor-pointer {kode === op.kode ? 'bg-primary text-white shadow-2xs' : 'bg-slate-100 hover:bg-slate-200 text-slate-700 border border-slate-300'}"
                  >
                    {op.kode}
                  </button>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Password -->
          <div>
            <label for="input-password" class="block text-xs font-bold text-slate-700 uppercase tracking-wider mb-1.5">
              Password Operator
            </label>
            <div class="relative">
              <span class="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-slate-400 text-[18px]">
                lock
              </span>
              <input
                id="input-password"
                type={showPassword ? "text" : "password"}
                bind:this={passwordInput}
                bind:value={password}
                placeholder="Masukkan password operator"
                autocomplete="current-password"
                class="w-full bg-slate-50 hover:bg-white focus:bg-white border border-slate-300 focus:border-primary focus:ring-1 focus:ring-primary rounded-xl pl-10 pr-11 py-2.5 text-sm font-medium text-slate-800 placeholder-slate-400 transition-all font-sans"
              />
              <button
                type="button"
                onclick={() => (showPassword = !showPassword)}
                class="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600 transition-colors cursor-pointer border-none bg-transparent p-0 flex items-center"
                title={showPassword ? "Sembunyikan password" : "Lihat password"}
              >
                <span class="material-symbols-outlined text-[18px]">
                  {showPassword ? "visibility_off" : "visibility"}
                </span>
              </button>
            </div>
          </div>

          <!-- Submit Button -->
          <button
            type="submit"
            disabled={isLoading}
            class="w-full py-2.5 px-4 bg-primary hover:bg-primary-dark active:bg-sky-800 disabled:opacity-50 text-white text-sm font-bold rounded-xl shadow-xs transition-all cursor-pointer border-none flex items-center justify-center gap-2 mt-4"
          >
            {#if isLoading}
              <span class="material-symbols-outlined text-[18px] animate-spin">progress_activity</span>
              <span>Memverifikasi...</span>
            {:else}
              <span class="material-symbols-outlined text-[18px]">login</span>
              <span>Masuk Sistem</span>
              <span class="font-mono text-[10px] bg-white/20 text-white px-1.5 py-0.5 rounded border border-white/30 font-bold ml-1">↵ Enter</span>
            {/if}
          </button>
        </form>

      </div>
    </div>
  </main>

  <!-- Footer Bar (Satu Konsep dengan FooterBar Layar Transaksi) -->
  <footer class="h-7 bg-slate-200 text-slate-800 border-t border-slate-300 flex items-center justify-between px-4 font-mono text-[10px] shrink-0 select-none shadow-xs">
    <div class="flex items-center gap-2">
      <span class="font-bold text-slate-700 uppercase tracking-wider font-sans text-xs">Pintasan:</span>

      <div class="flex items-center gap-1 bg-white border border-slate-300 px-1.5 py-0.5 rounded shadow-2xs">
        <span class="text-primary font-bold bg-blue-50 px-1 py-0.2 rounded border border-blue-200 text-[10px]">ENTER</span>
        <span class="text-slate-800 font-bold text-[10px]">Masuk Sistem</span>
      </div>

      <span class="text-slate-400">|</span>

      <div class="flex items-center gap-1 bg-white border border-slate-300 px-1.5 py-0.5 rounded shadow-2xs">
        <span class="text-slate-900 font-bold bg-slate-100 px-1 py-0.2 rounded border border-slate-300 text-[10px]">TAB</span>
        <span class="text-slate-800 font-bold text-[10px]">Pindah Input</span>
      </div>

      <span class="text-slate-400">|</span>

      <button
        type="button"
        onclick={() => api.closeWindow()}
        class="flex items-center gap-1 bg-white border border-rose-300 px-1.5 py-0.5 rounded hover:bg-rose-50 text-rose-700 font-bold text-[10px] cursor-pointer shadow-2xs transition-colors"
      >
        <span class="bg-rose-100 px-1 py-0.2 rounded border border-rose-300">ESC</span>
        <span>Keluar Aplikasi</span>
      </button>
    </div>

    <div class="flex items-center gap-2 text-slate-600 text-[10px]">
      <span class="flex items-center gap-1">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
        <span>SQLite Local WAL Engine</span>
      </span>
      <span class="text-slate-400">&bull;</span>
      <span>FazPos Desktop Standalone</span>
    </div>
  </footer>
</div>
