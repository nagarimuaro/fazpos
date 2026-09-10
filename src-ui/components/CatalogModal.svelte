<script lang="ts">
  import { api, formatRupiah, type ProductDTO, type CartSummaryDTO } from "../lib/api";

  let {
    isOpen,
    onClose,
    onCartUpdate,
  }: {
    isOpen: boolean;
    onClose: () => void;
    onCartUpdate: (e: CustomEvent<CartSummaryDTO>) => void;
  } = $props();

  let products = $state<ProductDTO[]>([]);
  let searchQuery = $state("");
  let activeCat = $state("Semua");
  let categories = $state<string[]>(["Semua"]);

  async function loadData() {
    try {
      products = await api.getCatalogProducts(searchQuery.trim() || undefined);
      const set = new Set(products.map((p) => p.kategori).filter(Boolean));
      categories = ["Semua", ...set];
    } catch (err) {
      console.error(err);
    }
  }

  async function handleSelect(p: ProductDTO) {
    try {
      const res = await api.scanBarcode(p.kode);
      onCartUpdate(new CustomEvent("cartUpdate", { detail: res }));
      onClose();
    } catch (err) {
      console.error(err);
    }
  }

  let filtered = $derived(
    activeCat === "Semua"
      ? products
      : products.filter((p) => p.kategori === activeCat)
  );

  $effect(() => {
    if (isOpen) {
      loadData();
    }
  });
</script>

{#if isOpen}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 bg-slate-900/50 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none"
    onclick={onClose}
    role="button"
    tabindex="0"
    onkeydown={(e) => e.key === "Escape" && onClose()}
  >
    <!-- Modal Card -->
    <div
      class="bg-white border border-slate-300 rounded-lg shadow-xl w-[720px] max-h-[80vh] flex flex-col overflow-hidden"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <!-- Modal Header -->
      <div class="px-4 py-3 bg-slate-50 border-b border-slate-200 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="material-symbols-outlined text-primary text-[20px]">inventory_2</span>
          <span class="font-sans font-bold text-slate-800 text-sm">Pencarian Katalog Produk</span>
        </div>
        <button
          onclick={onClose}
          class="text-slate-400 hover:text-slate-700 p-1 hover:bg-slate-200 rounded transition-colors border-none bg-transparent cursor-pointer"
        >
          <span class="material-symbols-outlined text-[18px]">close</span>
        </button>
      </div>

      <!-- Search Input & Categories -->
      <div class="p-3 border-b border-slate-200 bg-white flex flex-col gap-2">
        <div class="relative flex items-center">
          <span class="absolute left-3 text-slate-400 material-symbols-outlined text-[18px]">search</span>
          <input
            type="text"
            bind:value={searchQuery}
            oninput={loadData}
            placeholder="Cari nama produk, SKU, atau kategori..."
            class="w-full pl-9 pr-3 py-1.5 bg-slate-50 border border-slate-300 rounded text-xs font-sans focus:outline-none focus:border-primary focus:bg-white"
          />
        </div>

        <div class="flex items-center gap-1.5 overflow-x-auto py-1">
          {#each categories as c}
            <button
              class="px-2.5 py-0.5 rounded text-[11px] font-sans transition-colors cursor-pointer border {activeCat === c ? 'bg-slate-900 text-white border-slate-900 font-semibold' : 'bg-slate-100 text-slate-700 border-slate-200 hover:bg-slate-200'}"
              onclick={() => (activeCat = c)}
            >
              {c}
            </button>
          {/each}
        </div>
      </div>

      <!-- Product List -->
      <div class="flex-1 overflow-y-auto p-3 grid grid-cols-2 gap-2">
        {#each filtered as p}
          <div
            class="p-2.5 border border-slate-200 rounded hover:border-primary hover:bg-sky-50/40 transition-colors cursor-pointer flex flex-col justify-between"
            onclick={() => handleSelect(p)}
            role="button"
            tabindex="0"
            onkeydown={(e) => e.key === "Enter" && handleSelect(p)}
          >
            <div>
              <div class="flex items-center justify-between">
                <span class="font-mono text-[10px] text-slate-500 font-bold">{p.kode}</span>
                <span class="text-[10px] px-1.5 py-0.2 bg-slate-100 text-slate-600 rounded">{p.satuan}</span>
              </div>
              <div class="font-sans font-semibold text-xs text-slate-900 mt-1">{p.nama}</div>
              <div class="text-[10px] text-slate-500 font-sans">{p.kategori}</div>
            </div>

            <div class="flex items-center justify-between mt-2 pt-2 border-t border-slate-100">
              <span class="font-mono text-xs font-bold text-primary">{formatRupiah(p.hargajual1)}</span>
              <span class="text-[10px] font-mono {p.is_kritis ? 'text-amber-600 font-bold' : 'text-emerald-600'}">
                Stok: {p.stok}
              </span>
            </div>
          </div>
        {:else}
          <div class="col-span-2 py-8 text-center text-slate-400 font-mono text-xs">
            Tidak ada produk yang cocok dengan pencarian
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}
