<script lang="ts">
  let { onInput }: { onInput: (val: string) => void } = $props();

  const buttons = [
    { label: "7", val: "7", class: "num" },
    { label: "8", val: "8", class: "num" },
    { label: "9", val: "9", class: "num" },
    { label: "QTY ×", val: "QTY", class: "fn" },

    { label: "4", val: "4", class: "num" },
    { label: "5", val: "5", class: "num" },
    { label: "6", val: "6", class: "num" },
    { label: "% DISC", val: "DISC", class: "fn" },

    { label: "1", val: "1", class: "num" },
    { label: "2", val: "2", class: "num" },
    { label: "3", val: "3", class: "num" },
    { label: "⌫", val: "DEL", class: "del" },

    { label: "0", val: "0", class: "num" },
    { label: "00", val: "00", class: "num" },
    { label: "C", val: "CLR", class: "clear" },
    { label: "OK", val: "OK", class: "ok" },
  ];
</script>

<div class="numpad-container">
  <div class="numpad-grid">
    {#each buttons as btn}
      <button
        class="numpad-btn {btn.class}"
        onclick={() => onInput(btn.val)}
      >
        {btn.label}
      </button>
    {/each}
  </div>
</div>

<style>
  .numpad-container {
    height: 145px;
    background: #f1f5f9;
    padding: 6px 8px;
    border-top: 1px solid #cbd5e1;
    border-bottom: 1px solid #cbd5e1;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }

  .numpad-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    grid-template-rows: repeat(4, 1fr);
    gap: 4px;
    height: 100%;
  }

  .numpad-btn {
    background: #ffffff;
    border: 1px solid #cbd5e1;
    color: #0f172a;
    border-radius: 4px;
    font-size: 13px;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background-color 0.1s;
  }

  .numpad-btn:hover {
    background: #f8fafc;
  }

  .numpad-btn.fn {
    color: #0369a1;
    font-size: 11px;
    font-weight: 700;
    background: #f0f9ff;
    border-color: #bae6fd;
  }

  .numpad-btn.del {
    color: #dc2626;
    font-size: 14px;
  }

  .numpad-btn.clear {
    color: #64748b;
  }

  .numpad-btn.ok {
    background: #0284c7;
    border-color: #0284c7;
    color: #ffffff;
    font-weight: 700;
  }

  .numpad-btn.ok:hover {
    background: #0369a1;
  }
</style>
