<script lang="ts">
  import { api, formatRupiah, type CartSummaryDTO } from "../lib/api";
  import Numpad from "./Numpad.svelte";

  let { cart, onCartUpdate }: {
    cart: CartSummaryDTO;
    onCartUpdate: (e: CustomEvent<CartSummaryDTO>) => void;
  } = $props();

  let paymentMethod = $state("TUNAI");
  let cashTender = $state<number | null>(null);
  let orderNumber = $state("#ORD-8829");
  let lastMessage = $state("");
  let selectedCartIndex = $state<number | null>(null);
  let isEditingMember = $state(false);
  let memberInputText = $state("");

  function emit(result: CartSummaryDTO) {
    onCartUpdate(new CustomEvent("cartUpdate", { detail: result }));
  }

  async function updateQty(index: number, qty: number) {
    try {
      if (qty <= 0) {
        emit(await api.removeCartItem(index));
      } else {
        emit(await api.updateCartQty(index, qty));
      }
    } catch (e) {
      console.error("updateQty:", e);
    }
  }

  async function removeItem(index: number) {
    try {
      emit(await api.removeCartItem(index));
    } catch (e) {
      console.error("removeItem:", e);
    }
  }

  async function clearCart() {
    try {
      emit(await api.clearCart());
      cashTender = null;
      lastMessage = "Nota dibatalkan / void.";
      setTimeout(() => (lastMessage = ""), 3000);
    } catch (e) {
      console.error("clearCart:", e);
    }
  }

  async function doCheckout() {
    if (cart.items.length === 0) return;
    try {
      const bayar = cashTender ?? cart.total_akhir;
      const result = await api.checkout(bayar, paymentMethod);
      lastMessage = result.pesan + (result.kembalian > 0 ? ` Kembali: ${formatRupiah(result.kembalian)}` : "");
      emit(await api.getCart());
      cashTender = null;
      setTimeout(() => (lastMessage = ""), 6000);
    } catch (e: any) {
      lastMessage = String(e);
    }
  }

  async function holdOrder() {
    if (cart.items.length === 0) return;
    try {
      const faktur = await api.holdOrder("Antrean ditahan");
      emit(await api.getCart());
      lastMessage = `Nota ${faktur} berhasil ditahan [F10].`;
      setTimeout(() => (lastMessage = ""), 4000);
    } catch (e) {
      console.error("holdOrder:", e);
    }
  }

  function handleNumpad(action: string) {
    if (action === "CLR") {
      cashTender = null;
    } else if (action === "DEL") {
      if (cashTender !== null) {
        const s = String(cashTender);
        cashTender = s.length > 1 ? parseInt(s.slice(0, -1), 10) : null;
      }
    } else if (action === "OK") {
      doCheckout();
    } else if (action === "QTY") {
      if (selectedCartIndex !== null && cashTender !== null && cashTender > 0) {
        updateQty(selectedCartIndex, cashTender);
        cashTender = null;
      }
    } else if (action === "DISC") {
      // Toggle / discount simulation
      lastMessage = "Diskon diterapkan.";
      setTimeout(() => (lastMessage = ""), 2000);
    } else {
      // Numbers "0", "1" ... "9", "00"
      const cur = cashTender !== null ? String(cashTender) : "";
      cashTender = parseInt(cur + action, 10);
    }
  }

  async function saveMember() {
    if (!memberInputText.trim()) return;
    try {
      emit(await api.attachMember(memberInputText.trim()));
      isEditingMember = false;
      memberInputText = "";
    } catch (e: any) {
      lastMessage = String(e);
      setTimeout(() => (lastMessage = ""), 3000);
    }
  }

  // Keyboard shortcuts (F5, F7, F8, F9, F10, F11, ESC, Space)
  function handleWindowKeydown(e: KeyboardEvent) {
    if (e.key === "F7") {
      e.preventDefault();
      paymentMethod = "TUNAI";
    } else if (e.key === "F8") {
      e.preventDefault();
      paymentMethod = "QRIS";
    } else if (e.key === "F9") {
      e.preventDefault();
      paymentMethod = "EDC";
    } else if (e.key === "F10") {
      e.preventDefault();
      holdOrder();
    } else if (e.key === "F5") {
      e.preventDefault();
      isEditingMember = !isEditingMember;
    } else if (e.key === "Escape") {
      e.preventDefault();
      clearCart();
    } else if (e.key === "Enter" && (e.ctrlKey || e.target === document.body)) {
      doCheckout();
    }
  }

  let totalItemsCount = $derived(
    cart.items.reduce((acc, it) => acc + it.jumlah, 0)
  );

  let effectivePay = $derived(cashTender ?? cart.total_akhir);
  let kembalian = $derived(Math.max(0, effectivePay - cart.total_akhir));

  $effect(() => {
    window.addEventListener("keydown", handleWindowKeydown);
    return () => window.removeEventListener("keydown", handleWindowKeydown);
  });
