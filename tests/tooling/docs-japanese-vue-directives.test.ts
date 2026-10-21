import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { repoRoot } from "./_helpers/tool-commands.ts";
import { CATALOGUE_SOURCES } from "../../docs/scripts/materialize-content.ts";
import { defaultNavigationRenderRoutes } from "../../docs/scripts/navigation-render-routes.ts";
import { createRenderJobs } from "../../docs/scripts/navigation-render-concurrency.ts";
import { modifiedOnSpellings } from "../../tools/support/compat/davinci/lib/v-on-corpus.ts";

const route = "/ja/rules/vue-directives";
const generated = "docs/content/generated/rules/ja/vue-directives.md";
const read = (path: string) => readFileSync(join(repoRoot, path), "utf8");
const blocks = (text: string) =>
  [...text.matchAll(/^```(\w+)[^\n]*\n([\s\S]*?)^```[^\n]*$/gm)].map((match) => [
    match[1],
    match[2],
  ]);
const headings = [...read(generated).matchAll(/^### `([^`]+)`$/gm)];
const rules = headings.map((match) => match[1]);

await test("Japanese directive packets match their rule references and rule metadata", () => {
  const source = read(generated);
  const index = read("docs/content/ja/rules/vue-directives.md");
  assert.ok(rules.length > 0);
  const allRows = read("docs/content/rules/all.md").split("\n");
  for (const [position, heading] of headings.entries()) {
    const name = heading[1];
    const id = name.replaceAll("/", "-");
    const packet = source.slice(heading.index, headings[position + 1]?.index);
    const reference = read(`docs/content/ja/rules/reference/${id}.md`);
    assert.deepEqual(blocks(packet), blocks(reference), name);
    assert.deepEqual(blocks(packet), blocks(read(`docs/content/rules/reference/${id}.md`)), name);
    const path = reference.match(/\/blob\/main\/([^#]+)#L\d+/)?.[1];
    assert.ok(path, name);
    const metadata = read(path).match(/static META: RuleMeta = RuleMeta \{([\s\S]*?)\n\};/)?.[1];
    assert.ok(metadata, name);
    assert.ok(metadata.includes(`name: "${name}"`));
    const severity = metadata.match(/default_severity: Severity::(\w+)/)?.[1]?.toLowerCase();
    assert.ok(packet.includes(`既定の重大度: \`${severity}\``), name);
    assert.equal(
      packet.includes("自動修正: 対応している診断で利用できます"),
      metadata.includes("fixable: true"),
      name,
    );
    const row = allRows.find((line) => line.startsWith(`| [\`${name}\`](`));
    assert.ok(row, name);
    const cells = row.split(" | ");
    const column = cells.findIndex((cell) => /^`(?:warning|error)`$/.test(cell));
    assert.equal(packet.match(/^プリセット: (.+)$/m)?.[1].trim(), cells[column + 1], name);
    for (const kind of ["bad", "good"]) {
      assert.equal(packet.split(`<span id="${id}-${kind}"></span>`).length, 2);
      assert.ok(index.includes(`](https://vizejs.dev${route}.html#${id}-${kind})`));
    }
  }
  for (const text of [source, index])
    assert.equal(text.split('<span id="vue-rules-directive-conventions"></span>').length, 2);
});

await test("the directive page enrolls both viewports and its modified-event occurrences", () => {
  assert.equal(
    CATALOGUE_SOURCES.filter((path) => path === generated.slice("docs/content/".length)).length,
    1,
  );
  assert.equal(defaultNavigationRenderRoutes.filter((path) => path === route).length, 1);
  assert.deepEqual(
    createRenderJobs([route]).map((job) => job.device),
    ["desktop", "mobile"],
  );
  const spellings = modifiedOnSpellings(read(generated));
  const referenceSpellings = rules.flatMap((name) =>
    modifiedOnSpellings(read(`docs/content/ja/rules/reference/${name.replaceAll("/", "-")}.md`)),
  );
  assert.deepEqual(spellings, referenceSpellings);
  const committed = read("docs/davinci/plan/v-on-corpus/docs--content.tsv")
    .split("\n")
    .filter((line) => line.startsWith(`${generated}\t`))
    .map((line) => line.split("\t")[1]);
  assert.deepEqual(committed, spellings);
});
