<script lang="ts">
  import { api, formatRupiah, type ProductDTO, type CartSummaryDTO } from "../lib/api";

  let { onCartUpdate }: { onCartUpdate: (e: CustomEvent<CartSummaryDTO>) => void } = $props();

  let products: ProductDTO[] = $state([]);
  let searchQuery = $state("");
  let activeCategory = $state("Semua");
  let inputElement: HTMLInputElement | null = $state(null);

  // Default mockup categories
  const defaultCategories = [
    { label: "Semua", count: 84 },
    { label: "Minuman Kopi", count: 22 },
    { label: "Minuman Non-Kopi", count: 18 },
    { label: "Bakery & Pastry", count: 16 },
    { label: "Makanan Berat", count: 14 },
    { label: "Retail / Kemasan", count: 14 },
    { label: "● Paket Hemat", count: null },
  ];

  async function loadProducts() {
    try {
      const kw = searchQuery.trim() || undefined;
      products = await api.getCatalogProducts(kw);
    } catch (e) {
      console.error("loadProducts:", e);
    }
  }

  async function addToCart(product: ProductDTO) {
    try {
      const result = await api.scanBarcode(product.kode);
      onCartUpdate(new CustomEvent("cartUpdate", { detail: result }));
    } catch (e) {
      console.error("addToCart:", e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      if (searchQuery.trim()) {
        api.scanBarcode(searchQuery.trim()).then((res) => {
          onCartUpdate(new CustomEvent("cartUpdate", { detail: res }));
          searchQuery = "";
        }).catch(() => {
          loadProducts();
        });
      } else {
        loadProducts();
      }
    }
  }

  // Global hotkey F2 to focus scan
  function handleWindowKeydown(e: KeyboardEvent) {
    if (e.key === "F2") {
      e.preventDefault();
      inputElement?.focus();
      inputElement?.select();
    }
  }

  let filtered = $derived(
    activeCategory === "Semua"
      ? products
      : products.filter((p) => p.kategori.toLowerCase().includes(activeCategory.toLowerCase().replace("●", "").trim()))
  );

  $effect(() => {
    loadProducts();
    window.addEventListener("keydown", handleWindowKeydown);
    return () => window.removeEventListener("keydown", handleWindowKeydown);
  });
</script>

<div class="product-catalog-card">
  <!-- Search & Category Area (82px) -->
  <div class="search-section">
    <!-- Top row: Barcode input + buttons -->
    <div class="search-top-row">
      <div class="input-wrapper">
        <input
          bind:this={inputElement}
          type="text"
          class="barcode-input"
          placeholder="Scan Barcode EAN-13 / Masukkan SKU / Nama Barang... [F2]"
          bind:value={searchQuery}
          onkeydown={handleKeydown}
        />
      </div>
      <button class="btn-tool" onclick={loadProducts}>
        <span class="icon">☷</span> Filter Cepat
      </button>
      <button class="btn-tool btn-non-katalog">
        <span class="icon">+</span> Item Non-Katalog [F6]
      </button>
    </div>

    <!-- Bottom row: Categories -->
    <div class="category-row">
      <span class="category-label">KATEGORI:</span>
      <div class="category-buttons">
        {#each defaultCategories as cat}
          <button
            class="btn-category"
            class:active={activeCategory === cat.label}
            onclick={() => (activeCategory = cat.label)}
          >
            {cat.label}{cat.count !== null ? ` (${cat.count})` : ""}
          </button>
        {/each}
      </div>
    </div>
  </div>

  <!-- Table Area -->
  <div class="table-container">
    <div class="table-header-row">
      <div class="th col-sku">SKU / BARCODE</div>
      <div class="th col-name">NAMA PRODUK</div>
      <div class="th col-cat">KATEGORI</div>
      <div class="th col-stock">STOK</div>
      <div class="th col-price">HARGA SATUAN</div>
      <div class="th col-qty">QTY</div>
      <div class="th col-action">AKSI</div>
    </div>

    <div class="table-body">
      {#each filtered as product}
        <div class="table-row">
          <div class="td col-sku sku-text">{product.kode}</div>
          <div class="td col-name name-text">{product.nama}</div>
          <div class="td col-cat cat-text">{product.kategori}</div>
          <div class="td col-stock stock-text" class:kritis={product.is_kritis}>
            {product.stok} unit
          </div>
          <div class="td col-price price-text">{formatRupiah(product.hargajual1)}</div>
          <div class="td col-qty">
            <button class="btn-qty-mini" onclick={() => addToCart(product)}>-  1  +</button>
          </div>
          <div class="td col-action">
            <button class="btn-add" onclick={() => addToCart(product)}>+ Tambah</button>
          </div>
        </div>
      {:else}
        <div class="empty-state">
          <span>Tidak ada produk yang cocok</span>
        </div>
      {/each}
    </div>
  </div>

  <!-- Catalog Footer Info Bar (28px) -->
  <div class="catalog-footer">
    <span class="footer-item">Menampilkan: {filtered.length} dari {products.length || 84} SKU</span>
    <span class="footer-item">Index SQLite: 48ms</span>
    <span class="footer-item sync-ok">● Katalog Sinkron 100%</span>
  </div>
</div>

<style>
  .product-catalog-card {
    width: 65%;
    height: 100%;
    background: #ffffff;
    border: 1px solid #cbd5e1;
    border-radius: 4px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* ── Search Area (82px) ── */
  .search-section {
    height: 82px;
    background: #f1f5f9;
    border-bottom: 1px solid #e2e8f0;
    padding: 8px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    flex-shrink: 0;
  }

  .search-top-row {
    display: flex;
    gap: 8px;
    height: 42px;
  }

  .input-wrapper {
    flex: 1;
    display: flex;
  }

  .barcode-input {
    width: 100%;
    height: 42px;
    padding: 0 12px;
    border: 1px solid #cbd5e1;
    border-radius: 4px;
    background: #ffffff;
    font-size: 13px;
    color: #0f172a;
  }

  .barcode-input:focus {
    border-color: #0284c7;
    box-shadow: 0 0 0 1px #0284c7;
  }

  .barcode-input::placeholder {
    color: #94a3b8;
    font-size: 12px;
  }

  .btn-tool {
    height: 42px;
    padding: 0 12px;
    font-size: 12px;
    font-weight: 500;
    background: #ffffff;
    border: 1px solid #cbd5e1;
    color: #334155;
    border-radius: 4px;
    white-space: nowrap;
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .btn-tool:hover {
    background: #f8fafc;
  }

  .btn-non-katalog {
    color: #0369a1;
  }

  .category-row {
    display: flex;
    align-items: center;
    gap: 6px;
    overflow-x: auto;
  }

  .category-label {
    color: #94a3b8;
    font-size: 10px;
    font-weight: 700;
    white-space: nowrap;
  }

  .category-buttons {
    display: flex;
    gap: 4px;
    overflow-x: auto;
  }

  .btn-category {
    height: 22px;
    padding: 0 8px;
    font-size: 11px;
    font-weight: 500;
    background: #ffffff;
    border: 1px solid #cbd5e1;
    color: #334155;
    border-radius: 3px;
    white-space: nowrap;
  }

  .btn-category:hover {
    background: #f8fafc;
  }

  .btn-category.active {
    background: #0f172a;
    border-color: #0f172a;
    color: #ffffff;
    font-weight: 600;
  }

  /* ── Table Container ── */
  .table-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .table-header-row {
    height: 32px;
    background: #e8edf3;
    display: flex;
    align-items: center;
    padding: 0 8px;
    border-bottom: 1px solid #cbd5e1;
    flex-shrink: 0;
  }

  .th {
    font-size: 10px;
    font-weight: 700;
    color: #475569;
    letter-spacing: 0.3px;
  }

  .col-sku { width: 120px; }
  .col-name { flex: 1; min-width: 140px; }
  .col-cat { width: 120px; }
  .col-stock { width: 80px; text-align: right; }
  .col-price { width: 120px; text-align: right; }
  .col-qty { width: 80px; text-align: center; }
  .col-action { width: 90px; text-align: center; }

  .table-body {
    flex: 1;
    overflow-y: auto;
  }

  .table-row {
    height: 42px;
    display: flex;
    align-items: center;
    padding: 0 8px;
    border-bottom: 1px solid #f1f5f9;
    transition: background-color 0.1s;
  }

  .table-row:hover {
    background: #f8fafc;
  }

  .td {
    font-size: 12px;
  }

  .sku-text {
    color: #64748b;
    font-family: inherit;
  }

  .name-text {
    color: #0f172a;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .cat-text {
    color: #64748b;
  }

  .stock-text {
    color: #047857;
    font-weight: 600;
  }

  .stock-text.kritis {
    color: #d97706;
  }

  .price-text {
    color: #0f172a;
    font-weight: 700;
  }

  .btn-qty-mini {
    width: 80px;
    height: 28px;
    font-size: 11px;
    font-weight: 600;
    background: #ffffff;
    border: 1px solid #cbd5e1;
    color: #334155;
    border-radius: 4px;
  }

  .btn-qty-mini:hover {
    background: #f1f5f9;
  }

  .btn-add {
    width: 90px;
    height: 28px;
    font-size: 11px;
    font-weight: 600;
    background: #0284c7;
    border: 1px solid #0284c7;
    color: #ffffff;
    border-radius: 4px;
  }

  .btn-add:hover {
    background: #0369a1;
  }

  .empty-state {
    padding: 40px;
    text-align: center;
    color: #94a3b8;
    font-size: 12px;
  }

  /* ── Catalog Footer (28px) ── */
  .catalog-footer {
    height: 28px;
    background: #f1f5f9;
    border-top: 1px solid #e2e8f0;
    padding: 0 10px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
  }

  .footer-item {
    color: #475569;
    font-size: 10px;
  }

  .sync-ok {
    color: #047857;
    font-weight: 600;
  }
</style>
