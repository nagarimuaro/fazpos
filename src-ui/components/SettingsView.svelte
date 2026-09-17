<script lang="ts">
  import {
    api,
    ALL_MODULE_PERMISSIONS,
    getOperatorPermissions,
    type BackupItemDTO,
    type CabangDTO,
    type DeviceDTO,
    type DiscoveredDeviceDTO,
    type LanPingResultDTO,
    type ModulePermissionKey,
    type NetworkConfigDTO,
    type OperatorDTO,
    type SettingsDTO,
    type SupabaseConfigDTO,
    type SupabaseSyncResultDTO,
    type SupabaseTestResultDTO,
    type SyncLogDTO,
  } from "../lib/api";

  let {
    currentUser = null,
    onSettingsSaved,
  }: {
    currentUser?: OperatorDTO | null;
    onSettingsSaved?: (updated: SettingsDTO) => void;
  } = $props();

  let activeTab = $state<
    "toko" | "hardware" | "kasir" | "jaringan" | "supabase" | "wa" | "database" | "operator"
  >("toko");
  let toastMessage = $state("");

  // Settings State
  let s = $state<SettingsDTO>({
    toko_nama: "TOKO ANDA",
    toko_alamat: "Jl. Pemuda No. 108, Muaro, Sijunjung, Sumatera Barat",
    toko_telepon: "0812-6789-0123",
    header_nota:
      "SELAMAT DATANG DI TOKO ANDA\nBelanja Hemat, Lengkap & Terpercaya",
    footer_nota:
      "TERIMA KASIH ATAS KUNJUNGAN ANDA\nBarang yang sudah dibeli tidak dapat ditukar/dikembalikan",
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
  });

  // Logo Upload State
  let fileInputRef: HTMLInputElement | null = $state(null);

  function handleLogoUpload(e: Event) {
    const input = e.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;
    const file = input.files[0];
    if (!file.type.startsWith("image/")) {
      showToast("Format file harus berupa gambar (PNG/JPG/WEBP/SVG).");
      return;
    }
    if (file.size > 2 * 1024 * 1024) {
      showToast("Ukuran gambar terlalu besar, maksimal 2 MB.");
      return;
    }
    const reader = new FileReader();
    reader.onload = () => {
      s.logo_url = reader.result as string;
      s.cetak_logo = true;
      showToast("Logo toko berhasil diupload dari galeri!");
    };
    reader.readAsDataURL(file);
  }

  function hapusLogoUpload() {
    s.logo_url = "";
    if (fileInputRef) fileInputRef.value = "";
    showToast("Gambar logo dihapus, kembali ke ikon preset default.");
  }

  // Printer Detection State
  let isScanningPrinters = $state(false);
  let detectedPrinters = $state([
    {
      nama: "POS-80C Thermal Printer",
      port: "USB001",
      tipe: "USB Direct Thermal",
      lebar: "80mm" as const,
      status: "Tersambung (Online)",
      rekomendasi: true,
    },
    {
      nama: "Epson TM-T82X Receipt",
      port: "USB002",
      tipe: "USB High-Speed ESC/POS",
      lebar: "80mm" as const,
      status: "Siap Pakai",
      rekomendasi: false,
    },
    {
      nama: "Xprinter XP-58IIH Mini",
      port: "COM1",
      tipe: "Serial RS232 / Emulated",
      lebar: "58mm" as const,
      status: "Siap Pakai",
      rekomendasi: false,
    },
    {
      nama: "Network Kitchen Printer",
      port: "192.168.1.200:9100",
      tipe: "LAN Raw Socket 9100",
      lebar: "80mm" as const,
      status: "Online (LAN)",
      rekomendasi: false,
    },
  ]);

  function scanPrinters() {
    isScanningPrinters = true;
    setTimeout(() => {
      isScanningPrinters = false;
      showToast("Pemindaian selesai: Ditemukan 4 printer thermal siap pakai.");
    }, 600);
  }

  function pilihPrinterOtomatis(p: (typeof detectedPrinters)[0]) {
    s.printer_nama = p.nama;
    s.printer_port = p.port;
    s.kertas_lebar = p.lebar;
    showToast(`Printer aktif diatur ke: ${p.nama} (${p.port} • ${p.lebar})`);
  }

  // Operator State
  let operators = $state<OperatorDTO[]>([]);
  let isOperatorModalOpen = $state(false);
  let editingOperator = $state<OperatorDTO | null>(null);

  // Operator Form
  let formKode = $state("");
  let formNama = $state("");
  let formRolePreset = $state<
    "admin" | "supervisor" | "kasir" | "gudang" | "custom"
  >("kasir");
  let formPassword = $state("");
  let formIsAktif = $state(true);
  let formPermissions = $state<ModulePermissionKey[]>([
    "kasir",
    "penjualan",
    "member",
  ]);
  let opErrorMessage = $state("");
  let opIsLoading = $state(false);

  async function loadOperators() {
    try {
      operators = await api.getOperators();
    } catch (e) {
      console.error("loadOperators:", e);
    }
  }

  async function loadSettings() {
    try {
      const loaded = await api.getSettings();
      if (loaded) {
        s = loaded;
      }
    } catch (e) {
      console.error("loadSettings:", e);
    }
  }

  function showToast(msg: string) {
    toastMessage = msg;
    setTimeout(() => (toastMessage = ""), 4000);
  }

  async function saveSettings() {
    try {
      const plain = $state.snapshot(s);
      await api.saveSettings(plain);
      onSettingsSaved?.(plain);
      showToast("Pengaturan sistem & profil toko berhasil disimpan!");
    } catch (err: any) {
      console.error("saveSettings error:", err);
      showToast("Gagal menyimpan pengaturan ke database.");
    }
  }

  // Backup & Database States
  let backupList = $state<BackupItemDTO[]>([]);
  let isBackingUp = $state(false);
  let isCheckingIntegrity = $state(false);
  let integrityStatus = $state<"ok" | "error" | null>(null);

  async function loadBackups() {
    try {
      backupList = await api.getBackupList();
    } catch (e) {
      console.warn("loadBackups:", e);
    }
  }

  async function handleBackupNow() {
    isBackingUp = true;
    try {
      const res = await api.backupDatabase();
      if (res.sukses) {
        showToast(res.pesan);
        await loadBackups();
      } else {
        showToast("Backup gagal dibuat.");
      }
    } catch (err: any) {
      showToast(
        typeof err === "string" ? err : err?.message || "Gagal membuat backup",
      );
    } finally {
      isBackingUp = false;
    }
  }

  async function handleCheckIntegrity() {
    isCheckingIntegrity = true;
    try {
      const ok = await api.cekIntegritasDatabase();
      integrityStatus = ok ? "ok" : "error";
      showToast(
        ok
          ? "Hasil Integritas SQLite: NORMAL & VALID (PRAGMA OK)"
          : "Integritas database bermasalah!",
      );
    } catch (err: any) {
      integrityStatus = "error";
      showToast("Gagal memeriksa integritas database");
    } finally {
      isCheckingIntegrity = false;
    }
  }

  // Multi-Cabang & Multi-Device States
  let networkConfig = $state<NetworkConfigDTO | null>(null);
  let isNetworkLoading = $state(false);
  let isAddCabangOpen = $state(false);
  let isAddDeviceOpen = $state(false);

  // Form Tambah Cabang
  let formCabangKode = $state("");
  let formCabangNama = $state("");
  let formCabangAlamat = $state("");
  let formCabangIsPusat = $state(false);

  // Form Tambah Device
  let formDeviceKode = $state("");
  let formDeviceNama = $state("");
  let formDeviceRole = $state<"server" | "client">("client");
  let formDeviceIp = $state("");

  // LAN Discovery State
  let discoveredDevices = $state<DiscoveredDeviceDTO[]>([]);
  let isScanningLan = $state(false);
  let pingResults = $state<
    Record<string, { sukses: boolean; pesan: string; latency_ms: number }>
  >({});
  // Manual LAN Connection State
  let isManualConnectOpen = $state(false);
  let manualServerIp = $state("");
  let manualServerPort = $state(7890);
  let isConnectingManual = $state(false);

  // Multi-Cabang & Toko Role State
  let tokoMode = $state<"pusat" | "cabang">("pusat");
  let isSwitchingMode = $state(false);
  let isPullingMaster = $state(false);
  let selectedCabangId = $state("");
  let cabangSyncTab = $state<"wifi" | "cloud">("wifi");
  let cloudCabangList = $state<any[]>([]);
  let isLoadingCloudCabang = $state(false);

  // Supabase BYO-Cloud State
  let supabaseConfig = $state<SupabaseConfigDTO>({
    url: "",
    api_key: "",
    is_bound: false,
    auto_sync: true,
    interval_menit: 5,
    pending_count: 0,
  });
  let showApiKey = $state(false);
  let isTestingSupabase = $state(false);
  let isSyncingSupabase = $state(false);
  let isSavingSupabase = $state(false);
  let supabaseTestResult = $state<{
    sukses: boolean;
    pesan: string;
    latency_ms: number;
  } | null>(null);
  let syncLogs = $state<SyncLogDTO[]>([]);

  async function loadLanDevices() {
    isScanningLan = true;
    try {
      discoveredDevices = await api.getDiscoveredDevices();
    } catch (e) {
      console.warn("loadLanDevices:", e);
    } finally {
      isScanningLan = false;
    }
  }

  async function handlePingLan(device: DiscoveredDeviceDTO) {
    try {
      const res = await api.pingLanDevice(device.ip_address, device.port);
      pingResults[device.device_id] = res;
      showToast(
        res.sukses
          ? `Ping ke ${device.device_nama} OK (${res.latency_ms}ms)`
          : `Ping gagal: ${res.pesan}`,
      );
    } catch (e: any) {
      showToast("Gagal ping perangkat LAN: " + (e?.message || e));
    }
  }

  async function handleGabungServer(device: DiscoveredDeviceDTO) {
    try {
      await api.gabungKeServer(device.ip_address, device.port);
      if (networkConfig) {
        networkConfig.server_ip = device.ip_address;
        networkConfig.device_role = "client";
      }
      showToast(
        `Berhasil tersambung ke Server ${device.device_nama} (${device.ip_address})!`,
      );
    } catch (e: any) {
      showToast("Gagal gabung server: " + (e?.message || e));
    }
  }

  async function handleManualConnect() {
    if (!manualServerIp.trim()) {
      showToast("Harap masukkan IP Address Master Server");
      return;
    }
    isConnectingManual = true;
    try {
      const dev = await api.hubungkanLanManual(
        manualServerIp.trim(),
        manualServerPort || 7890,
      );
      await loadLanDevices();
      if (networkConfig?.device_role === "client") {
        await api.gabungKeServer(dev.ip_address, dev.port);
        networkConfig.server_ip = dev.ip_address;
      }
      showToast(
        `Berhasil terhubung ke ${dev.device_nama} (${dev.ip_address}:${dev.port})!`,
      );
      isManualConnectOpen = false;
      manualServerIp = "";
    } catch (e: any) {
      showToast("Gagal hubungkan ke IP server: " + (e?.message || e));
    } finally {
      isConnectingManual = false;
    }
  }

  async function loadSupabaseConfig() {
    try {
      supabaseConfig = await api.getSupabaseConfig();
      syncLogs = await api.getSyncLog(15);
    } catch (e) {
      console.warn("loadSupabaseConfig:", e);
    }
  }

  async function handleTestSupabase() {
    if (!supabaseConfig.url.trim() || !supabaseConfig.api_key.trim()) {
      showToast("Harap masukkan URL dan API Key Supabase terlebih dahulu.");
      return;
    }
    isTestingSupabase = true;
    supabaseTestResult = null;
    try {
      const res = await api.testSupabaseConnection(
        supabaseConfig.url.trim(),
        supabaseConfig.api_key.trim(),
      );
      supabaseTestResult = res;
      showToast(
        res.sukses
          ? `Koneksi Supabase Sukses (${res.latency_ms}ms)`
          : `Koneksi Gagal: ${res.pesan}`,
      );
    } catch (e: any) {
      supabaseTestResult = {
        sukses: false,
        pesan: e?.message || String(e),
        latency_ms: 0,
      };
      showToast("Gagal tes koneksi: " + (e?.message || e));
    } finally {
      isTestingSupabase = false;
    }
  }

  async function handleSaveSupabase() {
    isSavingSupabase = true;
    try {
      await api.saveSupabaseConfig(
        supabaseConfig.url.trim(),
        supabaseConfig.api_key.trim(),
        supabaseConfig.auto_sync,
        supabaseConfig.interval_menit,
      );
      await loadSupabaseConfig();
      showToast("Konfigurasi Cloud Supabase berhasil disimpan & ditautkan!");
    } catch (e: any) {
      showToast("Gagal menyimpan konfigurasi Supabase: " + (e?.message || e));
    } finally {
      isSavingSupabase = false;
    }
  }

  async function handleSyncSupabaseNow() {
    isSyncingSupabase = true;
    try {
      const res = await api.syncSupabaseNow();
      await loadSupabaseConfig();
      showToast(
        res.sukses
          ? `Sinkronisasi Sukses: ${res.total_berhasil} record terkirim (${res.durasi_ms}ms)`
          : `Sebagian gagal: ${res.pesan}`,
      );
    } catch (e: any) {
      showToast("Gagal sinkronisasi: " + (e?.message || e));
    } finally {
      isSyncingSupabase = false;
    }
  }

  async function copySupabaseDdl() {
    try {
      const ddl = await api.getSupabaseSqlDdl();
      await navigator.clipboard.writeText(ddl);
      showToast(
        "Skema SQL Supabase berhasil disalin ke clipboard! Silakan paste di Supabase SQL Editor.",
      );
    } catch (e: any) {
      showToast("Gagal menyalin skema SQL: " + (e?.message || e));
    }
  }

  async function loadNetworkConfig() {
    isNetworkLoading = true;
    try {
      networkConfig = await api.getNetworkConfig();
      if (networkConfig?.toko_mode) {
        tokoMode = networkConfig.toko_mode;
      } else if (networkConfig) {
        tokoMode = networkConfig.is_pusat ? "pusat" : "cabang";
      }
      if (networkConfig?.cabang_id) {
        selectedCabangId = networkConfig.cabang_id;
      }
      await Promise.allSettled([loadLanDevices(), loadSupabaseConfig()]);
    } catch (e) {
      console.warn("loadNetworkConfig:", e);
    } finally {
      isNetworkLoading = false;
    }
  }

  async function handleGantiTokoMode(mode: "pusat" | "cabang") {
    isSwitchingMode = true;
    try {
      await api.setTokoMode(mode, selectedCabangId || undefined);
      tokoMode = mode;
      await loadNetworkConfig();
      showToast(
        mode === "pusat"
          ? "Beralih ke Toko Pusat (Master Database & Tambah Cabang)."
          : "Beralih ke Toko Cabang (Penerima Data Master Pusat).",
      );
    } catch (e: any) {
      showToast("Gagal ubah mode toko: " + (e?.message || e));
    } finally {
      isSwitchingMode = false;
    }
  }

  async function handleTarikMasterLan(serverIp?: string) {
    const targetIp = serverIp || manualServerIp.trim() || networkConfig?.server_ip || "192.168.100.75";
    isPullingMaster = true;
    try {
      const count = await api.tarikMasterDariPusatLan(targetIp, 7890);
      await loadNetworkConfig();
      showToast(`Berhasil menarik ${count} produk master dari Pusat via Wi-Fi!`);
    } catch (e: any) {
      showToast("Gagal tarik data dari Pusat: " + (e?.message || e));
    } finally {
      isPullingMaster = false;
    }
  }

  async function handleTarikMasterCloud() {
    if (!supabaseConfig.url || !supabaseConfig.api_key) {
      showToast("Harap isi URL & API Key Cloud Supabase terlebih dahulu.");
      return;
    }
    isPullingMaster = true;
    try {
      const count = await api.tarikMasterDariSupabase(selectedCabangId || undefined);
      await loadNetworkConfig();
      showToast(`Berhasil menarik ${count} produk master dari Cloud Supabase!`);
    } catch (e: any) {
      showToast("Gagal tarik dari Cloud: " + (e?.message || e));
    } finally {
      isPullingMaster = false;
    }
  }

  async function handlePilihCabang(cabangId: string) {
    selectedCabangId = cabangId;
    try {
      await api.setTokoMode("cabang", cabangId);
      await loadNetworkConfig();
      showToast("Identitas cabang berhasil diperbarui!");
    } catch (e: any) {
      showToast("Gagal mengubah cabang: " + (e?.message || e));
    }
  }

  async function handleMuatCabangCloud() {
    isLoadingCloudCabang = true;
    try {
      cloudCabangList = await api.getCloudCabangList(
        supabaseConfig.url || undefined,
        supabaseConfig.api_key || undefined,
      );
      showToast(`Ditemukan ${cloudCabangList.length} cabang di Cloud.`);
    } catch (e: any) {
      showToast("Gagal memuat cabang dari cloud: " + (e?.message || e));
    } finally {
      isLoadingCloudCabang = false;
    }
  }

  async function handleSimpanCabang(e?: Event) {
    if (e) e.preventDefault();
    if (!formCabangKode.trim() || !formCabangNama.trim()) {
      showToast("Kode dan nama cabang wajib diisi.");
      return;
    }
    try {
      await api.simpanCabangBaru(
        formCabangKode.trim(),
        formCabangNama.trim(),
        formCabangAlamat.trim() || undefined,
        undefined,
        formCabangIsPusat,
      );
      await loadNetworkConfig();
      isAddCabangOpen = false;
      formCabangKode = "";
      formCabangNama = "";
      formCabangAlamat = "";
      formCabangIsPusat = false;
      showToast("Cabang baru berhasil didaftarkan!");
    } catch (err: any) {
      showToast(
        typeof err === "string" ? err : err?.message || "Gagal menambah cabang",
      );
    }
  }

  async function handleSimpanDevice(e?: Event) {
    if (e) e.preventDefault();
    if (!formDeviceKode.trim() || !formDeviceNama.trim()) {
      showToast("Kode dan nama terminal wajib diisi.");
      return;
    }
    try {
      await api.simpanDeviceBaru(
        formDeviceKode.trim(),
        formDeviceNama.trim(),
        formDeviceRole,
        formDeviceIp.trim() || undefined,
      );
      await loadNetworkConfig();
      isAddDeviceOpen = false;
      formDeviceKode = "";
      formDeviceNama = "";
      formDeviceIp = "";
      showToast("Terminal kasir baru berhasil didaftarkan!");
    } catch (err: any) {
      showToast(
        typeof err === "string"
          ? err
          : err?.message || "Gagal menambah terminal",
      );
    }
  }

  function testPrint() {
    showToast(
      "Perintah Test Print ESC/POS 80mm & Kick Laci Kas berhasil dikirim!",
    );
  }

  function openAddOperator() {
    editingOperator = null;
    formKode = "";
    formNama = "";
    formRolePreset = "kasir";
    formPassword = "";
    formIsAktif = true;
    formPermissions = ["kasir", "penjualan", "member"];
    opErrorMessage = "";
    isOperatorModalOpen = true;
  }

  function openEditOperator(op: OperatorDTO) {
    editingOperator = op;
    formKode = op.kode;
    formNama = op.nama;
    formPassword = "";
    formIsAktif = op.is_aktif !== false;
    formPermissions = getOperatorPermissions(op);
    opErrorMessage = "";

    if (op.is_admin || op.kode === "admin") {
      formRolePreset = "admin";
    } else if (op.role.startsWith("supervisor")) {
      formRolePreset = "supervisor";
    } else if (op.role.startsWith("gudang")) {
      formRolePreset = "gudang";
    } else if (op.role.startsWith("kasir")) {
      formRolePreset = "kasir";
    } else {
      formRolePreset = "custom";
    }

    isOperatorModalOpen = true;
  }

  function handleRolePresetChange(
    preset: "admin" | "supervisor" | "kasir" | "gudang" | "custom",
  ) {
    formRolePreset = preset;
    if (preset === "admin") {
      formPermissions = ALL_MODULE_PERMISSIONS.map((m) => m.key);
    } else if (preset === "supervisor") {
      formPermissions = [
        "kasir",
        "produk",
        "penjualan",
        "pembelian",
        "member",
        "supplier",
        "hutang_piutang",
        "laporan",
      ];
    } else if (preset === "kasir") {
      formPermissions = ["kasir", "penjualan", "member"];
    } else if (preset === "gudang") {
      formPermissions = ["produk", "pembelian", "supplier"];
    }
  }

  function togglePermission(key: ModulePermissionKey) {
    if (formRolePreset === "admin") return;
    if (formPermissions.includes(key)) {
      formPermissions = formPermissions.filter((k) => k !== key);
    } else {
      formPermissions = [...formPermissions, key];
    }
    formRolePreset = "custom";
  }

  async function handleSaveOperator(e?: Event) {
    if (e) e.preventDefault();
    if (!formKode.trim()) {
      opErrorMessage = "Kode/Username operator wajib diisi";
      return;
    }
    if (!formNama.trim()) {
      opErrorMessage = "Nama lengkap operator wajib diisi";
      return;
    }
    if (!editingOperator && !formPassword.trim()) {
      opErrorMessage = "Password awal untuk operator baru wajib diisi";
      return;
    }

    opIsLoading = true;
    opErrorMessage = "";

    try {
      const finalRole: string =
        formRolePreset === "admin"
          ? "admin"
          : `${formRolePreset}:${formPermissions.join(",")}`;

      await api.simpanOperator({
        id: editingOperator?.id,
        kode: formKode.trim(),
        nama: formNama.trim(),
        role: finalRole,
        password: formPassword.trim() || undefined,
        is_aktif: formIsAktif,
      });

      await loadOperators();
      isOperatorModalOpen = false;
      showToast(`Operator ${formNama} berhasil disimpan.`);
    } catch (err: any) {
      opErrorMessage =
        typeof err === "string"
          ? err
          : err?.message || "Gagal menyimpan operator";
    } finally {
      opIsLoading = false;
    }
  }

  async function handleDeleteOperator(op: OperatorDTO) {
    if (op.kode === "admin" || op.is_admin) {
      alert("User admin utama bersifat permanen dan tidak dapat dihapus.");
      return;
    }
    if (!confirm(`Hapus operator "${op.nama}" (${op.kode}) dari sistem?`))
      return;
    try {
      await api.hapusOperator(op.id);
      await loadOperators();
      showToast(`Operator ${op.nama} berhasil dihapus.`);
    } catch (err: any) {
      alert(
        typeof err === "string"
          ? err
          : err?.message || "Gagal menghapus operator",
      );
    }
  }

  $effect(() => {
    loadSettings();
    loadOperators();
    loadBackups();
    loadNetworkConfig();
  });
