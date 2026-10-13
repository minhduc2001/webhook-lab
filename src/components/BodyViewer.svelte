<script lang="ts">
  import { tick } from 'svelte';
  import {
    parseBody,
    collectJsonMatches,
    countMatches,
    splitMatches,
    copyToClipboard,
  } from '../lib/utils';
  import Icons from './Icons.svelte';
  import JsonNode from './JsonNode.svelte';

  let {
    body,
    contentType = '',
    emptyText = '<Không có nội dung body>',
  } = $props<{
    body: string;
    contentType?: string;
    emptyText?: string;
  }>();

  let parsed = $derived(parseBody(body, contentType));
  let isStructured = $derived(parsed.kind !== 'text');

  let isRaw = $state(false);
  let copied = $state(false);
  // Mức mở rộng cây; tăng treeVersion để dựng lại cây với mức mới
  let expandLevel = $state(Infinity);
  let treeVersion = $state(0);

  let query = $state('');
  let currentMatch = $state(-1);
  let searchInput = $state<HTMLInputElement>();
  let scopeEl = $state<HTMLElement>();

  let showTree = $derived(isStructured && !isRaw);
  let matches = $derived(
    showTree
      ? collectJsonMatches(parsed.value, query)
      : { openPaths: new Set<string>(), count: countMatches(body, query) }
  );

  function setExpandLevel(level: number) {
    expandLevel = level;
    treeVersion++;
  }

  async function handleCopy() {
    if (await copyToClipboard(body)) {
      copied = true;
      setTimeout(() => (copied = false), 2000);
    }
  }

  function onQueryInput() {
    currentMatch = -1;
  }

  // Nhảy tới kết quả kế tiếp / trước đó
  async function gotoMatch(dir: 1 | -1) {
    await tick();
    const marks = scopeEl?.querySelectorAll<HTMLElement>('mark.hl');
    if (!marks || marks.length === 0) return;
    marks.forEach((m) => m.classList.remove('current'));
    currentMatch = (currentMatch + dir + marks.length) % marks.length;
    const m = marks[currentMatch];
    m.classList.add('current');
    m.scrollIntoView({ block: 'center', behavior: 'smooth' });
  }

  function onSearchKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      gotoMatch(e.shiftKey ? -1 : 1);
    } else if (e.key === 'Escape') {
      query = '';
      currentMatch = -1;
      searchInput?.blur();
    }
  }

  function onWindowKeydown(e: KeyboardEvent) {
    // Bỏ qua khi đang mở hộp thoại (ô tìm nằm phía sau)
    if (document.querySelector('.modal-overlay')) return;
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'f') {
      e.preventDefault();
      searchInput?.focus();
      searchInput?.select();
    }
  }
</script>

<svelte:window onkeydown={onWindowKeydown} />

<div class="body-viewer">
  <div class="tab-toolbar">
    <div class="tab-toolbar-left">
      <span class="content-type-badge">{contentType || 'text/plain'}</span>
      {#if parsed.kind === 'json'}
        <span class="json-valid-badge">JSON</span>
      {:else if parsed.kind === 'form'}
        <span class="json-valid-badge form">Form URL-encoded</span>
      {/if}
    </div>
    <div class="tab-toolbar-right">
      {#if body}
        <div class="bv-search" class:active={query}>
          <Icons name="search" size={11} color="#64748b" />
          <input
            bind:this={searchInput}
            bind:value={query}
            oninput={onQueryInput}
            onkeydown={onSearchKeydown}
            placeholder="Tìm (Ctrl+F)"
            spellcheck="false"
          />
          {#if query}
            <span class="bv-count" class:none={matches.count === 0}>
              {currentMatch >= 0 ? `${currentMatch + 1}/` : ''}{matches.count}
            </span>
            <button class="bv-nav" onclick={() => gotoMatch(-1)} title="Kết quả trước (Shift+Enter)">↑</button>
            <button class="bv-nav" onclick={() => gotoMatch(1)} title="Kết quả sau (Enter)">↓</button>
          {/if}
        </div>
      {/if}
      {#if showTree}
        <button class="btn-toggle-raw" onclick={() => setExpandLevel(Infinity)} title="Mở rộng tất cả">Mở hết</button>
        <button class="btn-toggle-raw" onclick={() => setExpandLevel(1)} title="Thu gọn về cấp đầu tiên">Thu gọn</button>
      {/if}
      {#if isStructured}
        <button class="btn-toggle-raw" onclick={() => (isRaw = !isRaw)}>
          {isRaw ? 'Định dạng đẹp' : 'Raw'}
        </button>
      {/if}
      {#if body}
        <button class="btn-copy-toolbar" onclick={handleCopy}>
          <Icons name={copied ? 'check' : 'copy'} size={12} />
          <span>{copied ? 'Đã chép' : 'Chép'}</span>
        </button>
      {/if}
    </div>
  </div>

  {#if !body}
    <div class="empty-inline">{emptyText}</div>
  {:else if showTree}
    <div class="code-box json-tree" bind:this={scopeEl}>
      {#key treeVersion}
        <JsonNode value={parsed.value} {expandLevel} {query} openPaths={matches.openPaths} />
      {/key}
    </div>
  {:else}
    <pre class="code-box" bind:this={scopeEl}><code>{#each splitMatches(body, query) as seg}{#if seg.hit}<mark class="hl">{seg.text}</mark>{:else}{seg.text}{/if}{/each}</code></pre>
  {/if}
</div>

<style>
  .json-tree {
    white-space: normal;
    word-break: normal;
  }

  .json-valid-badge.form {
    color: #fbbf24;
    background: rgba(245, 158, 11, 0.12);
    border-color: rgba(245, 158, 11, 0.25);
  }

  .bv-search {
    display: flex;
    align-items: center;
    gap: 4px;
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 1px 6px;
  }

  .bv-search:focus-within,
  .bv-search.active {
    border-color: var(--border-focus);
  }

  .bv-search input {
    width: 96px;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-main);
    font-size: 11px;
    font-family: inherit;
    padding: 2px 0;
  }

  .bv-count {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .bv-count.none {
    color: #f87171;
  }

  .bv-nav {
    background: transparent;
    border: none;
    color: var(--text-dim);
    cursor: pointer;
    font-size: 11px;
    padding: 0 2px;
    line-height: 1;
  }

  .bv-nav:hover {
    color: var(--text-main);
  }
</style>
