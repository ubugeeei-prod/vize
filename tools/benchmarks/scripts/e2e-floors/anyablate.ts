// anyablate.ts: projection text -> same-byte-length text where Vue contract/listener/slot/event/model
// typing declarations are rewritten to `any` (names stay declared, expressions stay checked).
const TYPE_DECL = /^\s*type (__\w+_\d+__(listener_args|library|props|emit_props|fallthrough|props_members)\b|__\w+_\d+_[\w$]+_(prop_args|fallthrough_args|inferred_emit_args|static_emit_args|emit_args|args|event|return_native_fallthrough|return|prop_arrays|listener|handler)\b|__\w+_\d+_emit_resolver\b|__\w+_(Props|Component|CheckProps|CheckTail|ValueProps|FallthroughValue|Check)_\d+\b|__\w+_\d+_prop_\w+\b|__\w+_\d+_model_\w+\b|__vize_events_resolver_\d+\b|\$SlotContract_\d+\b)/;
const CONST_DECL = /^\s*const (__vize_emit_props_\d+|__vize_event_keys_\d+|__vize_events_resolved_\d+|__vize_kebab_events_nav_\d+|__vize_props_nav_\d+|__vize_slots_nav_\d+|__vize_slot_host_\d+(?:_instances)?|__vize_required_slots_\d+|__vize_prop_check_\w+)\b/;
const BLANK = /^\s*const \{ \} = undefined as unknown as \$ComponentNavigationProps|^\s*const \{ \w+: __vize_slot_nav_|^\s*void __vize_slot_nav_|^\s*\/\* __vize_template_hover \*\/|^\s*\/\/(?! @ts-| <reference| @vize-map)/;
function statementEnd(lines: string[], start: number): number {
  let depth = 0;
  for (let i = start; i < lines.length; i++) {
    const line = lines[i].replace(/(["'`])(?:\\.|(?!\1).)*\1/g, '""');
    for (const ch of line) { if ("([{".includes(ch)) depth++; else if (")]}".includes(ch)) depth--; }
    if (depth <= 0 && /[;}]\s*(\/\/.*)?$/.test(lines[i])) return i;
  }
  return start;
}
const spaces = (s: string) => s.replace(/[^\n]/gu, (c) => " ".repeat(Buffer.byteLength(c)));
// Index just after `type NAME<...>` (balanced generics) or `const NAME`.
function afterHead(text: string, re: RegExp): number {
  const m = re.exec(text)!; let i = m.index + m[0].length;
  if (text[i] === "<") { let d = 0; for (; i < text.length; i++) { const c = text[i]; if (c === "<" || c === "(" || c === "[" || c === "{") d++; else if (c === ">" || c === ")" || c === "]" || c === "}") { d--; if (d === 0) { i++; break; } } } }
  return i;
}
export function anyAblate(text: string): { text: string; ranges: [number, number][]; counts: Record<string, number> } {
  const lines = text.split("\n"); const counts: Record<string, number> = {};
  const out = [...lines]; const touched: [number, number][] = [];
  for (let i = 0; i < lines.length; i++) {
    const kind = TYPE_DECL.test(lines[i]) ? "type" : CONST_DECL.test(lines[i]) ? "const" : BLANK.test(lines[i]) ? "blank" : null;
    if (!kind) continue;
    const end = kind === "blank" && /\/\/|__vize_template_hover/.test(lines[i]) ? (/__vize_template_hover/.test(lines[i]) ? i + 1 : i) : statementEnd(lines, i);
    const stmt = lines.slice(i, end + 1).join("\n");
    let next: string;
    if (kind === "blank") next = spaces(stmt);
    else {
      const head = afterHead(stmt, kind === "type" ? /^\s*type [\w$]+/ : /^\s*const [\w$]+/);
      const semi = stmt.lastIndexOf(";"); if (semi < head) continue;
      const body = stmt.slice(head, semi); const fn = kind === "type" && /_(listener|handler)\s*$/.test(stmt.slice(0, head));
      const never = kind === "type" && /_prop_arrays\s*$/.test(stmt.slice(0, head));
      const repl = fn ? " = (...a: any[]) => any" : never ? " = never" : kind === "type" ? " = any" : " = 0 as any";
      if (Buffer.byteLength(body) < repl.length) continue;
      next = stmt.slice(0, head) + repl + spaces(body).slice(repl.length) + stmt.slice(semi);
    }
    counts[kind] = (counts[kind] ?? 0) + 1;
    const parts = next.split("\n"); for (let k = 0; k < parts.length; k++) out[i + k] = parts[k];
    touched.push([i, end]); i = end;
  }
  const lineStart: number[] = []; let off = 0; for (const l of lines) { lineStart.push(off); off += Buffer.byteLength(l) + 1; }
  const ranges = touched.map(([a, b]) => [lineStart[a], lineStart[b] + Buffer.byteLength(lines[b])] as [number, number]);
  const result = out.join("\n");
  if (Buffer.byteLength(result) !== Buffer.byteLength(text)) throw new Error("anyAblate changed byte length");
  return { text: result, ranges, counts };
}
