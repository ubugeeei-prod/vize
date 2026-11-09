// compact-text.ts: drop leading indentation and non-directive whole-line `//` comment text from a
// transform result (newlines kept), outside every mapped span, semantic link and authored directive;
// remap all virtual offsets. Experiment only: no lexer, so a template literal in unmapped text would break.
type Result = { text: string; mappings: number[][]; semanticLinks?: any[]; diagnosticDirectives?: { directives: number[][] } };
export function compactResult(r: Result): { removed: number } {
  const buf = Buffer.from(r.text);
  const blocked: [number, number][] = [];
  for (const m of r.mappings) blocked.push([m[0], m[0] + Math.max(1, m[1])]);
  for (const l of r.semanticLinks ?? []) blocked.push([l.targetStart, l.targetStart + Math.max(1, l.targetLength)]);
  for (const d of r.diagnosticDirectives?.directives ?? []) if (d[0] !== 0 || d[1] !== 0 || d[4] !== 0) blocked.push([d[2], Math.max(d[2] + 1, d[3])]);
  blocked.sort((a, b) => a[0] - b[0]);
  // merge blocked
  const bl: [number, number][] = [];
  for (const b of blocked) { const last = bl.at(-1); if (last && b[0] <= last[1]) last[1] = Math.max(last[1], b[1]); else bl.push([b[0], b[1]]); }
  const isBlocked = (s: number, e: number) => { let lo = 0, hi = bl.length; while (lo < hi) { const mid = (lo + hi) >> 1; if (bl[mid][1] <= s) lo = mid + 1; else hi = mid; } return lo < bl.length && bl[lo][0] < e; };
  const cuts: [number, number][] = [];
  let off = 0;
  while (off < buf.length) {
    let end = buf.indexOf(10, off); if (end < 0) end = buf.length;
    let i = off; while (i < end && (buf[i] === 32 || buf[i] === 9)) i++;
    const rest = buf.subarray(i, end).toString();
    const isComment = rest.startsWith("//") && !rest.startsWith("///") && !rest.startsWith("// @ts-");
    const s = off, e = isComment ? end : i;
    if (e > s && !isBlocked(s, e)) cuts.push([s, e]);
    off = end + 1;
  }
  if (!cuts.length) return { removed: 0 };
  const starts = cuts.map((c) => c[0]); const prefix: number[] = [0];
  for (const [s, e] of cuts) prefix.push(prefix.at(-1)! + (e - s));
  const remap = (p: number) => { let lo = 0, hi = cuts.length; while (lo < hi) { const mid = (lo + hi) >> 1; if (starts[mid] < p) lo = mid + 1; else hi = mid; } const k = lo - 1; if (k < 0) return p; const [s, e] = cuts[k]; return p >= e ? p - prefix[k + 1] : s - prefix[k]; };
  const parts: Buffer[] = []; let last = 0;
  for (const [s, e] of cuts) { parts.push(buf.subarray(last, s)); last = e; }
  parts.push(buf.subarray(last));
  r.text = Buffer.concat(parts).toString();
  for (const m of r.mappings) { const s = remap(m[0]); m[1] = remap(m[0] + m[1]) - s; m[0] = s; }
  for (const l of r.semanticLinks ?? []) { const s = remap(l.targetStart); l.targetLength = remap(l.targetStart + l.targetLength) - s; l.targetStart = s; }
  if (r.diagnosticDirectives) {
    r.diagnosticDirectives.directives = r.diagnosticDirectives.directives.map((d) => { const x = [...d]; x[2] = remap(d[2]); x[3] = remap(d[3]); return x; }).filter((d) => !(d[0] === 0 && d[1] === 0 && d[4] === 0 && d[3] <= d[2]));
  }
  return { removed: prefix.at(-1)! };
}
