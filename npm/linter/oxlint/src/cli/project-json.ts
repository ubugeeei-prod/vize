/** Locate top-level values in an already validated JSON document, retaining text. */
function values(text: string): Map<string, [number, number]> {
  const result = new Map<string, [number, number]>();
  let cursor = 0;
  const whitespace = () => {
    while (/\s/u.test(text[cursor] ?? "")) cursor++;
  };
  const string = () => {
    if (text[cursor++] !== '"') throw new Error("Invalid Oxlint JSON property.");
    while (cursor < text.length) {
      if (text[cursor] === "\\") cursor += 2;
      else if (text[cursor++] === '"') return;
    }
    throw new Error("Incomplete Oxlint JSON string.");
  };
  const value = () => {
    if (text[cursor] === '"') return string();
    if (text[cursor] === "[" || text[cursor] === "{") {
      let depth = 0;
      do {
        if (text[cursor] === '"') string();
        else {
          if (text[cursor] === "[" || text[cursor] === "{") depth++;
          if (text[cursor] === "]" || text[cursor] === "}") depth--;
          cursor++;
        }
      } while (depth > 0 && cursor < text.length);
      if (depth !== 0) throw new Error("Incomplete Oxlint JSON value.");
      return;
    }
    while (cursor < text.length && !/[\s,}]/u.test(text[cursor])) cursor++;
  };
  whitespace();
  if (text[cursor++] !== "{") throw new Error("Oxlint JSON report must be an object.");
  whitespace();
  while (text[cursor] !== "}") {
    const keyStart = cursor;
    string();
    const key = JSON.parse(text.slice(keyStart, cursor)) as string;
    if (result.has(key)) throw new Error("Duplicate Oxlint JSON report property.");
    whitespace();
    if (text[cursor++] !== ":") throw new Error("Invalid Oxlint JSON property separator.");
    whitespace();
    const start = cursor;
    value();
    result.set(key, [start, cursor]);
    whitespace();
    if (text[cursor] === "}") break;
    if (text[cursor++] !== ",") throw new Error("Invalid Oxlint JSON report separator.");
    whitespace();
  }
  return result;
}

/** Keep the original reporter layout and both diagnostic arrays' authored bytes. */
export function joinProjectJson(
  source: string,
  bridge: string,
  totals: Readonly<Record<string, number>>,
): string {
  const original = values(source);
  const template = values(bridge);
  const diagnostics = (text: string, fields: Map<string, [number, number]>) => {
    const span = fields.get("diagnostics");
    if (!span || text[span[0]] !== "[" || text[span[1] - 1] !== "]")
      throw new Error("Missing complete Oxlint diagnostics array.");
    return text.slice(span[0] + 1, span[1] - 1);
  };
  const left = diagnostics(source, original),
    right = diagnostics(bridge, template);
  const comma = left.trim() && right.trim() ? "," : "";
  const replacements = { ...totals, diagnostics: `[${left}${comma}${right}]` };
  const patches = Object.entries(replacements).map(([key, replacement]) => {
    if (typeof replacement === "number" && !Number.isFinite(replacement))
      throw new Error(`Invalid Oxlint JSON report total: ${key}`);
    const span = original.get(key);
    if (!span) throw new Error(`Missing original Oxlint JSON report property: ${key}`);
    return { start: span[0], end: span[1], value: String(replacement) };
  });
  for (const patch of patches.sort((left, right) => right.start - left.start))
    source = source.slice(0, patch.start) + patch.value + source.slice(patch.end);
  return source;
}
