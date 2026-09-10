<script lang="ts">
  import { api, formatRupiah, type CartSummaryDTO } from "../lib/api";

  let {
    cart,
    pendingCount = 3,
    onCartUpdate,
    onOpenCatalog,
    onOpenPending,
  }: {
    cart: CartSummaryDTO;
    pendingCount?: number;
    onCartUpdate: (e: CustomEvent<CartSummaryDTO>) => void;
    onOpenCatalog: () => void;
    onOpenPending: () => void;
  } = $props();

  let barcodeInput = $state("");
  let selectedIndex = $state<number>(0);
  let barcodeInputElement: HTMLInputElement | null = $state(null);

  function emit(result: CartSummaryDTO) {
    onCartUpdate(new CustomEvent("cartUpdate", { detail: result }));
  }

  async function handleBarcodeSubmit() {
    const raw = barcodeInput.trim();
    if (!raw) return;

    let multiplier = 1;
    let code = raw;

    // Support syntax 2*SKU-1001 or 5*12345
    if (raw.includes("*")) {
      const parts = raw.split("*");
      const parsedMult = parseFloat(parts[0]);
      if (!isNaN(parsedMult) && parsedMult > 0 && parts[1]) {
        multiplier = parsedMult;
        code = parts[1].trim();
      }
    }

    try {
      let res = await api.scanBarcode(code);
      if (multiplier !== 1) {
        // Find the index of the newly added/updated item
        const idx = res.items.findIndex((it) => it.kode === code || it.id === code);
        if (idx !== -1) {
          res = await api.updateCartQty(idx, multiplier);
        }
      }
      emit(res);
      selectedIndex = res.items.length - 1;
      barcodeInput = "";
    } catch (err) {
      console.error("Scan error:", err);
    }
  }

  async function updateQty(index: number, newQty: number) {
    if (index < 0 || index >= cart.items.length) return;
    if (newQty <= 0) {
      emit(await api.removeCartItem(index));
      if (selectedIndex >= cart.items.length - 1) {
        selectedIndex = Math.max(0, cart.items.length - 2);
      }
    } else {
      emit(await api.updateCartQty(index, newQty));
    }
  }

  async function removeItem(index: number) {
    emit(await api.removeCartItem(index));
    if (selectedIndex >= cart.items.length - 1) {
      selectedIndex = Math.max(0, cart.items.length - 2);
    }
  }

  function handleKeydownGlobal(e: KeyboardEvent) {
    if (e.key === "F2") {
      e.preventDefault();
      barcodeInputElement?.focus();
      barcodeInputElement?.select();
    } else if (e.key === "F6") {
      e.preventDefault();
      onOpenPending();
    } else if (e.key === "ArrowDown") {
      if (document.activeElement !== barcodeInputElement) {
        e.preventDefault();
        selectedIndex = Math.min(cart.items.length - 1, selectedIndex + 1);
      }
    } else if (e.key === "ArrowUp") {
      if (document.activeElement !== barcodeInputElement) {
        e.preventDefault();
        selectedIndex = Math.max(0, selectedIndex - 1);
      }
    } else if (e.key === "+" && document.activeElement !== barcodeInputElement) {
      e.preventDefault();
      if (cart.items[selectedIndex]) {
        updateQty(selectedIndex, cart.items[selectedIndex].jumlah + 1);
      }
    } else if (e.key === "-" && document.activeElement !== barcodeInputElement) {
      e.preventDefault();
      if (cart.items[selectedIndex]) {
        updateQty(selectedIndex, cart.items[selectedIndex].jumlah - 1);
      }
    } else if (e.key === "Delete" && document.activeElement !== barcodeInputElement) {
      e.preventDefault();
      if (cart.items[selectedIndex]) {
        removeItem(selectedIndex);
      }
    }
  }

  let totalQty = $derived(
    cart.items.reduce((sum, it) => sum + it.jumlah, 0)
  );

  let lastItemName = $derived(
    cart.items.length > 0
      ? `${cart.items[cart.items.length - 1].kode} (${cart.items[cart.items.length - 1].nama})`
      : "-"
  );

  $effect(() => {
    window.addEventListener("keydown", handleKeydownGlobal);
    return () => window.removeEventListener("keydown", handleKeydownGlobal);
  });
</script>

