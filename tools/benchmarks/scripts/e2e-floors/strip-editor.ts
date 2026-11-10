// strip-editor.ts: drop whole generated lines that exist only for editor features — navigation
// statements, and lines whose every span carries an editor-only feature mask (hover, signature,
// completion, navigation) — with their spans and directives; remap all other offsets.
// Experiment only: the corpora's diagnostics must stay identical for the cut to count.
type Result = { text: string; mappings: number[][]; semanticLinks?: any[]; diagnosticDirectives?: { directives: number[][] } };
const NAV = /\$ComponentNavigationProps|__vize_props_nav_|__vize_slot_nav_|^\s*const \{[^}]*\} = undefined as unknown as/u;
const ALL = (1 << 20) - 1;
// A whole statement on one line: ends with `;` and opens no bracket it does not close.
const standalone = (line: string) => {
  const t = line.trim(); if (!t.endsWith(";") || t.startsWith("}")) return false;
  const n = (c: string) => t.split(c).length - 1;
  return n("(") === n(")") && n("{") === n("}") && n("[") === n("]") && n("<") >= 0;
};
export function stripEditor(r: Result): number {
  const buf = Buffer.from(r.text);
  const text = r.text;
  const count = (name: string) => text.split(new RegExp(`(?<![\\w$])${name.replace(/\$/g, "\\$")}(?![\\w$])`)).length - 1;
  // Every name the line declares must be used nowhere else in the file.
  const unreferenced = (line: string) => {
    const declared = [...line.matchAll(/\b(?:const|let|var|type|function|interface)\s+([\w$]+)/g)].map((m) => m[1]!);
    const destructured = /\bconst \{([^}]*)\}/.exec(line)?.[1]?.split(",").map((part) => part.split(":").pop()!.trim()).filter(Boolean) ?? [];
    return [...declared, ...destructured].every((name) => count(name) <= 1);
  };
  const spans = [...r.mappings].sort((a, b) => a[0]! - b[0]!);
  const cuts: [number, number][] = [];
  let off = 0, k = 0;
  while (off < buf.length) {
    let end = buf.indexOf(10, off); if (end < 0) end = buf.length;
    while (k < spans.length && spans[k]![0]! < off) k++;
    const line = buf.subarray(off, end).toString();
    const inside: number[][] = [];
    for (let j = k; j < spans.length && spans[j]![0]! < end; j++) inside.push(spans[j]!);
    const editorOnly = inside.length > 0 && inside.every((m) => { const f = m[5] ?? ALL; return f !== 0 && f !== ALL && (f & ~0x1ff) === 0; });
    if ((NAV.test(line) || editorOnly) && standalone(line) && unreferenced(line)) cuts.push([off, Math.min(end + 1, buf.length)]);
    off = end + 1;
  }
  if (!cuts.length) return 0;
  const inCut = (s: number, e: number) => cuts.some(([a, b]) => a <= s && e <= b);
  r.mappings = r.mappings.filter((m) => !inCut(m[0]!, m[0]! + m[1]!));
  r.semanticLinks = (r.semanticLinks ?? []).filter((l) => !inCut(l.targetStart, l.targetStart + l.targetLength) && !inCut(l.sourceStart, l.sourceStart + l.sourceLength));
  if (r.diagnosticDirectives) r.diagnosticDirectives.directives = r.diagnosticDirectives.directives.filter((d) => !inCut(d[2]!, d[3]!));
  const starts = cuts.map((c) => c[0]); const prefix: number[] = [0];
  for (const [s, e] of cuts) prefix.push(prefix.at(-1)! + (e - s));
  const remap = (p: number) => { let lo = 0, hi = cuts.length; while (lo < hi) { const mid = (lo + hi) >> 1; if (starts[mid]! < p) lo = mid + 1; else hi = mid; } const q = lo - 1; if (q < 0) return p; const [s, e] = cuts[q]!; return p >= e ? p - prefix[q + 1]! : s - prefix[q]!; };
  const parts: Buffer[] = []; let last = 0;
  for (const [s, e] of cuts) { parts.push(buf.subarray(last, s)); last = e; }
  parts.push(buf.subarray(last));
  r.text = Buffer.concat(parts).toString();
  for (const m of r.mappings) { const s = remap(m[0]!); m[1] = remap(m[0]! + m[1]!) - s; m[0] = s; }
  for (const l of r.semanticLinks) { const t = remap(l.targetStart); l.targetLength = remap(l.targetStart + l.targetLength) - t; l.targetStart = t; const s = remap(l.sourceStart); l.sourceLength = remap(l.sourceStart + l.sourceLength) - s; l.sourceStart = s; }
  if (r.diagnosticDirectives) r.diagnosticDirectives.directives = r.diagnosticDirectives.directives.map((d) => { const x = [...d]; x[2] = remap(d[2]!); x[3] = remap(d[3]!); return x; }).filter((d) => !(d[0] === 0 && d[1] === 0 && d[4] === 0 && d[3]! <= d[2]!));
  return prefix.at(-1)!;
}
