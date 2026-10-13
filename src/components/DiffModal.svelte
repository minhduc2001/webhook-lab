<script lang="ts">
  import type { WebhookRequest } from '../types';
  import { diffMaps, diffObjects, diffLines, type DiffRow } from '../lib/diff';
  import { parseBody, formatDateTime } from '../lib/utils';
  import Icons from './Icons.svelte';

  let { a, b, onClose, onSwap } = $props<{
    a: WebhookRequest;
    b: WebhookRequest;
    onClose: () => void;
    onSwap: () => void;
  }>();

  let onlyChanges = $state(true);

  function metaMap(r: WebhookRequest): Map<string, string> {
    return new Map([
      ['method', r.method],
      ['path', r.path],
      ['content_type', r.content_type],
      ['response_status', String(r.response_status)],
      ['response_time_ms', `${r.response_time_ms} ms`],
      ['ip_address', r.ip_address],
      ['matched_rule', r.matched_rule_name ?? '—'],
    ]);
  }

  function lowerKeys(h: Record<string, string> = {}): Record<string, string> {
    return Object.fromEntries(Object.entries(h).map(([k, v]) => [k.toLowerCase(), v]));
  }

  let bodyA = $derived(parseBody(a.body, a.content_type));
  let bodyB = $derived(parseBody(b.body, b.content_type));
  let structuredBody = $derived(bodyA.kind !== 'text' && bodyB.kind !== 'text');

  let sections = $derived<{ title: string; rows: DiffRow[] }[]>([
    { title: 'Tổng quan', rows: diffMaps(metaMap(a), metaMap(b)) },
    { title: 'Headers', rows: diffObjects(lowerKeys(a.headers), lowerKeys(b.headers)) },
    { title: 'Query', rows: diffObjects(a.query_params, b.query_params) },
    ...(structuredBody ? [{ title: 'Body', rows: diffObjects(bodyA.value, bodyB.value) }] : []),
  ]);
  let lineDiff = $derived(structuredBody ? [] : diffLines(a.body, b.body));
  let lineChanges = $derived(lineDiff.filter((l) => l.kind !== 'same').length);

  const changedCount = (rows: DiffRow[]) => rows.filter((r) => r.kind !== 'same').length;
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onClose()} />

