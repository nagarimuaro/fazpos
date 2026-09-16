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

export interface PurchaseHistoryItemDTO {
  id: string;
  no: number;
  faktur: string;
  faktur_supplier: string;
  tanggal: string;
  suplier: string;
  total_item: number;
  total_qty: number;
  total_beli: number;
  metode: string;
  jatuh_tempo: string;
  status: string;
  penerima: string;
}

export interface PurchaseStatsDTO {
  total_belanja_bulan_ini: number;
  total_qty_masuk: number;
  total_faktur: number;
  total_hutang_tempo: number;
  supplier_teraktif: string;
}

export interface MemberDTO {
  id: string;
  kode: string;
  nama: string;
  telepon: string;
  alamat: string;
  tier: "VIP" | "GOLD" | "SILVER" | "REGULAR";
  poin: number;
  total_belanja: number;
  kunjungan_terakhir: string;
  is_aktif: boolean;
}

export interface MemberStatsDTO {
  total_member: number;
  member_aktif: number;
  total_poin: number;
  vip_gold_count: number;
  member_baru_minggu_ini: number;
}

export interface SupplierDTO {
  id: string;
  kode: string;
  nama: string;
  pic: string;
  telepon: string;
  alamat: string;
  rekening: string;
  termin_hari: number;
  total_transaksi: number;
  saldo_hutang: number;
  is_aktif: boolean;
}

export interface SupplierStatsDTO {
  total_supplier: number;
  total_belanja: number;
  total_hutang: number;
  jatuh_tempo_segera: number;
  avg_termin: number;
}

export interface DebtItemDTO {
  id: string;
  jenis: "HUTANG" | "PIUTANG";
  faktur_ref: string;
  tanggal: string;
  pihak: string;
  kontak: string;
  jatuh_tempo: string;
  tagihan_awal: number;
  telah_dibayar: number;
  sisa: number;
  status: "LUNAS" | "TEMPO" | "LEWAT_TEMPO";
}

export interface DebtStatsDTO {
  total_hutang: number;
  total_piutang: number;
  jatuh_tempo_minggu_ini: number;
  terbayar_bulan_ini: number;
  rasio_lancar: number;
}

export interface ReportSummaryDTO {
  omset_kotor: number;
  total_hpp: number;
  laba_kotor: number;
  biaya_operasional: number;
  laba_bersih: number;
  margin_persen: number;
}

export interface SettingsDTO {
  toko_nama: string;
  toko_alamat: string;
  toko_telepon: string;
  header_nota: string;
  footer_nota: string;
  printer_nama: string;
  printer_port: string;
  kertas_lebar: "58mm" | "80mm";
  auto_kick_drawer: boolean;
  ppn_aktif: boolean;
  ppn_persen: number;
  wa_notif_nomor: string;
  wa_notif_jam: string;
  cloud_sync_aktif: boolean;
  margin_atas?: number;
  margin_bawah?: number;
  cetak_logo?: boolean;
  logo_icon?: string;
  logo_url?: string;
  cetak_barcode?: boolean;
  cetak_telepon?: boolean;
  cetak_kasir?: boolean;
  ukuran_font?: "normal" | "compact";
  auto_cut?: boolean;
}

export interface OperatorDTO {
  id: string;
  cabang_id: string;
  kode: string;
  nama: string;
  role: string;
  is_admin: boolean;
  is_aktif?: boolean;
}

export interface BackupItemDTO {
  nama_file: string;
  path: string;
  ukuran_bytes: number;
  ukuran_formatted: string;
  waktu: string;
}

export interface BackupResultDTO {
  sukses: boolean;
  pesan: string;
  file?: BackupItemDTO;
  total_rotasi_dihapus: number;
}

export interface CabangDTO {
  id: string;
  kode: string;
  nama: string;
  alamat?: string;
  telepon?: string;
  is_pusat: boolean;
  sync_status: string;
}

export interface DeviceDTO {
  id: string;
  cabang_id: string;
  kode: string;
  nama: string;
  role: string; // "server" | "client"
  machine_id: string;
  ip_address?: string;
  is_active: boolean;
}

export interface NetworkConfigDTO {
  cabang_id: string;
  cabang_nama: string;
  cabang_kode: string;
  is_pusat: boolean;
  device_id: string;
  device_kode: string;
  device_nama: string;
  device_role: string;
  machine_id: string;
  ip_address: string;
  server_ip: string;
  lan_port: number;
  cloud_url: string;
  cloud_sync_enabled: boolean;
  daftar_cabang: CabangDTO[];
  daftar_device: DeviceDTO[];
}

export type ModulePermissionKey =
  | "beranda"
  | "kasir"
  | "produk"
  | "penjualan"
  | "pembelian"
  | "operasional"
  | "member"
  | "supplier"
  | "hutang_piutang"
  | "laporan"
  | "pengaturan";

export interface ModulePermissionDef {
  key: ModulePermissionKey;
  label: string;
  icon: string;
  desc: string;
}

