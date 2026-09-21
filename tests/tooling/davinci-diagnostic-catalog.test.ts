import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  type Entry,
  block,
  catalogEntries,
  compilerProblems,
  locales,
  parseCompilerCodes,
  parseVocabulary,
  read,
  repoRoot,
  rustString,
  tableProblems,
  vocabularyProblems,
} from "./davinci-diagnostic-catalog-sources.ts";

// TS-53 (davinci-road/plan/test-suites.md), catalog half: every word the
// Davinci diagnostic renderer prints around producer text, and every
// diagnostic code a producer can emit, exists in en, ja and zh; every renderer
// snapshot case is committed in all three locales. Codes are enumerated from
// the producers' sources mechanically — never from a hand-kept list — so a new
// phrase, code or case without its translations fails here. The rendering
// half is `cargo test -p vize_davinci --test diagnostic_render`.

const vocabularySource = read("crates", "vize_davinci", "src", "render", "catalog.rs");

/** `entries` with `key`'s entry replaced by `edit(entry)`, or dropped. */
function edited(
  entries: Array<[string, Entry]>,
  key: string,
  edit: ((entry: Entry) => Entry) | null,
): Array<[string, Entry]> {
  return entries.flatMap(([name, entry]): Array<[string, Entry]> => {
    if (name !== key) return [[name, entry]];
    return edit === null ? [] : [[name, edit(entry)]];
  });
}

test("TS-53: every catalog entry is complete and well-formed in en, ja and zh", () => {
  assert.deepEqual(tableProblems(catalogEntries()), []);
});

test("TS-53: every renderer phrase is catalogued", () => {
  const vocabulary = parseVocabulary(vocabularySource);
  assert.deepEqual(vocabulary.variants, [
    "Error",
    "Warning",
    "Info",
    "Hint",
    "Help",
    "SuggestedFix",
  ]);
  assert.deepEqual(vocabularyProblems(vocabulary, catalogEntries()), []);
});

test("TS-53: every compiler error code is named and catalogued, English unchanged", () => {
  const compiler = parseCompilerCodes();
  assert.equal(compiler.variants.length, 56, "the parser reads the whole ErrorCode enum");
  assert.equal(compiler.messages.size, 56, "the parser reads every message() arm");
  assert.deepEqual(compilerProblems(compiler, catalogEntries()), []);
});

test("TS-53: the catalog check fails on a removed, emptied or drifted entry", () => {
  const vocabulary = parseVocabulary(vocabularySource);
  const compiler = parseCompilerCodes();
  const entries = catalogEntries();

  assert.deepEqual(vocabularyProblems(vocabulary, edited(entries, "render.help", null)), [
    "Phrase::Help (`render.help`) is missing from the catalog",
  ]);
  assert.deepEqual(
    tableProblems(edited(entries, "render.warning", (entry) => ({ ...entry, zh: " " }))),
    ["`render.warning` is empty in zh"],
  );
  assert.deepEqual(
    tableProblems(edited(entries, "render.summary", (entry) => ({ ...entry, ja: "{files}" }))),
    ["`render.summary` in ja does not use the en placeholders"],
  );
  assert.deepEqual(vocabularyProblems({ ...vocabulary, all: vocabulary.all.slice(1) }, entries), [
    "Phrase::ALL does not list every variant in declaration order",
  ]);

  const missingHelp = edited(entries, "compiler/v-if-same-key.help", null);
  assert.deepEqual(compilerProblems(compiler, missingHelp), [
    "`compiler/v-if-same-key.help` is not catalogued",
  ]);
  const reworded = edited(entries, "compiler/eof-in-tag.message", (entry) => ({
    ...entry,
    en: "Unexpected end of file in tag.",
  }));
  assert.deepEqual(compilerProblems(compiler, reworded), [
    "`compiler/eof-in-tag.message` en differs from ErrorCode::message()",
  ]);
  const renamed = new Map(compiler.codes).set("VShowNoExpression", "compiler/v-show-empty");
  assert.deepEqual(compilerProblems({ ...compiler, codes: renamed }, entries), [
    "ErrorCode::VShowNoExpression is not named `compiler/v-show-no-expression`",
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
