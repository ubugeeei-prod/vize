/**
 * The generated-name table of `davinci/vize_l0/src/ts_emit/generated_name.rs`, for
 * tooling that reads producer source text.
 *
 * Producers spell every generated name through that table: a format string
 * captures the constant (`{PROPS_NAV}{idx}`) and a literal is built with
 * `concat!(…, vize_l0::generated_name!(PROPS_NAV), …)`. These helpers turn
 * such source back into the text the producer writes without a hygiene
 * binding.
 */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const table = readFileSync(
  new URL("../../../davinci/vize_l0/src/ts_emit/generated_name.rs", import.meta.url),
  "utf8",
);

/** Constant name → spelling, e.g. `PROPS_NAV` → `$propsNav`. */
export const GENERATED_NAMES: ReadonlyMap<string, string> = new Map(
  [...table.matchAll(/^\s*\(([A-Z][A-Z0-9_]*)\) => \{\s*"([^"]+)"\s*\};$/gmu)].map((match) => [
    match[1]!,
    match[2]!,
  ]),
);
assert.ok(GENERATED_NAMES.size > 0, "generated name table");

/** The spelling of one constant. */
export function generatedName(name: string): string {
  const spelling = GENERATED_NAMES.get(name);
  assert.ok(spelling, `unknown generated name ${name}`);
  return spelling;
}

/** A producer format string with every `{CONST}` hole of the table spelled. */
export function spellHoles(template: string): string {
  return template.replace(
    /\{([A-Z][A-Z0-9_]*)\}/gu,
    (hole, name: string) => GENERATED_NAMES.get(name) ?? hole,
  );
}

/**
 * The value of a Rust string expression: a literal, or a `concat!` of
 * literals and `generated_name!` calls. Escapes stay as written.
 */
export function rustStringExpression(expression: string): string {
  const trimmed = expression.trim();
  const concat = /^concat!\(([\s\S]*)\)$/u.exec(trimmed);
  const parts = concat ? concat[1]! : trimmed;
  const pieces = [
    ...parts.matchAll(/"((?:[^"\\]|\\.)*)"|(?:vize_l0::)?generated_name!\(([A-Z][A-Z0-9_]*)\)/gu),
  ];
  assert.ok(pieces.length > 0, `Rust string expression: ${trimmed.slice(0, 80)}`);
  return pieces.map((piece) => (piece[2] ? generatedName(piece[2]) : piece[1]!)).join("");
}

/** The value of `const NAME: &str = <expression>;` in `source`. */
export function rustStringConst(source: string, name: string): string {
  const start = new RegExp(`const ${name}: &str =\\s*`, "u").exec(source);
  assert.ok(start, name);
  let index = start.index + start[0].length;
  let depth = 0;
  let inString = false;
  for (; index < source.length; index += 1) {
    const char = source[index];
    if (inString) {
      if (char === "\\") index += 1;
      else if (char === '"') inString = false;
      continue;
    }
    if (char === '"') inString = true;
    else if (char === "(") depth += 1;
    else if (char === ")") depth -= 1;
    else if (char === ";" && depth === 0) break;
  }
  return rustStringExpression(source.slice(start.index + start[0].length, index));
}
