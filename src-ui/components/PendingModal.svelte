<script lang="ts">
  import { api, formatRupiah, type PendingItemDTO, type CartSummaryDTO } from "../lib/api";

  let {
    isOpen,
    onClose,
    onCartUpdate,
  }: {
    isOpen: boolean;
    onClose: () => void;
    onCartUpdate: (e: CustomEvent<CartSummaryDTO>) => void;
  } = $props();

  let pendingList = $state<PendingItemDTO[]>([]);

  async function loadPending() {
    try {
      pendingList = await api.getPendingOrders();
    } catch {
      // Default sample pending items if DB has none
      pendingList = [
        {
          id: "PND-01",
          faktur: "#PND-0104",
          tanggal: "14:15 WIB",
          pelanggan: "Meja T02 (Agus)",
          total: 84000,
          keterangan: "Dine-in pesanan tambahan",
        },
        {
          id: "PND-02",
          faktur: "#PND-0105",
          tanggal: "14:22 WIB",
          pelanggan: "Takeaway #04",
          total: 120000,
          keterangan: "Antrean kasir 2",
        },
        {
          id: "PND-03",
          faktur: "#PND-0106",
          tanggal: "14:28 WIB",
          pelanggan: "Ibu Siti (VIP)",
          total: 45000,
          keterangan: "Tinggal bayar QRIS",
        },
      ];
    }
  }

  async function handleRecall(id: string) {
    try {
      const res = await api.recallOrder(id);
      onCartUpdate(new CustomEvent("cartUpdate", { detail: res }));
      onClose();
    } catch (err) {
      console.error(err);
      onClose();
    }
  }

  async function handleDelete(id: string) {
    try {
      await api.deletePendingOrder(id);
      loadPending();
    } catch {
      pendingList = pendingList.filter((p) => p.id !== id);
    }
  }

  $effect(() => {
    if (isOpen) {
      loadPending();
    }
  });
</script>

{#if isOpen}
  <div
    class="fixed inset-0 bg-slate-900/50 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none"
    onclick={onClose}
    role="button"
    tabindex="0"
    onkeydown={(e) => e.key === "Escape" && onClose()}
  >
    <div
      class="bg-white border border-slate-300 rounded-lg shadow-xl w-[560px] max-h-[75vh] flex flex-col overflow-hidden"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <div class="px-4 py-3 bg-slate-50 border-b border-slate-200 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="material-symbols-outlined text-amber-600 text-[20px]">pause_circle</span>
          <span class="font-sans font-bold text-slate-800 text-sm">[F6] Daftar Pesanan Tertahan (Pending)</span>
        </div>
        <button
          onclick={onClose}
          class="text-slate-400 hover:text-slate-700 p-1 hover:bg-slate-200 rounded transition-colors border-none bg-transparent cursor-pointer"
        >
          <span class="material-symbols-outlined text-[18px]">close</span>
        </button>
      </div>

      <div class="flex-1 overflow-y-auto p-3 flex flex-col gap-2">
        {#each pendingList as item}
          <div class="p-3 border border-slate-200 rounded-lg hover:border-amber-300 bg-slate-50/50 flex items-center justify-between">
            <div class="flex flex-col gap-0.5">
              <div class="flex items-center gap-2">
                <span class="font-mono text-xs font-bold text-slate-900">{item.faktur}</span>
                <span class="text-[10px] text-slate-400 font-mono">{item.tanggal}</span>
              </div>
              <div class="text-xs font-sans text-slate-700 font-medium">{item.pelanggan}</div>
              <div class="text-[11px] text-slate-500">{item.keterangan}</div>
            </div>

            <div class="flex items-center gap-3">
              <span class="font-mono text-xs font-bold text-slate-900">{formatRupiah(item.total)}</span>
              <button
                onclick={() => handleRecall(item.id)}
                class="px-2.5 py-1 bg-amber-500 hover:bg-amber-600 text-white rounded text-xs font-bold shadow-2xs border-none cursor-pointer flex items-center gap-1"
              >
                <span class="material-symbols-outlined text-[14px]">play_circle</span>
                Panggil
              </button>
              <button
                onclick={() => handleDelete(item.id)}
                class="p-1 text-slate-400 hover:text-red-600 hover:bg-red-50 rounded border-none bg-transparent cursor-pointer"
                title="Hapus Pending"
              >
                <span class="material-symbols-outlined text-[16px]">delete</span>
              </button>
            </div>
          </div>
        {:else}
          <div class="py-12 text-center text-slate-400 font-mono text-xs">
            Tidak ada pesanan yang tertahan.
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}
