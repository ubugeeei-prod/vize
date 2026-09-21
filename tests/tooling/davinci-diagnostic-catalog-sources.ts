import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

// Readers for the TS-53 catalog check (davinci-diagnostic-catalog.test.ts):
// they enumerate diagnostic codes and translations from the Rust sources that
// define them, so the check never depends on a hand-kept list.

export const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
export const read = (...parts: string[]): string =>
  fs.readFileSync(path.join(repoRoot, ...parts), "utf8");

export const locales = ["en", "ja", "zh"] as const;
export type Entry = Record<(typeof locales)[number], string>;

const stringLiteral = String.raw`"(?:[^"\\]|\\.)*"`;

/** Decode a plain (non-raw) Rust string literal, quotes included. */
export function rustString(literal: string): string {
  return literal.slice(1, -1).replace(/\\(u\{([0-9a-fA-F]+)\}|.)/gu, (_, escape, hex) => {
    if (hex !== undefined) return String.fromCodePoint(Number.parseInt(hex, 16));
    const simple: Record<string, string> = { n: "\n", t: "\t", r: "\r", "0": "\0" };
    return simple[escape] ?? escape;
  });
}

/** The `(key, en, ja, zh)` tuples of a Rust translation table, in order. */
export function parseEntries(source: string): Array<[string, Entry]> {
  const tuple = new RegExp(
    String.raw`\(\s*(${stringLiteral})\s*,\s*(${stringLiteral})\s*,\s*(${stringLiteral})\s*,\s*(${stringLiteral})\s*,?\s*\)`,
    "gu",
  );
  return [...source.matchAll(tuple)].map((match) => [
    rustString(match[1]),
    { en: rustString(match[2]), ja: rustString(match[3]), zh: rustString(match[4]) },
  ]);
}

/** The shipped Rust-side catalog tables this lane owns, in registration order. */
export function catalogEntries(): Array<[string, Entry]> {
  return [
    "i18n_render.rs",
    "i18n_compiler.rs",
    "i18n_compiler_template.rs",
    "i18n_compiler_directive.rs",
  ].flatMap((file) => parseEntries(read("crates", "vize_carton", "src", file)));
}

/** The body of the first `{ … }` block after `anchor`. */
export function block(source: string, anchor: string): string {
  const start = source.indexOf(anchor);
  assert.notEqual(start, -1, `missing \`${anchor}\``);
  const open = source.indexOf("{", start);
  let depth = 0;
  for (let at = open; at < source.length; at += 1) {
    if (source[at] === "{") depth += 1;
    if (source[at] === "}") depth -= 1;
    if (depth === 0) return source.slice(open + 1, at);
  }
  throw new Error(`unterminated block after \`${anchor}\``);
}

/** The `Name::Variant` list between `anchor` and the next `];`. */
export function constList(source: string, anchor: string): string[] {
  const start = source.indexOf(anchor);
  assert.notEqual(start, -1, `missing \`${anchor}\``);
  return [...source.slice(start, source.indexOf("];", start)).matchAll(/Self::(\w+)/gu)].map(
    (match) => match[1],
  );
}

/** `Prefix::Variant => "text"` arms (a braced single-string arm included). */
export function stringArms(body: string, prefix: string): Map<string, string> {
  const arm = new RegExp(`${prefix}::(\\w+) =>\\s*\\{?\\s*(${stringLiteral})`, "gu");
  return new Map([...body.matchAll(arm)].map((match) => [match[1], rustString(match[2])]));
}

export interface Vocabulary {
  variants: string[];
  all: string[];
  keys: Map<string, string>;
  english: Map<string, string>;
}

/** The renderer's `Phrase` vocabulary, read from its source. */
export function parseVocabulary(source: string): Vocabulary {
  const variants = [...block(source, "pub enum Phrase").matchAll(/^\s*(\w+),$/gmu)].map(
    (match) => match[1],
  );
  return {
    variants,
    all: constList(source, "pub const ALL"),
    keys: stringArms(block(source, "pub const fn key"), "Self"),
    english: stringArms(block(source, "impl Catalog for EnglishCatalog"), "Phrase"),
  };
}

