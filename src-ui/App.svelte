<script lang="ts">
  import "./global.css";
  import TopBar from "./components/TopBar.svelte";
  import SubHeader from "./components/SubHeader.svelte";
  import TransactionTable from "./components/TransactionTable.svelte";
  import PaymentSidebar from "./components/PaymentSidebar.svelte";
  import FooterBar from "./components/FooterBar.svelte";
  import CatalogModal from "./components/CatalogModal.svelte";
  import PendingModal from "./components/PendingModal.svelte";
  import CloseKasirModal from "./components/CloseKasirModal.svelte";
  import TransactionHistoryView from "./components/TransactionHistoryView.svelte";
  import ProductMenuView from "./components/ProductMenuView.svelte";
  import { api, type CartSummaryDTO, type StatusInfoDTO } from "./lib/api";

  // Navigation state: 'transaksi' | 'produk' | 'kasir' (starts at 'transaksi')
  let currentView = $state<"transaksi" | "produk" | "kasir">("transaksi");
  let isCloseKasirModalOpen = $state(false);

  let cart: CartSummaryDTO = $state({
    items: [],
    subtotal: 0,
    nilai_tukar_poin: 0,
    total_akhir: 0,
    member_nama: "Budi Santoso",
    member_kode: "MBR-0421",
    member_poin_saldo: 128,
    is_member_attached: true,
  });

  let status: StatusInfoDTO | null = $state(null);
  let isCatalogOpen = $state(false);
  let isPendingOpen = $state(false);
  let pendingCount = $state(3);

  async function refreshCart() {
    try {
      const res = await api.getCart();
      if (res.items.length === 0) {
        await seedDefaultMockupCart();
      } else {
        cart = res;
      }
    } catch (e) {
      console.error("refreshCart:", e);
      seedMockupCartLocally();
    }
  }

  async function seedDefaultMockupCart() {
    try {
      await api.scanBarcode("SKU-1001");
      await api.updateCartQty(0, 2);
      await api.scanBarcode("SKU-1002");
      await api.updateCartQty(1, 2);
      await api.scanBarcode("SKU-1005");
      await api.updateCartQty(2, 1);
      await api.scanBarcode("SKU-1007");
      await api.updateCartQty(3, 1);
      const updated = await api.getCart();
      cart = {
        ...updated,
        member_nama: "Budi Santoso",
        member_kode: "MBR-0421",
        member_poin_saldo: 128,
        is_member_attached: true,
      };
    } catch {
      seedMockupCartLocally();
    }
  }

  function seedMockupCartLocally() {
    cart = {
      items: [
        {
          id: "item-1",
          kode: "SKU-1001",
          nama: "Iced Caramel Macchiato",
          satuan: "Cup",
          harga: 32000,
          jumlah: 2,
          subtotal: 57600,
          suplier: "PT Kopi Mandiri",
          rak: "Chiller A",
          hargajual1: 32000,
          hargajual2: 30000,
          hargajual3: 28000,
          expired: "20/12/2026",
        },
        {
          id: "item-2",
          kode: "SKU-1002",
          nama: "Artisan Butter Croissant",
          satuan: "Pcs",
          harga: 22000,
          jumlah: 2,
          subtotal: 44000,
          suplier: "Prima Bakery",
          rak: "Display 01",
          hargajual1: 22000,
          hargajual2: 20000,
          hargajual3: 19000,
          expired: "15/09/2026",
        },
        {
          id: "item-3",
          kode: "SKU-1005",
          nama: "Club Sandwich Triple Decker",
          satuan: "Porsi",
          harga: 38000,
          jumlah: 1,
          subtotal: 38000,
          suplier: "Dapur Utama",
          rak: "Warm Box",
          hargajual1: 38000,
          hargajual2: 36000,
          hargajual3: 35000,
          expired: "11/09/2026",
        },
        {
          id: "item-4",
          kode: "SKU-1007",
          nama: "Air Mineral Artesian 600ml",
          satuan: "Btl",
          harga: 8000,
          jumlah: 1,
          subtotal: 8000,
          suplier: "Danone Tirta",
          rak: "Rak B-02",
          hargajual1: 8000,
          hargajual2: 7500,
          hargajual3: 7000,
          expired: "10/05/2027",
        },
      ],
      subtotal: 148000,
      nilai_tukar_poin: 14800,
      total_akhir: 147520,
      member_nama: "Budi Santoso",
      member_kode: "MBR-0421",
      member_poin_saldo: 128,
      is_member_attached: true,
    };
  }

  async function refreshStatus() {
    try {
      status = await api.getStatusInfo();
    } catch (e) {
      console.error("refreshStatus:", e);
    }
  }

  async function refreshPendingCount() {
    try {
      const p = await api.getPendingOrders();
      pendingCount = p.length || 3;
    } catch {
      pendingCount = 3;
    }
  }

  function handleCartUpdate(e: CustomEvent<CartSummaryDTO>) {
    cart = e.detail;
  }

  async function handleVoidCart() {
    try {
      const res = await api.clearCart();
      cart = res;
    } catch {
      cart = {
        items: [],
        subtotal: 0,
        nilai_tukar_poin: 0,
        total_akhir: 0,
        member_nama: "UMUM (Non-Member)",
        member_kode: "UMUM",
        member_poin_saldo: 0,
        is_member_attached: false,
      };
    }
  }

  function handleNavigation(target: "transaksi" | "produk" | "kasir") {
    if (currentView === "kasir" && target !== "kasir") {
      isCloseKasirModalOpen = true;
    } else {
      currentView = target;
    }
  }

  function handleKeydownGlobal(e: KeyboardEvent) {
    if (e.key === "F1") {
      e.preventDefault();
      currentView = "kasir";
    } else if (e.key === "F2" && currentView !== "kasir") {
      e.preventDefault();
      currentView = "produk";
    } else if (e.key === "F3" && currentView !== "kasir") {
      e.preventDefault();
      currentView = "transaksi";
    } else if (currentView === "kasir" && e.key === "Escape") {
      if (!isCatalogOpen && !isPendingOpen && !isCloseKasirModalOpen) {
        e.preventDefault();
        isCloseKasirModalOpen = true;
      }
    }
  }

  $effect(() => {
    refreshCart();
    refreshStatus();
    refreshPendingCount();
    window.addEventListener("keydown", handleKeydownGlobal);
    const clockInterval = setInterval(refreshStatus, 1000);
    return () => {
      window.removeEventListener("keydown", handleKeydownGlobal);
      clearInterval(clockInterval);
    };
  });