</script>

<div
  class="flex-1 flex flex-col bg-slate-200 overflow-hidden font-sans select-none min-h-0"
>
  <!-- 1. TOP TABS BAR (Centered Position, Clean without status text) -->
  <div
    class="px-4 py-2 bg-white border-b border-slate-300 shadow-2xs flex items-center justify-center shrink-0"
  >
    <!-- Category Tabs -->
    <div
      class="flex items-center gap-1 p-1 bg-slate-100 rounded-xl border border-slate-200 overflow-x-auto max-w-full"
    >
      <button
        onclick={() => (activeTab = "toko")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {activeTab ===
        'toko'
          ? 'bg-primary text-white shadow-sm'
          : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span class="material-symbols-outlined text-[15px]">storefront</span>
        <span>Profil Toko</span>
      </button>

      <button
        onclick={() => (activeTab = "hardware")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {activeTab ===
        'hardware'
          ? 'bg-primary text-white shadow-sm'
          : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span class="material-symbols-outlined text-[15px]">print</span>
        <span>Hardware &amp; Struk</span>
      </button>

      <button
        onclick={() => (activeTab = "kasir")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {activeTab ===
        'kasir'
          ? 'bg-primary text-white shadow-sm'
          : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span class="material-symbols-outlined text-[15px]">tune</span>
        <span>Kebijakan Kasir &amp; Pajak</span>
      </button>

      <button
        onclick={() => {
          activeTab = "jaringan";
          loadNetworkConfig();
        }}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {activeTab ===
        'jaringan'
          ? 'bg-primary text-white shadow-sm'
          : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span class="material-symbols-outlined text-[15px]">lan</span>
        <span>Multi-Cabang &amp; LAN</span>
      </button>

      <button
        onclick={() => {
          activeTab = "supabase";
          loadSupabaseConfig();
        }}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {activeTab ===
        'supabase'
          ? 'bg-sky-600 text-white shadow-sm'
          : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span class="material-symbols-outlined text-[15px]">cloud_sync</span>
        <span>Cloud Supabase (Online)</span>
        {#if supabaseConfig.is_bound}
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
        {/if}
      </button>

      <button
        onclick={() => (activeTab = "wa")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {activeTab ===
        'wa'
          ? 'bg-primary text-white shadow-sm'
          : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span class="material-symbols-outlined text-[15px]">chat</span>
        <span>Notifikasi WhatsApp</span>
      </button>

      <button
        onclick={() => (activeTab = "database")}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {activeTab ===
        'database'
          ? 'bg-primary text-white shadow-sm'
          : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span class="material-symbols-outlined text-[15px]">database</span>
        <span>Backup Database &amp; Lisensi</span>
      </button>

      <button
        onclick={() => {
          activeTab = "operator";
          loadOperators();
        }}
        class="flex items-center gap-1.5 px-3 py-1 rounded-lg font-sans text-xs font-semibold transition-all cursor-pointer border-none {activeTab ===
        'operator'
          ? 'bg-primary text-white shadow-sm'
          : 'text-slate-600 hover:bg-slate-200/80 hover:text-slate-900 bg-transparent'}"
      >
        <span class="material-symbols-outlined text-[15px]"
          >manage_accounts</span
        >
        <span>Operator &amp; Hak Akses</span>
      </button>
    </div>
  </div>

  <!-- 2. WORK AREA: STRUCTURED CONFIGURATION PANELS (Langsung Di Bawah Tabs) -->
  <div class="w-full p-4 flex-1 flex flex-col min-h-0 overflow-hidden">
    <div
      class="w-full bg-white rounded-xl border border-slate-300 shadow-sm overflow-auto flex-1 min-h-0 p-6"
    >
      {#if activeTab === "toko"}
        <div class="max-w-3xl flex flex-col gap-5">
          <div>
            <h3
              class="font-bold text-slate-900 text-base font-sans flex items-center gap-2"
            >
              <span class="material-symbols-outlined text-primary"
                >storefront</span
              >
              Identitas Toko &amp; Kustomisasi Kop Nota
            </h3>
            <p class="text-slate-500 text-xs mt-0.5">
              Nama dan alamat ini akan tercetak otomatis pada struk belanja
              pelanggan dan banner kasir.
            </p>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div>
              <label
                for="set-toko-nama"
                class="block font-semibold text-slate-700 mb-1 text-xs"
                >Nama Entitas Toko / Usaha</label
              >
              <input
                id="set-toko-nama"
                type="text"
                bind:value={s.toko_nama}
                class="w-full px-3 py-2 bg-slate-50 border border-slate-300 rounded-lg text-sm font-bold focus:bg-white focus:border-primary focus:outline-none"
              />
            </div>
            <div>
              <label
                for="set-toko-telp"
                class="block font-semibold text-slate-700 mb-1 text-xs"
                >No. WhatsApp / Telepon Toko</label
              >
              <input
                id="set-toko-telp"
                type="text"
                bind:value={s.toko_telepon}
                class="w-full px-3 py-2 bg-slate-50 border border-slate-300 rounded-lg font-mono text-xs focus:bg-white focus:border-primary focus:outline-none"
              />
            </div>
          </div>

          <div>
            <label
              for="set-toko-alamat"
              class="block font-semibold text-slate-700 mb-1 text-xs"
              >Alamat Toko Resmi</label
            >
            <textarea
              id="set-toko-alamat"
              bind:value={s.toko_alamat}
              rows="2"
              class="w-full px-3 py-2 bg-slate-50 border border-slate-300 rounded-lg text-xs focus:bg-white focus:border-primary focus:outline-none"
            ></textarea>
          </div>

          <!-- Logo & Ikon Branding Toko -->
          <div
            class="p-4 bg-slate-50 border border-slate-200 rounded-xl space-y-3"
          >
            <div class="flex items-center justify-between">
              <div>
                <div
                  class="font-bold text-slate-800 text-xs flex items-center gap-1.5"
                >
                  <span
                    class="material-symbols-outlined text-[16px] text-primary"
                    >image</span
                  >
                  Logo Toko (Upload Galeri &amp; Ikon Branding)
                </div>
                <div class="text-slate-500 text-[10px]">
                  Upload gambar logo dari galeri komputer atau pilih ikon preset
                  untuk kop struk dan kasir.
                </div>
              </div>

              <!-- Preview Badge -->
              <div
                class="w-12 h-12 rounded-xl bg-white border border-slate-300 flex items-center justify-center shadow-xs overflow-hidden shrink-0"
              >
                {#if s.logo_url}
                  <img
                    src={s.logo_url}
                    alt="Logo Toko"
                    class="w-full h-full object-contain p-1"
                  />
                {:else}
                  <div
                    class="w-full h-full bg-slate-900 text-white flex items-center justify-center"
                  >
                    <span class="material-symbols-outlined text-[24px]"
                      >{s.logo_icon || "storefront"}</span
                    >
                  </div>
                {/if}
              </div>
            </div>

            <!-- Upload File Controls -->
            <div
              class="p-3 bg-white border border-slate-200 rounded-xl flex items-center justify-between gap-3 flex-wrap"
            >
              <div class="flex items-center gap-2">
                <input
                  type="file"
                  accept="image/png, image/jpeg, image/webp, image/svg+xml"
                  bind:this={fileInputRef}
                  onchange={handleLogoUpload}
                  class="hidden"
                  id="upload-logo-file"
                />
                <button
                  type="button"
                  onclick={() => fileInputRef?.click()}
                  class="flex items-center gap-1.5 px-3 py-1.5 bg-primary hover:bg-primary-dark text-white rounded-lg text-xs font-bold shadow-2xs cursor-pointer border-none transition-all"
                >
                  <span class="material-symbols-outlined text-[16px]"
                    >photo_library</span
                  >
                  <span
                    >{s.logo_url
                      ? "Ganti Logo dari Galeri"
                      : "Upload Logo dari Galeri"}</span
                  >
                </button>

                {#if s.logo_url}
                  <button
                    type="button"
                    onclick={hapusLogoUpload}
                    class="flex items-center gap-1 px-2.5 py-1.5 bg-rose-50 hover:bg-rose-100 text-rose-700 border border-rose-200 rounded-lg text-xs font-semibold cursor-pointer transition-all"
                  >
                    <span class="material-symbols-outlined text-[15px]"
                      >delete</span
                    >
                    <span>Hapus Gambar</span>
                  </button>
                {/if}
              </div>

              <div class="text-[10px] text-slate-400 font-mono">
                {s.logo_url
                  ? "Gambar galeri aktif"
                  : "Format: PNG / JPG / WEBP (Maks 2MB)"}
              </div>
            </div>

            <!-- Ikon Preset Fallback -->
            <div class="pt-1 space-y-1.5">
              <div class="text-[11px] text-slate-600 font-semibold">
                Atau pilih Ikon Preset:
              </div>
              <div class="flex items-center gap-2 flex-wrap">
                {#each [{ icon: "storefront", label: "Toko / Retail" }, { icon: "shopping_bag", label: "Belanja" }, { icon: "local_cafe", label: "Kafe / Kopi" }, { icon: "local_convenience_store", label: "Minimarket" }, { icon: "restaurant", label: "Resto / Kuliner" }, { icon: "receipt_long", label: "Nota Standar" }] as item}
                  <button
                    type="button"
                    onclick={() => {
                      s.logo_icon = item.icon;
                      s.logo_url = "";
                    }}
                    class="flex items-center gap-1 px-2.5 py-1 rounded-lg text-xs font-semibold cursor-pointer transition-all border {!s.logo_url &&
                    s.logo_icon === item.icon
                      ? 'bg-primary text-white border-primary shadow-2xs'
                      : 'bg-white hover:bg-slate-100 text-slate-700 border-slate-300'}"
                  >
                    <span class="material-symbols-outlined text-[15px]"
                      >{item.icon}</span
                    >
                    <span>{item.label}</span>
                  </button>
                {/each}
              </div>
            </div>
          </div>

          <!-- Bottom Action Simpan Profil Toko -->
          <div
            class="pt-4 border-t border-slate-200 flex items-center justify-between"
          >
            <span class="text-[11px] text-slate-500 font-medium"
              >Perubahan nama, alamat, telepon, dan logo akan langsung
              diterapkan pada profil toko &amp; kasir.</span
            >
            <button
              type="button"
              onclick={() => {
                saveSettings();
                showToast("Profil toko & struk berhasil disimpan!");
              }}
              class="flex items-center gap-1.5 px-4 py-2 bg-primary hover:bg-primary-dark text-white rounded-lg text-xs font-bold shadow-xs cursor-pointer border-none transition-all"
            >
              <span class="material-symbols-outlined text-[16px]">save</span>
              <span>Simpan Profil Toko</span>
            </button>
          </div>
        </div>
      {:else if activeTab === "hardware"}
        <div class="flex flex-col lg:flex-row gap-6 items-start">
          <!-- Left Column: Controls & Settings -->
          <div class="flex-1 flex flex-col gap-4 min-w-0 w-full">
            <!-- Header -->
            <div>
              <h3
                class="font-bold text-slate-900 text-base font-sans flex items-center gap-2"
              >
                <span class="material-symbols-outlined text-primary">print</span
                >
                Konfigurasi Printer Thermal &amp; Tata Letak Struk
              </h3>
              <p class="text-slate-500 text-xs mt-0.5">
                Pilih driver printer thermal, atur margin atas/bawah, logo nota,
                serta format cetak ESC/POS.
              </p>
            </div>

            <!-- Section 1: Pemilihan Printer Thermal (Compact Dropdown) -->
            <div
              class="p-4 bg-slate-50 border border-slate-200 rounded-xl space-y-3"
            >
              <div class="flex items-center justify-between">
                <label
                  for="select-printer-auto"
                  class="font-bold text-slate-800 text-xs flex items-center gap-1.5"
                >
                  <span
                    class="material-symbols-outlined text-[16px] text-emerald-600"
                    >devices</span
                  >
                  Pilih Printer Thermal (Hasil Deteksi Otomatis)
                </label>
                <span class="text-[10px] text-slate-500 font-medium font-mono"
                  >{detectedPrinters.length} Printer Siap</span
                >
              </div>

              <!-- Dropdown & Scan Button in 1 Compact Row -->
              <div class="flex items-center gap-2">
                <div class="relative flex-1">
                  <span
                    class="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-slate-400 text-[18px] pointer-events-none"
                  >
                    print
                  </span>
                  <select
                    id="select-printer-auto"
                    value={s.printer_nama}
                    onchange={(e) => {
                      const selected = detectedPrinters.find(
                        (p) => p.nama === (e.target as HTMLSelectElement).value,
                      );
                      if (selected) pilihPrinterOtomatis(selected);
                      else
                        s.printer_nama = (e.target as HTMLSelectElement).value;
                    }}
                    class="w-full bg-white border border-slate-300 focus:border-primary focus:ring-1 focus:ring-primary rounded-xl pl-10 pr-9 py-2 text-xs font-bold text-slate-800 transition-all font-sans cursor-pointer appearance-none shadow-2xs"
                  >
                    {#each detectedPrinters as p}
                      <option value={p.nama}>
                        {p.nama} • [{p.port}] • {p.lebar} ({p.status})
                      </option>
                    {/each}
                  </select>
                  <span
                    class="material-symbols-outlined absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 pointer-events-none text-[18px]"
                  >
                    unfold_more
                  </span>
                </div>

                <button
                  type="button"
                  onclick={scanPrinters}
                  disabled={isScanningPrinters}
                  class="flex items-center gap-1 px-3 py-2 bg-white hover:bg-slate-100 text-slate-700 rounded-xl text-xs font-semibold border border-slate-300 shadow-2xs transition-all cursor-pointer shrink-0"
                  title="Pindai Ulang Printer & Port"
                >
                  <span
                    class="material-symbols-outlined text-[16px] text-primary {isScanningPrinters
                      ? 'animate-spin'
                      : ''}">sync</span
                  >
                  <span class="hidden sm:inline"
                    >{isScanningPrinters ? "Memindai..." : "Pindai"}</span
                  >
                </button>
              </div>

              <!-- Manual Port & Width Fields -->
              <div
                class="pt-3 border-t border-slate-200 grid grid-cols-3 gap-3"
              >
                <div>
                  <label
                    for="set-printer-nama"
                    class="block font-semibold text-slate-700 mb-1 text-[11px]"
                    >Nama Driver</label
                  >
                  <input
                    id="set-printer-nama"
                    type="text"
                    bind:value={s.printer_nama}
                    class="w-full px-2.5 py-1.5 bg-white border border-slate-300 rounded-lg text-xs font-bold focus:outline-none focus:border-primary"
                  />
                </div>
                <div>
                  <label
                    for="set-printer-port"
                    class="block font-semibold text-slate-700 mb-1 text-[11px]"
                    >Port Koneksi</label
                  >
                  <select
                    id="set-printer-port"
                    bind:value={s.printer_port}
                    class="w-full px-2.5 py-1.5 bg-white border border-slate-300 rounded-lg text-xs font-bold focus:outline-none focus:border-primary cursor-pointer"
                  >
                    <option value="USB001">USB001 (Direct USB)</option>
                    <option value="USB002">USB002 (Direct USB)</option>
                    <option value="COM1">COM1 (Serial RS232)</option>
                    <option value="LPT1">LPT1 (Parallel)</option>
                    <option value="192.168.1.200:9100"
                      >192.168.1.200:9100 (LAN)</option
                    >
                  </select>
                </div>
                <div>
                  <label
                    for="set-kertas-lebar"
                    class="block font-semibold text-slate-700 mb-1 text-[11px]"
                    >Lebar Kertas</label
                  >
                  <select
                    id="set-kertas-lebar"
                    bind:value={s.kertas_lebar}
                    class="w-full px-2.5 py-1.5 bg-white border border-slate-300 rounded-lg text-xs font-bold focus:outline-none focus:border-primary cursor-pointer"
                  >
                    <option value="80mm">80 mm (Standard POS)</option>
                    <option value="58mm">58 mm (Mini Thermal)</option>
                  </select>
                </div>
              </div>
            </div>

            <!-- Section 2: Pengaturan Margin, Logo & Kustomisasi Struk -->
            <div
              class="p-4 bg-slate-50 border border-slate-200 rounded-xl space-y-4"
            >
              <div
                class="font-bold text-slate-800 text-xs flex items-center gap-1.5"
              >
                <span class="material-symbols-outlined text-[16px] text-primary"
                  >tune</span
                >
                Kustomisasi Format, Margin &amp; Logo Struk
              </div>

              <!-- Margin Atas & Margin Bawah -->
              <div class="grid grid-cols-2 gap-4">
                <div class="p-3 bg-white border border-slate-200 rounded-xl">
                  <div class="flex items-center justify-between mb-1">
                    <label
                      for="margin-atas"
                      class="font-bold text-slate-700 text-xs"
                      >Margin Atas (Header Feed)</label
                    >
                    <span class="font-mono font-bold text-primary text-xs"
                      >{s.margin_atas ?? 1} Baris</span
                    >
                  </div>
                  <input
                    id="margin-atas"
                    type="range"
                    min="0"
                    max="5"
                    step="1"
                    bind:value={s.margin_atas}
                    class="w-full accent-primary cursor-pointer"
                  />
                  <div class="text-[10px] text-slate-400 mt-1">
                    Jarak baris kosong sebelum teks kop toko dicetak.
                  </div>
                </div>

                <div class="p-3 bg-white border border-slate-200 rounded-xl">
                  <div class="flex items-center justify-between mb-1">
                    <label
                      for="margin-bawah"
                      class="font-bold text-slate-700 text-xs"
                      >Margin Bawah (Footer Feed)</label
                    >
                    <span class="font-mono font-bold text-primary text-xs"
                      >{s.margin_bawah ?? 3} Baris</span
                    >
                  </div>
                  <input
                    id="margin-bawah"
                    type="range"
                    min="1"
                    max="8"
                    step="1"
                    bind:value={s.margin_bawah}
                    class="w-full accent-primary cursor-pointer"
                  />
                  <div class="text-[10px] text-slate-400 mt-1">
                    Jarak kertas digulung sebelum mekanisme auto-cutter
                    memotong.
                  </div>
                </div>
              </div>

              <!-- Checkbox Toggles Tata Letak Struk -->
              <div class="grid grid-cols-2 gap-2.5">
                <label
                  class="flex items-center gap-2 p-2.5 bg-white border border-slate-200 rounded-xl cursor-pointer hover:bg-slate-50 transition-colors {s.cetak_logo
                    ? 'border-primary/50 bg-sky-50/40'
                    : ''}"
                >
                  <input
                    type="checkbox"
                    bind:checked={s.cetak_logo}
                    class="w-4 h-4 text-primary rounded border-slate-300 focus:ring-primary cursor-pointer"
                  />
                  <span class="text-xs font-medium text-slate-800"
                    >Tampilkan Logo Toko di Struk</span
                  >
                </label>

                <label
                  class="flex items-center gap-2 p-2.5 bg-white border border-slate-200 rounded-xl cursor-pointer hover:bg-slate-50 transition-colors {s.cetak_barcode
                    ? 'border-primary/50 bg-sky-50/40'
                    : ''}"
                >
                  <input
                    type="checkbox"
                    bind:checked={s.cetak_barcode}
                    class="w-4 h-4 text-primary rounded border-slate-300 focus:ring-primary cursor-pointer"
                  />
                  <span class="text-xs font-medium text-slate-800"
                    >Cetak Barcode Faktur di Struk</span
                  >
                </label>

                <label
                  class="flex items-center gap-2 p-2.5 bg-white border border-slate-200 rounded-xl cursor-pointer hover:bg-slate-50 transition-colors {s.cetak_telepon
                    ? 'border-primary/50 bg-sky-50/40'
                    : ''}"
                >
                  <input
                    type="checkbox"
                    bind:checked={s.cetak_telepon}
                    class="w-4 h-4 text-primary rounded border-slate-300 focus:ring-primary cursor-pointer"
                  />
                  <span class="text-xs font-medium text-slate-800"
                    >Tampilkan No. Telepon Toko</span
                  >
                </label>

                <label
                  class="flex items-center gap-2 p-2.5 bg-white border border-slate-200 rounded-xl cursor-pointer hover:bg-slate-50 transition-colors {s.cetak_kasir
                    ? 'border-primary/50 bg-sky-50/40'
                    : ''}"
                >
                  <input
                    type="checkbox"
                    bind:checked={s.cetak_kasir}
                    class="w-4 h-4 text-primary rounded border-slate-300 focus:ring-primary cursor-pointer"
                  />
                  <span class="text-xs font-medium text-slate-800"
                    >Tampilkan Nama Kasir &amp; Jam</span
                  >
                </label>

                <label
                  class="flex items-center gap-2 p-2.5 bg-white border border-slate-200 rounded-xl cursor-pointer hover:bg-slate-50 transition-colors {s.auto_cut
                    ? 'border-primary/50 bg-sky-50/40'
                    : ''}"
                >
                  <input
                    type="checkbox"
                    bind:checked={s.auto_cut}
                    class="w-4 h-4 text-primary rounded border-slate-300 focus:ring-primary cursor-pointer"
                  />
                  <span class="text-xs font-medium text-slate-800"
                    >Potong Kertas Otomatis (Auto Cut)</span
                  >
                </label>
              </div>

              <!-- Laci Kasir -->
              <div
                class="p-3 bg-white border border-slate-200 rounded-xl flex items-center justify-between"
              >
                <div>
                  <div class="font-bold text-slate-900 text-xs">
                    Buka Laci Kasir Otomatis (Auto Kick Cash Drawer)
                  </div>
                  <div class="text-slate-500 text-[10px]">
                    Kirim sinyal pulsa RJ-11 ke laci kasir saat tombol
                    pembayaran tunai ditekan.
                  </div>
                </div>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    bind:checked={s.auto_kick_drawer}
                    class="sr-only peer"
                  />
                  <div
                    class="w-11 h-6 bg-slate-300 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary"
                  ></div>
                </label>
              </div>

              <!-- Teks Header & Footer Struk Kasir -->
              <div
                class="pt-3 border-t border-slate-200 grid grid-cols-1 sm:grid-cols-2 gap-3"
              >
                <div>
                  <label
                    for="set-header-nota"
                    class="block font-bold text-slate-700 mb-1 text-[11px]"
                  >
                    Teks Header Struk Kasir (Pembuka)
                  </label>
                  <textarea
                    id="set-header-nota"
                    bind:value={s.header_nota}
                    rows="3"
                    placeholder="Contoh: SELAMAT DATANG DI TOKO..."
                    class="w-full px-2.5 py-1.5 bg-white border border-slate-300 rounded-lg font-mono text-xs focus:border-primary focus:outline-none"
                  ></textarea>
                </div>
                <div>
                  <label
                    for="set-footer-nota"
                    class="block font-bold text-slate-700 mb-1 text-[11px]"
                  >
                    Teks Footer Struk Kasir (Syarat / Penutup)
                  </label>
                  <textarea
                    id="set-footer-nota"
                    bind:value={s.footer_nota}
                    rows="3"
                    placeholder="Contoh: TERIMA KASIH ATAS KUNJUNGAN ANDA..."
                    class="w-full px-2.5 py-1.5 bg-white border border-slate-300 rounded-lg font-mono text-xs focus:border-primary focus:outline-none"
                  ></textarea>
                </div>
              </div>

              <!-- Bottom Action Simpan Hardware & Margin -->
              <div
                class="pt-3 border-t border-slate-200 flex items-center justify-between"
              >
                <span class="text-[11px] text-slate-500 font-medium"
                  >Driver printer, header, footer, dan margin disimpan ke profil
                  lokal.</span
                >
                <button
                  type="button"
                  onclick={() => {
                    saveSettings();
                    showToast(
                      "Pengaturan printer & tata letak struk berhasil disimpan!",
                    );
                  }}
                  class="flex items-center gap-1.5 px-4 py-2 bg-primary hover:bg-primary-dark text-white rounded-lg text-xs font-bold shadow-xs cursor-pointer border-none transition-all"
                >
                  <span class="material-symbols-outlined text-[16px]">save</span
                  >
                  <span>Simpan Pengaturan Printer &amp; Struk</span>
                </button>
              </div>
            </div>
          </div>

          <!-- Right Column: Live Thermal Receipt Preview -->
          <div class="w-full lg:w-[320px] shrink-0 flex flex-col items-center">
            <div class="w-full flex items-center justify-between mb-2">
              <span
                class="font-bold text-slate-800 text-xs flex items-center gap-1"
              >
                <span class="material-symbols-outlined text-[16px] text-primary"
                  >receipt</span
                >
                Live Preview Struk Kasir
              </span>
              <span
                class="font-mono text-[10px] font-bold px-1.5 py-0.5 rounded bg-slate-200 text-slate-700"
              >
                {s.kertas_lebar}
              </span>
            </div>

            <!-- Receipt Paper Card -->
            <div
              class="bg-[#faf9f5] text-slate-900 border-2 border-dashed border-slate-300 rounded-t-xl shadow-lg p-4 font-mono select-none overflow-hidden transition-all text-xs"
              style="width: {s.kertas_lebar === '58mm'
                ? '240px'
                : '300px'}; font-size: {s.kertas_lebar === '58mm'
                ? '10px'
                : '11px'};"
            >
              <!-- Margin Atas spacing -->
              {#if (s.margin_atas ?? 1) > 0}
                <div style="height: {(s.margin_atas ?? 1) * 8}px;"></div>
              {/if}

              <!-- Logo Toko -->
              {#if s.cetak_logo}
                <div class="text-center mb-2 flex justify-center">
                  {#if s.logo_url}
                    <img
                      src={s.logo_url}
                      alt="Logo Toko"
                      class="max-h-12 max-w-[120px] object-contain grayscale contrast-150"
                    />
                  {:else}
                    <div
                      class="inline-flex items-center justify-center w-10 h-10 rounded-full bg-slate-900 text-white shadow-xs"
                    >
                      <span class="material-symbols-outlined text-[22px]"
                        >{s.logo_icon || "storefront"}</span
                      >
                    </div>
                  {/if}
                </div>
              {/if}

              <!-- Store Header -->
              <div
                class="text-center font-bold text-sm tracking-tight leading-tight uppercase mb-0.5"
              >
                {s.toko_nama || "TOKO ANDA"}
              </div>
              <div
                class="text-center text-[10px] leading-tight text-slate-600 mb-1"
              >
                {s.toko_alamat || "Jl. Pemuda No. 108, Muaro, Sijunjung"}
              </div>
              {#if s.cetak_telepon}
                <div
                  class="text-center text-[10px] font-mono text-slate-600 mb-1"
                >
                  Telp: {s.toko_telepon || "0812-6789-0123"}
                </div>
              {/if}

              {#if s.header_nota}
                <div
                  class="text-center text-[10px] italic text-slate-500 my-1 whitespace-pre-line border-t border-dashed border-slate-300 pt-1"
                >
                  {s.header_nota}
                </div>
              {/if}

              <div class="border-b border-dashed border-slate-400 my-1.5"></div>

              <!-- Metadata Faktur -->
              <div class="text-[10px] space-y-0.5">
                <div class="flex justify-between">
                  <span>No.Nota:</span>
                  <strong class="font-bold">INV/260914-0042</strong>
                </div>
                {#if s.cetak_kasir}
                  <div class="flex justify-between">
                    <span>Kasir: Siti (OP01)</span>
                    <span>14/09/26 14:32</span>
                  </div>
                {/if}
              </div>

              <div class="border-b border-dashed border-slate-400 my-1.5"></div>

              <!-- Sample Items List -->
              <div class="space-y-1 text-[10px]">
                <div class="flex justify-between">
                  <span class="truncate">2x Iced Macchiato</span>
                  <span class="tabular-nums">64.000</span>
                </div>
                <div
                  class="flex justify-between text-slate-500 text-[9px] pl-2"
                >
                  <span>@ 32.000</span>
                </div>

                <div class="flex justify-between">
                  <span class="truncate">1x Artisan Croissant</span>
                  <span class="tabular-nums">22.000</span>
                </div>

                <div class="flex justify-between">
                  <span class="truncate">1x Air Mineral 600ml</span>
                  <span class="tabular-nums">8.000</span>
                </div>
              </div>

              <div class="border-b border-dashed border-slate-400 my-1.5"></div>

              <!-- Totals -->
              <div class="text-[10px] space-y-0.5">
                <div class="flex justify-between">
                  <span>Subtotal</span>
                  <span class="tabular-nums">94.000</span>
                </div>
                {#if s.ppn_aktif}
                  <div class="flex justify-between text-slate-500">
                    <span>PPN ({s.ppn_persen}%)</span>
                    <span class="tabular-nums">10.340</span>
                  </div>
                {/if}
                <div
                  class="flex justify-between font-bold text-xs pt-1 border-t border-slate-300"
                >
                  <span>TOTAL AKHIR</span>
                  <span class="tabular-nums font-black"
                    >{s.ppn_aktif ? "104.340" : "94.000"}</span
                  >
                </div>
                <div class="flex justify-between pt-0.5">
                  <span>Bayar Tunai</span>
                  <span class="tabular-nums">110.000</span>
                </div>
                <div class="flex justify-between">
                  <span>Kembalian</span>
                  <span class="tabular-nums font-bold"
                    >{s.ppn_aktif ? "5.660" : "16.000"}</span
                  >
                </div>
              </div>

              <div class="border-b-2 border-slate-400 my-2"></div>

              <!-- Footer Nota -->
              {#if s.footer_nota}
                <div
                  class="text-center text-[10px] text-slate-600 my-1 whitespace-pre-line leading-tight"
                >
                  {s.footer_nota}
                </div>
              {/if}

              <!-- Barcode Mockup -->
              {#if s.cetak_barcode}
                <div
                  class="text-center mt-2.5 pt-1 border-t border-dashed border-slate-300"
                >
                  <div
                    class="font-mono text-[14px] tracking-widest text-slate-800 leading-none"
                  >
                    ||| | |||| | || |||| | |||
                  </div>
                  <div
                    class="text-[8px] text-slate-500 font-mono tracking-wider mt-0.5"
                  >
                    *INV260914-0042*
                  </div>
                </div>
              {/if}

              <!-- Margin Bawah Feed spacing -->
              {#if (s.margin_bawah ?? 3) > 0}
                <div style="height: {(s.margin_bawah ?? 3) * 8}px;"></div>
              {/if}

              <!-- Cut Line -->
              <div
                class="text-center text-[8px] text-slate-400 border-t border-dotted border-slate-400 pt-1 mt-1"
              >
                - - - - - - [ Gunting Kertas ] - - - - - -
              </div>
            </div>

            <button
              type="button"
              onclick={testPrint}
              class="w-full mt-3 py-2 px-3 bg-primary hover:bg-primary-dark text-white rounded-lg text-xs font-bold shadow-xs cursor-pointer border-none flex items-center justify-center gap-1.5 transition-all"
            >
              <span class="material-symbols-outlined text-[16px]">print</span>
              <span>Test Cetak Preview Struk</span>
            </button>
          </div>
        </div>
      {:else if activeTab === "kasir"}
        <div class="max-w-3xl flex flex-col gap-5">
          <div>
            <h3
              class="font-bold text-slate-900 text-base font-sans flex items-center gap-2"
            >
              <span class="material-symbols-outlined text-primary">tune</span>
              Kebijakan Transaksi Kasir, Pajak &amp; Pembulatan
            </h3>
            <p class="text-slate-500 text-xs mt-0.5">
              Aturan tarif PPN, pembulatan nominal kas, dan proteksi transaksi
              kasir.
            </p>
          </div>

          <div
            class="p-4 bg-slate-50 border border-slate-200 rounded-xl flex items-center justify-between"
          >
            <div>
              <div class="font-bold text-slate-900 text-xs">
                Pajak Pertambahan Nilai (PPN) Kasir
              </div>
              <div class="text-slate-500 text-[11px]">
                Kalkulasi otomatis PPN pada keranjang kasir dan cetak rincian
                pada faktur.
              </div>
            </div>
            <div class="flex items-center gap-3">
              <div class="flex items-center gap-1 font-mono">
                <input
                  type="number"
                  bind:value={s.ppn_persen}
                  class="w-14 px-2 py-1 bg-white border border-slate-300 rounded text-right font-bold text-xs"
                />
                <span class="text-xs font-bold">%</span>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  bind:checked={s.ppn_aktif}
                  class="sr-only peer"
                />
                <div
                  class="w-11 h-6 bg-slate-300 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary"
                ></div>
              </label>
            </div>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div class="p-3 bg-slate-50 border border-slate-200 rounded-lg">
              <div class="font-bold text-slate-900 text-xs mb-1">
                Cegah Stok Minus Saat Kasir
              </div>
              <p class="text-slate-500 text-[11px] mb-2">
                Tolak scan barang di kasir apabila stok fisik di database
                bernilai 0.
              </p>
              <span
                class="px-2 py-0.5 rounded bg-emerald-100 text-emerald-800 font-mono text-[10px] font-bold"
                >AKTIF</span
              >
            </div>
            <div class="p-3 bg-slate-50 border border-slate-200 rounded-lg">
              <div class="font-bold text-slate-900 text-xs mb-1">
                Multi-Tier Harga Grosir Otomatis
              </div>
              <p class="text-slate-500 text-[11px] mb-2">
                Ubah harga satuan otomatis jika kuantiti belanja pelanggan
                mencapai tier 2 atau 3.
              </p>
              <span
                class="px-2 py-0.5 rounded bg-emerald-100 text-emerald-800 font-mono text-[10px] font-bold"
                >AKTIF (5 Level)</span
              >
            </div>
          </div>

          <!-- Bottom Action Simpan Kebijakan Kasir -->
          <div
            class="pt-4 border-t border-slate-200 flex items-center justify-between"
          >
            <span class="text-[11px] text-slate-500 font-medium"
              >Kebijakan tarif PPN dan proteksi stok diterapkan saat itu juga di
              kasir.</span
            >
            <button
              type="button"
              onclick={() => {
                saveSettings();
                showToast("Kebijakan kasir & pajak berhasil disimpan!");
              }}
              class="flex items-center gap-1.5 px-4 py-2 bg-primary hover:bg-primary-dark text-white rounded-lg text-xs font-bold shadow-xs cursor-pointer border-none transition-all"
            >
              <span class="material-symbols-outlined text-[16px]">save</span>
              <span>Simpan Kebijakan Kasir</span>
            </button>
          </div>
        </div>
      {:else if activeTab === "wa"}
        <div class="max-w-3xl flex flex-col gap-5">
          <div>
            <h3
              class="font-bold text-slate-900 text-base font-sans flex items-center gap-2"
            >
              <span class="material-symbols-outlined text-primary">chat</span>
              Notifikasi WhatsApp Otomatis &amp; Sinkronisasi Cloud
            </h3>
            <p class="text-slate-500 text-xs mt-0.5">
              Kirim rekap omset penjualan harian, rekap shift kasir, dan
              notifikasi stok kritis otomatis ke WhatsApp Owner.
            </p>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div>
              <label
                for="set-wa-nomor"
                class="block font-semibold text-slate-700 mb-1 text-xs"
                >Nomor WhatsApp Owner (Tujuan Notif)</label
              >
              <input
                id="set-wa-nomor"
                type="text"
                bind:value={s.wa_notif_nomor}
                placeholder="0812-..."
                class="w-full px-3 py-2 bg-slate-50 border border-slate-300 rounded-lg font-mono text-xs focus:outline-none font-bold"
              />
            </div>
            <div>
              <label
                for="set-wa-jam"
                class="block font-semibold text-slate-700 mb-1 text-xs"
                >Jadwal Kirim Laporan Harian (Pukul)</label
              >
              <input
                id="set-wa-jam"
                type="text"
                bind:value={s.wa_notif_jam}
                placeholder="21:00"
                class="w-full px-3 py-2 bg-slate-50 border border-slate-300 rounded-lg font-mono text-xs focus:outline-none font-bold"
              />
            </div>
          </div>

          <div
            class="p-4 bg-slate-50 border border-slate-200 rounded-xl flex items-center justify-between"
          >
            <div>
              <div class="font-bold text-slate-900 text-xs">
                Integrasi Laporan Cloud Online
              </div>
              <div class="text-slate-500 text-[11px]">
                Kirim rekap encrypted ke dashboard cloud monitoring pemilik
                usaha.
              </div>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                bind:checked={s.cloud_sync_aktif}
                class="sr-only peer"
              />
              <div
                class="w-11 h-6 bg-slate-300 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary"
              ></div>
            </label>
          </div>

          <!-- Bottom Action Simpan WA & Cloud -->
          <div
            class="pt-4 border-t border-slate-200 flex items-center justify-between"
          >
            <span class="text-[11px] text-slate-500 font-medium"
              >Jadwal rekap otomatis dikirim ke WhatsApp owner setiap tutup
              toko.</span
            >
            <button
              type="button"
              onclick={() => {
                saveSettings();
                showToast("Pengaturan WhatsApp & Cloud berhasil disimpan!");
              }}
              class="flex items-center gap-1.5 px-4 py-2 bg-primary hover:bg-primary-dark text-white rounded-lg text-xs font-bold shadow-xs cursor-pointer border-none transition-all"
            >
              <span class="material-symbols-outlined text-[16px]">save</span>
              <span>Simpan Pengaturan WA &amp; Cloud</span>
            </button>
          </div>
        </div>
      {:else if activeTab === "jaringan"}
        <div class="max-w-4xl flex flex-col gap-5">
          <!-- Header & Mode Switcher -->
          <div class="bg-white p-4 sm:p-5 rounded-2xl border border-slate-200 shadow-2xs space-y-4">
            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
              <div>
                <h3 class="font-bold text-slate-900 text-base flex items-center gap-2">
                  <span class="material-symbols-outlined text-primary text-[22px]">storefront</span>
                  <span>Pengaturan Multi-Cabang &amp; Sinkronisasi Data</span>
                </h3>
                <p class="text-slate-500 text-xs mt-0.5">
                  Satu database terpusat untuk seluruh cabang (Wi-Fi lokal maupun luar kota via Cloud).
                </p>
              </div>

              <button
                type="button"
                onclick={() => {
                  loadNetworkConfig();
                  showToast("Data jaringan & cabang diperbarui.");
                }}
                class="flex items-center gap-1.5 px-3 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-800 rounded-lg text-xs font-semibold border border-slate-300 shadow-2xs transition-all cursor-pointer self-start sm:self-auto shrink-0"
              >
                <span
                  class="material-symbols-outlined text-[16px] text-primary {isNetworkLoading
                    ? 'animate-spin'
                    : ''}">sync</span
                >
                <span>Segarkan Data</span>
              </button>
            </div>

            <!-- Switcher Peran Toko: Toko Pusat vs Toko Cabang -->
            <div class="pt-3 border-t border-slate-100">
              <span class="block text-xs font-bold text-slate-700 mb-2">Pilih Peran Komputer Ini:</span>
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <!-- Pilihan 1: Toko Pusat -->
                <button
                  type="button"
                  onclick={() => handleGantiTokoMode("pusat")}
                  disabled={isSwitchingMode}
                  class="p-3.5 rounded-xl border text-left cursor-pointer transition-all flex items-start gap-3 {tokoMode === 'pusat'
                    ? 'border-primary bg-primary/5 ring-2 ring-primary/20'
                    : 'border-slate-200 bg-white hover:border-slate-300'}"
                >
                  <div class="w-9 h-9 rounded-lg flex items-center justify-center shrink-0 {tokoMode === 'pusat' ? 'bg-primary text-white' : 'bg-slate-100 text-slate-500'}">
                    <span class="material-symbols-outlined text-[20px]">domain</span>
                  </div>
                  <div class="space-y-0.5">
                    <div class="flex items-center gap-2">
                      <span class="font-bold text-xs text-slate-900">🏢 Toko Pusat (Headquarters)</span>
                      {#if tokoMode === "pusat"}
                        <span class="px-1.5 py-0.2 rounded text-[9px] font-bold bg-primary text-white">AKTIF</span>
                      {/if}
                    </div>
                    <p class="text-[11px] text-slate-500 leading-snug">
                      Pusat master produk, harga, dan pendaftaran cabang. Hanya Pusat yang berhak menambah cabang baru.
                    </p>
                  </div>
                </button>

                <!-- Pilihan 2: Toko Cabang -->
                <button
                  type="button"
                  onclick={() => handleGantiTokoMode("cabang")}
                  disabled={isSwitchingMode}
                  class="p-3.5 rounded-xl border text-left cursor-pointer transition-all flex items-start gap-3 {tokoMode === 'cabang'
                    ? 'border-emerald-600 bg-emerald-50/40 ring-2 ring-emerald-500/20'
                    : 'border-slate-200 bg-white hover:border-slate-300'}"
                >
                  <div class="w-9 h-9 rounded-lg flex items-center justify-center shrink-0 {tokoMode === 'cabang' ? 'bg-emerald-600 text-white' : 'bg-slate-100 text-slate-500'}">
                    <span class="material-symbols-outlined text-[20px]">store</span>
                  </div>
                  <div class="space-y-0.5">
                    <div class="flex items-center gap-2">
                      <span class="font-bold text-xs text-slate-900">🏪 Toko Cabang (Outlet)</span>
                      {#if tokoMode === "cabang"}
                        <span class="px-1.5 py-0.2 rounded text-[9px] font-bold bg-emerald-600 text-white">AKTIF</span>
                      {/if}
                    </div>
                    <p class="text-[11px] text-slate-500 leading-snug">
                      Mengambil katalog produk dari Toko Pusat. Kasir tetap bisa jualan lancar saat offline (tanpa internet).
                    </p>
                  </div>
                </button>
              </div>
            </div>
          </div>

          <!-- TAMPILAN JIKA MODE TOKO PUSAT -->
          {#if tokoMode === "pusat"}
            <!-- Kartu 1: Daftar Cabang (Hanya Pusat yang menambah cabang) -->
            <div class="p-4 bg-white border border-slate-200 rounded-2xl shadow-2xs space-y-3">
              <div class="flex items-center justify-between">
                <div>
                  <div class="font-bold text-slate-900 text-xs flex items-center gap-2">
                    <span class="material-symbols-outlined text-primary text-[18px]">store</span>
                    <span>Daftar Cabang Toko Anda</span>
                    <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-primary/10 text-primary">
                      {networkConfig?.daftar_cabang?.length || 1} Cabang
                    </span>
                  </div>
                  <p class="text-[11px] text-slate-500 mt-0.5">
                    Hanya Toko Pusat yang dapat mendaftarkan cabang baru agar struktur database tetap teratur dan seragam.
                  </p>
                </div>

                <button
                  type="button"
                  onclick={() => (isAddCabangOpen = true)}
                  class="flex items-center gap-1.5 px-3 py-1.5 bg-primary hover:bg-primary-dark text-white rounded-lg text-xs font-bold shadow-xs cursor-pointer border-none transition-all"
                >
                  <span class="material-symbols-outlined text-[16px]">add</span>
                  <span>+ Tambah Cabang Baru</span>
                </button>
              </div>

              <!-- Tabel Cabang -->
              <div class="border border-slate-200 rounded-xl overflow-hidden shadow-2xs">
                <table class="w-full text-left border-collapse text-xs">
                  <thead>
                    <tr class="bg-slate-100 border-b border-slate-200 text-slate-700 font-bold uppercase tracking-wider text-[10px]">
                      <th class="py-2.5 px-3">Kode</th>
                      <th class="py-2.5 px-3">Nama Cabang</th>
                      <th class="py-2.5 px-3">Alamat / Lokasi</th>
                      <th class="py-2.5 px-3">Telepon</th>
                      <th class="py-2.5 px-3 text-center">Tipe</th>
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-slate-100">
                    {#if !networkConfig?.daftar_cabang || networkConfig.daftar_cabang.length === 0}
                      <tr>
                        <td colspan="5" class="py-4 text-center text-slate-400 text-xs">
                          Belum ada cabang terdaftar.
                        </td>
                      </tr>
                    {:else}
                      {#each networkConfig.daftar_cabang as c}
                        <tr class="hover:bg-slate-50/80 transition-all {c.is_pusat ? 'bg-primary/5 font-semibold' : ''}">
                          <td class="py-2.5 px-3 font-mono text-[11px] font-bold text-slate-900">
                            {c.kode}
                          </td>
                          <td class="py-2.5 px-3 text-slate-900 font-medium">
                            {c.nama}
                          </td>
                          <td class="py-2.5 px-3 text-slate-500 text-[11px]">
                            {c.alamat || "-"}
                          </td>
                          <td class="py-2.5 px-3 text-slate-500 text-[11px] font-mono">
                            {c.telepon || "-"}
                          </td>
                          <td class="py-2.5 px-3 text-center">
                            {#if c.is_pusat}
                              <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-primary text-white">
                                TOKO PUSAT
                              </span>
                            {:else}
                              <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-slate-200 text-slate-700">
                                CABANG
                              </span>
                            {/if}
                          </td>
                        </tr>
                      {/each}
                    {/if}
                  </tbody>
                </table>
              </div>
            </div>

            <!-- Kartu 2: Hubungkan Cabang Di Luar Jaringan (Cloud Supabase) -->
            <div class="p-4 bg-white border border-slate-200 rounded-2xl shadow-2xs space-y-3">
              <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                <div class="flex items-center gap-2.5">
                  <div class="w-8 h-8 rounded-lg bg-sky-600 text-white flex items-center justify-center shrink-0">
                    <span class="material-symbols-outlined text-[18px]">cloud_sync</span>
                  </div>
                  <div>
                    <div class="font-bold text-slate-900 text-xs flex items-center gap-2">
                      <span>Jembatan Cloud Supabase (Cabang Luar Jaringan / Beda Kota)</span>
                      <span class="px-2 py-0.5 rounded text-[10px] font-bold font-mono {supabaseConfig.is_bound ? 'bg-emerald-100 text-emerald-800' : 'bg-slate-200 text-slate-600'}">
                        {supabaseConfig.is_bound ? "TERHUBUNG" : "BELUM AKTIF"}
                      </span>
                    </div>
                    <p class="text-[11px] text-slate-500 mt-0.5">
                      Untuk cabang di ruko lain atau beda kota, aktifkan Cloud agar cabang bisa mengambil master produk Pusat.
                    </p>
                  </div>
                </div>

                <div class="flex items-center gap-2">
                  <button
                    type="button"
                    onclick={() => {
                      activeTab = "supabase";
                      loadSupabaseConfig();
                    }}
                    class="px-3 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg text-xs font-bold cursor-pointer border border-slate-300"
                  >
                    Atur Cloud
                  </button>
                  <button
                    type="button"
                    onclick={handleSyncSupabaseNow}
                    disabled={isSyncingSupabase || !supabaseConfig.is_bound}
                    class="px-3 py-1.5 bg-sky-600 hover:bg-sky-700 disabled:opacity-50 text-white rounded-lg text-xs font-bold cursor-pointer border-none shadow-xs flex items-center gap-1.5"
                  >
                    {#if isSyncingSupabase}
                      <span class="material-symbols-outlined text-[14px] animate-spin">progress_activity</span>
                      <span>Mengirim...</span>
                    {:else}
                      <span class="material-symbols-outlined text-[14px]">cloud_upload</span>
                      <span>Kirim Master Produk ke Cloud</span>
                    {/if}
                  </button>
                </div>
              </div>
            </div>

            <!-- Kartu 3: Jaringan Wi-Fi Toko (Satu Gedung) -->
            <div class="p-4 bg-white border border-slate-200 rounded-2xl shadow-2xs space-y-3">
              <div class="flex items-center justify-between">
                <div>
                  <div class="font-bold text-slate-900 text-xs flex items-center gap-2">
                    <span class="material-symbols-outlined text-primary text-[18px]">wifi</span>
                    <span>Jaringan Wi-Fi Toko (Satu Gedung)</span>
                  </div>
                  <p class="text-[11px] text-slate-500 mt-0.5">
                    Kasir cabang di ruangan/gedung yang sama dapat terhubung langsung ke komputer Server ini.
                  </p>
                </div>
                <div class="px-2.5 py-1 rounded-lg bg-slate-100 border border-slate-200 font-mono text-xs font-bold text-slate-800">
                  IP Server Ini: {networkConfig?.ip_address || "192.168.100.75"}:7890
                </div>
              </div>

              {#if discoveredDevices.length > 0}
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 pt-2 border-t border-slate-100">
                  {#each discoveredDevices as dev}
                    <div class="p-2.5 bg-slate-50 rounded-lg border border-slate-200 text-xs flex items-center justify-between">
                      <div class="flex items-center gap-2">
                        <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
                        <span class="font-bold text-slate-800">{dev.device_nama}</span>
                        <span class="text-slate-400 font-mono text-[10px]">({dev.ip_address})</span>
                      </div>
                      <span class="px-1.5 py-0.2 rounded text-[9px] font-bold bg-blue-100 text-blue-800 uppercase">
                        {dev.role}
                      </span>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>

          <!-- TAMPILAN JIKA MODE TOKO CABANG -->
          {:else}
            <!-- Banner Penjelasan Toko Cabang -->
            <div class="p-4 bg-emerald-50/80 border border-emerald-200 rounded-2xl flex items-start gap-3 shadow-2xs">
              <span class="material-symbols-outlined text-[24px] text-emerald-700 shrink-0 mt-0.5">verified_user</span>
              <div class="space-y-1 text-xs text-emerald-950">
                <div class="font-bold text-emerald-900 text-sm">Mode Toko Cabang Aktif</div>
                <p class="text-emerald-800 text-[11px] leading-relaxed">
                  Komputer ini menggunakan <strong>database master dari Toko Pusat</strong>. Semua katalog produk, kategori, dan harga diatur terpusat agar tidak campur aduk. Toko Cabang tidak bisa membuat cabang baru, melainkan hanya bertugas menerima data dan bertransaksi.
                </p>
              </div>
            </div>

            <!-- Langkah 1: Pilih Identitas Cabang Ini -->
            <div class="p-4 bg-white border border-slate-200 rounded-2xl shadow-2xs space-y-3">
              <div class="flex items-center justify-between">
                <div>
                  <div class="font-bold text-slate-900 text-xs flex items-center gap-2">
                    <span class="w-5 h-5 rounded-full bg-emerald-600 text-white text-[11px] font-bold flex items-center justify-center">1</span>
                    <span>Identitas Cabang Ini</span>
                  </div>
                  <p class="text-[11px] text-slate-500 mt-0.5">
                    Pilih nama cabang yang telah didaftarkan oleh Toko Pusat.
                  </p>
                </div>
                <button
                  type="button"
                  onclick={handleMuatCabangCloud}
                  disabled={isLoadingCloudCabang}
                  class="flex items-center gap-1 px-2.5 py-1 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg text-xs font-semibold border border-slate-300 cursor-pointer"
                >
                  <span class="material-symbols-outlined text-[14px] {isLoadingCloudCabang ? 'animate-spin' : ''}">refresh</span>
                  <span>Muat Cabang dari Cloud</span>
                </button>
              </div>

              <div class="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 pt-1">
                <select
                  value={selectedCabangId}
                  onchange={(e) => handlePilihCabang((e.target as HTMLSelectElement).value)}
                  class="flex-1 px-3 py-2 border border-slate-300 rounded-lg text-xs font-medium bg-white focus:ring-1 focus:ring-emerald-500 outline-hidden"
                >
                  {#if networkConfig?.daftar_cabang}
                    {#each networkConfig.daftar_cabang as c}
                      <option value={c.id}>
                        {c.nama} ({c.kode}) {c.is_pusat ? "- Toko Pusat" : ""}
                      </option>
                    {/each}
                  {/if}
                  {#if cloudCabangList.length > 0}
                    {#each cloudCabangList as cc}
                      {#if !networkConfig?.daftar_cabang?.some((x) => x.id === cc.id)}
                        <option value={cc.id}>
                          {cc.nama} ({cc.kode}) - Dari Cloud
                        </option>
                      {/if}
                    {/each}
                  {/if}
                </select>

                <div class="px-3 py-2 bg-emerald-50 border border-emerald-200 rounded-lg text-emerald-800 font-bold text-xs shrink-0 flex items-center gap-1.5">
                  <span class="material-symbols-outlined text-[16px]">check_circle</span>
                  <span>Cabang Terpilih: {networkConfig?.cabang_nama || "Pusat"}</span>
                </div>
              </div>
            </div>

            <!-- Langkah 2: Ambil & Sinkronkan Data Produk dari Pusat -->
            <div class="p-4 bg-white border border-slate-200 rounded-2xl shadow-2xs space-y-3">
              <div>
                <div class="font-bold text-slate-900 text-xs flex items-center gap-2">
                  <span class="w-5 h-5 rounded-full bg-emerald-600 text-white text-[11px] font-bold flex items-center justify-center">2</span>
                  <span>Tarik Data Master Produk dari Pusat</span>
                </div>
                <p class="text-[11px] text-slate-500 mt-0.5">
                  Pilih cara menghubungkan toko ini ke Pusat (Wi-Fi lokal jika satu gedung, atau Cloud jika beda kota/ruko).
                </p>
              </div>

              <!-- Pilihan Jalur Koneksi: Wi-Fi vs Cloud -->
              <div class="flex border-b border-slate-200 text-xs">
                <button
                  type="button"
                  onclick={() => (cabangSyncTab = "wifi")}
                  class="px-4 py-2 font-bold cursor-pointer border-b-2 flex items-center gap-1.5 {cabangSyncTab === 'wifi' ? 'border-primary text-primary' : 'border-transparent text-slate-500 hover:text-slate-800'}"
                >
                  <span class="material-symbols-outlined text-[16px]">wifi</span>
                  <span>📶 Melalui Wi-Fi Toko (Satu Gedung)</span>
                </button>

                <button
                  type="button"
                  onclick={() => (cabangSyncTab = "cloud")}
                  class="px-4 py-2 font-bold cursor-pointer border-b-2 flex items-center gap-1.5 {cabangSyncTab === 'cloud' ? 'border-sky-600 text-sky-600' : 'border-transparent text-slate-500 hover:text-slate-800'}"
                >
                  <span class="material-symbols-outlined text-[16px]">cloud_sync</span>
                  <span>☁️ Melalui Cloud Internet (Luar Kota / Beda Lokasi)</span>
                </button>
              </div>

              {#if cabangSyncTab === "wifi"}
                <div class="p-3 bg-slate-50 rounded-xl space-y-3 text-xs">
                  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
                    <div class="space-y-1">
                      <span class="font-bold text-slate-700">Alamat IP Komputer Server Pusat:</span>
                      <input
                        type="text"
                        bind:value={manualServerIp}
                        placeholder="Contoh: 192.168.100.75"
                        class="w-full sm:w-64 px-3 py-1.5 border border-slate-300 rounded-lg text-xs font-mono bg-white outline-hidden"
                      />
                    </div>

                    <button
                      type="button"
                      onclick={() => handleTarikMasterLan(manualServerIp)}
                      disabled={isPullingMaster}
                      class="px-4 py-2 bg-primary hover:bg-primary-dark disabled:opacity-50 text-white rounded-lg font-bold text-xs cursor-pointer border-none shadow-xs flex items-center gap-1.5 self-start sm:self-end"
                    >
                      {#if isPullingMaster}
                        <span class="material-symbols-outlined text-[16px] animate-spin">progress_activity</span>
                        <span>Menarik Data...</span>
                      {:else}
                        <span class="material-symbols-outlined text-[16px]">download</span>
                        <span>Tarik Data via Wi-Fi</span>
                      {/if}
                    </button>
                  </div>
                </div>
              {:else}
                <div class="p-3 bg-sky-50/60 border border-sky-200 rounded-xl space-y-3 text-xs">
                  <div class="flex items-center justify-between">
                    <div>
                      <span class="font-bold text-sky-900 block">Koneksi Cloud Supabase:</span>
                      <span class="text-[11px] text-sky-700">
                        {supabaseConfig.is_bound ? `Terhubung ke Cloud (${supabaseConfig.url})` : "Belum terhubung ke Cloud. Silakan atur di tab Cloud Supabase."}
                      </span>
                    </div>

                    <button
                      type="button"
                      onclick={handleTarikMasterCloud}
                      disabled={isPullingMaster || !supabaseConfig.is_bound}
                      class="px-4 py-2 bg-sky-600 hover:bg-sky-700 disabled:opacity-50 text-white rounded-lg font-bold text-xs cursor-pointer border-none shadow-xs flex items-center gap-1.5"
                    >
                      {#if isPullingMaster}
                        <span class="material-symbols-outlined text-[16px] animate-spin">progress_activity</span>
                        <span>Menarik Data dari Cloud...</span>
                      {:else}
                        <span class="material-symbols-outlined text-[16px]">cloud_download</span>
                        <span>📥 Tarik &amp; Sinkronkan Data dari Cloud</span>
                      {/if}
                    </button>
                  </div>
                </div>
              {/if}
            </div>

            <!-- Langkah 3: Status Database Lokal & Siap Jualan Offline -->
            <div class="p-4 bg-white border border-slate-200 rounded-2xl shadow-2xs space-y-3">
              <div class="font-bold text-slate-900 text-xs flex items-center gap-2">
                <span class="w-5 h-5 rounded-full bg-emerald-600 text-white text-[11px] font-bold flex items-center justify-center">3</span>
                <span>Status Database Lokal (100% Offline-Ready)</span>
              </div>

              <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div class="p-3 bg-slate-50 border border-slate-200 rounded-xl flex items-center gap-3">
                  <div class="w-10 h-10 rounded-lg bg-emerald-100 text-emerald-800 flex items-center justify-center shrink-0">
                    <span class="material-symbols-outlined text-[22px]">inventory_2</span>
                  </div>
                  <div>
                    <div class="font-bold text-slate-900 text-sm">
                      {networkConfig?.total_produk_lokal || 0} Produk
                    </div>
                    <p class="text-[11px] text-slate-500">Tersimpan di database komputer ini</p>
                  </div>
                </div>

                <div class="p-3 bg-emerald-50 border border-emerald-200 rounded-xl flex items-center gap-3">
                  <div class="w-10 h-10 rounded-lg bg-emerald-600 text-white flex items-center justify-center shrink-0">
                    <span class="material-symbols-outlined text-[22px]">wifi_off</span>
                  </div>
                  <div>
                    <div class="font-bold text-emerald-900 text-sm">Siap Transaksi Offline</div>
                    <p class="text-[11px] text-emerald-700">Kasir tetap lancar mencetak struk walau internet mati</p>
                  </div>
                </div>
              </div>

              <div class="pt-2 border-t border-slate-100 flex items-center justify-between">
                <span class="text-[11px] text-slate-500 font-medium">
                  Kirim transaksi yang belum tersinkronkan ke Toko Pusat:
                </span>
                <button
                  type="button"
                  onclick={handleSyncSupabaseNow}
                  disabled={isSyncingSupabase}
                  class="px-3.5 py-1.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg text-xs font-bold cursor-pointer border-none shadow-xs flex items-center gap-1.5"
                >
                  <span class="material-symbols-outlined text-[15px]">sync</span>
                  <span>Kirim Transaksi ke Pusat Sekarang</span>
                </button>
              </div>
            </div>
          {/if}
        </div>
      {:else if activeTab === "supabase"}
        <div class="max-w-4xl flex flex-col gap-5">
          <!-- Header -->
          <div class="flex items-center justify-between">
            <div>
              <h3
                class="font-bold text-slate-900 text-base font-sans flex items-center gap-2"
              >
                <span class="material-symbols-outlined text-sky-600">cloud_sync</span>
                Integrasi Cloud Supabase (BYO-Cloud Multi-Cabang)
              </h3>
              <p class="text-slate-500 text-xs mt-0.5">
                Hubungkan database PostgreSQL Supabase Anda sendiri untuk sinkronisasi transaksi, master produk, pelanggan, dan shift kasir antar cabang secara online.
              </p>
            </div>

            <div class="flex items-center gap-2">
              <span
                class="px-2.5 py-1 rounded-lg text-xs font-bold font-mono {supabaseConfig.is_bound
                  ? 'bg-emerald-100 text-emerald-800 border border-emerald-300'
                  : 'bg-slate-200 text-slate-600'}"
              >
                {supabaseConfig.is_bound ? "TERHUBUNG KE CLOUD" : "BELUM DITAUTKAN"}
              </span>

              <button
                type="button"
                onclick={async () => {
                  await loadSupabaseConfig();
                  showToast("Data Supabase diperbarui.");
                }}
                class="flex items-center gap-1.5 px-3 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-800 rounded-lg text-xs font-semibold border border-slate-300 shadow-2xs transition-all cursor-pointer"
              >
                <span class="material-symbols-outlined text-[16px] text-sky-600">sync</span>
                <span>Segarkan</span>
              </button>
            </div>
          </div>

          <!-- Branch Identity & Sync Status Banner -->
          <div class="p-4 bg-gradient-to-r from-sky-50 to-indigo-50 border border-sky-200 rounded-xl flex flex-wrap items-center justify-between gap-3 shadow-2xs">
            <div class="space-y-1">
              <div class="text-[11px] text-sky-800 font-bold uppercase tracking-wider flex items-center gap-1.5">
                <span class="material-symbols-outlined text-[15px]">store</span>
                <span>Cabang Aktif Pengirim Sinkronisasi</span>
              </div>
              <div class="flex items-center gap-2 text-xs font-bold text-slate-900">
                <span>{networkConfig?.cabang_nama || "Toko Utama"}</span>
                <span class="px-2 py-0.5 bg-sky-200/80 text-sky-900 rounded font-mono text-[11px]">Kode: {networkConfig?.cabang_kode || "CAB-01"}</span>
                <span class="px-2 py-0.5 bg-indigo-200/80 text-indigo-900 rounded font-mono text-[11px]">Perangkat: {networkConfig?.device_nama || "Kasir Utama"} ({networkConfig?.device_role || "server"})</span>
              </div>
            </div>

            <div class="flex items-center gap-4">
              <div class="text-right">
                <div class="text-[11px] text-slate-500 font-medium">Antrean Menunggu Sync:</div>
                <div class="text-base font-black font-mono {supabaseConfig.pending_count > 0 ? 'text-amber-600' : 'text-emerald-600'}">
                  {supabaseConfig.pending_count} record
                </div>
              </div>

              {#if supabaseConfig.is_bound}
                <button
                  type="button"
                  onclick={handleSyncSupabaseNow}
                  disabled={isSyncingSupabase}
                  class="px-3.5 py-2 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg text-xs font-bold shadow-xs flex items-center gap-1.5 cursor-pointer border-none transition-all disabled:opacity-50"
                >
                  <span class="material-symbols-outlined text-[16px] {isSyncingSupabase ? 'animate-spin' : ''}">sync</span>
                  <span>{isSyncingSupabase ? "Menyinkronkan..." : "Sync Sekarang"}</span>
                </button>
              {/if}
            </div>
          </div>

          <!-- Card 1: Form Kredensial URL & Key -->
          <div class="bg-white border border-slate-200 rounded-xl p-5 space-y-4 shadow-2xs">
            <div class="flex items-center justify-between pb-2 border-b border-slate-100">
              <div class="font-bold text-slate-800 text-xs flex items-center gap-1.5">
                <span class="material-symbols-outlined text-[16px] text-primary">key</span>
                <span>Kredensial Akses Project Supabase</span>
              </div>
              <span class="text-[11px] text-slate-400">Bring Your Own Cloud (BYO-Cloud)</span>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <label for="supa-url" class="block text-slate-700 text-xs font-bold mb-1">
                  Supabase Project URL <span class="text-red-500">*</span>
                </label>
                <input
                  id="supa-url"
                  type="text"
                  bind:value={supabaseConfig.url}
                  placeholder="https://xxxxxxxxxxxxxxxx.supabase.co"
                  class="w-full px-3 py-2 bg-slate-50 border border-slate-300 rounded-lg text-xs font-mono focus:bg-white focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary"
                />
                <span class="text-[10px] text-slate-500 mt-1 block">
                  Buka Supabase Dashboard &gt; <strong>Project Settings &gt; Data API &gt; Project URL</strong>
                </span>
              </div>

              <div>
                <label for="supa-key" class="block text-slate-700 text-xs font-bold mb-1">
                  Supabase API Key (service_role atau anon) <span class="text-red-500">*</span>
                </label>
                <div class="relative">
                  <input
                    id="supa-key"
                    type={showApiKey ? "text" : "password"}
                    bind:value={supabaseConfig.api_key}
                    placeholder="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
                    class="w-full px-3 py-2 pr-9 bg-slate-50 border border-slate-300 rounded-lg text-xs font-mono focus:bg-white focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary"
                  />
                  <button
                    type="button"
                    onclick={() => (showApiKey = !showApiKey)}
                    class="absolute right-2 top-2 text-slate-400 hover:text-slate-600 cursor-pointer border-none bg-transparent"
                    title={showApiKey ? "Sembunyikan API Key" : "Tampilkan API Key"}
                  >
                    <span class="material-symbols-outlined text-[16px]">{showApiKey ? "visibility_off" : "visibility"}</span>
                  </button>
                </div>
                <span class="text-[10px] text-slate-500 mt-1 block">
                  Disarankan menggunakan <strong>service_role secret key</strong> agar sinkronisasi tidak terhalang Row Level Security.
                </span>
              </div>
            </div>

            <!-- Auto-Sync Configuration -->
            <div class="p-3 bg-slate-50 border border-slate-200 rounded-lg flex flex-wrap items-center justify-between gap-3">
              <div class="flex items-center gap-4">
                <label class="flex items-center gap-2 cursor-pointer text-xs font-bold text-slate-800 select-none">
                  <input
                    type="checkbox"
                    bind:checked={supabaseConfig.auto_sync}
                    class="rounded text-primary focus:ring-primary w-4 h-4"
                  />
                  <span>Aktifkan Sinkronisasi Otomatis di Latar Belakang</span>
                </label>

                {#if supabaseConfig.auto_sync}
                  <div class="flex items-center gap-1.5 text-xs text-slate-700 font-medium">
                    <span>Setiap:</span>
                    <input
                      type="number"
                      min="1"
                      max="60"
                      bind:value={supabaseConfig.interval_menit}
                      class="w-14 px-2 py-1 border border-slate-300 rounded text-center font-bold text-xs bg-white"
                    />
                    <span>menit</span>
                  </div>
                {/if}
              </div>

              <!-- Action Buttons -->
              <div class="flex items-center gap-2">
                <button
                  type="button"
                  onclick={handleTestSupabase}
                  disabled={isTestingSupabase}
                  class="px-3 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-800 rounded-lg text-xs font-bold border border-slate-300 flex items-center gap-1 cursor-pointer transition-all disabled:opacity-50"
                >
                  <span class="material-symbols-outlined text-[15px] {isTestingSupabase ? 'animate-spin' : ''}">cable</span>
                  <span>{isTestingSupabase ? "Menghubungkan..." : "Uji Koneksi"}</span>
                </button>

                <button
                  type="button"
                  onclick={handleSaveSupabase}
                  disabled={isSavingSupabase}
                  class="px-3.5 py-1.5 bg-primary hover:bg-primary-dark text-white rounded-lg text-xs font-bold shadow-xs flex items-center gap-1 cursor-pointer border-none transition-all disabled:opacity-50"
                >
                  <span class="material-symbols-outlined text-[15px]">save</span>
                  <span>Simpan &amp; Tautkan</span>
                </button>
              </div>
            </div>

            <!-- Test Result Banner -->
            {#if supabaseTestResult}
              <div
                class="p-3 rounded-lg text-xs font-medium flex items-center gap-2 {supabaseTestResult.sukses
                  ? 'bg-emerald-50 text-emerald-800 border border-emerald-200'
                  : 'bg-rose-50 text-rose-800 border border-rose-200'}"
              >
                <span class="material-symbols-outlined text-[18px] shrink-0">
                  {supabaseTestResult.sukses ? "check_circle" : "error"}
                </span>
                <span>{supabaseTestResult.pesan}</span>
              </div>
            {/if}
          </div>

          <!-- Card 2: Panduan Setup Skema Database (DDL) -->
          <div class="p-4 bg-slate-50 border border-slate-200 rounded-xl space-y-3 shadow-2xs">
            <div class="flex items-center justify-between">
              <div>
                <div class="font-bold text-slate-800 text-xs flex items-center gap-1.5">
                  <span class="material-symbols-outlined text-[16px] text-sky-600">terminal</span>
                  <span>Setup Tabel Database Supabase (8 Tabel)</span>
                </div>
                <div class="text-slate-500 text-[10px]">
                  FAZPOS memerlukan 8 tabel PostgreSQL: cabang, device, dbarang, dpelanggan, tpenjualan, tpenjualandetail, tshift, keluarmasuk.
                </div>
              </div>

              <button
                type="button"
                onclick={copySupabaseDdl}
                class="px-3 py-1.5 bg-sky-100 hover:bg-sky-200 text-sky-900 rounded-lg text-xs font-bold border border-sky-300 flex items-center gap-1 cursor-pointer transition-all shadow-2xs"
              >
                <span class="material-symbols-outlined text-[15px]">content_copy</span>
                <span>Salin Skema SQL (DDL)</span>
              </button>
            </div>

            <div class="p-3 bg-white border border-slate-200 rounded-lg text-xs text-slate-600 space-y-1.5 font-sans">
              <div class="font-bold text-slate-800">Petunjuk Menyiapkan Supabase Baru:</div>
              <ol class="list-decimal list-inside space-y-1 text-[11px] text-slate-600">
                <li>Buka project Supabase Anda di browser, masuk ke menu <strong>SQL Editor</strong>.</li>
                <li>Klik tombol <strong>"Salin Skema SQL (DDL)"</strong> di atas, lalu tempel (*paste*) ke SQL Editor Supabase.</li>
                <li>Klik <strong>Run</strong> untuk membuat 8 tabel sinkronisasi beserta indeks performanya.</li>
                <li>Salin <strong>Project URL</strong> dan <strong>service_role Key</strong> dari Project Settings &gt; API ke form di atas, lalu klik <strong>Simpan &amp; Tautkan</strong>.</li>
              </ol>
            </div>
          </div>

          <!-- Card 3: Riwayat Log Sinkronisasi -->
          <div class="p-4 bg-slate-50 border border-slate-200 rounded-xl space-y-3 shadow-2xs">
            <div class="flex items-center justify-between">
              <div class="font-bold text-slate-800 text-xs flex items-center gap-1.5">
                <span class="material-symbols-outlined text-[16px] text-slate-600">history</span>
                <span>Riwayat Sinkronisasi Terakhir (Sync Log)</span>
              </div>
              <span class="text-[10px] text-slate-400">
                {syncLogs.length > 0 ? `Menampilkan ${syncLogs.length} entri riwayat` : "Belum ada riwayat"}
              </span>
            </div>

            {#if syncLogs.length === 0}
              <div class="p-6 bg-white border border-slate-200 rounded-xl text-center text-slate-400 text-xs">
                Belum ada log sinkronisasi. Klik tombol <strong>"Sync Sekarang"</strong> di atas untuk memulai sinkronisasi pertama.
              </div>
            {:else}
              <div class="bg-white border border-slate-200 rounded-xl overflow-hidden shadow-2xs">
                <table class="w-full text-left border-collapse text-xs">
                  <thead>
                    <tr class="bg-slate-100 border-b border-slate-200 text-slate-700 font-bold uppercase tracking-wider text-[10px]">
                      <th class="py-2 px-3">Waktu</th>
                      <th class="py-2 px-3">Tabel</th>
                      <th class="py-2 px-3 text-center">Jumlah Record</th>
                      <th class="py-2 px-3 text-center">Status</th>
                      <th class="py-2 px-3">Keterangan / Durasi</th>
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-slate-100">
                    {#each syncLogs as log}
                      <tr class="hover:bg-slate-50">
                        <td class="py-1.5 px-3 font-mono text-[11px] text-slate-600">
                          {new Date(log.waktu).toLocaleString("id-ID")}
                        </td>
                        <td class="py-1.5 px-3 font-mono font-bold text-slate-800">
                          {log.tabel}
                        </td>
                        <td class="py-1.5 px-3 text-center font-bold font-mono">
                          {log.jumlah_record}
                        </td>
                        <td class="py-1.5 px-3 text-center">
                          <span
                            class="px-2 py-0.5 rounded text-[9px] font-bold uppercase {log.status === 'sukses'
                              ? 'bg-emerald-100 text-emerald-800 border border-emerald-200'
                              : 'bg-rose-100 text-rose-800 border border-rose-200'}"
                          >
                            {log.status}
                          </span>
                        </td>
                        <td class="py-1.5 px-3 text-slate-600 text-[11px] truncate max-w-xs font-mono">
                          {log.pesan || (log.durasi_ms ? `${log.durasi_ms} ms` : "-")}
                        </td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            {/if}
          </div>
        </div>
      {:else if activeTab === "database"}
        <div class="max-w-4xl flex flex-col gap-5">
          <!-- Header -->
          <div class="flex items-center justify-between">
            <div>
              <h3
                class="font-bold text-slate-900 text-base font-sans flex items-center gap-2"
              >
                <span class="material-symbols-outlined text-primary"
                  >database</span
                >
                Pemeliharaan Database SQLite &amp; Riwayat Backup Atomik
              </h3>
              <p class="text-slate-500 text-xs mt-0.5">
                Mekanisme snapshot atomik online (VACUUM INTO), rotasi otomatis
                10 file, dan audit integritas fisik.
              </p>
            </div>

            <!-- Actions Bar -->
            <div class="flex items-center gap-2">
              <button
                type="button"
                onclick={handleCheckIntegrity}
                disabled={isCheckingIntegrity}
                class="flex items-center gap-1 px-3 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-800 rounded-lg text-xs font-semibold border border-slate-300 shadow-2xs transition-all cursor-pointer"
                title="Jalankan PRAGMA integrity_check"
              >
                <span
                  class="material-symbols-outlined text-[16px] text-emerald-600 {isCheckingIntegrity
                    ? 'animate-spin'
                    : ''}">verified</span
                >
                <span
                  >{isCheckingIntegrity
                    ? "Memeriksa..."
                    : "Cek Integritas DB"}</span
                >
              </button>

              <button
                type="button"
                onclick={handleBackupNow}
                disabled={isBackingUp}
                class="flex items-center gap-1.5 px-3.5 py-1.5 bg-primary hover:bg-primary-dark text-white rounded-lg text-xs font-bold shadow-2xs cursor-pointer border-none transition-all"
                title="Jalankan VACUUM INTO ke folder backups/"
              >
                <span
                  class="material-symbols-outlined text-[16px] {isBackingUp
                    ? 'animate-spin'
                    : ''}">backup</span
                >
                <span
                  >{isBackingUp ? "Membuat Backup..." : "Backup Sekarang"}</span
                >
              </button>
            </div>
          </div>

          <!-- Status Box -->
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
            <div class="p-3.5 bg-slate-50 border border-slate-200 rounded-xl">
              <div
                class="flex items-center justify-between text-[11px] text-slate-500 font-semibold mb-1"
              >
                <span>Status Engine Database</span>
                <span
                  class="px-1.5 py-0.2 rounded bg-emerald-100 text-emerald-800 font-bold font-mono text-[9px]"
                  >ONLINE</span
                >
              </div>
              <div class="font-mono font-bold text-slate-900 text-sm">
                pos_data.db
              </div>
              <div class="text-[10px] text-slate-500 font-mono mt-1">
                SQLite WAL Mode • Synchronous Normal
              </div>
            </div>

            <div class="p-3.5 bg-slate-50 border border-slate-200 rounded-xl">
              <div
                class="flex items-center justify-between text-[11px] text-slate-500 font-semibold mb-1"
              >
                <span>Integritas Fisik (B-Tree)</span>
                {#if integrityStatus === "ok"}
                  <span
                    class="px-1.5 py-0.2 rounded bg-emerald-100 text-emerald-800 font-bold font-mono text-[9px]"
                    >NORMAL (OK)</span
                  >
                {:else if integrityStatus === "error"}
                  <span
                    class="px-1.5 py-0.2 rounded bg-red-100 text-red-800 font-bold font-mono text-[9px]"
                    >ERROR</span
                  >
                {:else}
                  <span
                    class="px-1.5 py-0.2 rounded bg-slate-200 text-slate-700 font-bold font-mono text-[9px]"
                    >TERVERIFIKASI</span
                  >
                {/if}
              </div>
              <div
                class="font-semibold text-slate-900 text-sm flex items-center gap-1"
              >
                <span
                  class="material-symbols-outlined text-[18px] text-emerald-600"
                  >check_circle</span
                >
                <span>Bebas Korup Fisik</span>
              </div>
              <div class="text-[10px] text-slate-500 mt-1">
                PRAGMA integrity_check: ok
              </div>
            </div>

            <div class="p-3.5 bg-slate-50 border border-slate-200 rounded-xl">
              <div
                class="flex items-center justify-between text-[11px] text-slate-500 font-semibold mb-1"
              >
                <span>Kebijakan Rotasi Backup</span>
                <span
                  class="px-1.5 py-0.2 rounded bg-blue-100 text-blue-800 font-bold font-mono text-[9px]"
                  >AKTIF</span
                >
              </div>
              <div class="font-mono font-bold text-slate-900 text-sm">
                {backupList.length} File Tersedia
              </div>
              <div class="text-[10px] text-slate-500 mt-1">
                Maksimal simpan 10 backup terbaru otomatis
              </div>
            </div>
          </div>

          <!-- Tabel Riwayat File Backup -->
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <span
                class="font-bold text-slate-800 text-xs flex items-center gap-1.5"
              >
                <span class="material-symbols-outlined text-[16px] text-primary"
                  >history</span
                >
                Daftar File Backup SQLite Atomik (Folder ./backups/)
              </span>
              <button
                type="button"
                onclick={loadBackups}
                class="text-[11px] text-primary hover:underline cursor-pointer border-none bg-transparent flex items-center gap-1 font-semibold"
              >
                <span class="material-symbols-outlined text-[14px]"
                  >refresh</span
                >
                <span>Refresh List</span>
              </button>
            </div>

            <div
              class="bg-white border border-slate-200 rounded-xl overflow-hidden shadow-2xs"
            >
              <table class="w-full text-left border-collapse text-xs">
                <thead>
                  <tr
                    class="bg-slate-100 border-b border-slate-200 text-slate-700 font-bold uppercase tracking-wider text-[10px]"
                  >
                    <th class="py-2.5 px-3 w-10 text-center">No</th>
                    <th class="py-2.5 px-3">Nama File Backup</th>
                    <th class="py-2.5 px-3">Waktu Pembuatan</th>
                    <th class="py-2.5 px-3">Ukuran File</th>
                    <th class="py-2.5 px-3">Lokasi / Path</th>
                    <th class="py-2.5 px-3 text-center">Status</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-slate-100">
                  {#if backupList.length === 0}
                    <tr>
                      <td colspan="6" class="py-8 text-center text-slate-400">
                        <div class="flex flex-col items-center gap-1.5">
                          <span
                            class="material-symbols-outlined text-[32px] text-slate-300"
                            >backup</span
                          >
                          <span
                            >Belum ada file backup database. Klik tombol <strong
                              >Backup Sekarang</strong
                            > di atas.</span
                          >
                        </div>
                      </td>
                    </tr>
                  {:else}
                    {#each backupList as b, idx (b.nama_file)}
                      <tr class="hover:bg-slate-50 transition-colors">
                        <td
                          class="py-2 px-3 text-center font-mono text-slate-400"
                          >{idx + 1}</td
                        >
                        <td
                          class="py-2 px-3 font-mono font-bold text-slate-900 flex items-center gap-1.5"
                        >
                          <span
                            class="material-symbols-outlined text-[16px] text-primary"
                            >description</span
                          >
                          <span>{b.nama_file}</span>
                        </td>
                        <td
                          class="py-2 px-3 text-slate-600 font-mono text-[11px]"
                          >{b.waktu}</td
                        >
                        <td class="py-2 px-3 font-mono font-bold text-slate-800"
                          >{b.ukuran_formatted}</td
                        >
                        <td
                          class="py-2 px-3 font-mono text-slate-500 text-[10px] truncate max-w-xs"
                          >{b.path}</td
                        >
                        <td class="py-2 px-3 text-center">
                          <span
                            class="px-2 py-0.5 rounded text-[10px] font-bold bg-emerald-100 text-emerald-800"
                          >
                            VALID
                          </span>
                        </td>
                      </tr>
                    {/each}
                  {/if}
                </tbody>
              </table>
            </div>
          </div>

          <!-- License Info -->
          <div
            class="p-4 bg-slate-100 border border-slate-300 rounded-xl font-mono text-xs flex flex-col gap-2"
          >
            <div class="flex justify-between">
              <span class="text-slate-600">Nama Aplikasi:</span>
              <strong class="text-slate-900"
                >FazPos Retail Edition (iB Retago 5 Engine)</strong
              >
            </div>
            <div class="flex justify-between">
              <span class="text-slate-600">Versi Build:</span>
              <strong class="text-slate-900"
                >v5.2.210 - RTG02 (Tauri v2 + Rust)</strong
              >
            </div>
            <div class="flex justify-between">
              <span class="text-slate-600">Hardware Machine ID:</span>
              <strong class="text-slate-900"
                >{networkConfig?.machine_id ||
                  "MACHINE-LOCAL-01 (Registered)"}</strong
              >
            </div>
            <div class="flex justify-between">
              <span class="text-slate-600">Tipe Lisensi:</span>
              <strong class="text-emerald-700 font-bold"
                >LIFETIME UNLIMITED STANDALONE</strong
              >
            </div>
          </div>

          <!-- Bottom Action Simpan Database Preferences -->
          <div
            class="pt-4 border-t border-slate-200 flex items-center justify-between"
          >
            <span class="text-[11px] text-slate-500 font-medium"
              >Pengaturan database WAL &amp; direktori backup otomatis.</span
            >
            <button
              type="button"
              onclick={() => {
                saveSettings();
                showToast("Preferensi database berhasil disimpan!");
              }}
              class="flex items-center gap-1.5 px-4 py-2 bg-primary hover:bg-primary-dark text-white rounded-lg text-xs font-bold shadow-xs cursor-pointer border-none transition-all"
            >
              <span class="material-symbols-outlined text-[16px]">save</span>
              <span>Simpan Preferensi Database</span>
            </button>
          </div>
        </div>
      {:else if activeTab === "operator"}
        <div class="flex flex-col gap-4">
          <!-- Header of Operator Tab -->
          <div class="flex items-center justify-between">
            <div>
              <h3
                class="font-bold text-slate-900 text-base font-sans flex items-center gap-2"
              >
                <span class="material-symbols-outlined text-primary"
                  >manage_accounts</span
                >
                Manajemen Operator &amp; Pembatasan Hak Akses
              </h3>
              <p class="text-slate-500 text-xs mt-0.5">
                Kelola akun operator kasir, tentukan peran kerja, dan batasi
                modul apa saja yang dapat dibuka pengguna.
              </p>
            </div>

            <button
              type="button"
              onclick={openAddOperator}
              class="flex items-center gap-1.5 px-3.5 py-1.5 bg-primary hover:bg-primary-dark text-white rounded-lg font-sans text-xs font-bold shadow-2xs cursor-pointer border-none transition-all"
            >
              <span class="material-symbols-outlined text-[16px]"
                >person_add</span
              >
              <span>Tambah Operator</span>
            </button>
          </div>

          <!-- Tabel Operator -->
          <div
            class="bg-white border border-slate-300 rounded-xl overflow-hidden shadow-2xs"
          >
            <table class="w-full text-left border-collapse text-xs">
              <thead>
                <tr
                  class="bg-slate-100 border-b border-slate-300 text-slate-700 font-bold uppercase tracking-wider text-[10px]"
                >
                  <th class="py-2.5 px-3 w-10 text-center">No</th>
                  <th class="py-2.5 px-3">Kode / User</th>
                  <th class="py-2.5 px-3">Nama Lengkap</th>
                  <th class="py-2.5 px-3">Peran (Role)</th>
                  <th class="py-2.5 px-3">Hak Akses Modul</th>
                  <th class="py-2.5 px-3 text-center">Status</th>
                  <th class="py-2.5 px-3 text-right">Aksi</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-slate-200">
                {#if operators.length === 0}
                  <tr>
                    <td colspan="7" class="py-6 text-center text-slate-400">
                      Memuat daftar operator...
                    </td>
                  </tr>
                {:else}
                  {#each operators as op, idx (op.id)}
                    {@const perms = getOperatorPermissions(op)}
                    <tr class="hover:bg-slate-50 transition-colors">
                      <td
                        class="py-2.5 px-3 text-center font-mono text-slate-500"
                        >{idx + 1}</td
                      >
                      <td
                        class="py-2.5 px-3 font-mono font-bold text-slate-900"
                      >
                        <div class="flex items-center gap-1.5">
                          <span
                            class="material-symbols-outlined text-[16px] text-slate-400"
                            >badge</span
                          >
                          <span>{op.kode}</span>
                          {#if op.is_admin || op.kode === "admin"}
                            <span
                              class="text-[9px] font-bold px-1 rounded bg-amber-100 text-amber-800 border border-amber-300"
                              >ADMIN</span
                            >
                          {/if}
                        </div>
                      </td>
                      <td class="py-2.5 px-3 font-medium text-slate-800"
                        >{op.nama}</td
                      >
                      <td class="py-2.5 px-3">
                        <span
                          class="px-2 py-0.5 rounded text-[10px] font-bold font-sans uppercase {op.is_admin ||
                          op.role.startsWith('admin')
                            ? 'bg-amber-100 text-amber-800 border border-amber-300'
                            : op.role.startsWith('supervisor')
                              ? 'bg-emerald-100 text-emerald-800 border border-emerald-300'
                              : op.role.startsWith('gudang')
                                ? 'bg-purple-100 text-purple-800 border border-purple-300'
                                : 'bg-blue-100 text-blue-800 border border-blue-200'}"
                        >
                          {op.is_admin || op.role.startsWith("admin")
                            ? "Admin"
                            : op.role.split(":")[0]}
                        </span>
                      </td>
                      <td class="py-2.5 px-3">
                        {#if op.is_admin || op.role === "admin" || op.role.startsWith("admin")}
                          <span
                            class="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-emerald-50 text-emerald-800 text-[10px] font-bold border border-emerald-200"
                          >
                            <span class="material-symbols-outlined text-[13px]"
                              >lock_open</span
                            >
                            Akses Penuh (9 Modul)
                          </span>
                        {:else}
                          <div
                            class="flex items-center gap-1 flex-wrap max-w-md"
                          >
                            {#each perms as p}
                              <span
                                class="px-1.5 py-0.2 rounded bg-slate-100 text-slate-700 text-[10px] font-mono border border-slate-300"
                              >
                                {p}
                              </span>
                            {/each}
                            <span class="text-[10px] text-slate-400 font-mono"
                              >({perms.length} modul)</span
                            >
                          </div>
                        {/if}
                      </td>
                      <td class="py-2.5 px-3 text-center">
                        {#if op.is_aktif !== false}
                          <span
                            class="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-emerald-100 text-emerald-800 font-mono text-[10px] font-bold"
                          >
                            <span
                              class="w-1.5 h-1.5 rounded-full bg-emerald-500"
                            ></span> Aktif
                          </span>
                        {:else}
                          <span
                            class="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-slate-200 text-slate-600 font-mono text-[10px] font-bold"
                          >
                            Non-Aktif
                          </span>
                        {/if}
                      </td>
                      <td class="py-2.5 px-3 text-right">
                        <div class="flex items-center justify-end gap-1">
                          <button
                            type="button"
                            onclick={() => openEditOperator(op)}
                            class="p-1 rounded text-primary hover:bg-sky-50 border border-slate-200 cursor-pointer transition-colors"
                            title="Edit Operator & Hak Akses"
                          >
                            <span class="material-symbols-outlined text-[16px]"
                              >edit</span
                            >
                          </button>
                          {#if !op.is_admin && op.kode !== "admin"}
                            <button
                              type="button"
                              onclick={() => handleDeleteOperator(op)}
                              class="p-1 rounded text-red-600 hover:bg-red-50 border border-slate-200 cursor-pointer transition-colors"
                              title="Hapus Operator"
                            >
                              <span
                                class="material-symbols-outlined text-[16px]"
                                >delete</span
                              >
                            </button>
                          {/if}
                        </div>
                      </td>
                    </tr>
                  {/each}
                {/if}
              </tbody>
            </table>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- Modal Add / Edit Operator -->
  {#if isOperatorModalOpen}
    <div
      class="fixed inset-0 bg-slate-900/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onkeydown={(e) => e.key === "Escape" && (isOperatorModalOpen = false)}
    >
      <div
        class="bg-white border border-slate-300 rounded-2xl shadow-xl w-full max-w-xl max-h-[92vh] flex flex-col overflow-hidden animate-in fade-in zoom-in-95 duration-150 text-slate-800 font-sans"
      >
        <!-- Modal Header -->
        <div
          class="px-5 py-3.5 bg-slate-100 border-b border-slate-200 flex items-center justify-between shrink-0"
        >
          <div class="flex items-center gap-2.5">
            <div
              class="w-8 h-8 rounded-lg bg-blue-100 text-primary flex items-center justify-center border border-blue-200"
            >
              <span class="material-symbols-outlined text-[20px]"
                >manage_accounts</span
              >
            </div>
            <div>
              <h3 class="font-bold text-slate-900 text-sm">
                {editingOperator
                  ? `Edit Operator: ${editingOperator.nama}`
                  : "Tambah Operator Baru"}
              </h3>
              <p class="text-[11px] text-slate-500">
                Atur kredensial dan pembatasan modul aplikasi
              </p>
            </div>
          </div>
          <button
            type="button"
            onclick={() => (isOperatorModalOpen = false)}
            class="text-slate-400 hover:text-slate-600 p-1 rounded-lg hover:bg-slate-200 cursor-pointer border-none bg-transparent"
          >
            <span class="material-symbols-outlined text-[20px]">close</span>
          </button>
        </div>

        <!-- Modal Body (Scrollable) -->
        <form
          onsubmit={handleSaveOperator}
          class="p-5 overflow-y-auto space-y-4 text-xs"
        >
          {#if opErrorMessage}
            <div
              class="p-3 rounded-xl bg-red-50 border border-red-200 text-red-700 flex items-center gap-2"
            >
              <span class="material-symbols-outlined text-[18px] shrink-0"
                >error</span
              >
              <span>{opErrorMessage}</span>
            </div>
          {/if}

          {#if editingOperator?.kode === "admin"}
            <div
              class="p-3 bg-amber-50 border border-amber-200 rounded-xl text-amber-800 text-[11px] flex items-start gap-2"
            >
              <span
                class="material-symbols-outlined text-[16px] text-amber-600 shrink-0 mt-0.5"
                >verified_user</span
              >
              <span>
                <strong>Akun Administrator Utama:</strong> Username dan akses modul
                bersifat permanen (seluruh modul terbuka). Anda hanya dapat memperbarui
                nama atau password.
              </span>
            </div>
          {/if}

          <div class="grid grid-cols-2 gap-3">
            <div>
              <label for="op-kode" class="block font-bold text-slate-700 mb-1">
                Kode / Username Operator <span class="text-red-500">*</span>
              </label>
              <input
                id="op-kode"
                type="text"
                bind:value={formKode}
                disabled={editingOperator?.kode === "admin"}
                placeholder="Contoh: KASIR-01"
                class="w-full px-3 py-2 bg-slate-50 border border-slate-300 rounded-lg font-mono text-xs font-bold focus:bg-white focus:border-primary focus:outline-none disabled:opacity-60"
              />
            </div>

            <div>
              <label for="op-nama" class="block font-bold text-slate-700 mb-1">
                Nama Lengkap Operator <span class="text-red-500">*</span>
              </label>
              <input
                id="op-nama"
                type="text"
                bind:value={formNama}
                placeholder="Contoh: Siti Aminah"
                class="w-full px-3 py-2 bg-slate-50 border border-slate-300 rounded-lg text-xs font-semibold focus:bg-white focus:border-primary focus:outline-none"
              />
            </div>
          </div>

          <div class="grid grid-cols-2 gap-3">
            <div>
              <label
                for="op-password"
                class="block font-bold text-slate-700 mb-1"
              >
                Password {editingOperator ? "(Kosongkan jika tak diubah)" : "*"}
              </label>
              <input
                id="op-password"
                type="password"
                bind:value={formPassword}
                placeholder={editingOperator
                  ? "Tetap gunakan password lama"
                  : "Password baru"}
                class="w-full px-3 py-2 bg-slate-50 border border-slate-300 rounded-lg text-xs font-mono focus:bg-white focus:border-primary focus:outline-none"
              />
            </div>

            <div>
              <label for="op-role" class="block font-bold text-slate-700 mb-1">
                Template Peran (Role Preset)
              </label>
              <select
                id="op-role"
                value={formRolePreset}
                disabled={editingOperator?.kode === "admin"}
                onchange={(e) =>
                  handleRolePresetChange(
                    (e.target as HTMLSelectElement).value as any,
                  )}
                class="w-full px-3 py-2 bg-slate-50 border border-slate-300 rounded-lg text-xs font-semibold focus:bg-white focus:border-primary focus:outline-none disabled:opacity-60 cursor-pointer"
              >
                <option value="kasir"
                  >Kasir (POS, Riwayat Penjualan, Member)</option
                >
                <option value="supervisor"
                  >Supervisor (Semua kecuali Pengaturan)</option
                >
                <option value="gudang"
                  >Staf Gudang (Produk, Pembelian, Supplier)</option
                >
                <option value="admin"
                  >Administrator (Akses Penuh Semua Modul)</option
                >
                <option value="custom"
                  >Kustom Mandiri (Pilih Checklist di bawah)</option
                >
              </select>
            </div>
          </div>

          {#if editingOperator?.kode !== "admin"}
            <div
              class="flex items-center gap-2 p-2 bg-slate-50 rounded-lg border border-slate-200"
            >
              <input
                id="op-aktif"
                type="checkbox"
                bind:checked={formIsAktif}
                class="w-4 h-4 text-primary rounded border-slate-300 focus:ring-primary cursor-pointer"
              />
              <label
                for="op-aktif"
                class="text-slate-800 font-semibold cursor-pointer"
              >
                Status Akun Aktif (Dapat digunakan untuk login kasir)
              </label>
            </div>
          {/if}

          <!-- Checklist Pembatasan Akses Modul -->
          <div class="pt-2 border-t border-slate-200">
            <div class="flex items-center justify-between mb-2">
              <div>
                <span
                  class="font-bold text-slate-800 uppercase tracking-wider text-[11px] block"
                >
                  Pembatasan Akses Modul Aplikasi
                </span>
                <span class="text-slate-500 text-[10px]">
                  Tentukan modul apa saja yang boleh dibuka oleh operator ini
                </span>
              </div>
              {#if formRolePreset !== "admin"}
                <div class="flex items-center gap-1">
                  <button
                    type="button"
                    onclick={() =>
                      (formPermissions = ALL_MODULE_PERMISSIONS.map(
                        (m) => m.key,
                      ))}
                    class="text-[10px] text-primary hover:underline cursor-pointer border-none bg-transparent"
                  >
                    Pilih Semua
                  </button>
                  <span class="text-slate-300">|</span>
                  <button
                    type="button"
                    onclick={() => (formPermissions = [])}
                    class="text-[10px] text-slate-500 hover:underline cursor-pointer border-none bg-transparent"
                  >
                    Kosongkan
                  </button>
                </div>
              {/if}
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
              {#each ALL_MODULE_PERMISSIONS as m (m.key)}
                {@const isAllowed =
                  formRolePreset === "admin" || formPermissions.includes(m.key)}
                <label
                  class="flex items-start gap-2.5 p-2 rounded-xl border transition-all cursor-pointer {isAllowed
                    ? 'bg-sky-50/50 border-sky-300'
                    : 'bg-slate-50 border-slate-200 hover:bg-slate-100'}"
                >
                  <input
                    type="checkbox"
                    checked={isAllowed}
                    disabled={formRolePreset === "admin"}
                    onchange={() => togglePermission(m.key)}
                    class="mt-0.5 w-4 h-4 text-primary rounded border-slate-300 focus:ring-primary cursor-pointer disabled:opacity-50"
                  />
                  <div class="flex-1">
                    <div
                      class="flex items-center gap-1 font-bold text-slate-900 text-xs"
                    >
                      <span
                        class="material-symbols-outlined text-[16px] text-primary"
                        >{m.icon}</span
                      >
                      <span>{m.label}</span>
                    </div>
                    <div
                      class="text-[10px] text-slate-500 mt-0.5 leading-tight"
                    >
                      {m.desc}
                    </div>
                  </div>
                </label>
              {/each}
            </div>
          </div>

          <!-- Modal Footer -->
          <div
            class="pt-3 border-t border-slate-200 flex items-center justify-end gap-2"
          >
            <button
              type="button"
              onclick={() => (isOperatorModalOpen = false)}
              class="px-3.5 py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg text-xs font-semibold cursor-pointer border border-slate-300 transition-colors"
            >
              Batal
            </button>
            <button
              type="submit"
              disabled={opIsLoading}
              class="px-4 py-1.5 bg-primary hover:bg-primary-dark active:bg-sky-800 disabled:opacity-50 text-white rounded-lg text-xs font-bold shadow-xs cursor-pointer border-none flex items-center gap-1.5 transition-all"
            >
              {#if opIsLoading}
                <span class="material-symbols-outlined text-[16px] animate-spin"
                  >progress_activity</span
                >
                <span>Menyimpan...</span>
              {:else}
                <span class="material-symbols-outlined text-[16px]">save</span>
                <span>Simpan Data &amp; Hak Akses</span>
              {/if}
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}

  <!-- Modal Tambah Terminal Kasir (LAN) -->
  {#if isAddDeviceOpen}
    <div
      class="fixed inset-0 bg-slate-900/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onkeydown={(e) => e.key === "Escape" && (isAddDeviceOpen = false)}
    >
      <div
        class="bg-white border border-slate-300 rounded-2xl shadow-xl w-full max-w-md flex flex-col overflow-hidden animate-in fade-in zoom-in-95 duration-150 text-slate-800 font-sans"
      >
        <!-- Modal Header -->
        <div
          class="px-5 py-3.5 bg-slate-100 border-b border-slate-200 flex items-center justify-between shrink-0"
        >
          <div class="flex items-center gap-2.5">
            <div
              class="w-8 h-8 rounded-lg bg-blue-100 text-primary flex items-center justify-center border border-blue-200"
            >
              <span class="material-symbols-outlined text-[20px]">devices</span>
            </div>
            <div>
              <h3 class="font-bold text-slate-900 text-sm">
                Tambah Terminal Kasir
              </h3>
              <p class="text-[11px] text-slate-500">
                Daftarkan perangkat terminal kasir baru untuk cabang aktif
              </p>
            </div>
          </div>
          <button
            type="button"
            onclick={() => (isAddDeviceOpen = false)}
            class="text-slate-400 hover:text-slate-600 p-1 rounded-lg hover:bg-slate-200 cursor-pointer border-none bg-transparent"
          >
            <span class="material-symbols-outlined text-[20px]">close</span>
          </button>
        </div>

        <!-- Modal Body -->
        <form
          onsubmit={handleSimpanDevice}
          class="p-5 space-y-4 text-xs"
        >
          <div class="space-y-1">
            <label for="dev-kode" class="block font-bold text-slate-700">Kode Terminal <span class="text-red-500">*</span></label>
            <input
              id="dev-kode"
              type="text"
              bind:value={formDeviceKode}
              placeholder="Contoh: DEV-02"
              required
              class="w-full px-3 py-2 border border-slate-300 rounded-lg text-xs font-mono uppercase focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
            />
          </div>

          <div class="space-y-1">
            <label for="dev-nama" class="block font-bold text-slate-700">Nama Terminal <span class="text-red-500">*</span></label>
            <input
              id="dev-nama"
              type="text"
              bind:value={formDeviceNama}
              placeholder="Contoh: Kasir Depan 2"
              required
              class="w-full px-3 py-2 border border-slate-300 rounded-lg text-xs focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
            />
          </div>

          <div class="space-y-1">
            <label for="dev-role" class="block font-bold text-slate-700">Peran Terminal (Role)</label>
            <select
              id="dev-role"
              bind:value={formDeviceRole}
              class="w-full px-3 py-2 border border-slate-300 rounded-lg text-xs focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none bg-white"
            >
              <option value="client">Client Kasir (Klien Transaksi)</option>
              <option value="server">Server Utama (Pusat Database Toko)</option>
            </select>
            <p class="text-[10px] text-slate-400">Pilih 'Client Kasir' untuk kasir tambahan yang terhubung ke server utama.</p>
          </div>

          <div class="space-y-1">
            <label for="dev-ip" class="block font-bold text-slate-700">IP Address (Opsional)</label>
            <input
              id="dev-ip"
              type="text"
              bind:value={formDeviceIp}
              placeholder="Contoh: 192.168.1.105 (kosongkan jika DHCP)"
              class="w-full px-3 py-2 border border-slate-300 rounded-lg text-xs font-mono focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
            />
          </div>

          <!-- Modal Footer -->
          <div class="pt-3 border-t border-slate-200 flex items-center justify-end gap-2">
            <button
              type="button"
              onclick={() => (isAddDeviceOpen = false)}
              class="px-4 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg font-bold text-xs cursor-pointer border border-slate-300"
            >
              Batal
            </button>
            <button
              type="submit"
              class="px-4 py-2 bg-primary hover:bg-primary-dark text-white rounded-lg font-bold text-xs cursor-pointer border-none shadow-xs flex items-center gap-1.5"
            >
              <span class="material-symbols-outlined text-[16px]">save</span>
              <span>Simpan Terminal</span>
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}

  <!-- Modal Tambah Cabang (Outlets) -->
  {#if isAddCabangOpen}
    <div
      class="fixed inset-0 bg-slate-900/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onkeydown={(e) => e.key === "Escape" && (isAddCabangOpen = false)}
    >
      <div
        class="bg-white border border-slate-300 rounded-2xl shadow-xl w-full max-w-md flex flex-col overflow-hidden animate-in fade-in zoom-in-95 duration-150 text-slate-800 font-sans"
      >
        <!-- Modal Header -->
        <div
          class="px-5 py-3.5 bg-slate-100 border-b border-slate-200 flex items-center justify-between shrink-0"
        >
          <div class="flex items-center gap-2.5">
            <div
              class="w-8 h-8 rounded-lg bg-primary/10 text-primary flex items-center justify-center border border-primary/20"
            >
              <span class="material-symbols-outlined text-[20px]">store</span>
            </div>
            <div>
              <h3 class="font-bold text-slate-900 text-sm">
                Tambah Cabang Baru
              </h3>
              <p class="text-[11px] text-slate-500">
                Daftarkan outlet / toko cabang baru untuk integrasi multi-cabang
              </p>
            </div>
          </div>
          <button
            type="button"
            onclick={() => (isAddCabangOpen = false)}
            class="text-slate-400 hover:text-slate-600 p-1 rounded-lg hover:bg-slate-200 cursor-pointer border-none bg-transparent"
          >
            <span class="material-symbols-outlined text-[20px]">close</span>
          </button>
        </div>

        <!-- Modal Body -->
        <form
          onsubmit={handleSimpanCabang}
          class="p-5 space-y-4 text-xs"
        >
          <div class="space-y-1">
            <label for="cab-kode" class="block font-bold text-slate-700">Kode Cabang <span class="text-red-500">*</span></label>
            <input
              id="cab-kode"
              type="text"
              bind:value={formCabangKode}
              placeholder="Contoh: CAB-02"
              required
              class="w-full px-3 py-2 border border-slate-300 rounded-lg text-xs font-mono uppercase focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
            />
          </div>

          <div class="space-y-1">
            <label for="cab-nama" class="block font-bold text-slate-700">Nama Cabang / Toko <span class="text-red-500">*</span></label>
            <input
              id="cab-nama"
              type="text"
              bind:value={formCabangNama}
              placeholder="Contoh: Cabang Padang Timur"
              required
              class="w-full px-3 py-2 border border-slate-300 rounded-lg text-xs focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
            />
          </div>

          <div class="space-y-1">
            <label for="cab-alamat" class="block font-bold text-slate-700">Alamat Cabang (Opsional)</label>
            <textarea
              id="cab-alamat"
              bind:value={formCabangAlamat}
              placeholder="Alamat lengkap cabang..."
              rows="2"
              class="w-full px-3 py-2 border border-slate-300 rounded-lg text-xs focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none resize-none"
            ></textarea>
          </div>

          <div class="flex items-center gap-2 p-2.5 bg-slate-50 border border-slate-200 rounded-lg">
            <input
              id="cab-is-pusat"
              type="checkbox"
              bind:checked={formCabangIsPusat}
              class="w-4 h-4 text-primary rounded border-slate-300 focus:ring-primary"
            />
            <label for="cab-is-pusat" class="text-xs text-slate-700 font-medium cursor-pointer select-none">
              Jadikan sebagai Toko Pusat (Headquarters)
            </label>
          </div>

          <!-- Modal Footer -->
          <div class="pt-3 border-t border-slate-200 flex items-center justify-end gap-2">
            <button
              type="button"
              onclick={() => (isAddCabangOpen = false)}
              class="px-4 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg font-bold text-xs cursor-pointer border border-slate-300"
            >
              Batal
            </button>
            <button
              type="submit"
              class="px-4 py-2 bg-primary hover:bg-primary-dark text-white rounded-lg font-bold text-xs cursor-pointer border-none shadow-xs flex items-center gap-1.5"
            >
              <span class="material-symbols-outlined text-[16px]">save</span>
              <span>Simpan Cabang</span>
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}

  <!-- Modal Sambung Manual via IP Server -->
  {#if isManualConnectOpen}
    <div
      class="fixed inset-0 bg-slate-900/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onkeydown={(e) => e.key === "Escape" && (isManualConnectOpen = false)}
    >
      <div
        class="bg-white border border-slate-300 rounded-2xl shadow-xl w-full max-w-md flex flex-col overflow-hidden animate-in fade-in zoom-in-95 duration-150 text-slate-800 font-sans"
      >
        <!-- Modal Header -->
        <div
          class="px-5 py-3.5 bg-slate-100 border-b border-slate-200 flex items-center justify-between shrink-0"
        >
          <div class="flex items-center gap-2.5">
            <div
              class="w-8 h-8 rounded-lg bg-primary/10 text-primary flex items-center justify-center border border-primary/20"
            >
              <span class="material-symbols-outlined text-[20px]">lan</span>
            </div>
            <div>
              <h3 class="font-bold text-slate-900 text-sm">
                Sambung Manual ke Server
              </h3>
              <p class="text-[11px] text-slate-500">
                Sambung langsung menggunakan alamat IP Master Server
              </p>
            </div>
          </div>
          <button
            type="button"
            onclick={() => (isManualConnectOpen = false)}
            class="text-slate-400 hover:text-slate-600 p-1 rounded-lg hover:bg-slate-200 cursor-pointer border-none bg-transparent"
          >
            <span class="material-symbols-outlined text-[20px]">close</span>
          </button>
        </div>

        <!-- Modal Body -->
        <form
          onsubmit={(e) => {
            e.preventDefault();
            handleManualConnect();
          }}
          class="p-5 space-y-4 text-xs"
        >
          <div class="p-3 bg-blue-50/60 border border-blue-200 rounded-lg text-blue-900 text-[11px] flex items-start gap-2">
            <span class="material-symbols-outlined text-[16px] text-blue-600 shrink-0 mt-0.5">info</span>
            <span>
              Masukkan alamat IP komputer yang bertindak sebagai <strong>Master Server</strong> (contoh: <code>192.168.100.75</code>). Alamat IP ini dapat dilihat di layar FAZPOS komputer server.
            </span>
          </div>

          <div class="space-y-1.5">
            <label for="manual-ip" class="font-bold text-slate-700 block">
              Alamat IP Master Server <span class="text-rose-500">*</span>
            </label>
            <input
              id="manual-ip"
              type="text"
              bind:value={manualServerIp}
              placeholder="Contoh: 192.168.100.75"
              class="w-full px-3 py-2 border border-slate-300 rounded-lg text-xs font-mono focus:ring-1 focus:ring-primary focus:border-primary outline-hidden"
              required
            />
          </div>

          <div class="space-y-1.5">
            <label for="manual-port" class="font-bold text-slate-700 block">
              Port Server (Default: 7890)
            </label>
            <input
              id="manual-port"
              type="number"
              bind:value={manualServerPort}
              placeholder="7890"
              class="w-full px-3 py-2 border border-slate-300 rounded-lg text-xs font-mono focus:ring-1 focus:ring-primary focus:border-primary outline-hidden"
            />
          </div>

          <!-- Modal Footer -->
          <div class="pt-3 border-t border-slate-200 flex items-center justify-end gap-2">
            <button
              type="button"
              onclick={() => (isManualConnectOpen = false)}
              class="px-4 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg font-bold text-xs cursor-pointer border border-slate-300"
            >
              Batal
            </button>
            <button
              type="submit"
              disabled={isConnectingManual}
              class="px-4 py-2 bg-primary hover:bg-primary-dark disabled:opacity-50 text-white rounded-lg font-bold text-xs cursor-pointer border-none shadow-xs flex items-center gap-1.5"
            >
              {#if isConnectingManual}
                <span class="material-symbols-outlined text-[16px] animate-spin">progress_activity</span>
                <span>Menghubungkan...</span>
              {:else}
                <span class="material-symbols-outlined text-[16px]">link</span>
                <span>Hubungkan Sekarang</span>
              {/if}
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}

  <!-- Toast Message -->
  {#if toastMessage}
    <div
      class="fixed bottom-4 right-4 bg-slate-900 text-emerald-300 px-4 py-2.5 rounded-lg shadow-xl font-mono text-xs border border-slate-700 animate-in fade-in z-50 flex items-center gap-2"
    >
      <span class="material-symbols-outlined text-[18px] text-emerald-400"
        >info</span
      >
      <span>{toastMessage}</span>
    </div>
  {/if}
</div>
