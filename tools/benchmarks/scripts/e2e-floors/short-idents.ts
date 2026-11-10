// short-idents.ts: rename projection-declared internal identifiers (`__x`, `$X`) to `$` + base36 per file,
// outside every mapped span, semantic link and authored directive; never names on import/export lines.
// Experiment only: proves bytes and parse cost, and diagnostics identity on the corpora.
type Result = { text: string; mappings: number[][]; semanticLinks?: any[]; diagnosticDirectives?: { directives: number[][] } };
const declared = /(?:\bconst|\blet|\bvar|\btype|\binterface|\bfunction|\bclass)\s+((?:__|\$[A-Z])[\w$]*)/g;
export function shortenIdentifiers(r: Result): number {
  const text = r.text;
  const blocked: [number, number][] = [];
  for (const m of r.mappings) blocked.push([m[0], m[0] + Math.max(1, m[1])]);
  for (const l of r.semanticLinks ?? []) blocked.push([l.targetStart, l.targetStart + Math.max(1, l.targetLength)], [l.sourceStart, l.sourceStart + Math.max(1, l.sourceLength)]);
  for (const d of r.diagnosticDirectives?.directives ?? []) if (d[0] !== 0 || d[1] !== 0 || d[4] !== 0) blocked.push([d[2], Math.max(d[2] + 1, d[3])]);
  const buf = Buffer.from(text);
  const inBlocked = (s: number, e: number) => blocked.some(([a, b]) => a < e && s < b);
  const exposed = new Set<string>();
  for (const line of text.split("\n")) if (/^\s*(import|export)\b/.test(line)) for (const m of line.matchAll(/[\w$]+/g)) exposed.add(m[0]);
  const names = new Set<string>();
  for (const m of text.matchAll(declared)) if (!exposed.has(m[1]!)) names.add(m[1]!);
  if (!names.size) return 0;
  const words = new Set(text.match(/[\w$]+/g) ?? []);
  let counter = 0;
  const fresh = () => { for (;;) { const id = `$${(counter++).toString(36)}`; if (!words.has(id)) return id; } };
  const rename = new Map<string, string>();
  // Byte-accurate occurrences; a name with any blocked occurrence keeps its spelling.
  const occurrences = new Map<string, [number, number][]>();
  // A name also spelled after `.` or inside quotes (a key or a string) keeps its spelling.
  for (const m of text.matchAll(/[.'"`]((?:__|\$[A-Z])[\w$]*)/g)) names.delete(m[1]!);
  const pattern = /(?<![\w$])((?:__|\$[A-Z])[\w$]*)(?![\w$])/g;
  let byteAt = 0, charAt = 0;
  for (const m of text.matchAll(pattern)) {
    if (!names.has(m[1]!)) continue;
    byteAt += Buffer.byteLength(text.slice(charAt, m.index)); charAt = m.index!;
    const s = byteAt, e = s + Buffer.byteLength(m[1]!);
    const list = occurrences.get(m[1]!) ?? []; list.push([s, e]); occurrences.set(m[1]!, list);
  }
  const edits: [number, number, string][] = [];
  for (const [name, list] of occurrences) {
    if (list.some(([s, e]) => inBlocked(s, e))) continue;
    const short = fresh(); if (short.length >= name.length) continue;
    rename.set(name, short);
    for (const [s, e] of list) edits.push([s, e, short]);
  }
  if (!edits.length) return 0;
  edits.sort((a, b) => a[0] - b[0]);
  const starts = edits.map((e) => e[0]); const shift: number[] = [0];
  for (const [s, e, rep] of edits) shift.push(shift.at(-1)! + (e - s) - Buffer.byteLength(rep));
  const remap = (p: number) => { let lo = 0, hi = edits.length; while (lo < hi) { const mid = (lo + hi) >> 1; if (starts[mid]! < p) lo = mid + 1; else hi = mid; } const k = lo - 1; if (k < 0) return p; const [s, e] = edits[k]!; return p >= e ? p - shift[k + 1]! : s - shift[k]!; };
  const parts: Buffer[] = []; let last = 0;
  for (const [s, e, rep] of edits) { parts.push(buf.subarray(last, s), Buffer.from(rep)); last = e; }
  parts.push(buf.subarray(last));
  r.text = Buffer.concat(parts).toString();
  for (const m of r.mappings) { const s = remap(m[0]); m[1] = remap(m[0] + m[1]) - s; m[0] = s; }
  for (const l of r.semanticLinks ?? []) {
    const t = remap(l.targetStart); l.targetLength = remap(l.targetStart + l.targetLength) - t; l.targetStart = t;
    const s = remap(l.sourceStart); l.sourceLength = remap(l.sourceStart + l.sourceLength) - s; l.sourceStart = s;
  }
  if (r.diagnosticDirectives) r.diagnosticDirectives.directives = r.diagnosticDirectives.directives.map((d) => { const x = [...d]; x[2] = remap(d[2]!); x[3] = remap(d[3]!); return x; });
  return shift.at(-1)!;
}
