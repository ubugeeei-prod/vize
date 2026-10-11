import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { test } from "node:test";
import {
  assertEditorialParagraph,
  editorialText,
} from "../../docs/scripts/ja-rule-editorial-render-assertions.ts";

const root = resolve(import.meta.dirname, "../..");
const read = (path: string) => readFileSync(resolve(root, path), "utf8");
const fixture = JSON.parse(read("tests/_fixtures/docs-ja-rule-editorial.json"));
const custody = JSON.parse(read("tests/_fixtures/docs-ja-rule-original-custody.json"));

test("all reviewed rule pairs preserve the original complete executable and graph sources", () => {
  assert.equal(custody.entries.length, 317);
  assert.equal(
    custody.entries.filter((entry: { path: string }) => entry.path.includes("/reference/")).length,
    251,
  );
  for (const entry of custody.entries) {
    for (const path of [entry.path, entry.englishPath]) {
      const source = read(path);
      assert(source.includes(`# \`${entry.id}\``), `${path}: original rule identity`);
      const fences = [...source.matchAll(/^```[^\n]*\n[\s\S]*?^```[^\n]*$/gm)].map(
        (match) => match[0],
      );
      assert.equal(
        createHash("sha256").update(fences.join("\0")).digest("hex"),
        entry.wholeCodeSha256,
        `${path}: original whole examples/configuration/graphs`,
      );
    }
    const facts = [
      ...read(entry.path).matchAll(
        /^(?:現在の対応|悪い例での診断|Current support|Bad diagnostic|Example qualification):[^\n]+/gm,
      ),
    ].map((match) => match[0]);
    assert.deepEqual(facts, entry.supportFacts, `${entry.id}: original support qualification`);
  }
});

test("whole paragraph witnesses reject original mistranslations and survive regeneration", () => {
  assert.equal(fixture.cases.length, 24);
  for (const entry of fixture.cases) {
    const section = entry.rule.startsWith("vize:") ? "project" : "reference";
    const slug = entry.rule.replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase();
    const source = read(`docs/content/ja/rules/${section}/${slug}.md`);
    assert(
      source.split("\n").includes(entry.after),
      `${entry.rule}: whole authored after paragraph`,
    );
    assert(
      !source.split("\n").includes(entry.before),
      `${entry.rule}: original bad paragraph removed`,
    );
    assertEditorialParagraph(editorialText(entry.after), entry);
    assert.throws(() => assertEditorialParagraph(editorialText(entry.before), entry));
  }
});

test("the original native project chapter bookmark survives the new Japanese label", () => {
  for (const file of ["docs/content/ja/rules/all.md", "docs/content/generated/rules/ja/all.md"])
    assert.equal(
      read(file).split('<span id="プロジェクト-ルールと-analyzer-契約-66"></span>').length,
      2,
    );
});
