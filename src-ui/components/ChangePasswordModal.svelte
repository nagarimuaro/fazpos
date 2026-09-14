<script lang="ts">
  import { api, type OperatorDTO } from "../lib/api";

  let {
    isOpen,
    currentUser,
    onClose,
  }: {
    isOpen: boolean;
    currentUser: OperatorDTO | null;
    onClose: () => void;
  } = $props();

  let passwordLama = $state("");
  let passwordBaru = $state("");
  let konfirmasiBaru = $state("");
  let errorMessage = $state("");
  let successMessage = $state("");
  let isLoading = $state(false);

  function resetForm() {
    passwordLama = "";
    passwordBaru = "";
    konfirmasiBaru = "";
    errorMessage = "";
    successMessage = "";
    isLoading = false;
  }

  async function handleSimpan(e?: Event) {
    if (e) e.preventDefault();
    if (!currentUser) return;

    if (!passwordLama) {
      errorMessage = "Silakan masukkan password saat ini / lama";
      return;
    }
    if (!passwordBaru) {
      errorMessage = "Silakan masukkan password baru";
      return;
    }
    if (passwordBaru.length < 3) {
      errorMessage = "Password baru minimal 3 karakter";
      return;
    }
    if (passwordBaru !== konfirmasiBaru) {
      errorMessage = "Konfirmasi password baru tidak cocok";
      return;
    }

    isLoading = true;
    errorMessage = "";
    successMessage = "";

    try {
      await api.ubahPassword(currentUser.kode, passwordLama, passwordBaru);
      successMessage = "Password berhasil diperbarui!";
      setTimeout(() => {
        resetForm();
        onClose();
      }, 1200);
    } catch (err: any) {
      errorMessage = typeof err === "string" ? err : err?.message || "Gagal mengubah password";
    } finally {
      isLoading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!isOpen) return;
    if (e.key === "Escape") {
      e.preventDefault();
      resetForm();
      onClose();
    }
  }

  $effect(() => {
    if (isOpen) {
      resetForm();
      window.addEventListener("keydown", handleKeydown);
      return () => window.removeEventListener("keydown", handleKeydown);
    }
  });
</script>

{#if isOpen}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 bg-slate-900/70 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none"
    onclick={() => { resetForm(); onClose(); }}
    role="button"
    tabindex="0"
    onkeydown={(e) => e.key === "Escape" && onClose()}
  >
    <!-- Modal Dialog -->
    <div
      class="bg-white border border-slate-200 rounded-2xl shadow-2xl w-[440px] overflow-hidden animate-in fade-in zoom-in-95 duration-150 text-slate-800 font-sans"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="px-5 py-4 bg-slate-100 border-b border-slate-200 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-blue-100 text-blue-700 flex items-center justify-center shrink-0 border border-blue-200">
            <span class="material-symbols-outlined text-[22px]">lock_reset</span>
          </div>
          <div>
            <h3 class="font-bold text-slate-900 text-sm">Ubah Password Operator</h3>
            <p class="text-xs text-slate-500 font-medium">
              Akun: <span class="font-bold text-slate-700">{currentUser?.nama}</span> ({currentUser?.kode})
            </p>
          </div>
        </div>
        <button
          onclick={() => { resetForm(); onClose(); }}
          class="text-slate-400 hover:text-slate-600 p-1 rounded-lg hover:bg-slate-200/60 cursor-pointer border-none bg-transparent"
        >
          <span class="material-symbols-outlined text-[20px]">close</span>
        </button>
      </div>

      <!-- Body Form -->
      <form onsubmit={handleSimpan} class="p-5 space-y-4">
        {#if currentUser?.is_admin}
          <div class="p-3 bg-amber-50 border border-amber-200 rounded-xl text-amber-800 text-[11px] flex items-start gap-2">
            <span class="material-symbols-outlined text-[16px] text-amber-600 shrink-0 mt-0.5">info</span>
            <span>
              <strong>Catatan User Admin:</strong> Akun ini bersifat permanen sistem. Identitas username tidak dapat diubah, namun Anda bebas memperbarui password kapan saja.
            </span>
          </div>
        {/if}

        {#if errorMessage}
          <div class="p-3 rounded-xl bg-red-50 border border-red-200 text-red-700 text-xs flex items-center gap-2">
            <span class="material-symbols-outlined text-[18px] shrink-0">error</span>
            <span>{errorMessage}</span>
          </div>
        {/if}

        {#if successMessage}
          <div class="p-3 rounded-xl bg-emerald-50 border border-emerald-200 text-emerald-700 text-xs flex items-center gap-2 font-medium">
            <span class="material-symbols-outlined text-[18px] shrink-0">check_circle</span>
            <span>{successMessage}</span>
          </div>
        {/if}

        <div>
          <label for="pass-lama" class="block text-xs font-semibold text-slate-600 mb-1">
            Password Lama / Saat Ini:
          </label>
          <input
            id="pass-lama"
            type="password"
            bind:value={passwordLama}
            placeholder="Masukkan password saat ini"
            class="w-full px-3 py-2 text-sm border border-slate-300 rounded-xl focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
          />
        </div>

        <div>
          <label for="pass-baru" class="block text-xs font-semibold text-slate-600 mb-1">
            Password Baru:
          </label>
          <input
            id="pass-baru"
            type="password"
            bind:value={passwordBaru}
            placeholder="Minimal 3 karakter"
            class="w-full px-3 py-2 text-sm border border-slate-300 rounded-xl focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
          />
        </div>

        <div>
          <label for="pass-konfirm" class="block text-xs font-semibold text-slate-600 mb-1">
            Konfirmasi Password Baru:
          </label>
          <input
            id="pass-konfirm"
            type="password"
            bind:value={konfirmasiBaru}
            placeholder="Ulangi password baru"
            class="w-full px-3 py-2 text-sm border border-slate-300 rounded-xl focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
          />
        </div>

        <!-- Footer Buttons -->
        <div class="pt-3 border-t border-slate-100 flex items-center justify-end gap-2">
          <button
            type="button"
            onclick={() => { resetForm(); onClose(); }}
            class="px-4 py-2 text-xs font-semibold text-slate-600 hover:bg-slate-100 rounded-xl cursor-pointer border border-slate-200 transition-colors"
          >
            Batal
          </button>
          <button
            type="submit"
            disabled={isLoading}
            class="px-4 py-2 text-xs font-bold text-white bg-blue-600 hover:bg-blue-500 disabled:opacity-50 rounded-xl cursor-pointer border-none transition-colors flex items-center gap-1.5 shadow-xs"
          >
            {#if isLoading}
              <span class="material-symbols-outlined text-[16px] animate-spin">progress_activity</span>
              <span>Menyimpan...</span>
            {:else}
              <span class="material-symbols-outlined text-[16px]">check</span>
              <span>Simpan Password</span>
            {/if}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
