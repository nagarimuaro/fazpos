<script lang="ts">
  let {
    isOpen,
    onConfirm,
    onClose,
  }: {
    isOpen: boolean;
    onConfirm: () => void;
    onClose: () => void;
  } = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (!isOpen) return;
    if (e.key === "Enter") {
      e.preventDefault();
      onConfirm();
    } else if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    }
  }

  $effect(() => {
    if (isOpen) {
      window.addEventListener("keydown", handleKeydown);
      return () => window.removeEventListener("keydown", handleKeydown);
    }
  });
</script>

{#if isOpen}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 bg-slate-900/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none"
    onclick={onClose}
    role="button"
    tabindex="0"
    onkeydown={(e) => e.key === "Escape" && onClose()}
  >
    <!-- Modal Card -->
    <div
      class="bg-white border border-slate-300 rounded-xl shadow-2xl w-[440px] overflow-hidden animate-in fade-in zoom-in-95 duration-150"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="px-5 py-4 bg-slate-100 border-b border-slate-200 flex items-center gap-3">
        <div class="w-10 h-10 rounded-full bg-amber-100 text-amber-700 flex items-center justify-center shrink-0 border border-amber-300">
          <span class="material-symbols-outlined text-[24px]">logout</span>
        </div>
        <div>
          <div class="font-bold text-slate-900 text-sm font-sans">Konfirmasi Tutup Kasir</div>
          <div class="text-xs text-slate-500 font-sans">Kembali ke Halaman Riwayat Transaksi</div>
        </div>
      </div>

      <!-- Body -->
      <div class="p-5 text-xs text-slate-700 font-sans leading-relaxed">
        Apakah Anda yakin ingin keluar dari sesi kasir aktif dan kembali ke daftar transaksi? Keranjang belanja aktif akan tetap tersimpan.
      </div>

      <!-- Footer Buttons -->
      <div class="px-5 py-3.5 bg-slate-50 border-t border-slate-200 flex items-center justify-end gap-2.5">
        <button
          type="button"
          onclick={onClose}
          class="px-4 py-2 bg-white hover:bg-slate-100 text-slate-700 border border-slate-300 rounded-lg text-xs font-bold shadow-2xs transition-colors cursor-pointer"
        >
          Batal / Lanjut Kasir (ESC)
        </button>
        <button
          type="button"
          onclick={onConfirm}
          class="px-4 py-2 bg-rose-600 hover:bg-rose-700 text-white rounded-lg text-xs font-bold shadow-sm transition-colors cursor-pointer border-none flex items-center gap-1"
        >
          <span class="material-symbols-outlined text-[16px]">check</span>
          Ya, Tutup Kasir (Enter)
        </button>
      </div>
    </div>
  </div>
{/if}
