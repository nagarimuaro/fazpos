<script lang="ts">
  let {
    tokoNama = "MUEEZA STORE",
    pelanggan = "",
    logoUrl = "",
    logoIcon = "storefront",
  }: {
    tokoNama?: string;
    faktur?: string;
    pelanggan?: string;
    logoUrl?: string;
    logoIcon?: string;
  } = $props();
</script>

<div class="bg-slate-100 border-b border-slate-300 px-2.5 sm:px-5 py-2 sm:py-3 flex items-center justify-between shadow-xs shrink-0 select-none gap-2">
  <div class="flex items-center gap-2 sm:gap-3.5 min-w-0">
    <div class="w-8 h-8 sm:w-11 sm:h-11 rounded-lg sm:rounded-xl {logoUrl ? 'bg-white border border-slate-300 p-0.5 sm:p-1' : 'bg-primary text-white'} flex items-center justify-center shadow-xs shrink-0 overflow-hidden">
      {#if logoUrl}
        <img src={logoUrl} alt={tokoNama} class="w-full h-full object-contain" />
      {:else}
        <span class="material-symbols-outlined text-[20px] sm:text-[28px]">{logoIcon || "storefront"}</span>
      {/if}
    </div>
    <div class="flex flex-col sm:flex-row sm:items-baseline gap-0.5 sm:gap-3 min-w-0">
      <div class="text-base sm:text-2xl md:text-3xl font-black text-slate-900 uppercase tracking-tight leading-tight sm:leading-none truncate">
        {tokoNama}
      </div>
      {#if pelanggan && pelanggan !== "UMUM" && !pelanggan.toLowerCase().includes("umum") && !pelanggan.includes("Non-Member")}
        <div class="text-[10px] sm:text-xs text-slate-700 font-sans font-semibold bg-white/70 px-1.5 sm:px-2 py-0.5 rounded border border-slate-200 truncate">
          Pelanggan: <span class="text-slate-900 font-bold">{pelanggan}</span>
        </div>
      {/if}
    </div>
  </div>

  <div class="flex items-center gap-1 sm:gap-2 font-mono text-[10px] sm:text-[11px] shrink-0">
    <span class="px-2 sm:px-2.5 py-0.5 sm:py-1 rounded bg-emerald-100 text-emerald-800 font-semibold border border-emerald-300 flex items-center gap-1 shadow-2xs">
      <span class="w-1.5 sm:w-2 h-1.5 sm:h-2 rounded-full bg-emerald-500"></span>
      <span class="hidden sm:inline">Sistem Normal</span>
      <span class="sm:hidden">OK</span>
    </span>
  </div>
</div>
