<script lang="ts">
  import type { Endpoint, ResponseRule, ConditionType, ConditionOperator } from '../types';
  import Icons from './Icons.svelte';

  let {
    isOpen,
    endpoint,
    rules,
    initialPrefill,
    onClose,
    onSaveRule,
    onDeleteRule,
  } = $props<{
    isOpen: boolean;
    endpoint: Endpoint | null;
    rules: ResponseRule[];
    initialPrefill?: Partial<ResponseRule> | null;
    onClose: () => void;
    onSaveRule: (rule: Partial<ResponseRule>) => void;
    onDeleteRule: (id: string) => void;
  }>();

  let isEditing = $state(false);
  let editRuleId = $state<string | null>(null);

  let ruleName = $state('');
  let conditionType = $state<ConditionType>('header');
  let conditionField = $state('');
  let conditionOperator = $state<ConditionOperator>('equals');
  let conditionValue = $state('');
  let responseStatus = $state(200);
  let responseBody = $state('{\n  "matched": true\n}');
  let responseDelayMs = $state(0);
  let isEnabled = $state(true);

  $effect(() => {
    if (isOpen) {
      if (initialPrefill) {
        startCreatePrefill(initialPrefill);
      } else {
        isEditing = false;
        editRuleId = null;
      }
    }
  });

  function startCreatePrefill(prefill: Partial<ResponseRule>) {
    isEditing = true;
    editRuleId = null;
    ruleName = prefill.name || 'Quy Tắc Tùy Biến';
    conditionType = prefill.condition_type || 'header';
    conditionField = prefill.condition_field || '';
    conditionOperator = prefill.condition_operator || 'equals';
    conditionValue = prefill.condition_value || '';
    responseStatus = prefill.response_status ?? 200;
    responseBody = prefill.response_body || '{\n  "status": "matched"\n}';
    responseDelayMs = prefill.response_delay_ms ?? 0;
    isEnabled = prefill.is_enabled ?? true;
  }

  function startCreateNew() {
    isEditing = true;
    editRuleId = null;
    ruleName = 'Quy Tắc Điều Kiện Mới';
    conditionType = 'header';
    conditionField = 'x-event-type';
    conditionOperator = 'equals';
    conditionValue = 'order.created';
    responseStatus = 200;
    responseBody = '{\n  "result": "success",\n  "processed": true\n}';
    responseDelayMs = 0;
    isEnabled = true;
  }

  function startEdit(r: ResponseRule) {
    isEditing = true;
    editRuleId = r.id;
    ruleName = r.name;
    conditionType = r.condition_type;
    conditionField = r.condition_field;
    conditionOperator = r.condition_operator;
    conditionValue = r.condition_value;
    responseStatus = r.response_status;
    responseBody = r.response_body;
    responseDelayMs = r.response_delay_ms;
    isEnabled = r.is_enabled;
  }

  function handleSave() {
    if (!endpoint || !ruleName.trim()) return;

    onSaveRule({
      id: editRuleId || undefined,
      endpoint_id: endpoint.id,
      name: ruleName.trim(),
      condition_type: conditionType,
      condition_field: conditionField.trim(),
      condition_operator: conditionOperator,
      condition_value: conditionValue.trim(),
      response_status: Number(responseStatus),
      response_headers: { 'Content-Type': 'application/json' },
      response_body: responseBody,
      response_delay_ms: Number(responseDelayMs),
      is_enabled: isEnabled,
    });

    isEditing = false;
    editRuleId = null;
  }

  function toggleRuleEnabled(r: ResponseRule) {
    onSaveRule({
      ...r,
      is_enabled: !r.is_enabled,
    });
  }
</script>

