// So sánh hai request: phẳng hoá object thành path -> giá trị rồi đối chiếu
import { jsonLeafText, toJsonPath } from './utils';

export type DiffKind = 'same' | 'changed' | 'added' | 'removed';

export interface DiffRow {
  path: string;
  kind: DiffKind;
  a?: string;
  b?: string;
}

export function flatten(value: unknown, path: (string | number)[] = [], out = new Map<string, string>()) {
  if (value !== null && typeof value === 'object') {
    const entries: [string | number, unknown][] = Array.isArray(value)
      ? value.map((v, i) => [i, v])
      : Object.entries(value as Record<string, unknown>);
    if (entries.length === 0) {
      out.set(toJsonPath(path) || '(gốc)', Array.isArray(value) ? '[]' : '{}');
    }
    for (const [k, v] of entries) flatten(v, [...path, k], out);
  } else {
    out.set(toJsonPath(path) || '(gốc)', jsonLeafText(value));
  }
  return out;
}

export function diffMaps(a: Map<string, string>, b: Map<string, string>): DiffRow[] {
  const rows: DiffRow[] = [];
  for (const [path, av] of a) {
    if (!b.has(path)) rows.push({ path, kind: 'removed', a: av });
    else {
      const bv = b.get(path)!;
      rows.push({ path, kind: av === bv ? 'same' : 'changed', a: av, b: bv });
    }
  }
  for (const [path, bv] of b) {
    if (!a.has(path)) rows.push({ path, kind: 'added', b: bv });
  }
  return rows;
}

export function diffObjects(a: unknown, b: unknown): DiffRow[] {
  return diffMaps(flatten(a), flatten(b));
}

export interface LineDiff {
  kind: 'same' | 'added' | 'removed';
  text: string;
}

const MAX_LINES = 2000;

// Diff theo dòng bằng LCS (dùng khi body không phải JSON/form)
export function diffLines(aText: string, bText: string): LineDiff[] {
  const a = aText.split('\n').slice(0, MAX_LINES);
  const b = bText.split('\n').slice(0, MAX_LINES);
  const n = a.length;
  const m = b.length;
  const lcs: Uint32Array[] = Array.from({ length: n + 1 }, () => new Uint32Array(m + 1));
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      lcs[i][j] = a[i] === b[j] ? lcs[i + 1][j + 1] + 1 : Math.max(lcs[i + 1][j], lcs[i][j + 1]);
    }
  }
  const out: LineDiff[] = [];
  let i = 0;
  let j = 0;
  while (i < n && j < m) {
    if (a[i] === b[j]) {
      out.push({ kind: 'same', text: a[i] });
      i++;
      j++;
    } else if (lcs[i + 1][j] >= lcs[i][j + 1]) {
      out.push({ kind: 'removed', text: a[i++] });
    } else {
      out.push({ kind: 'added', text: b[j++] });
    }
  }
  while (i < n) out.push({ kind: 'removed', text: a[i++] });
  while (j < m) out.push({ kind: 'added', text: b[j++] });
  return out;
}
