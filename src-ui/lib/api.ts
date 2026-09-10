import { invoke } from "@tauri-apps/api/core";

/* ── DTO Types matching Rust structs ── */

export interface ProductDTO {
  id: string;
  kode: string;
  barcode: string;
  nama: string;
  kategori: string;
  satuan: string;
  rak: string;
  supplier: string;
  hargapokok: number;
  hargajual1: number;
  hargajual2: number;
  hargajual3: number;
  margin_persen: number;
  stok: number;
  stokminimum: number;
  is_kritis: boolean;
  expired?: string;
  tag?: string;
}

export interface ProductStatsDTO {
  total_produk: number;
  stok_optimal: number;
  stok_menipis: number;
  stok_kosong: number;
  valuasi_aset: number;
  avg_margin: number;
}

export interface CartItemDTO {
  id: string;
  kode: string;
  nama: string;
  satuan: string;
  harga: number;
  jumlah: number;
  subtotal: number;
  suplier?: string;
  rak?: string;
  hargajual1?: number;
  hargajual2?: number;
  hargajual3?: number;
  expired?: string;
}

export interface CartSummaryDTO {
  items: CartItemDTO[];
  subtotal: number;
  nilai_tukar_poin: number;
  total_akhir: number;
  member_nama: string;
  member_kode: string;
  member_poin_saldo: number;
  is_member_attached: boolean;
}

export interface StatusInfoDTO {
  toko_nama: string;
  operator_nama: string;
  shift_status: string;
  terminal_id: string;
  clock: string;
  escpos_ready: boolean;
  sqlite_status: string;
  machine_id: string;
  license_status: string;
}

export interface CheckoutResultDTO {
  sukses: boolean;
  faktur: string;
  total: number;
  bayar: number;
  kembalian: number;
  pesan: string;
}

export interface PendingItemDTO {
  id: string;
  faktur: string;
  tanggal: string;
  pelanggan: string;
  total: number;
  keterangan: string;
}

export interface TransactionHistoryItemDTO {
  id: string;
  no: number;
  faktur: string;
  waktu: string;
  kasir: string;
  shift: string;
  pelanggan: string;
  pelanggan_badge: string;
  pelanggan_info: string;
  total_qty: number;
  total_sku: number;
  metode: string;
  total_penjualan: number;
}

export interface TransactionSummaryStatsDTO {
  total_omset: number;
  target_omset: number;
  persentase_omset: number;
  total_qty: number;
  avg_item_per_trx: number;
  total_transaksi: number;
  void_count: number;
  retur_count: number;
  avg_basket_size: number;
  max_basket: number;
  min_basket: number;
  tunai_count: number;
  qris_count: number;
  edc_count: number;
  tunai_pct: number;
  qris_pct: number;
  edc_pct: number;
}

/* ── API Calls ── */

export const api = {
  getStatusInfo: () => invoke<StatusInfoDTO>("get_status_info"),
  getCatalogProducts: (keyword?: string) =>
    invoke<ProductDTO[]>("get_catalog_products", { keyword }),
  scanBarcode: (code: string) =>
    invoke<CartSummaryDTO>("scan_barcode", { code }),
  updateCartQty: (index: number, qty: number) =>
    invoke<CartSummaryDTO>("update_cart_qty", { index, qty }),
  removeCartItem: (index: number) =>
    invoke<CartSummaryDTO>("remove_cart_item", { index }),
  clearCart: () => invoke<CartSummaryDTO>("clear_cart"),
  getCart: () => invoke<CartSummaryDTO>("get_cart"),
  checkout: (tunai: number, metode: string) =>
    invoke<CheckoutResultDTO>("checkout", { tunai, metode }),
  attachMember: (query: string) =>
    invoke<CartSummaryDTO>("attach_member", { query }),
  detachMember: () => invoke<CartSummaryDTO>("detach_member"),
  tukarPoin: (poin: number) =>
    invoke<CartSummaryDTO>("tukar_poin", { poin }),
  holdOrder: (keterangan?: string) =>
    invoke<string>("hold_order", { keterangan }),
  recallOrder: (id: string) =>
    invoke<CartSummaryDTO>("recall_order", { id }),
  getPendingOrders: () => invoke<PendingItemDTO[]>("get_pending_orders"),
  deletePendingOrder: (id: string) =>
    invoke<boolean>("delete_pending_order", { id }),
  printLastReceipt: (faktur: string) =>
    invoke<boolean>("print_last_receipt", { faktur }),
  getTransactions: (metode?: string, keyword?: string) =>
    invoke<TransactionHistoryItemDTO[]>("get_transactions", { metode, keyword }),
  getTransactionStats: () =>
    invoke<TransactionSummaryStatsDTO>("get_transaction_stats"),
  getProductStats: () =>
    invoke<ProductStatsDTO>("get_product_stats"),
  minimizeWindow: () => invoke("minimize_window"),
  toggleMaximizeWindow: () => invoke("toggle_maximize_window"),
  closeWindow: () => invoke("close_window"),
};

/* ── Helpers ── */

export function formatRupiah(n: number): string {
  return "Rp " + n.toLocaleString("id-ID", { minimumFractionDigits: 0 });
}