</script>

<div class="h-screen w-screen flex flex-col bg-slate-200 overflow-hidden font-sans">
  <!-- Top Bar (Header Konsisten) -->
  <TopBar
    {status}
    {currentView}
    onNavigate={handleNavigation}
  />

  {#if currentView === "transaksi"}
    <!-- Halaman Riwayat Transaksi (Default Startup Screen) -->
    <TransactionHistoryView
      onOpenKasir={() => (currentView = "kasir")}
    />
  {:else if currentView === "produk"}
    <!-- Halaman Menu Produk & Stok (Master Data) -->
    <ProductMenuView />
  {:else}
    <!-- Halaman Kasir (POS) -->
    <SubHeader
      tokoNama="MUEEZA STORE"
      pelanggan={cart.is_member_attached ? `${cart.member_nama} (VIP - #${cart.member_kode})` : ""}
    />

    <main class="flex-1 flex overflow-hidden p-2 gap-2 bg-slate-200">
      <TransactionTable
        {cart}
        {pendingCount}
        onCartUpdate={handleCartUpdate}
        onOpenCatalog={() => (isCatalogOpen = true)}
        onOpenPending={() => (isPendingOpen = true)}
      />

      <PaymentSidebar
        {cart}
        {pendingCount}
        onCartUpdate={handleCartUpdate}
        onOpenPending={() => (isPendingOpen = true)}
      />
    </main>

    <FooterBar
      {pendingCount}
      onOpenPending={() => (isPendingOpen = true)}
      onVoidCart={() => (isCloseKasirModalOpen = true)}
    />
  {/if}

  <!-- Modals -->
  <CatalogModal
    isOpen={isCatalogOpen}
    onClose={() => (isCatalogOpen = false)}
    onCartUpdate={handleCartUpdate}
  />

  <PendingModal
    isOpen={isPendingOpen}
    onClose={() => (isPendingOpen = false)}
    onCartUpdate={handleCartUpdate}
  />

  <!-- Modal Konfirmasi Tutup Kasir (ESC) -->
  <CloseKasirModal
    isOpen={isCloseKasirModalOpen}
    onConfirm={() => {
      isCloseKasirModalOpen = false;
      currentView = "transaksi";
    }}
    onClose={() => (isCloseKasirModalOpen = false)}
  />
</div>
