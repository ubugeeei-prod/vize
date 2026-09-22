import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  type Entry,
  block,
  catalogEntries,
  compilerProblems,
  legacyTranslations,
  locales,
  parseCompilerCodes,
  parseRules,
  parseVocabulary,
  read,
  repoRoot,
  ruleProblems,
  rustString,
  tableProblems,
  vocabularyProblems,
} from "./davinci-diagnostic-catalog-sources.ts";
import {
  canonProblems,
  parseCanonCodes,
  parseS3Codes,
  s3Problems,
} from "./davinci-diagnostic-catalog-producers.ts";

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
    "Why",
    "Because",
    "FactFallback",
    "UnusedBinding",
    "HtmlElement",
    "HtmlComposedNesting",
  ]);
  assert.deepEqual(vocabularyProblems(vocabulary, catalogEntries()), []);
});

test("TS-53: every compiler error code is named and catalogued, English unchanged", () => {
  const compiler = parseCompilerCodes();
  assert.equal(compiler.variants.length, 56, "the parser reads the whole ErrorCode enum");
  assert.equal(compiler.messages.size, 56, "the parser reads every message() arm");
  assert.deepEqual(compilerProblems(compiler, catalogEntries()), []);
});

test("TS-53: every one of the 249 lint rules has a description in every locale", () => {
  const rules = parseRules();
  assert.equal(rules.size, 249, "the parser reads every RuleMeta-family declaration");
  assert.deepEqual(ruleProblems(rules, legacyTranslations(), catalogEntries()), []);
});

test("TS-53: every Canon type-error code is catalogued in en, ja and zh", () => {
  const canon = parseCanonCodes();
  assert.equal(canon.variants.length, 22, "the parser reads the whole TypeErrorCode enum");
  const legacy = legacyTranslations();
  assert.deepEqual(canonProblems(canon, legacy), []);
  const dropped = new Map(legacy);
  dropped.delete("ts/2304.help");
  assert.deepEqual(canonProblems(canon, dropped), [
    "`ts/2304.help` has no en catalog text",
    "`ts/2304.help` has no ja catalog text",
    "`ts/2304.help` has no zh catalog text",
  ]);
});

test("TS-53: every S3 verifier code is catalogued in en, ja and zh", () => {
  const codes = parseS3Codes();
  assert.equal(codes.size, 10, "the parser reads every ViolationCode arm");
  const entries = catalogEntries();
  assert.deepEqual(s3Problems(codes, entries), []);
  const dropped = entries.filter(([key]) => key !== "s3/S3V001.message");
  assert.deepEqual(s3Problems(codes, dropped), ["`s3/S3V001.message` is not catalogued"]);
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

  const rules = parseRules();
  const legacy = legacyTranslations();
  const undescribed = edited(entries, "script/prefer-computed.description", null);
  assert.deepEqual(ruleProblems(rules, legacy, undescribed), [
    "`script/prefer-computed` has no description in en",
    "`script/prefer-computed` has no description in ja",
    "`script/prefer-computed` has no description in zh",
  ]);
  const drift = edited(entries, "vue/no-mutating-props.description", (entry) => ({
    ...entry,
    en: "Disallow mutating props",
  }));
  assert.deepEqual(ruleProblems(rules, legacy, drift), [
    "`vue/no-mutating-props.description` en differs from its RuleMeta",
  ]);
  const legacyWithoutJa = new Map(legacy).set("vue/require-v-for-key.description", {
    ...legacy.get("vue/require-v-for-key.description"),
    ja: "",
  });
  assert.deepEqual(ruleProblems(rules, legacyWithoutJa, entries), [
    "`vue/require-v-for-key` has no description in ja",
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

/** `=== <code>` headers of one locale's explain snapshot, in order. */
function explainCodes(locale: string): string[] {
  return read("crates", "vize", "src", "commands", "explain", "snapshots", `${locale}.txt`)
    .split("\n")
    .filter((line) => line.startsWith("=== "))
    .map((line) => line.slice(4));
}

test("TS-53: vize explain has one generated page per compiler code and lint rule", () => {
  const expected = [
    ...[...parseCompilerCodes().codes.values()].sort(),
    ...[...parseRules().keys()].sort(),
  ];
  const en = explainCodes("en");
  assert.deepEqual(en, expected);
  for (const locale of ["ja", "zh"] as const) {
    assert.deepEqual(explainCodes(locale), en, locale);
  }
  const dropped = en.filter((code) => code !== "vue/require-v-for-key");
  const missing = expected
    .filter((code) => !dropped.includes(code))
    .map((code) => `\`${code}\` has no explain page`);
  assert.deepEqual(missing, ["`vue/require-v-for-key` has no explain page"]);

  const page = read("crates", "vize", "src", "commands", "explain", "snapshots", "en.txt");
  const ruleAt = page.indexOf("=== vue/require-v-for-key\n");
  const rule = page.slice(ruleAt, page.indexOf("\n=== ", ruleAt + 1));
  assert.match(rule, /^tier: exact$/m);
  assert.match(rule, /^domain: elements and Vue directive syntax/m);
  assert.match(rule, /<li v-for="item in items">\{\{ item \}\}<\/li>/);
  const compilerAt = page.indexOf("=== compiler/v-if-no-expression\n");
  const compiler = page.slice(compilerAt, page.indexOf("\n=== ", compilerAt + 1));
  assert.match(compiler, /v-if\/v-else-if is missing expression/);
  assert.match(compiler, /give the condition, as in v-if="visible"/);
});

test("TS-53: witness-why snapshots for the P4-3c and P4-11b witnesses", () => {
  for (const name of ["witness_unused_binding", "witness_composed_nesting"]) {
    for (const locale of locales) {
      const text = read(
        "crates",
        "vize_davinci",
        "tests",
        "snapshots",
        "diagnostic_render",
        `${name}.${locale}.txt`,
      );
      const why = text.search(/= (?:note|根拠|依据): /u);
      const help = text.search(/= (?:help|ヒント|帮助): /u);
      assert.ok(why >= 0 && help > why, `${name}.${locale} note before help`);
    }
  }
  const composed = read(
    "crates",
    "vize_davinci",
    "tests",
    "snapshots",
    "diagnostic_render",
    "witness_composed_nesting.en.txt",
  );
  assert.equal(composed.split("\n").filter((line) => line.includes("= note:")).length, 2);
  const unused = read(
    "crates",
    "vize_davinci",
    "tests",
    "snapshots",
    "diagnostic_render",
    "witness_unused_binding.en.txt",
  );
  assert.match(unused, /because `total` is a <script setup> binding that nothing reads/);
  assert.match(composed, /because `<p` is the ancestor element this nesting proof cites/);
  assert.match(composed, /because `<InfoCard` is rendered where its parent forbids that child/);
});

test("TS-53: every renderer case is committed in every locale, and nothing else is", () => {
  const names = renderCaseNames();
  assert.equal(new Set(names).size, names.length, "case names are unique");
  assert.equal(names.length, 16, "the parser reads the whole TS-53 case list");
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