const placeholders = (text: string): string =>
  [...text.matchAll(/\{(\w+)\}/gu)]
    .map((match) => match[1])
    .sort()
    .join(",");

/** Every malformed table entry: duplicates, blanks, drifted placeholders. */
export function tableProblems(entries: Array<[string, Entry]>): string[] {
  const problems: string[] = [];
  const seen = new Set<string>();
  for (const [key, entry] of entries) {
    if (seen.has(key)) problems.push(`duplicate catalog key \`${key}\``);
    seen.add(key);
    for (const locale of locales) {
      if (entry[locale].trim() === "") problems.push(`\`${key}\` is empty in ${locale}`);
    }
    for (const locale of ["ja", "zh"] as const) {
      if (placeholders(entry[locale]) !== placeholders(entry.en)) {
        problems.push(`\`${key}\` in ${locale} does not use the en placeholders`);
      }
    }
  }
  return problems;
}

/** Every way the renderer vocabulary and the table disagree. */
export function vocabularyProblems(
  vocabulary: Vocabulary,
  entries: Array<[string, Entry]>,
): string[] {
  const problems: string[] = [];
  const table = new Map(entries);
  if (vocabulary.all.join(",") !== vocabulary.variants.join(",")) {
    problems.push("Phrase::ALL does not list every variant in declaration order");
  }
  for (const variant of vocabulary.variants) {
    const key = vocabulary.keys.get(variant);
    if (key === undefined) {
      problems.push(`Phrase::${variant} has no key`);
      continue;
    }
    const entry = table.get(key);
    if (entry === undefined) {
      problems.push(`Phrase::${variant} (\`${key}\`) is missing from the catalog`);
    } else if (vocabulary.english.get(variant) !== entry.en) {
      problems.push(`EnglishCatalog disagrees with the en catalog for \`${key}\``);
    }
  }
  return problems;
}

export interface CompilerCodes {
  variants: string[];
  all: string[];
  codes: Map<string, string>;
  messages: Map<string, string>;
}

/** `vize_relief::ErrorCode`: variants, `ALL`, `code()` and `message()`. */
export function parseCompilerCodes(): CompilerCodes {
  const errors = read("crates", "vize_relief", "src", "errors.rs");
  const codes = read("crates", "vize_relief", "src", "errors", "codes.rs");
  const variants = [...block(errors, "pub enum ErrorCode").matchAll(/^\s*(\w+) = \d+,$/gmu)].map(
    (match) => match[1],
  );
  return {
    variants,
    all: constList(codes, "pub const ALL"),
    codes: stringArms(block(codes, "pub const fn code"), "Self"),
    messages: stringArms(block(errors, "pub fn message"), "Self"),
  };
}

const kebab = (variant: string): string =>
  variant.replace(/(?<=[a-z0-9])(?=[A-Z])|(?<=[A-Z])(?=[A-Z][a-z])/gu, "-").toLowerCase();

/** Every compiler code without a stable name, a catalog entry or its English. */
export function compilerProblems(
  compiler: CompilerCodes,
  entries: Array<[string, Entry]>,
): string[] {
  const problems: string[] = [];
  const table = new Map(entries);
  if (compiler.all.join(",") !== compiler.variants.join(",")) {
    problems.push("ErrorCode::ALL does not list every variant in declaration order");
  }
  for (const variant of compiler.variants) {
    const code = compiler.codes.get(variant);
    if (code !== `compiler/${kebab(variant)}`) {
      problems.push(`ErrorCode::${variant} is not named \`compiler/${kebab(variant)}\``);
      continue;
    }
    for (const part of ["message", "help"]) {
      if (!table.has(`${code}.${part}`)) problems.push(`\`${code}.${part}\` is not catalogued`);
    }
    const english = table.get(`${code}.message`)?.en;
    if (english !== undefined && english !== compiler.messages.get(variant)) {
      problems.push(`\`${code}.message\` en differs from ErrorCode::message()`);
    }
  }
  return problems;
}