<section class="flex-1 flex flex-col bg-white border border-slate-300 rounded-md shadow-sm overflow-hidden">
  <!-- Top Barcode Search & Fast Multipliers -->
  <div class="p-2 border-b border-slate-300 bg-slate-100/90 flex flex-col gap-2">
    <div class="flex items-center gap-2">
      <!-- Input Barcode -->
      <div class="relative flex-1">
        <div class="absolute inset-y-0 left-0 pl-2.5 flex items-center pointer-events-none text-primary">
          <span class="material-symbols-outlined text-[18px]">barcode_scanner</span>
        </div>
        <input
          bind:this={barcodeInputElement}
          bind:value={barcodeInput}
          onkeydown={(e) => e.key === "Enter" && handleBarcodeSubmit()}
          class="w-full pl-9 pr-24 py-2 bg-white border border-slate-400 rounded font-mono text-xs text-slate-900 placeholder-slate-400 focus:outline-none focus:bg-white focus:border-primary focus:ring-1 focus:ring-primary shadow-2xs"
          placeholder="Scan Barcode / Ketik SKU atau Nama Produk..."
          type="text"
        />
        <div class="absolute inset-y-0 right-0 pr-2 flex items-center gap-1.5 pointer-events-none">
          <span class="px-1.5 py-0.5 rounded bg-emerald-100 text-emerald-800 font-mono text-[10px] font-bold border border-emerald-300 flex items-center gap-1">
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-600 animate-ping"></span> READY
          </span>
        </div>
      </div>

      <!-- Cari Katalog Button -->
      <button
        onclick={onOpenCatalog}
        class="px-3 py-2 bg-slate-50 hover:bg-slate-200 border border-slate-300 rounded font-mono text-[11px] font-medium text-slate-700 flex items-center gap-1.5 shadow-2xs transition-colors cursor-pointer"
        title="Pencarian Katalog Produk"
      >
        <span class="material-symbols-outlined text-[15px] text-slate-600">search</span>
        <span>Cari Katalog</span>
      </button>

      <!-- Non-Katalog Button -->
      <button
        class="px-3 py-2 bg-blue-100/70 text-primary-dark hover:bg-blue-100 border border-blue-300 rounded font-mono text-[11px] font-medium flex items-center gap-1.5 shadow-2xs transition-colors cursor-pointer"
        onclick={() => {
          const name = prompt("Nama Item Non-Katalog:");
          if (name) {
            const price = parseFloat(prompt("Harga Satuan (Rp):") || "0");
            if (price > 0) {
              api.scanBarcode("SKU-1007").then((r) => emit(r));
            }
          }
        }}
      >
        <span class="material-symbols-outlined text-[15px]">add_box</span>
        <span>Non-Katalog</span>
      </button>

      <!-- List Pending Button [F6] -->
      <button
        onclick={onOpenPending}
        class="relative px-3 py-2 bg-amber-100 hover:bg-amber-200/80 text-amber-800 border border-amber-300 rounded font-mono text-[11px] font-medium flex items-center gap-1.5 shadow-2xs cursor-pointer transition-colors"
        title="Daftar Pesanan Ditahan [F6]"
      >
        <span class="material-symbols-outlined text-[15px] text-amber-700">pause_circle</span>
        <span class="font-bold text-slate-900">[F6] List Pending</span>
        <span class="px-1.5 py-0.5 rounded-full bg-amber-400 text-slate-900 text-[10px] font-bold shadow-2xs tabular-nums leading-tight ml-0.5">
          {pendingCount}
        </span>
      </button>
    </div>

    <!-- Multiplier Syntax & Totals Row -->
    <div class="flex items-center justify-between px-2 py-1 bg-slate-200/80 rounded border border-slate-300 text-[11px] font-mono text-slate-600">
      <div class="flex items-center gap-2">
        <span class="font-bold text-slate-700">Sintaks Multiplier Cepat:</span>
        <span class="px-1.5 py-0.5 bg-white text-slate-800 rounded font-bold border border-slate-300 shadow-2xs">2*SKU-1001</span>
        <span class="text-slate-500">atau</span>
        <span class="px-1.5 py-0.5 bg-white text-slate-800 rounded font-bold border border-slate-300 shadow-2xs">5*[BARCODE]</span>
        <span class="text-slate-500">• Scan langsung otomatis menambahkan baris</span>
      </div>
      <div class="flex items-center gap-2 text-slate-700 font-medium">
        <span>Total Baris: <strong class="text-slate-900">{cart.items.length}</strong></span>
        <span class="text-slate-400">|</span>
        <span>Total Qty: <strong class="text-slate-900">{totalQty}</strong> Unit</span>
      </div>
    </div>
  </div>

  <!-- Cart Table -->
  <div class="flex-1 overflow-y-auto bg-white">
    <table class="w-full text-left border-collapse font-sans text-xs">
      <thead class="sticky top-0 bg-slate-800 text-slate-100 font-mono text-[11px] font-bold border-b-2 border-slate-900 select-none z-10 shadow-sm">
        <tr>
          <th class="py-2.5 px-2.5 font-bold w-10 text-center">NO</th>
          <th class="py-2.5 px-2 font-bold w-28">BARCODE / SKU</th>
          <th class="py-2.5 px-2 font-bold min-w-[180px]">NAMA BARANG</th>
          <th class="py-2.5 px-1 font-bold text-center w-28">QTY</th>
          <th class="py-2.5 px-2 font-bold text-center w-16">SATUAN</th>
          <th class="py-2.5 px-2 font-bold text-right w-28">HARGA SATUAN</th>
          <th class="py-2.5 px-2 font-bold text-center w-20">DISC (%)</th>
          <th class="py-2.5 px-2 font-bold text-right w-32">SUBTOTAL</th>
          <th class="py-2.5 px-2 font-bold text-center w-14">AKSI</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-slate-200 font-mono text-[11px]" id="cart-table-body">
        {#each cart.items as item, i}
          <tr
            class="transition-colors group {selectedIndex === i ? 'bg-sky-100/90 font-medium border-l-4 border-primary' : i % 2 === 1 ? 'bg-slate-100/70' : 'bg-white'} hover:bg-sky-50/80 cursor-pointer"
            onclick={() => (selectedIndex = i)}
          >
            <td class="py-2 px-2.5 text-center text-slate-500 font-semibold">{i + 1}</td>
            <td class="py-2 px-2 text-slate-700 font-bold">{item.kode}</td>
            <td class="py-2 px-2 font-sans font-semibold text-slate-900">
              <div class="flex items-center gap-1.5">
                <span>{item.nama}</span>
                {#if i === 0 && cart.nilai_tukar_poin > 0}
                  <span class="font-mono text-[9px] px-1.5 py-0.5 bg-emerald-100 text-emerald-800 rounded font-bold border border-emerald-300">Promo</span>
                {/if}
              </div>
            </td>
            <td class="py-2 px-1 text-center">
              <div class="inline-flex items-center border border-slate-400 rounded bg-white shadow-2xs">
                <button
                  class="w-5 h-5 bg-slate-50 hover:bg-slate-200 text-slate-800 font-bold leading-none border-r border-slate-300 cursor-pointer p-0"
                  onclick={(e) => { e.stopPropagation(); updateQty(i, item.jumlah - 1); }}
                >
                  -
                </button>
                <input
                  class="w-8 h-5 text-center font-mono text-xs font-bold p-0 border-none bg-transparent focus:ring-0"
                  type="text"
                  value={item.jumlah}
                  onchange={(e) => {
                    const v = parseFloat(e.currentTarget.value);
                    if (!isNaN(v)) updateQty(i, v);
                  }}
                  onclick={(e) => e.stopPropagation()}
                />
                <button
                  class="w-5 h-5 bg-slate-50 hover:bg-slate-200 text-slate-800 font-bold leading-none border-l border-slate-300 cursor-pointer p-0"
                  onclick={(e) => { e.stopPropagation(); updateQty(i, item.jumlah + 1); }}
                >
                  +
                </button>
              </div>
            </td>
            <td class="py-2 px-2 text-center text-slate-600 font-sans">{item.satuan}</td>
            <td class="py-2 px-2 text-right text-slate-700 tabular-nums font-medium">{formatRupiah(item.harga)}</td>
            <td class="py-2 px-2 text-center {i === 0 && cart.nilai_tukar_poin > 0 ? 'text-emerald-700 font-bold' : 'text-slate-500'} tabular-nums">
              {i === 0 && cart.nilai_tukar_poin > 0 ? "10%" : "0%"}
            </td>
            <td class="py-2 px-2 text-right font-bold text-slate-900 tabular-nums">
              {formatRupiah(item.subtotal)}
            </td>
            <td class="py-2 px-2 text-center">
              <button
                class="text-slate-500 hover:text-red-600 p-1 hover:bg-red-50 rounded transition-colors cursor-pointer border-none bg-transparent"
                title="Hapus Baris [DEL]"
                onclick={(e) => { e.stopPropagation(); removeItem(i); }}
              >
                <span class="material-symbols-outlined text-[16px]">delete</span>
              </button>
            </td>
          </tr>
        {:else}
          <tr>
            <td colspan="9" class="py-12 text-center text-slate-500 font-mono text-xs">
              Keranjang masih kosong. Scan barcode atau klik [Cari Katalog] di atas.
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  <!-- Table Footer (32px) -->
  <div class="h-8 px-3 bg-slate-200 text-slate-700 border-t border-slate-300 flex items-center justify-between font-mono text-[10px] select-none font-medium">
    <div class="flex items-center gap-3">
      <span>Baris Terakhir: <strong class="text-slate-900 font-bold">{lastItemName}</strong></span>
      <span class="text-slate-400">|</span>
      <span>Scan Engine: <strong class="text-emerald-700 font-bold">Ready (EAN-13, QR, SKU)</strong></span>
    </div>
    <div class="flex items-center gap-3">
      <span class="text-slate-600">
        Pintasan Tabel:
        <kbd class="px-1 py-0.5 bg-white border border-slate-300 rounded font-bold text-slate-800 shadow-2xs">▲/▼</kbd> Pilih Baris
        <kbd class="px-1 py-0.5 bg-white border border-slate-300 rounded font-bold text-slate-800 shadow-2xs">+/-</kbd> Ubah Qty
        <kbd class="px-1 py-0.5 bg-white border border-slate-300 rounded font-bold text-slate-800 shadow-2xs">DEL</kbd> Hapus
      </span>
    </div>
  </div>
</section>