</script>

<div class="cart-panel-card">
  <!-- Order Header (68px) -->
  <div class="order-header">
    <div class="order-row-top">
      <span class="order-number">{orderNumber}</span>
      <span class="order-type">Dine-In • Meja T04</span>
      <span class="order-clock">14:32 WIB</span>
    </div>

    {#if isEditingMember}
      <div class="member-edit-row">
        <input
          type="text"
          class="member-input"
          placeholder="Cari kode/nama member..."
          bind:value={memberInputText}
          onkeydown={(e) => e.key === "Enter" && saveMember()}
        />
        <button class="btn-save-member" onclick={saveMember}>Simpan</button>
        <button class="btn-cancel-member" onclick={() => (isEditingMember = false)}>Batal</button>
      </div>
    {:else}
      <div class="order-row-bottom">
        <div class="customer-info">
          <span class="customer-name">{cart.member_nama}</span>
          <span class="customer-tier">{cart.is_member_attached ? "VIP (-10%)" : "UMUM"}</span>
        </div>
        <button class="btn-ganti" onclick={() => (isEditingMember = true)}>Ganti [F5]</button>
      </div>
    {/if}
  </div>

  <!-- Cart Items Grid -->
  <div class="cart-items-container">
    <div class="cart-table-header">
      <span class="col-item">ITEM / DESKRIPSI</span>
      <span class="col-qty">QTY</span>
      <span class="col-price">HARGA</span>
      <span class="col-total">TOTAL</span>
      <span class="col-del"></span>
    </div>

    <div class="cart-rows-wrap">
      {#each cart.items as item, i}
        <div
          class="cart-row"
          class:selected={selectedCartIndex === i}
          onclick={() => (selectedCartIndex = i)}
          role="button"
          tabindex="0"
          onkeydown={(e) => e.key === "Enter" && (selectedCartIndex = i)}
        >
          <div class="col-item item-meta">
            <span class="item-title">{item.nama}</span>
            <span class="item-sub">{item.kode} • Sat: {item.satuan}</span>
          </div>

          <div class="col-qty qty-cell">
            <div class="qty-stepper">
              <button class="step-btn" onclick={(e) => { e.stopPropagation(); updateQty(i, item.jumlah - 1); }}>-</button>
              <span class="qty-num">{item.jumlah}</span>
              <button class="step-btn" onclick={(e) => { e.stopPropagation(); updateQty(i, item.jumlah + 1); }}>+</button>
            </div>
          </div>

          <div class="col-price price-cell">{item.harga.toLocaleString("id-ID")}</div>
          <div class="col-total total-cell">Rp {item.subtotal.toLocaleString("id-ID")}</div>

          <div class="col-del">
            <button class="btn-del" onclick={(e) => { e.stopPropagation(); removeItem(i); }}>×</button>
          </div>
        </div>
      {:else}
        <div class="cart-empty-message">Keranjang belanja kosong</div>
      {/each}
    </div>
  </div>

  <!-- Financial Summary (112px) -->
  <div class="financial-summary">
    <div class="summary-line">
      <span class="sum-label">Subtotal Item ({cart.items.length} Baris / {totalItemsCount} Item)</span>
      <span class="sum-val">{formatRupiah(cart.subtotal)}</span>
    </div>

    <div class="summary-line disc">
      <span class="sum-label">Diskon VIP Member (10%)</span>
      <span class="sum-val">{cart.nilai_tukar_poin > 0 ? `-${formatRupiah(cart.nilai_tukar_poin)}` : "- Rp 0"}</span>
    </div>

    <div class="summary-line">
      <span class="sum-label">Pajak Restoran PB1 / PPN 10%</span>
      <span class="sum-val">+ Rp 0</span>
    </div>

    <!-- Grand Total Box (#0f172a) -->
    <div class="grand-total-card">
      <div class="total-label-block">
        <span class="total-title">TOTAL AKHIR PEMBAYARAN</span>
        <span class="total-sub">PPN Terhitung Otomatis</span>
      </div>
      <span class="total-amount">{formatRupiah(cart.total_akhir)}</span>
    </div>
  </div>

  <!-- Quick Cash (36px) -->
  <div class="quick-cash-row">
    <button class="btn-cash" onclick={() => (cashTender = cart.total_akhir)}>Uang Pas</button>
    <button class="btn-cash" onclick={() => (cashTender = 150000)}>150.000</button>
    <button class="btn-cash" onclick={() => (cashTender = 200000)}>200.000</button>
    <button class="btn-cash" onclick={() => (cashTender = 300000)}>300.000</button>
  </div>

  <!-- Numpad Grid (145px) -->
  <Numpad onInput={handleNumpad} />

  <!-- Payment Methods (58px) -->
  <div class="payment-methods-row">
    <button
      class="btn-pay-method"
      class:active={paymentMethod === "TUNAI"}
      onclick={() => (paymentMethod = "TUNAI")}
    >
      <span class="pm-key">[F7]</span>
      <span class="pm-name">TUNAI</span>
    </button>
    <button
      class="btn-pay-method"
      class:active={paymentMethod === "QRIS"}
      onclick={() => (paymentMethod = "QRIS")}
    >
      <span class="pm-key">[F8]</span>
      <span class="pm-name">QRIS STATIC/DYN</span>
    </button>
    <button
      class="btn-pay-method"
      class:active={paymentMethod === "EDC"}
      onclick={() => (paymentMethod = "EDC")}
    >
      <span class="pm-key">[F9]</span>
      <span class="pm-name">KARTU EDC</span>
    </button>
  </div>

  <!-- Pay Button Area (96px) -->
  <div class="pay-button-section">
    <button
      class="btn-checkout-primary"
      onclick={doCheckout}
      disabled={cart.items.length === 0}
    >
      <span class="key-hint">SPACE / ENTER</span>
      <span class="action-text">PROSES BAYAR SEKARANG</span>
      <span class="amount-text">{formatRupiah(cart.total_akhir)} →</span>
    </button>

    <div class="sub-actions-row">
      <button class="btn-sub-act" onclick={holdOrder}>[F10] Tahan Nota</button>
      <button class="btn-sub-act" onclick={() => api.printLastReceipt(orderNumber)}>[F11] Cetak Bill</button>
      <button class="btn-sub-act btn-void" onclick={clearCart}>[ESC] Void Nota</button>
    </div>
  </div>

  {#if lastMessage}
    <div class="toast-message">{lastMessage}</div>
  {/if}
</div>

<style>
  .cart-panel-card {
    width: 35%;
    height: 100%;
    background: #ffffff;
    border: 1px solid #cbd5e1;
    border-radius: 4px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
  }

  /* ── Order Header (68px) ── */
  .order-header {
    height: 68px;
    background: #f8fafc;
    border-bottom: 1px solid #e2e8f0;
    padding: 8px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    flex-shrink: 0;
  }

  .order-row-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .order-number {
    font-size: 14px;
    font-weight: 700;
    color: #0f172a;
  }

  .order-type {
    color: #0369a1;
    font-size: 10px;
    font-weight: 600;
    background: #e0f2fe;
    padding: 2px 6px;
    border-radius: 3px;
  }

  .order-clock {
    font-size: 11px;
    color: #64748b;
  }

  .order-row-bottom {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .customer-info {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .customer-name {
    font-size: 11px;
    font-weight: 600;
    color: #0f172a;
  }

  .customer-tier {
    color: #92400e;
    background: #fef3c7;
    font-size: 9px;
    font-weight: 700;
    padding: 1px 5px;
    border-radius: 2px;
  }

  .btn-ganti {
    height: 24px;
    padding: 0 8px;
    font-size: 10px;
    font-weight: 500;
    background: #ffffff;
    border: 1px solid #cbd5e1;
    color: #334155;
    border-radius: 3px;
  }

  .member-edit-row {
    display: flex;
    gap: 4px;
    align-items: center;
  }

  .member-input {
    flex: 1;
    height: 24px;
    padding: 0 6px;
    font-size: 11px;
    border: 1px solid #0284c7;
    border-radius: 3px;
  }

  .btn-save-member {
    height: 24px;
    padding: 0 8px;
    font-size: 10px;
    background: #0284c7;
    color: #fff;
    border-color: #0284c7;
  }

  .btn-cancel-member {
    height: 24px;
    padding: 0 6px;
    font-size: 10px;
  }

  /* ── Cart Items Grid ── */
  .cart-items-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .cart-table-header {
    height: 30px;
    background: #e8edf3;
    display: flex;
    align-items: center;
    padding: 0 8px;
    font-size: 10px;
    font-weight: 700;
    color: #475569;
    border-bottom: 1px solid #cbd5e1;
    flex-shrink: 0;
  }

  .col-item { flex: 1; min-width: 100px; }
  .col-qty { width: 70px; text-align: center; }
  .col-price { width: 80px; text-align: right; }
  .col-total { width: 90px; text-align: right; }
  .col-del { width: 28px; text-align: center; }

  .cart-rows-wrap {
    flex: 1;
    overflow-y: auto;
  }

  .cart-row {
    height: 48px;
    display: flex;
    align-items: center;
    padding: 0 8px;
    border-bottom: 1px solid #f1f5f9;
    cursor: pointer;
    transition: background-color 0.1s;
  }

  .cart-row:hover {
    background: #f8fafc;
  }

  .cart-row.selected {
    background: #f0f9ff;
    border-left: 3px solid #0284c7;
  }

  .item-meta {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .item-title {
    font-size: 11px;
    font-weight: 600;
    color: #0f172a;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-sub {
    font-size: 9px;
    color: #64748b;
  }

  .qty-stepper {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 2px;
  }

  .step-btn {
    width: 20px;
    height: 22px;
    font-size: 12px;
    font-weight: 700;
    background: #f1f5f9;
    border: 1px solid #cbd5e1;
    color: #334155;
    border-radius: 3px;
    padding: 0;
  }

  .step-btn:hover {
    background: #e2e8f0;
  }

  .qty-num {
    width: 22px;
    text-align: center;
    font-size: 11px;
    font-weight: 700;
    color: #0f172a;
  }

  .price-cell {
    font-size: 11px;
    color: #475569;
  }

  .total-cell {
    font-size: 11px;
    font-weight: 700;
    color: #0f172a;
  }

  .btn-del {
    width: 20px;
    height: 20px;
    border-radius: 3px;
    background: transparent;
    border: none;
    color: #94a3b8;
    font-size: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .btn-del:hover {
    background: #fee2e2;
    color: #dc2626;
  }

  .cart-empty-message {
    padding: 30px;
    text-align: center;
    color: #94a3b8;
    font-size: 11px;
  }

  /* ── Financial Summary (112px) ── */
  .financial-summary {
    height: 112px;
    background: #f8fafc;
    border-top: 1px solid #e2e8f0;
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    flex-shrink: 0;
  }

  .summary-line {
    display: flex;
    justify-content: space-between;
    font-size: 10px;
    color: #475569;
  }

  .summary-line.disc {
    color: #047857;
  }

  .sum-val {
    font-weight: 600;
  }

  .grand-total-card {
    height: 44px;
    background: #0f172a;
    border-radius: 6px;
    padding: 6px 10px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .total-label-block {
    display: flex;
    flex-direction: column;
  }

  .total-title {
    color: #94a3b8;
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.3px;
  }

  .total-sub {
    color: #38bdf8;
    font-size: 10px;
  }

  .total-amount {
    color: #ffffff;
    font-size: 18px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  /* ── Quick Cash (36px) ── */
  .quick-cash-row {
    height: 36px;
    display: flex;
    gap: 4px;
    padding: 4px 8px;
    flex-shrink: 0;
  }

  .btn-cash {
    flex: 1;
    height: 28px;
    font-size: 10px;
    font-weight: 600;
    background: #ffffff;
    border: 1px solid #cbd5e1;
    color: #334155;
    border-radius: 3px;
  }

  .btn-cash:hover {
    background: #f8fafc;
  }

  /* ── Payment Methods (58px) ── */
  .payment-methods-row {
    height: 54px;
    display: flex;
    gap: 4px;
    padding: 4px 8px;
    flex-shrink: 0;
  }

  .btn-pay-method {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    background: #ffffff;
    border: 1px solid #cbd5e1;
    color: #334155;
    border-radius: 4px;
    padding: 2px;
  }

  .btn-pay-method:hover {
    background: #f8fafc;
  }

  .btn-pay-method.active {
    background: #047857;
    border-color: #047857;
    color: #ffffff;
  }

  .pm-key {
    font-size: 9px;
    opacity: 0.8;
  }

  .pm-name {
    font-size: 10px;
    font-weight: 700;
  }

  /* ── Pay Button Area (96px) ── */
  .pay-button-section {
    height: 96px;
    background: #f1f5f9;
    border-top: 1px solid #e2e8f0;
    padding: 8px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    flex-shrink: 0;
  }

  .btn-checkout-primary {
    height: 48px;
    background: #059669;
    border: 1px solid #059669;
    color: #ffffff;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    font-weight: 700;
    transition: background-color 0.15s;
  }

  .btn-checkout-primary:hover:not(:disabled) {
    background: #047857;
  }

  .btn-checkout-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .key-hint {
    font-size: 9px;
    opacity: 0.85;
    background: rgba(0, 0, 0, 0.2);
    padding: 2px 6px;
    border-radius: 3px;
  }

  .action-text {
    font-size: 11px;
    letter-spacing: 0.3px;
  }

  .amount-text {
    font-size: 14px;
  }

  .sub-actions-row {
    display: flex;
    gap: 4px;
  }

  .btn-sub-act {
    flex: 1;
    height: 26px;
    font-size: 10px;
    font-weight: 500;
    background: #ffffff;
    border: 1px solid #cbd5e1;
    color: #334155;
    border-radius: 3px;
  }

  .btn-sub-act:hover {
    background: #f8fafc;
  }

  .btn-void {
    color: #dc2626;
  }

  .toast-message {
    position: absolute;
    bottom: 100px;
    left: 12px;
    right: 12px;
    background: #0f172a;
    color: #6ee7b7;
    padding: 8px 12px;
    border-radius: 4px;
    font-size: 11px;
    text-align: center;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
    animation: fadeIn 0.2s;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