<div class="modal-overlay" onclick={onClose} role="presentation">
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-card diff-card" onclick={(e) => e.stopPropagation()}>
    <div class="modal-header">
      <span class="modal-title">So Sánh Request</span>
      <button class="btn-close" onclick={onClose} aria-label="Đóng">
        <Icons name="x" size={16} />
      </button>
    </div>

    <div class="diff-legend">
      <div class="diff-side side-a">
        <span class="side-tag">A</span>
        <span class="mono">{a.method} {a.path}</span>
        <span class="dim">{formatDateTime(a.timestamp, false)}</span>
      </div>
      <button class="btn-swap" onclick={onSwap} title="Đổi chỗ A ↔ B">⇄</button>
      <div class="diff-side side-b">
        <span class="side-tag">B</span>
        <span class="mono">{b.method} {b.path}</span>
        <span class="dim">{formatDateTime(b.timestamp, false)}</span>
      </div>
      <label class="only-changes">
        <input type="checkbox" bind:checked={onlyChanges} />
        Chỉ hiện khác biệt
      </label>
    </div>

    <div class="modal-body diff-body">
      {#each sections as section}
        {@const changes = changedCount(section.rows)}
        {@const rows = onlyChanges ? section.rows.filter((r) => r.kind !== 'same') : section.rows}
        <div class="diff-section">
          <div class="diff-section-title">
            {section.title}
            <span class="diff-badge" class:clean={changes === 0}>
              {changes === 0 ? 'giống nhau' : `${changes} khác biệt`}
            </span>
          </div>
          {#if rows.length > 0}
            <table class="diff-table">
              <tbody>
                {#each rows as row}
                  <tr class="row-{row.kind}">
                    <td class="diff-path">{row.path}</td>
                    <td class="diff-val val-a">{row.a ?? ''}</td>
                    <td class="diff-val val-b">{row.b ?? ''}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        </div>
      {/each}

      {#if !structuredBody}
        <div class="diff-section">
          <div class="diff-section-title">
            Body (theo dòng)
            <span class="diff-badge" class:clean={lineChanges === 0}>
              {lineChanges === 0 ? 'giống nhau' : `${lineChanges} dòng khác`}
            </span>
          </div>
          {#if lineChanges > 0 || !onlyChanges}
            <pre class="code-box line-diff">{#each lineDiff as line}{#if !onlyChanges || line.kind !== 'same'}<div class="line-{line.kind}"><span class="line-sign">{line.kind === 'added' ? '+' : line.kind === 'removed' ? '-' : ' '}</span>{line.text}</div>{/if}{/each}</pre>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .diff-card {
    max-width: 980px;
    width: 94%;
    height: 85vh;
  }

  .btn-close {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 3px;
    border-radius: 4px;
  }

  .btn-close:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-main);
  }

  .diff-legend {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    border-bottom: 1px solid var(--border-subtle);
    font-size: 11px;
    flex-wrap: wrap;
  }

  .diff-side {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .side-tag {
    font-weight: 700;
    font-size: 10px;
    padding: 0 5px;
    border-radius: 3px;
  }

  .side-a .side-tag {
    background: rgba(239, 68, 68, 0.15);
    color: #fca5a5;
  }

  .side-b .side-tag {
    background: rgba(16, 185, 129, 0.15);
    color: #86efac;
  }

  .mono {
    font-family: var(--font-mono);
    color: var(--text-main);
  }

  .dim {
    color: var(--text-dim);
  }

  .btn-swap {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    color: var(--text-muted);
    cursor: pointer;
    padding: 1px 6px;
  }

  .btn-swap:hover {
    color: var(--text-main);
  }

  .only-changes {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 4px;
    color: var(--text-muted);
    cursor: pointer;
  }

  .diff-body {
    flex: 1;
  }

  .diff-section-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    margin-bottom: 4px;
  }

  .diff-badge {
    font-size: 10px;
    font-weight: 600;
    text-transform: none;
    padding: 0 5px;
    border-radius: 3px;
    background: rgba(245, 158, 11, 0.15);
    color: #fbbf24;
  }

  .diff-badge.clean {
    background: rgba(255, 255, 255, 0.05);
    color: var(--text-dim);
  }

  .diff-table {
    width: 100%;
    border-collapse: collapse;
    table-layout: fixed;
    font-family: var(--font-mono);
    font-size: 11px;
  }

  .diff-table td {
    padding: 3px 8px;
    border-bottom: 1px solid var(--border-subtle);
    vertical-align: top;
    word-break: break-all;
  }

  .diff-path {
    width: 28%;
    color: #7dd3fc;
  }

  .diff-val {
    color: var(--text-muted);
  }

  .row-changed .val-a,
  .row-removed .val-a {
    background: rgba(239, 68, 68, 0.1);
    color: #fca5a5;
  }

  .row-changed .val-b,
  .row-added .val-b {
    background: rgba(16, 185, 129, 0.1);
    color: #86efac;
  }

  .row-same td {
    opacity: 0.6;
  }

  .line-diff {
    white-space: pre-wrap;
    padding: 6px 0;
  }

  .line-diff > div {
    padding: 0 10px;
  }

  .line-sign {
    display: inline-block;
    width: 14px;
    color: var(--text-dim);
    user-select: none;
  }

  .line-added {
    background: rgba(16, 185, 129, 0.1);
    color: #86efac;
  }

  .line-removed {
    background: rgba(239, 68, 68, 0.1);
    color: #fca5a5;
  }
</style>
