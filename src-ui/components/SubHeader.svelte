<script lang="ts">
  let {
    tokoNama = "MUEEZA STORE",
    pelanggan = "",
  }: {
    tokoNama?: string;
    faktur?: string;
    pelanggan?: string;
  } = $props();
</script>

<div class="bg-slate-100 border-b border-slate-300 px-5 py-3 flex items-center justify-between shadow-xs shrink-0 select-none">
  <div class="flex items-center gap-3">
    <div class="flex items-center gap-3.5">
      <div class="w-11 h-11 rounded-xl bg-primary text-white flex items-center justify-center shadow-xs shrink-0">
        <span class="material-symbols-outlined text-[28px]">storefront</span>
      </div>
      <div class="flex items-baseline gap-3">
        <div class="text-3xl font-black text-slate-900 uppercase tracking-tight leading-none">
          {tokoNama}
        </div>
        {#if pelanggan && pelanggan !== "UMUM" && !pelanggan.toLowerCase().includes("umum") && !pelanggan.includes("Non-Member")}
          <div class="text-xs text-slate-700 font-sans font-semibold bg-white/70 px-2 py-0.5 rounded border border-slate-200">
            Pelanggan: <span class="text-slate-900 font-bold">{pelanggan}</span>
          </div>
        {/if}
      </div>
    </div>
  </div>

  <div class="flex items-center gap-2 font-mono text-[11px]">
    <span class="px-2.5 py-1 rounded bg-emerald-100 text-emerald-800 font-semibold border border-emerald-300 flex items-center gap-1 shadow-2xs">
      <span class="w-2 h-2 rounded-full bg-emerald-500"></span> Sistem Normal
    </span>
  </div>
</div>
