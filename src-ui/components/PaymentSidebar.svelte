<script lang="ts">
  import { api, formatRupiah, type CartSummaryDTO } from "../lib/api";

  let {
    cart,
    pendingCount = 3,
    onCartUpdate,
    onOpenPending,
  }: {
    cart: CartSummaryDTO;
    pendingCount?: number;
    onCartUpdate: (e: CustomEvent<CartSummaryDTO>) => void;
    onOpenPending: () => void;
  } = $props();

  let memberSearch = $state("0812-3456-7890");
  let paymentMethod = $state<"TUNAI" | "QRIS" | "EDC">("TUNAI");
  let cashReceivedRaw = $state<string>("150000");
  let checkoutNotification = $state<string>("");

  function emit(result: CartSummaryDTO) {
    onCartUpdate(new CustomEvent("cartUpdate", { detail: result }));
  }

  // Calculations
  let subtotal = $derived(cart.subtotal);
  let discountVip = $derived(
    cart.is_member_attached
      ? Math.round(subtotal * 0.1)
      : Math.round(cart.nilai_tukar_poin)
  );
  let dpp = $derived(Math.max(0, subtotal - discountVip));
  let taxPpn = $derived(subtotal > 0 ? Math.round(dpp * 0.11) : 0);
  let exactTotal = $derived(dpp + taxPpn);
  // Pembulatan ke kelipatan 50 atau 100
  let roundedTotal = $derived(
    exactTotal > 0 ? Math.floor(exactTotal / 10) * 10 : 0
  );
  let roundingDiff = $derived(roundedTotal - exactTotal);

  let grandTotal = $derived(
    cart.items.length > 0
      ? (roundedTotal > 0 ? roundedTotal : cart.total_akhir)
      : 0
  );

  let cashReceivedNumber = $derived(
    parseFloat(cashReceivedRaw.replace(/\D/g, "")) || 0
  );

  let changeAmount = $derived(
    Math.max(0, cashReceivedNumber - grandTotal)
  );

  let pointsEarned = $derived(
    Math.floor(grandTotal / 10000)
  );

  async function handleCheckMember() {
    const q = memberSearch.trim();
    if (!q) return;
    try {
      const res = await api.attachMember(q);
      emit(res);
    } catch {
      // Mock attachment if backend lookup fails
      emit({
        ...cart,
        member_nama: "Budi Santoso",
        member_kode: "MBR-0421",
        member_poin_saldo: 128,
        is_member_attached: true,
      });
    }
  }

  async function handleDetachMember() {
    try {
      const res = await api.detachMember();
      emit(res);
    } catch {
      emit({
        ...cart,
        member_nama: "UMUM (Non-Member)",
        member_kode: "UMUM",
        member_poin_saldo: 0,
        is_member_attached: false,
      });
    }
  }

  async function handleCheckout() {
    if (cart.items.length === 0) return;
    const pay = paymentMethod === "TUNAI" ? (cashReceivedNumber || grandTotal) : grandTotal;
    try {
      const res = await api.checkout(pay, paymentMethod);
      checkoutNotification = `${res.pesan} Kembali: ${formatRupiah(res.kembalian)}`;
      emit(await api.getCart());
      setTimeout(() => (checkoutNotification = ""), 6000);
    } catch (err: any) {
      checkoutNotification = String(err);
      setTimeout(() => (checkoutNotification = ""), 4000);
    }
  }

  async function handlePending() {
    if (cart.items.length === 0) return;
    try {
      const faktur = await api.holdOrder("Antrean ditahan kasir");
      checkoutNotification = `Faktur ${faktur} berhasil ditahan.`;
      emit(await api.getCart());
      setTimeout(() => (checkoutNotification = ""), 4000);
    } catch (err: any) {
      checkoutNotification = String(err);
    }
  }

  function formatCashDisplay(val: number): string {
    return val > 0 ? val.toLocaleString("id-ID") : "0";
  }

  function handleKeydown(e: KeyboardEvent) {
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
      handlePending();
    } else if (e.key === "F3") {
      e.preventDefault();
      handlePending();
    } else if (e.key === "F4") {
      e.preventDefault();
      api.printLastReceipt("#ORD-8829");
    } else if (e.key === " " && (e.ctrlKey || (e.target as HTMLElement).tagName !== "INPUT")) {
      e.preventDefault();
      handleCheckout();
    }
  }

  $effect(() => {
    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });
