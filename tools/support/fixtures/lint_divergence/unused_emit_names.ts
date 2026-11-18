/**
 * Patina's no-unused-emit-declarations reads every call of the defineEmits
 * binding. A literal or static template literal names its event, as it does
 * for eslint-plugin-vue. Patina also resolves a conditional between names, a
 * top-level `const` and a static key of a `const` object. Any other argument
 * computes the name, so Patina reports no event unused. eslint-plugin-vue
 * counts literals only and reports the rest.
 */
export const computedEmitReason =
  "The emit function is called with a computed event name (`emit(eventName)`), so Patina cannot prove any declared event unused and reports none; eslint-plugin-vue counts only literal event names.";
export const resolvedEmitReason =
  "Patina resolves this event name through a const, a const object key or a conditional between literals; eslint-plugin-vue counts only literal event names and reports the event as unused.";

const identifier = String.raw`[A-Za-z_$][\w$]*`;

/** Why Patina reports `event` as used where eslint-plugin-vue does not. */
export function unusedEmitPatinaReading(event: string, source: string): string | null {
  const script = scriptSetup(source);
  if (script === null || !/\bdefineEmits\s*(?:<|\()/u.test(script)) return null;
  const binding = new RegExp(
    `\\b(?:const|let|var)\\s+(${identifier})\\s*=\\s*defineEmits\\b`,
    "u",
  ).exec(script)?.[1];
  const consts = constNames(script);
  let computed = false;
  const resolved = new Set<string>();
  if (binding !== undefined) {
    for (const argument of callArguments(script, binding)) {
      const names = argumentNames(argument, consts);
      if (names === null) computed = true;
      else if (!literal(argument)) for (const name of names) resolved.add(name);
    }
  }
  const template = templateText(source);
  for (const callee of [...(binding === undefined ? [] : [binding]), "$emit"])
    for (const argument of callArguments(template, callee))
      if (argument !== "" && !/^['"]/u.test(argument)) computed = true;
  if (computed) return computedEmitReason;
  return resolved.has(event) ? resolvedEmitReason : null;
}

function scriptSetup(source: string): string | null {
  for (const open of source.matchAll(/<script\b([^>]*)>/gu)) {
    if (!/\ssetup(?:[\s=/]|$)/u.test(open[1] ?? "")) continue;
    const start = (open.index ?? 0) + open[0].length;
    const end = source.indexOf("</script>", start);
    return source
      .slice(start, end < 0 ? source.length : end)
      .replace(/\/\*[\s\S]*?\*\/|\/\/[^\n]*/gu, " ");
  }
  return null;
}

/** Directive values and interpolations, the only template text Patina reads. */
function templateText(source: string): string {
  const start = source.indexOf("<template");
  const end = source.lastIndexOf("</template>");
  if (start < 0 || end < start) return "";
  const template = source.slice(start, end);
  const parts = [
    ...template.matchAll(/\s(?:[:@#]|v-)[^\s=>]*\s*=\s*(?:"([^"]*)"|'([^']*)')/gu),
  ].map((match) => match[1] ?? match[2] ?? "");
  for (const match of template.matchAll(/\{\{([\s\S]*?)\}\}/gu)) parts.push(match[1] ?? "");
  return parts.join("\n");
}

/** The trimmed first-argument text of every `callee(...)` call. */
function callArguments(text: string, callee: string): string[] {
  const escaped = callee.replace(/\$/gu, "\\$");
  const found: string[] = [];
  for (const match of text.matchAll(
    new RegExp(`(?<![\\w$.])${escaped}\\s*(?:\\?\\.)?\\s*\\(`, "gu"),
  )) {
    let depth = 0,
      quote: string | null = null;
    const start = (match.index ?? 0) + match[0].length;
    let end = start;
    for (; end < text.length; end++) {
      const character = text[end];
      if (quote !== null) {
        if (character === "\\") end++;
        else if (character === quote) quote = null;
      } else if (character === "'" || character === '"' || character === "`") quote = character;
      else if (character === "(" || character === "[" || character === "{") depth++;
      else if ((character === ")" || character === "]" || character === "}") && depth-- === 0)
        break;
      else if (character === "," && depth === 0) break;
    }
    found.push(text.slice(start, end).trim());
  }
  return found;
}

function literal(argument: string): boolean {
  return /^(?:'[^'\\]*'|"[^"\\]*"|`[^`$\\]*`)$/u.test(argument);
}

/** The events an argument names, or `null` when Patina treats it as computed. */
function argumentNames(argument: string, consts: Map<string, string[]>): string[] | null {
  const text = argument
    .replace(/\s+as\s+const$/u, "")
    .replace(/^\((.*)\)$/su, "$1")
    .trim();
  if (text === "") return [];
  const quoted = /^(?:'([^'\\]*)'|"([^"\\]*)"|`([^`$\\]*)`)$/u.exec(text);
  if (quoted !== null) return [quoted[1] ?? quoted[2] ?? quoted[3] ?? ""];
  const conditional = /^[^?:'"`]+\?\s*([^:]+?)\s*:\s*(.+)$/su.exec(text);
  if (conditional !== null) {
    const left = argumentNames(conditional[1] ?? "", consts),
      right = argumentNames(conditional[2] ?? "", consts);
    return left === null || right === null ? null : [...left, ...right];
  }
  return consts.get(text) ?? null;
}

/** Top-level `const NAME = 'x'` and `const NAME = { key: 'x' }` names. */
function constNames(script: string): Map<string, string[]> {
  const consts = new Map<string, string[]>();
  // Unindented statements only: Patina resolves top-level consts.
  for (const match of script.matchAll(
    new RegExp(`^(?:export\\s+)?const\\s+(${identifier})\\s*=\\s*`, "gmu"),
  )) {
    const name = match[1] ?? "";
    const start = (match.index ?? 0) + match[0].length;
    const rest = script.slice(start);
    if (rest.startsWith("{")) {
      const close = rest.indexOf("}");
      if (close < 0) continue;
      for (const entry of rest
        .slice(1, close)
        .matchAll(
          new RegExp(`(${identifier}|'[^']*'|"[^"]*")\\s*:\\s*('[^'\\\\]*'|"[^"\\\\]*")`, "gu"),
        ))
        consts.set(`${name}.${(entry[1] ?? "").replace(/^['"]|['"]$/gu, "")}`, [
          (entry[2] ?? "").slice(1, -1),
        ]);
      continue;
    }
    const names = argumentNames(rest.split(/[;\n]/u)[0] ?? "", consts);
    if (names !== null && names.length > 0) consts.set(name, names);
  }
  return consts;
}
