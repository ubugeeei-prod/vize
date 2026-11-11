/**
 * Expressions-only projection: Vize's virtual TS with the Vue contract,
 * listener, slot, event and model typing rewritten to `any`, at the same
 * length in the session's position encoding (UTF-8 for Vize). Every name stays declared and every script and template
 * expression stays checked, so its check cost is the floor of checking the
 * authored expressions alone. Spans, links and directives that overlap a
 * rewritten statement are dropped.
 */

const TYPE_DECL =
  /^\s*type (__\w+_\d+__(listener_args|library|props|emit_props|fallthrough|props_members)\b|__\w+_\d+_[\w$]+_(prop_args|fallthrough_args|inferred_emit_args|static_emit_args|emit_args|args|event|return_native_fallthrough|return|prop_arrays|listener|handler)\b|__\w+_\d+_emit_resolver\b|__\w+_(Props|Component|CheckProps|CheckTail|ValueProps|FallthroughValue|Check)_\d+\b|__\w+_\d+_prop_\w+\b|__\w+_\d+_model_\w+\b|__vize_events_resolver_\d+\b|\$SlotContract_\d+\b)/u;
const CONST_DECL =
  /^\s*const (__vize_emit_props_\d+|__vize_event_keys_\d+|__vize_events_resolved_\d+|__vize_kebab_events_nav_\d+|__vize_props_nav_\d+|__vize_slots_nav_\d+|__vize_slot_host_\d+(?:_instances)?|__vize_required_slots_\d+|__vize_prop_check_\w+)\b/u;
const BLANK =
  /^\s*const \{ \} = undefined as unknown as \$ComponentNavigationProps|^\s*const \{ \w+: __vize_slot_nav_|^\s*void __vize_slot_nav_|^\s*\/\* __vize_template_hover \*\/|^\s*\/\/(?! @ts-| <reference| @vize-map)/u;

/** Bracket depth change of one line, quoted text skipped (linear, unlike a backtracking regex). */
function depthChange(line: string): number {
  let depth = 0;
  for (let i = 0; i < line.length; i++) {
    const char = line[i] as string;
    if (char === '"' || char === "'" || char === "`") {
      for (i++; i < line.length && line[i] !== char; i++) if (line[i] === "\\") i++;
    } else if ("([{".includes(char)) depth++;
    else if (")]}".includes(char)) depth--;
  }
  return depth;
}

/** Last line of the statement starting at `start`: brackets balanced and a closing `;`. */
function statementEnd(lines: string[], start: number): number {
  let depth = 0;
  for (let i = start; i < lines.length; i++) {
    depth += depthChange(lines[i] as string);
    if (depth <= 0 && /;\s*(\/\/.*)?$/u.test(lines[i] as string)) return i;
  }
  return start;
}

export type Encoding = "utf-8" | "utf-16";
const lengthIn = (encoding: Encoding) =>
  encoding === "utf-8" ? (text: string) => Buffer.byteLength(text) : (text: string) => text.length;
/** Spaces of the same length, newlines kept. */
const spaces = (text: string, length: (text: string) => number) =>
  text.replace(/[^\n]/gu, (char) => " ".repeat(length(char)));

/** Index just after `type NAME<...>` (balanced generics) or `const NAME`. */
function afterHead(text: string, head: RegExp): number {
  const match = head.exec(text) as RegExpExecArray;
  let i = match.index + match[0].length;
  if (text[i] !== "<") return i;
  for (let depth = 0; i < text.length; i++) {
    const char = text[i] as string;
    if ("<([{".includes(char)) depth++;
    else if (">)]}".includes(char) && --depth === 0) return i + 1;
  }
  return i;
}

/** Rewrite one statement, or null when its body is shorter than the replacement. */
function rewrite(
  statement: string,
  kind: "type" | "const",
  length: (text: string) => number,
): string | null {
  const head = afterHead(statement, kind === "type" ? /^\s*type [\w$]+/u : /^\s*const [\w$]+/u);
  const semi = statement.lastIndexOf(";");
  if (semi < head) return null;
  const name = statement.slice(0, head);
  const replacement =
    kind === "const"
      ? " = 0 as any"
      : /_(listener|handler)\s*$/u.test(name)
        ? " = (...a: any[]) => any"
        : /_prop_arrays\s*$/u.test(name)
          ? " = never"
          : " = any";
  const body = statement.slice(head, semi);
  if (length(body) < replacement.length) return null;
  return (
    name + replacement + spaces(body, length).slice(replacement.length) + statement.slice(semi)
  );
}

export function ablateText(
  text: string,
  encoding: Encoding = "utf-8",
): {
  text: string;
  ranges: [number, number][];
  counts: Record<string, number>;
} {
  const length = lengthIn(encoding);
  const lines = text.split("\n");
  const out = [...lines];
  const counts: Record<string, number> = {};
  const touched: [number, number][] = [];
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i] as string;
    const kind = TYPE_DECL.test(line)
      ? "type"
      : CONST_DECL.test(line)
        ? "const"
        : BLANK.test(line)
          ? "blank"
          : null;
    if (!kind) continue;
    const end =
      kind !== "blank"
        ? statementEnd(lines, i)
        : /__vize_template_hover/u.test(line)
          ? i + 1
          : /\/\//u.test(line)
            ? i
            : statementEnd(lines, i);
    const statement = lines.slice(i, end + 1).join("\n");
    const next = kind === "blank" ? spaces(statement, length) : rewrite(statement, kind, length);
    if (next == null) continue;
    counts[kind] = (counts[kind] ?? 0) + 1;
    next.split("\n").forEach((part, k) => (out[i + k] = part));
    touched.push([i, end]);
    i = end;
  }
  const starts: number[] = [];
  let offset = 0;
  for (const line of lines) {
    starts.push(offset);
    offset += length(line) + 1;
  }
  const ranges = touched.map(
    ([a, b]) =>
      [starts[a] as number, (starts[b] as number) + length(lines[b] as string)] as [number, number],
  );
  const result = out.join("\n");
  if (length(result) !== length(text)) throw new Error(`ablation changed the ${encoding} length`);
  return { text: result, ranges, counts };
}

