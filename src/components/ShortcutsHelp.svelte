<script lang="ts">
  import Icons from './Icons.svelte';

  let { onClose } = $props<{ onClose: () => void }>();

  // combo: các phím bấm cùng lúc (Ctrl+F); ngược lại là các phím thay thế (j / ↓)
  const shortcuts: { keys: string[]; label: string; combo?: boolean }[] = [
    { keys: ['j', '↓'], label: 'Request kế tiếp' },
    { keys: ['k', '↑'], label: 'Request trước đó' },
    { keys: ['/'], label: 'Lọc danh sách request' },
    { keys: ['Ctrl', 'F'], label: 'Tìm trong body', combo: true },
    { keys: ['r'], label: 'Replay request đang chọn' },
    { keys: ['d'], label: 'So sánh với request khác' },
    { keys: ['Del'], label: 'Xoá request đang chọn' },
    { keys: ['p'], label: 'Tạm dừng / bật lại endpoint đang chọn' },
    { keys: ['Esc'], label: 'Huỷ so sánh / đóng hộp thoại' },
    { keys: ['?'], label: 'Bật / tắt bảng này' },
  ];
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onClose()} />

<div class="modal-overlay" onclick={onClose} role="presentation">
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-card shortcuts-card" onclick={(e) => e.stopPropagation()}>
    <div class="modal-header">
      <span class="modal-title">
        <Icons name="keyboard" size={14} />
        Phím Tắt
      </span>
    </div>
    <div class="modal-body">
      {#each shortcuts as { keys, label, combo }}
        <div class="shortcut-row">
          <span class="keys">
            {#each keys as k, i}
              {#if i > 0}<span class="sep">{combo ? '+' : '/'}</span>{/if}<kbd>{k}</kbd>
            {/each}
          </span>
          <span class="label">{label}</span>
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .shortcuts-card {
    max-width: 340px;
  }

  .modal-title {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 11.5px;
  }

  .keys {
    width: 90px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 3px;
  }

  .sep {
    color: var(--text-dim);
    font-size: 10px;
  }

  kbd {
    font-family: var(--font-mono);
    font-size: 10.5px;
    min-width: 18px;
    text-align: center;
    padding: 1px 5px;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--border-medium);
    border-bottom-width: 2px;
    color: var(--text-main);
  }

  .label {
    color: var(--text-muted);
  }
</style>