{#if isOpen && endpoint}
  <div
    class="modal-overlay"
    onclick={onClose}
    onkeydown={(e) => e.key === 'Escape' && onClose()}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="modal-card rules-card"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="modal-header">
        <div class="title-col">
          <h3 class="modal-title">Quy Tắc Phản Hồi</h3>
          <span class="modal-subtitle">/wh/{endpoint.slug}</span>
        </div>
        <button class="btn-close" onclick={onClose}>
          <Icons name="x" size={16} />
        </button>
      </div>

      <div class="modal-body">
        {#if !isEditing}
          <div class="rules-list-toolbar">
            <span class="rules-count-desc">{rules.length} quy tắc</span>
            <button class="btn-primary sm" onclick={startCreateNew}>
              <Icons name="plus" size={12} color="#ffffff" />
              <span>Thêm Quy Tắc</span>
            </button>
          </div>

          {#if rules.length === 0}
            <div class="empty-rules">
              <Icons name="sliders" size={24} color="#64748b" />
              <span>Chưa có quy tắc nào</span>
              <button class="btn-secondary" onclick={startCreateNew}>+ Thêm Quy Tắc</button>
            </div>
          {:else}
            <div class="rules-items-list">
              {#each rules as r (r.id)}
                <div class="rule-card" class:disabled={!r.is_enabled}>
                  <div class="rule-card-top">
                    <div class="rule-title-row">
                      <span class="rule-name">{r.name}</span>
                      <span class="rule-status-badge" class:error={r.response_status >= 400}>
                        {r.response_status}
                      </span>
                    </div>

                    <div class="rule-actions-top">
                      <button
                        class="btn-toggle-switch"
                        class:active={r.is_enabled}
                        onclick={() => toggleRuleEnabled(r)}
                        title={r.is_enabled ? 'Tắt quy tắc' : 'Bật quy tắc'}
                      >
                        {r.is_enabled ? 'Bật' : 'Tắt'}
                      </button>
                      <button class="btn-icon" onclick={() => startEdit(r)} title="Chỉnh sửa">
                        <Icons name="settings" size={12} color="#94a3b8" />
                      </button>
                      <button class="btn-icon danger" onclick={() => onDeleteRule(r.id)} title="Xóa">
                        <Icons name="trash" size={12} color="#f87171" />
                      </button>
                    </div>
                  </div>

                  <div class="rule-condition-overview">
                    <span class="cond-tag">NẾU</span>
                    <span class="cond-type">{r.condition_type}</span>
                    {#if r.condition_field}
                      <span class="cond-field">"{r.condition_field}"</span>
                    {/if}
                    <span class="cond-op">{r.condition_operator}</span>
                    <span class="cond-val">"{r.condition_value}"</span>
                    {#if r.response_delay_ms > 0}
                      <span class="cond-delay">+{r.response_delay_ms}ms</span>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        {:else}
          <!-- Editing / Creating Rule Form -->
          <div class="rule-edit-form">
            <div class="form-group">
              <label class="form-label" for="rule-name-input">Tên quy tắc</label>
              <input
                id="rule-name-input"
                type="text"
                class="form-input"
                placeholder="VD: Kiểm tra Token hoặc Đơn hàng"
                bind:value={ruleName}
              />
            </div>

            <div class="form-section-title">Điều Kiện Khớp</div>

            <div class="form-row-3">
              <div class="form-group">
                <label class="form-label" for="rule-cond-type-input">Loại</label>
                <select id="rule-cond-type-input" class="form-select" bind:value={conditionType}>
                  <option value="header">Header</option>
                  <option value="json_body">JSON Body</option>
                  <option value="path">Path</option>
                  <option value="query">URL Query</option>
                  <option value="method">Method</option>
                </select>
              </div>

              <div class="form-group">
                <label class="form-label" for="rule-cond-field-input">Trường / Key</label>
                <input
                  id="rule-cond-field-input"
                  type="text"
                  class="form-input mono"
                  placeholder={conditionType === 'header' ? 'x-event-type' : 'event.type'}
                  bind:value={conditionField}
                />
              </div>

              <div class="form-group">
                <label class="form-label" for="rule-cond-op-input">Toán tử</label>
                <select id="rule-cond-op-input" class="form-select" bind:value={conditionOperator}>
                  <option value="equals">Equals (=)</option>
                  <option value="contains">Contains</option>
                  <option value="regex">Regex</option>
                  <option value="exists">Exists</option>
                </select>
              </div>
            </div>

            {#if conditionOperator !== 'exists'}
              <div class="form-group">
                <label class="form-label" for="rule-cond-val-input">Giá trị khớp</label>
                <input
                  id="rule-cond-val-input"
                  type="text"
                  class="form-input mono"
                  placeholder="VD: payment.success"
                  bind:value={conditionValue}
                />
              </div>
            {/if}

            <div class="form-section-title">Phản Hồi Trả Về</div>

            <div class="form-row-2">
              <div class="form-group">
                <label class="form-label" for="rule-status-input">Mã HTTP</label>
                <select id="rule-status-input" class="form-select" bind:value={responseStatus}>
                  <option value={200}>200 OK</option>
                  <option value={201}>201 Created</option>
                  <option value={204}>204 No Content</option>
                  <option value={400}>400 Bad Request</option>
                  <option value={401}>401 Unauthorized</option>
                  <option value={403}>403 Forbidden</option>
                  <option value={404}>404 Not Found</option>
                  <option value={429}>429 Too Many Requests</option>
                  <option value={500}>500 Server Error</option>
                </select>
              </div>

              <div class="form-group">
                <label class="form-label" for="rule-delay-input">Độ trễ (ms)</label>
                <input
                  id="rule-delay-input"
                  type="number"
                  min="0"
                  max="10000"
                  step="50"
                  class="form-input mono"
                  bind:value={responseDelayMs}
                />
              </div>
            </div>

            <div class="form-group">
              <label class="form-label" for="rule-body-input">Body phản hồi</label>
              <textarea
                id="rule-body-input"
                rows="4"
                class="form-textarea"
                bind:value={responseBody}
              ></textarea>
            </div>
          </div>
        {/if}
      </div>

      <div class="modal-footer">
        {#if isEditing}
          <button class="btn-secondary" onclick={() => (isEditing = false)}>Hủy</button>
          <button class="btn-primary" onclick={handleSave}>Lưu Quy Tắc</button>
        {:else}
          <button class="btn-secondary" onclick={onClose}>Đóng</button>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .rules-card {
    max-width: 680px;
  }

  .title-col {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .modal-subtitle {
    font-size: 11.5px;
    font-family: var(--font-mono);
    color: var(--text-dim);
  }

  .btn-close {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    border-radius: 4px;
  }

  .rules-list-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .rules-count-desc {
    font-size: 11.5px;
    color: var(--text-dim);
  }

  .btn-primary.sm {
    padding: 4px 10px;
    font-size: 11.5px;
  }

  .empty-rules {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 36px 20px;
    gap: 8px;
    text-align: center;
    color: var(--text-muted);
  }

  .rules-items-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .rule-card {
    background: var(--bg-card);
    border: 1px solid var(--border-subtle);
    border-radius: 8px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    transition: all 0.15s;
  }

  .rule-card.disabled {
    opacity: 0.5;
  }

  .rule-card-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .rule-title-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .rule-name {
    font-size: 13px;
    font-weight: 600;
  }

  .rule-status-badge {
    font-family: var(--font-mono);
    font-size: 10.5px;
    padding: 1px 6px;
    border-radius: 4px;
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
  }

  .rule-status-badge.error {
    background: rgba(239, 68, 68, 0.15);
    color: #f87171;
  }

  .rule-actions-top {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .btn-toggle-switch {
    font-size: 10.5px;
    font-weight: 600;
    padding: 2px 7px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-dim);
    border: none;
    cursor: pointer;
  }

  .btn-toggle-switch.active {
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
  }

  .btn-icon {
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 3px;
    border-radius: 4px;
  }

  .btn-icon:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .btn-icon.danger:hover {
    background: rgba(239, 68, 68, 0.15);
  }

  .rule-condition-overview {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
    font-family: var(--font-mono);
    font-size: 11px;
    background: rgba(0, 0, 0, 0.25);
    padding: 6px 10px;
    border-radius: 6px;
  }

  .cond-tag {
    font-weight: 700;
    color: #a855f7;
  }

  .cond-type {
    color: #60a5fa;
  }

  .cond-field {
    color: #cbd5e1;
  }

  .cond-op {
    color: #f59e0b;
  }

  .cond-val {
    color: #34d399;
  }

  .cond-delay {
    margin-left: auto;
    color: var(--text-dim);
    font-size: 10px;
  }

  .form-section-title {
    font-size: 11px;
    font-weight: 700;
    color: #60a5fa;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-top: 4px;
  }

  .form-row-3 {
    display: grid;
    grid-template-columns: 140px 1fr 150px;
    gap: 10px;
  }

  .form-row-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .mono {
    font-family: var(--font-mono);
  }
</style>