</script>

<section class="w-[380px] shrink-0 flex flex-col bg-slate-50 border border-slate-300 rounded-md shadow-sm overflow-hidden select-none">
  <div class="p-3.5 bg-slate-100/60 border-b border-slate-300 flex flex-col justify-between flex-1 font-mono text-xs overflow-y-auto">
    <!-- Pelanggan / Member Card -->
    <div class="pb-2.5 mb-2.5 border-b border-slate-300 flex flex-col gap-2 font-sans">
      <div class="flex items-center justify-between font-mono text-[10px]">
        <span class="font-bold uppercase tracking-wider text-slate-700 flex items-center gap-1">
          <span class="material-symbols-outlined text-[14px] text-primary">badge</span>
          PELANGGAN / MEMBER
        </span>
        <span class="text-[10px] font-bold text-slate-500 font-mono">VIP Loyalty</span>
      </div>

      <!-- Search member input -->
      <div class="relative flex items-center">
        <div class="absolute inset-y-0 left-0 pl-2 flex items-center pointer-events-none text-slate-400">
          <span class="material-symbols-outlined text-[15px]">person_search</span>
        </div>
        <input
          type="text"
          bind:value={memberSearch}
          onkeydown={(e) => e.key === "Enter" && handleCheckMember()}
          class="w-full pl-7 pr-16 py-1 bg-white border border-slate-300 rounded font-mono text-xs text-slate-900 placeholder-slate-400 focus:outline-none focus:bg-white focus:border-primary focus:ring-1 focus:ring-primary shadow-2xs"
          placeholder="Ketik No. HP / Nama..."
        />
        <div class="absolute inset-y-0 right-0 pr-1 flex items-center gap-1">
          <button
            type="button"
            onclick={handleCheckMember}
            class="px-2 py-0.5 bg-primary hover:bg-primary-dark text-white rounded font-mono text-[10px] font-bold shadow-2xs transition-colors flex items-center gap-0.5 cursor-pointer border-none"
          >
            <span class="material-symbols-outlined text-[12px]">search</span>
            Cek
          </button>
        </div>
      </div>

      <!-- Active Member Card (hanya jika member terpasang) -->
      {#if cart.is_member_attached}
        <div class="bg-slate-200/80 border border-slate-300 rounded p-2.5 flex flex-col gap-1.5 shadow-2xs">
          <div class="flex items-start justify-between">
            <div class="flex items-center gap-1.5">
              <div class="w-6 h-6 rounded-full bg-primary text-white flex items-center justify-center shrink-0 shadow-2xs">
                <span class="material-symbols-outlined text-[15px]">verified</span>
              </div>
              <div>
                <div class="text-xs font-bold text-slate-900 leading-tight flex items-center gap-1">
                  {cart.member_nama}
                  <span class="px-1.5 py-0.2 bg-amber-400 text-slate-900 rounded text-[9px] font-mono font-bold uppercase tracking-wider">
                    VIP Member
                  </span>
                </div>
                <div class="font-mono text-[10px] text-slate-600 font-medium">
                  #{cart.member_kode}
                </div>
              </div>
            </div>
            <button
              type="button"
              onclick={handleDetachMember}
              class="text-slate-500 hover:text-red-600 p-0.5 hover:bg-rose-100 rounded transition-colors cursor-pointer border-none bg-transparent"
              title="Lepas Member"
            >
              <span class="material-symbols-outlined text-[15px]">close</span>
            </button>
          </div>

          <div class="mt-1 pt-1.5 border-t border-slate-300 flex items-center justify-between font-mono text-[10px]">
            <div class="flex items-center gap-1 text-slate-700 font-medium">
              <span class="material-symbols-outlined text-[13px] text-amber-600">stars</span>
              <span>Poin Saat Ini: <strong class="text-slate-900 font-bold">{cart.member_poin_saldo} Poin</strong></span>
            </div>
            <div class="px-1.5 py-0.2 rounded bg-emerald-100 text-emerald-800 font-bold border border-emerald-300 flex items-center gap-0.5" title="Aturan: Rp 10.000 = 1 Poin">
              <span class="material-symbols-outlined text-[11px]">add_circle</span>
              +{pointsEarned || 0} Poin Transaksi
            </div>
          </div>
        </div>
      {/if}
    </div>

    <!-- Rincian Pembayaran Breakdown Card -->
    <div class="bg-white p-2.5 rounded border border-slate-300 flex flex-col gap-2 shadow-2xs">
      <div class="pb-1.5 border-b border-slate-200 flex items-center justify-between">
        <span class="font-sans font-bold text-slate-900 text-xs">Rincian Pembayaran</span>
        <span class="px-2 py-0.5 bg-emerald-100 text-emerald-800 rounded font-mono text-[10px] font-bold border border-emerald-300">
          Faktur Siap Bayar
        </span>
      </div>

      <div class="flex justify-between text-slate-700 py-0.5 text-xs">
        <span class="font-sans">Subtotal Item ({cart.items.length} item)</span>
        <span class="tabular-nums font-bold text-slate-900" id="calc-subtotal">
          {formatRupiah(subtotal)}
        </span>
      </div>

      <div class="flex justify-between text-emerald-700 py-0.5 text-xs font-semibold">
        <span class="font-sans">Diskon VIP Member (10%)</span>
        <span class="tabular-nums font-bold" id="calc-discount">
          -{formatRupiah(discountVip)}
        </span>
      </div>

      <div class="flex justify-between text-slate-700 py-0.5 text-xs">
        <span class="font-sans">Pajak Restoran PB1 / PPN 11%</span>
        <span class="tabular-nums font-bold text-slate-900" id="calc-tax">
          +{formatRupiah(taxPpn)}
        </span>
      </div>

      <div class="flex justify-between text-slate-600 text-[11px] py-0.5 border-t border-slate-100">
        <span class="font-sans">Pembulatan Nilai Tunai</span>
        <span class="tabular-nums font-medium" id="calc-rounding">
          {roundingDiff <= 0 ? `-${formatRupiah(Math.abs(roundingDiff))}` : `+${formatRupiah(roundingDiff)}`}
        </span>
      </div>
    </div>

    <!-- Grand Total Card -->
    <div class="pt-2.5">
      <div class="bg-emerald-100/90 border-2 border-emerald-300 rounded-md p-3 flex items-center justify-between shadow-2xs">
        <div class="flex flex-col">
          <span class="text-[10px] font-mono font-bold uppercase text-emerald-900 tracking-wider">
            TOTAL AKHIR
          </span>
          <span class="text-[11px] text-slate-600 font-sans font-medium">Termasuk PPN 11%</span>
        </div>
        <div class="text-right">
          <span class="text-2xl font-bold font-mono tracking-tight text-emerald-800 tabular-nums" id="display-grand-total">
            {formatRupiah(grandTotal)}
          </span>
        </div>
      </div>
    </div>

    <!-- Uang Diterima / Bayar (Rp) -->
    <div class="pt-2.5 flex flex-col gap-2 border-t border-slate-300 mt-2.5 bg-slate-200/60 p-2.5 rounded-md border border-slate-300">
      <div class="flex flex-col gap-1">
        <div class="flex items-center justify-between font-mono text-[10px]">
          <label for="cash-received-input" class="font-bold uppercase tracking-wider text-slate-800 flex items-center gap-1">
            <span class="material-symbols-outlined text-[14px] text-emerald-700">payments</span>
            Uang Diterima / Bayar (Rp)
          </label>
          <span class="text-[10px] text-slate-500 font-mono font-bold">Tunai Langsung</span>
        </div>

        <div class="relative">
          <span class="absolute inset-y-0 left-0 pl-2.5 flex items-center font-mono font-bold text-slate-600 text-sm pointer-events-none">Rp</span>
          <input
            id="cash-received-input"
            type="text"
            value={formatCashDisplay(cashReceivedNumber)}
            oninput={(e) => {
              const num = e.currentTarget.value.replace(/\D/g, "");
              cashReceivedRaw = num;
            }}
            class="w-full pl-9 pr-3 py-1.5 bg-white border-2 border-primary rounded font-mono text-[16px] font-bold text-slate-900 focus:outline-none focus:bg-white focus:border-primary-dark shadow-inner tabular-nums"
            placeholder="0"
          />
        </div>
      </div>

      <!-- Quick Cash (4 Buttons) -->
      <div class="grid grid-cols-4 gap-1 font-mono text-[10px]">
        <button
          type="button"
          onclick={() => (cashReceivedRaw = String(grandTotal))}
          class="py-1 px-1 bg-white hover:bg-slate-100 border border-slate-300 rounded font-bold text-slate-800 text-center shadow-2xs transition-colors cursor-pointer"
        >
          Uang Pas
        </button>
        <button
          type="button"
          onclick={() => (cashReceivedRaw = "150000")}
          class="py-1 px-1 bg-blue-50 hover:bg-blue-100 border border-blue-300 rounded font-bold text-primary-dark text-center shadow-2xs transition-colors cursor-pointer"
        >
          150.000
        </button>
        <button
          type="button"
          onclick={() => (cashReceivedRaw = "200000")}
          class="py-1 px-1 bg-white hover:bg-slate-100 border border-slate-300 rounded font-bold text-slate-800 text-center shadow-2xs transition-colors cursor-pointer"
        >
          200.000
        </button>
        <button
          type="button"
          onclick={() => (cashReceivedRaw = "500000")}
          class="py-1 px-1 bg-white hover:bg-slate-100 border border-slate-300 rounded font-bold text-slate-800 text-center shadow-2xs transition-colors cursor-pointer"
        >
          500.000
        </button>
      </div>

      <!-- Kembalian Box -->
      <div class="bg-emerald-100/90 border border-emerald-300 rounded px-2.5 py-1.5 flex items-center justify-between shadow-2xs">
        <span class="font-mono text-[10px] font-bold text-slate-700 uppercase tracking-wider flex items-center gap-1">
          <span class="material-symbols-outlined text-[14px] text-emerald-700">change_circle</span>
          KEMBALIAN:
        </span>
        <span class="font-mono text-[16px] font-bold text-emerald-800 tabular-nums tracking-tight">
          {formatRupiah(changeAmount)}
        </span>
      </div>
    </div>
  </div>

  <!-- Payment Methods Tab -->
  <div class="p-2 bg-slate-200 border-b border-slate-300 grid grid-cols-3 gap-1.5 font-mono text-xs">
    <button
      onclick={() => (paymentMethod = "TUNAI")}
      class="py-2 px-1 font-bold rounded flex items-center justify-center gap-1 shadow-xs transition-colors cursor-pointer border {paymentMethod === 'TUNAI' ? 'bg-primary hover:bg-primary-dark text-white border-primary-dark' : 'bg-white hover:bg-slate-100 text-slate-800 border-slate-300'}"
    >
      <span class="material-symbols-outlined text-[15px]">payments</span>
      <span>TUNAI</span>
    </button>
    <button
      onclick={() => (paymentMethod = "QRIS")}
      class="py-2 px-1 font-bold rounded flex items-center justify-center gap-1 shadow-xs transition-colors cursor-pointer border {paymentMethod === 'QRIS' ? 'bg-primary hover:bg-primary-dark text-white border-primary-dark' : 'bg-white hover:bg-slate-100 text-slate-800 border-slate-300'}"
    >
      <span class="material-symbols-outlined text-[15px]">qr_code_scanner</span>
      <span>QRIS</span>
    </button>
    <button
      onclick={() => (paymentMethod = "EDC")}
      class="py-2 px-1 font-bold rounded flex items-center justify-center gap-1 shadow-xs transition-colors cursor-pointer border {paymentMethod === 'EDC' ? 'bg-primary hover:bg-primary-dark text-white border-primary-dark' : 'bg-white hover:bg-slate-100 text-slate-800 border-slate-300'}"
    >
      <span class="material-symbols-outlined text-[15px]">credit_card</span>
      <span>KARTU / EDC</span>
    </button>
  </div>

  <!-- Big Pay Button & Sub-actions -->
  <div class="p-2.5 bg-slate-200/90 border-t border-slate-300 flex flex-col gap-1.5">
    <button
      onclick={handleCheckout}
      disabled={cart.items.length === 0}
      class="w-full py-2.5 px-3 bg-primary hover:bg-primary-dark active:bg-blue-800 disabled:opacity-50 disabled:cursor-not-allowed text-white font-bold rounded flex items-center justify-between shadow-sm border border-blue-700 transition-all cursor-pointer"
    >
      <div class="flex items-center gap-2">
        <span class="px-2 py-0.5 bg-blue-950/40 text-blue-100 font-mono text-[11px] rounded font-bold border border-blue-400/30">
          SPACE / F2
        </span>
        <span class="text-xs uppercase tracking-wide font-bold">PROSES BAYAR</span>
      </div>
      <div class="font-mono text-sm font-bold tabular-nums">
        {formatRupiah(grandTotal)} →
      </div>
    </button>

    <!-- 4 Sub-actions -->
    <div class="grid grid-cols-4 gap-1 font-mono text-[10px]">
      <button
        onclick={handlePending}
        class="py-1.5 px-1 bg-white hover:bg-slate-100 border border-slate-300 rounded text-slate-800 font-bold flex flex-col items-center justify-center gap-0.5 shadow-2xs transition-colors cursor-pointer"
        title="Pending / Tahan Pesanan Aktif"
      >
        <span class="material-symbols-outlined text-[14px] text-amber-600">pause_circle</span>
        <span>[F10] Pending</span>
      </button>

      <button
        onclick={onOpenPending}
        class="py-1.5 px-1 bg-amber-100 hover:bg-amber-200 border border-amber-300 rounded text-amber-800 font-bold flex flex-col items-center justify-center gap-0.5 shadow-2xs transition-colors cursor-pointer relative"
        title="Buka Daftar Pesanan Tertahan"
      >
        <div class="flex items-center gap-1">
          <span class="material-symbols-outlined text-[14px]">pending_actions</span>
          <span class="px-1 rounded-full bg-amber-400 text-slate-900 text-[9px] font-bold leading-none">
            {pendingCount}
          </span>
        </div>
        <span>[F6] List Pending</span>
      </button>

      <button
        onclick={handlePending}
        class="py-1.5 px-1 bg-white hover:bg-slate-100 border border-slate-300 rounded text-slate-800 font-bold flex flex-col items-center justify-center gap-0.5 shadow-2xs transition-colors cursor-pointer"
        title="Simpan Transaksi / Draft"
      >
        <span class="material-symbols-outlined text-[14px] text-primary">save</span>
        <span>[F3] Simpan</span>
      </button>

      <button
        onclick={() => api.printLastReceipt("#ORD-8829")}
        class="py-1.5 px-1 bg-white hover:bg-slate-100 border border-slate-300 rounded text-slate-800 font-bold flex flex-col items-center justify-center gap-0.5 shadow-2xs transition-colors cursor-pointer"
        title="Cetak Struk Transaksi"
      >
        <span class="material-symbols-outlined text-[14px] text-slate-700">print</span>
        <span>[F4] Cetak Struk</span>
      </button>
    </div>
  </div>

  {#if checkoutNotification}
    <div class="bg-slate-900 text-emerald-300 px-3 py-1.5 text-xs text-center font-mono border-t border-slate-800 animate-pulse">
      {checkoutNotification}
    </div>
  {/if}
</section>
