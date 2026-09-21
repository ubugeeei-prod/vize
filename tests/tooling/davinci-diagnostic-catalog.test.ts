import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

// TS-53 (davinci-road/plan/test-suites.md), catalog half: every word the
// Davinci diagnostic renderer prints around producer text exists in en, ja
// and zh, and every renderer snapshot case is committed in all three locales.
// The codes are enumerated from the sources mechanically — the renderer's
// `Phrase` enum and the TS-53 case list — never from a hand-kept list, so a
// new phrase or case without its translations fails here. The rendering half
// is `cargo test -p vize_davinci --test diagnostic_render`.

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const read = (...parts: string[]): string => fs.readFileSync(path.join(repoRoot, ...parts), "utf8");

const locales = ["en", "ja", "zh"] as const;
type Entry = Record<(typeof locales)[number], string>;

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

/** The body of `name`'s first `{ … }` block after `anchor`. */
function block(source: string, anchor: string): string {
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
  const allStart = source.indexOf("pub const ALL");
  const all = [
    ...source.slice(allStart, source.indexOf("];", allStart)).matchAll(/Self::(\w+)/gu),
  ].map((match) => match[1]);
  const arms = (body: string, prefix: string) =>
    new Map(
      [...body.matchAll(new RegExp(`${prefix}::(\\w+) => (${stringLiteral})`, "gu"))].map(
        (match) => [match[1], rustString(match[2])] as const,
      ),
    );
  return {
    variants,
    all,
    keys: arms(block(source, "pub const fn key"), "Self"),
    english: arms(block(source, "impl Catalog for EnglishCatalog"), "Phrase"),
  };
}

const placeholders = (text: string): string =>
  [...text.matchAll(/\{(\w+)\}/gu)]
    .map((match) => match[1])
    .sort()
    .join(",");

/** Every way the vocabulary and the table disagree; empty when complete. */
export function catalogProblems(vocabulary: Vocabulary, entries: Array<[string, Entry]>): string[] {
  const problems: string[] = [];
  const table = new Map<string, Entry>();
  for (const [key, entry] of entries) {
    if (table.has(key)) problems.push(`duplicate catalog key \`${key}\``);
    table.set(key, entry);
    for (const locale of locales) {
      if (entry[locale].trim() === "") problems.push(`\`${key}\` is empty in ${locale}`);
    }
    for (const locale of ["ja", "zh"] as const) {
      if (placeholders(entry[locale]) !== placeholders(entry.en)) {
        problems.push(`\`${key}\` in ${locale} does not use the en placeholders`);
      }
    }
  }
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

const vocabularySource = read("crates", "vize_davinci", "src", "render", "catalog.rs");
const tableSource = read("crates", "vize_carton", "src", "i18n_render.rs");

test("TS-53: every renderer phrase is catalogued in en, ja and zh", () => {
  const vocabulary = parseVocabulary(vocabularySource);
  assert.deepEqual(vocabulary.variants, [
    "Error",
    "Warning",
    "Info",
    "Hint",
    "Help",
    "SuggestedFix",
  ]);
  assert.deepEqual(catalogProblems(vocabulary, parseEntries(tableSource)), []);
});

test("TS-53: the catalog check fails on a removed, emptied or drifted entry", () => {
  const vocabulary = parseVocabulary(vocabularySource);
  const entries = parseEntries(tableSource);
  const withoutHelp = entries.filter(([key]) => key !== "render.help");
  assert.deepEqual(catalogProblems(vocabulary, withoutHelp), [
    "Phrase::Help (`render.help`) is missing from the catalog",
  ]);

  const emptied = entries.map(([key, entry]): [string, Entry] =>
    key === "render.warning" ? [key, { ...entry, zh: " " }] : [key, entry],
  );
  assert.deepEqual(catalogProblems(vocabulary, emptied), ["`render.warning` is empty in zh"]);

  const drifted = entries.map(([key, entry]): [string, Entry] =>
    key === "render.summary" ? [key, { ...entry, ja: "{files}" }] : [key, entry],
  );
  assert.deepEqual(catalogProblems(vocabulary, drifted), [
    "`render.summary` in ja does not use the en placeholders",
  ]);

  const unlisted = { ...vocabulary, all: vocabulary.all.slice(1) };
  assert.deepEqual(catalogProblems(unlisted, entries), [
    "Phrase::ALL does not list every variant in declaration order",
  ]);
});

/** The TS-53 renderer case names, from the case list and its modules. */
export function renderCaseNames(): string[] {
  const dir = path.join("crates", "vize_davinci", "tests", "diagnostic_render");
  const cases = read(dir, "cases.rs");
  const listStart = cases.indexOf("pub const ALL: &[&Case] = &[");
  assert.notEqual(listStart, -1, "cases.rs declares the case list");
  const list = cases.slice(listStart, cases.indexOf("];", listStart));
  return [...list.matchAll(/&(\w+)::(\w+),/gu)].map(([, module, constant]) => {
    const source = read(dir, "cases", `${module}.rs`);
    const body = block(source, `pub const ${constant}: Case = Case`);
    const name = /name: ("[^"]+"),/u.exec(body);
    assert.ok(name, `${module}::${constant} has a name`);
    return rustString(name[1]);
  });
}

test("TS-53: every renderer case is committed in every locale, and nothing else is", () => {
  const names = renderCaseNames();
  assert.equal(new Set(names).size, names.length, "case names are unique");
  assert.equal(names.length, 14, "the parser reads the whole TS-53 case list");
  const expected = names
    .flatMap((name) => [...locales.map((locale) => `${name}.${locale}.txt`), `${name}.en.ansi`])
    .sort();
  const committed = fs
    .readdirSync(
      path.join(repoRoot, "crates", "vize_davinci", "tests", "snapshots", "diagnostic_render"),
    )
    .sort();
  assert.deepEqual(committed, expected);
});
