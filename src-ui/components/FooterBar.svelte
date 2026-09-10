<script lang="ts">
  let {
    pendingCount = 3,
    onOpenPending,
    onVoidCart,
  }: {
    pendingCount?: number;
    onOpenPending?: () => void;
    onVoidCart?: () => void;
  } = $props();
</script>

<footer class="h-7 bg-slate-200 text-slate-800 border-t border-slate-300 flex items-center justify-between px-3 font-mono text-[10px] shrink-0 select-none shadow-xs">
  <div class="flex items-center gap-3 w-full justify-between font-mono text-[11px]">
    <div class="flex items-center gap-2">
      <span class="text-xs font-bold text-slate-700 uppercase tracking-wider font-sans">Pintasan Kasir:</span>

      <div class="flex items-center gap-1 bg-white border border-slate-300 px-1.5 py-0.5 rounded shadow-2xs">
        <span class="text-primary font-bold bg-blue-50 px-1 py-0.5 rounded border border-blue-200 text-[10px]">SPACE / F2</span>
        <span class="text-primary-dark font-bold text-[10px]">Bayar</span>
      </div>

      <span class="text-slate-400">|</span>

      <div class="flex items-center gap-1 bg-white border border-slate-300 px-1.5 py-0.5 rounded shadow-2xs">
        <span class="text-slate-900 font-bold bg-slate-100 px-1 py-0.5 rounded border border-slate-300 text-[10px]">F3</span>
        <span class="text-slate-800 font-bold text-[10px]">Simpan Transaksi</span>
      </div>

      <span class="text-slate-400">|</span>

      <div class="flex items-center gap-1 bg-white border border-slate-300 px-1.5 py-0.5 rounded shadow-2xs">
        <span class="text-slate-900 font-bold bg-slate-100 px-1 py-0.5 rounded border border-slate-300 text-[10px]">F4</span>
        <span class="text-slate-800 font-bold text-[10px]">Cetak Struk</span>
      </div>

      <span class="text-slate-400">|</span>

      <button
        onclick={onOpenPending}
        class="flex items-center gap-1 bg-white border border-amber-300 px-1.5 py-0.5 rounded shadow-2xs cursor-pointer"
      >
        <span class="text-slate-900 font-bold bg-amber-400 px-1 py-0.5 rounded text-[10px]">F6</span>
        <span class="text-amber-800 font-bold text-[10px]">List Pending ({pendingCount})</span>
      </button>

      <span class="text-slate-400">|</span>

      <div class="flex items-center gap-1 bg-white border border-slate-300 px-1.5 py-0.5 rounded shadow-2xs">
        <span class="text-slate-900 font-bold bg-slate-100 px-1 py-0.5 rounded border border-slate-300 text-[10px]">F10</span>
        <span class="text-slate-800 font-bold text-[10px]">Pending Pesanan</span>
      </div>
    </div>

    <button
      onclick={onVoidCart}
      class="flex items-center gap-1 cursor-pointer border border-rose-300 bg-white px-2 py-0.5 rounded hover:bg-rose-50 transition-colors"
    >
      <span class="text-rose-700 font-bold bg-rose-100 px-1 py-0.5 rounded border border-rose-300 text-[10px]">ESC</span>
      <span class="text-rose-700 font-bold text-[10px]">Batal / Void</span>
    </button>
  </div>
</footer>