export const ALL_MODULE_PERMISSIONS: ModulePermissionDef[] = [
  { key: "beranda", label: "Beranda Utama", icon: "dashboard", desc: "Dashboard ringkasan omset, aktivitas & stok" },
  { key: "kasir", label: "Kasir (POS)", icon: "point_of_sale", desc: "Layar transaksi kasir & scan barcode [F1]" },
  { key: "produk", label: "Menu Produk & Stok", icon: "inventory_2", desc: "Katalog produk, edit harga & stok barang [F2]" },
  { key: "penjualan", label: "Riwayat Penjualan", icon: "receipt_long", desc: "Daftar nota penjualan & cetak ulang struk [F3]" },
  { key: "pembelian", label: "Riwayat Pembelian", icon: "local_shipping", desc: "Faktur pembelian & pengadaan barang masuk" },
  { key: "operasional", label: "Biaya & Operasional", icon: "payments", desc: "Buku kas operasional, beban toko & cashflow" },
  { key: "member", label: "Master Member", icon: "loyalty", desc: "Data pelanggan, tier member & reward poin" },
  { key: "supplier", label: "Master Supplier", icon: "factory", desc: "Daftar rekanan vendor & jadwal pasokan" },
  { key: "hutang_piutang", label: "Hutang & Piutang", icon: "account_balance_wallet", desc: "Buku jatuh tempo hutang dagang & piutang" },
  { key: "laporan", label: "Laporan Bisnis", icon: "monitoring", desc: "Laporan omset, laba rugi & produk terlaris" },
  { key: "pengaturan", label: "Pengaturan Sistem", icon: "settings", desc: "Profil toko, hardware ESC/POS & manajemen operator" },
];

export function getOperatorPermissions(op: OperatorDTO | null | undefined): ModulePermissionKey[] {
  if (!op) return [];
  if (op.is_admin || op.role === "admin" || op.role.startsWith("admin")) {
    return ALL_MODULE_PERMISSIONS.map((m) => m.key);
  }

  if (op.role.includes(":")) {
    const parts = op.role.split(":");
    const perms = parts[1].split(",").map((p) => p.trim() as ModulePermissionKey).filter(Boolean);
    if (!perms.includes("beranda")) perms.unshift("beranda");
    return perms;
  }

  const baseRole = op.role.toLowerCase();
  if (baseRole === "supervisor") {
    return ["beranda", "kasir", "produk", "penjualan", "pembelian", "operasional", "member", "supplier", "hutang_piutang", "laporan"];
  } else if (baseRole === "gudang") {
    return ["beranda", "produk", "pembelian", "supplier"];
  } else {
    return ["beranda", "kasir", "penjualan", "member"];
  }
}

export function canOperatorAccess(op: OperatorDTO | null | undefined, view: string): boolean {
  if (!op) return false;
  if (op.is_admin || op.role === "admin" || op.role.startsWith("admin")) return true;
  const perms = getOperatorPermissions(op);
  return perms.includes(view as ModulePermissionKey);
}

/* ── API Calls ── */