const overlaps = (ranges: [number, number][], start: number, end: number) =>
  ranges.some(([a, b]) => start < b && end > a);

type Result = {
  text?: unknown;
  mappings?: number[][];
  semanticLinks?: { targetStart: number; targetLength: number }[];
  diagnosticDirectives?: { directives: number[][] };
};

/** Ablate one transform result in place; counts rewritten statements and dropped tuples. */
export function ablateResult(
  result: Record<string, unknown>,
  encoding: Encoding = "utf-8",
): Record<string, number> {
  const reply = result as Result;
  if (typeof reply.text !== "string") return {};
  const { text, ranges, counts } = ablateText(reply.text, encoding);
  reply.text = text;
  if (ranges.length === 0) return counts;
  const before =
    (reply.mappings?.length ?? 0) + (reply.diagnosticDirectives?.directives.length ?? 0);
  reply.mappings = reply.mappings?.filter(
    ([start = 0, length = 0]) => !overlaps(ranges, start, start + Math.max(1, length)),
  );
  if (reply.semanticLinks) {
    reply.semanticLinks = reply.semanticLinks.filter(
      (link) => !overlaps(ranges, link.targetStart, link.targetStart + link.targetLength),
    );
  }
  if (reply.diagnosticDirectives) {
    reply.diagnosticDirectives.directives = reply.diagnosticDirectives.directives.filter(
      ([, , start = 0, end = 0]) => !overlaps(ranges, start, Math.max(start + 1, end)),
    );
  }
  const after =
    (reply.mappings?.length ?? 0) + (reply.diagnosticDirectives?.directives.length ?? 0);
  return { ...counts, "dropped tuples": before - after };
}
