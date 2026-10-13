<script lang="ts">
  import { untrack } from 'svelte';
  import JsonNode from './JsonNode.svelte';
  import { copyToClipboard, toJsonPath, splitMatches, jsonLeafText, pathKey } from '../lib/utils';

  let {
    value,
    name = null,
    path = [],
    depth = 0,
    isLast = true,
    expandLevel = Infinity,
    query = '',
    openPaths = new Set<string>(),
  } = $props<{
    value: unknown;
    name?: string | number | null;
    path?: (string | number)[];
    depth?: number;
    isLast?: boolean;
    expandLevel?: number;
    query?: string;
    openPaths?: Set<string>;
  }>();

  let isArray = $derived(Array.isArray(value));
  let isContainer = $derived(value !== null && typeof value === 'object');
  let entries = $derived<[string | number, unknown][]>(
    isContainer
      ? isArray
        ? (value as unknown[]).map((v, i) => [i, v])
        : Object.entries(value as Record<string, unknown>)
      : []
  );
  let open = $state(untrack(() => depth < expandLevel));

  let brackets = $derived(isArray ? ['[', ']'] : ['{', '}']);
  let summary = $derived(
    isArray ? `${entries.length} items` : `${entries.length} keys`
  );
  let primitiveType = $derived(
    value === null ? 'null' : typeof value
  );
  let pathText = $derived(toJsonPath(path));

  // Khi tìm kiếm: tự mở node nếu bên trong có kết quả
  $effect(() => {
    if (query && openPaths.has(pathKey(path))) open = true;
  });

  let copied = $state<'path' | 'value' | null>(null);
  let copiedTimer: ReturnType<typeof setTimeout> | undefined;

  async function copy(kind: 'path' | 'value', text: string) {
    if (!(await copyToClipboard(text))) return;
    copied = kind;
    clearTimeout(copiedTimer);
    copiedTimer = setTimeout(() => (copied = null), 1200);
  }
</script>

{#snippet hl(text: string)}{#each splitMatches(text, query) as seg}{#if seg.hit}<mark class="hl">{seg.text}</mark>{:else}{seg.text}{/if}{/each}{/snippet}

<div class="jn">
  <div class="jn-line">
    {#if isContainer && entries.length > 0}
      <button class="jn-toggle" class:open onclick={() => (open = !open)} aria-label={open ? 'Thu gọn' : 'Mở rộng'}>
        <svg width="8" height="8" viewBox="0 0 8 8"><path d="M2 1l4 3-4 3z" fill="currentColor" /></svg>
      </button>
    {:else}
      <span class="jn-toggle-spacer"></span>
    {/if}

    {#if name !== null}
      <button
        class="jn-key"
        class:jn-index={typeof name === 'number'}
        onclick={() => copy('path', pathText)}
        title={`Copy đường dẫn: ${pathText}`}
      >{#if typeof name === 'number'}{name}{:else}"{@render hl(name)}"{/if}</button><span class="jn-punc">: </span>
    {/if}

    {#if isContainer}
      {#if entries.length === 0}
        <span class="jn-punc">{brackets[0]}{brackets[1]}</span>{#if !isLast}<span class="jn-punc">,</span>{/if}
      {:else if open}
        <span class="jn-punc">{brackets[0]}</span>
      {:else}
        <button class="jn-collapsed" onclick={() => (open = true)}>
          <span class="jn-punc">{brackets[0]}</span><span class="jn-ellipsis">…</span><span class="jn-punc">{brackets[1]}</span>
        </button>{#if !isLast}<span class="jn-punc">,</span>{/if}
        <span class="jn-count">{summary}</span>
      {/if}
    {:else}
      <button
        class="jn-val jn-{primitiveType}"
        onclick={() => copy('value', String(value))}
        title="Copy giá trị"
      >{@render hl(jsonLeafText(value))}</button>{#if !isLast}<span class="jn-punc">,</span>{/if}
    {/if}

    {#if copied}
      <span class="jn-copied">✓ đã copy {copied === 'path' ? pathText : 'giá trị'}</span>
    {/if}
  </div>

  {#if isContainer && entries.length > 0 && open}
    <div class="jn-children">
      {#each entries as [k, v], i (k)}
        <JsonNode
          value={v}
          name={k}
          path={[...path, k]}
          depth={depth + 1}
          isLast={i === entries.length - 1}
          {expandLevel}
          {query}
          {openPaths}
        />
      {/each}
    </div>
    <div class="jn-line">
      <span class="jn-toggle-spacer"></span>
      <span class="jn-punc">{brackets[1]}</span>{#if !isLast}<span class="jn-punc">,</span>{/if}
    </div>
  {/if}
</div>

<style>
  .jn-line {
    display: flex;
    align-items: baseline;
    flex-wrap: wrap;
    border-radius: 3px;
    padding-right: 4px;
  }

  .jn-line:hover {
    background: rgba(255, 255, 255, 0.035);
  }

  .jn-toggle,
  .jn-toggle-spacer {
    flex-shrink: 0;
    width: 14px;
    height: 14px;
    align-self: center;
  }

  .jn-toggle {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: 3px;
    color: var(--text-dim);
    cursor: pointer;
    transition: transform 0.12s, color 0.12s;
  }

  .jn-toggle:hover {
    color: var(--text-main);
    background: rgba(255, 255, 255, 0.08);
  }

  .jn-toggle.open {
    transform: rotate(90deg);
  }

  .jn-children {
    margin-left: 6px;
    padding-left: 10px;
    border-left: 1px solid rgba(255, 255, 255, 0.06);
  }

  .jn-children:hover {
    border-left-color: rgba(96, 165, 250, 0.25);
  }

  .jn-key,
  .jn-val {
    padding: 0;
    font: inherit;
    text-align: left;
    background: transparent;
    border: none;
    border-radius: 2px;
    cursor: pointer;
  }

  .jn-key:hover,
  .jn-val:hover {
    background: rgba(255, 255, 255, 0.08);
    text-decoration: underline dotted rgba(255, 255, 255, 0.3);
  }

  .jn-key {
    color: #7dd3fc;
  }

  .jn-key.jn-index {
    color: var(--text-dim);
  }

  .jn-copied {
    margin-left: 8px;
    font-size: 10px;
    color: #34d399;
    white-space: nowrap;
  }

  .jn-punc {
    color: #64748b;
    white-space: pre;
  }

  .jn-val {
    white-space: pre-wrap;
    word-break: break-all;
  }

  .jn-string {
    color: #86efac;
  }

  .jn-number,
  .jn-bigint {
    color: #fdba74;
  }

  .jn-boolean {
    color: #c4b5fd;
    font-weight: 600;
  }

  .jn-null,
  .jn-undefined {
    color: #94a3b8;
    font-style: italic;
  }

  .jn-collapsed {
    display: inline;
    padding: 0 3px;
    font: inherit;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-subtle);
    border-radius: 3px;
    cursor: pointer;
  }

  .jn-collapsed:hover {
    background: rgba(96, 165, 250, 0.12);
    border-color: rgba(96, 165, 250, 0.3);
  }

  .jn-ellipsis {
    color: var(--text-muted);
    margin: 0 2px;
  }

  .jn-count {
    margin-left: 8px;
    font-size: 10px;
    color: var(--text-dim);
    font-style: italic;
  }
</style>