export const api = {
  login: (kode: string, password: string) =>
    invoke<OperatorDTO>("login", { kode, password }),
  logout: () => invoke<void>("logout"),
  getOperators: () =>
    invoke<OperatorDTO[]>("get_operators").catch(() => [
      {
        id: "op-admin",
        cabang_id: "CABANG-01",
        kode: "admin",
        nama: "Administrator",
        role: "admin",
        is_admin: true,
        is_aktif: true,
      },
    ]),
  simpanOperator: (op: {
    id?: string;
    kode: string;
    nama: string;
    role: string;
    password?: string;
    is_aktif: boolean;
  }) =>
    invoke<OperatorDTO>("simpan_operator", {
      id: op.id,
      kode: op.kode,
      nama: op.nama,
      role: op.role,
      password: op.password,
      isAktif: op.is_aktif,
      is_aktif: op.is_aktif,
    }),
  hapusOperator: (id: string) => invoke<void>("hapus_operator", { id }),
  getCurrentUser: () => invoke<OperatorDTO | null>("get_current_user"),
  ubahPassword: (kode: string, lama: string, baru: string) =>
    invoke<void>("ubah_password", { kode, lama, baru }),
  getSettings: async () => {
    try {
      const res = await invoke<SettingsDTO>("get_settings");
      if (res && res.toko_nama) {
        localStorage.setItem("fazpos_settings", JSON.stringify(res));
        return res;
      }
    } catch (e) {
      console.warn("api.getSettings fallback to localStorage:", e);
    }
    const cached = localStorage.getItem("fazpos_settings");
    if (cached) {
      try {
        return JSON.parse(cached) as SettingsDTO;
      } catch {}
    }
    return {
      toko_nama: "MUEEZA STORE",
      toko_alamat: "Jl. Pemuda No. 108, Muaro, Sijunjung, Sumatera Barat",
      toko_telepon: "0812-6789-0123",
      header_nota: "SELAMAT DATANG DI MUEEZA STORE\nBelanja Hemat, Lengkap & Terpercaya",
      footer_nota: "TERIMA KASIH ATAS KUNJUNGAN ANDA\nBarang yang sudah dibeli tidak dapat ditukar/dikembalikan",
      printer_nama: "POS-80C Thermal Printer",
      printer_port: "USB001",
      kertas_lebar: "80mm",
      auto_kick_drawer: true,
      ppn_aktif: true,
      ppn_persen: 11,
      wa_notif_nomor: "0812-3456-7890",
      wa_notif_jam: "21:00",
      cloud_sync_aktif: true,
      margin_atas: 1,
      margin_bawah: 3,
      cetak_logo: true,
      logo_icon: "storefront",
      logo_url: "",
      cetak_barcode: true,
      cetak_telepon: true,
      cetak_kasir: true,
      ukuran_font: "normal",
      auto_cut: true,
    } as SettingsDTO;
  },
  saveSettings: async (settings: SettingsDTO) => {
    localStorage.setItem("fazpos_settings", JSON.stringify(settings));
    return invoke<void>("save_settings", { settings });
  },
  backupDatabase: () => invoke<BackupResultDTO>("backup_database"),
  cekIntegritasDatabase: () => invoke<boolean>("cek_integritas_database"),
  getBackupList: () => invoke<BackupItemDTO[]>("get_backup_list"),
  getNetworkConfig: () => invoke<NetworkConfigDTO>("get_network_config"),
  simpanCabangBaru: (kode: string, nama: string, alamat?: string, telepon?: string, is_pusat?: boolean) =>
    invoke<CabangDTO>("simpan_cabang_baru", {
      kode,
      nama,
      alamat,
      telepon,
      isPusat: is_pusat ?? false,
      is_pusat: is_pusat ?? false,
    }),
  simpanDeviceBaru: (kode: string, nama: string, role: string, ip_address?: string) =>
    invoke<DeviceDTO>("simpan_device_baru", {
      kode,
      nama,
      role,
      ipAddress: ip_address,
      ip_address,
    }),
  getStatusInfo: () => invoke<StatusInfoDTO>("get_status_info"),
  aktivasiLisensi: (token: string) => invoke<StatusInfoDTO>("aktivasi_lisensi", { token }),
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

  /* ── LAN Discovery & Multi-Kasir ── */
  getDiscoveredDevices: () => invoke<DiscoveredDeviceDTO[]>("get_discovered_devices"),
  pingLanDevice: (ip: string, port: number) =>
    invoke<LanPingResultDTO>("ping_lan_device", { ip, port }),
  gabungKeServer: (serverIp: string, serverPort: number) =>
    invoke<void>("gabung_ke_server", {
      serverIp,
      serverPort,
      server_ip: serverIp,
      server_port: serverPort,
    }),

  /* ── Supabase BYO-Cloud Sync ── */
  getSupabaseConfig: () => invoke<SupabaseConfigDTO>("get_supabase_config"),
  saveSupabaseConfig: (url: string, key: string, autoSync: boolean, intervalMenit: number) =>
    invoke<void>("save_supabase_config", {
      url,
      key,
      autoSync,
      intervalMenit,
      auto_sync: autoSync,
      interval_menit: intervalMenit,
    }),
  testSupabaseConnection: (url: string, key: string) =>
    invoke<SupabaseTestResultDTO>("test_supabase_connection", { url, key }),
  syncSupabaseNow: () => invoke<SupabaseSyncResultDTO>("sync_supabase_now"),
  getSupabaseSqlDdl: () => invoke<string>("get_supabase_sql_ddl"),
  getSyncLog: (limit?: number) => invoke<SyncLogDTO[]>("get_sync_log", { limit }),
};

export interface DiscoveredDeviceDTO {
  device_id: string;
  device_nama: string;
  cabang_id: string;
  cabang_nama: string;
  role: string;
  ip_address: string;
  port: number;
  machine_id: string;
  license_status: string;
  versi: string;
  is_online: boolean;
  last_seen: string;
  latency_ms?: number;
}

export interface LanPingResultDTO {
  sukses: boolean;
  pesan: string;
  latency_ms: number;
}

export interface SupabaseConfigDTO {
  url: string;
  api_key: string;
  is_bound: boolean;
  auto_sync: boolean;
  interval_menit: number;
  pending_count: number;
  last_sync_waktu?: string;
  last_sync_status?: string;
}

export interface SupabaseTestResultDTO {
  sukses: boolean;
  pesan: string;
  latency_ms: number;
}

export interface SupabaseSyncResultDTO {
  sukses: boolean;
  total_dikirim: number;
  total_berhasil: number;
  total_gagal: number;
  durasi_ms: number;
  pesan: string;
}

export interface SyncLogDTO {
  id: string;
  waktu: string;
  tabel: string;
  jumlah_record: number;
  status: string;
  pesan?: string;
  durasi_ms?: number;
}

/* ── Helpers ── */

export function formatRupiah(n: number): string {
  return "Rp " + n.toLocaleString("id-ID", { minimumFractionDigits: 0 });
}
