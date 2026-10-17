import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { repoRoot } from "./_helpers/moonbit.ts";
import { CATALOGUE_SOURCES } from "../../docs/scripts/materialize-content.ts";
import { defaultNavigationRenderRoutes } from "../../docs/scripts/navigation-render-routes.ts";
import { createRenderJobs } from "../../docs/scripts/navigation-render-concurrency.ts";
import { modifiedOnSpellings } from "../../tools/support/compat/davinci/lib/v-on-corpus.mjs";

const route = "/ja/rules/vue-directives";
const generated = "docs/content/generated/rules/ja/vue-directives.md";
const expected = [
  "vue/scoped-event-names",
  "vue/use-v-on-exact",
  "vue/v-bind-style",
  "vue/v-on-style",
  "vue/v-slot-style",
  "vue/warn-custom-directive",
].sort();
const read = (path: string) => readFileSync(join(repoRoot, path), "utf8");
const blocks = (text: string) =>
  [...text.matchAll(/^```(\w+)[^\n]*\n([\s\S]*?)^```[^\n]*$/gm)].map((match) => [
    match[1],
    match[2],
  ]);

await test("Japanese directive conventions retain six whole original packets and twenty-four source blocks", () => {
  const source = read(generated);
  const index = read("docs/content/ja/rules/vue-directives.md");
  const headings = [...source.matchAll(/^### `([^`]+)`$/gm)];
  assert.deepEqual(
    headings.map((match) => match[1]),
    expected,
  );
  assert.equal(blocks(source).length, 24);
  const allRows = read("docs/content/rules/all.md").split("\n");
  for (const [position, heading] of headings.entries()) {
    const name = heading[1];
    const id = name.replaceAll("/", "-");
    const packet = source.slice(heading.index, headings[position + 1]?.index);
    const original = read(`docs/content/ja/rules/reference/${id}.md`);
    assert.equal(blocks(original).length, 4);
    assert.deepEqual(blocks(packet), blocks(original));
    assert.deepEqual(blocks(packet), blocks(read(`docs/content/rules/reference/${id}.md`)));
    const restored = packet
      .replace(/^### /, "# ")
      .replaceAll(`<span id="${id}-bad"></span>\n\n`, "")
      .replaceAll(`<span id="${id}-good"></span>\n\n`, "")
      .replace(/^\*\*(設定（Vite\+）|悪い|良い|ハンドラーが重なる条件)\*\*$/gm, "## $1")
      .replace(/^(```(?:ts|sh|vue)) annotate="(?:add|remove):\d+(?:,\d+)*"$/gm, "$1")
      .replaceAll(`](#${id}-bad)`, "](#悪い)")
      .replaceAll(`](#${id}-good)`, "](#良い)")
      .replaceAll("](all.md)", "](../all.md)")
      .trim();
    assert.equal(restored, original.replace(/^---\n[^\n]+\n---\n/, "").trim(), name);
    const path = original.match(/\/blob\/main\/([^#]+)#L\d+/)?.[1];
    assert.ok(path);
    const metadata = read(path).match(/static META: RuleMeta = RuleMeta \{([\s\S]*?)\n\};/)?.[1];
    assert.ok(metadata);
    assert.ok(metadata.includes(`name: "${name}"`));
    assert.match(metadata, /default_severity: Severity::Warning/);
    assert.match(packet, /既定の重大度: `warning`/);
    assert.equal(
      packet.includes("自動修正: 対応している診断で利用できます"),
      metadata.includes("fixable: true"),
    );
    const row = allRows.find((line) => line.startsWith(`| [\`${name}\`](`));
    assert.ok(row);
    const cells = row.split(" | ");
    const severity = cells.findIndex((cell) => /^`(?:warning|error)`$/.test(cell));
    assert.ok(severity >= 0);
    assert.equal(original.match(/^プリセット: (.+)$/m)?.[1].trim(), cells[severity + 1]);
    for (const kind of ["bad", "good"]) {
      assert.equal(packet.split(`<span id="${id}-${kind}"></span>`).length, 2);
      assert.ok(index.includes(`](https://vizejs.dev${route}.html#${id}-${kind})`));
    }
  }
  for (const text of [source, index]) {
    assert.equal(text.split('<span id="vue-rules-directive-conventions"></span>').length, 2);
    assert.match(text, /^# ディレクティブとイベントの書き方$/m);
    assert.match(text, /採用するルールだけを掲載された設定で有効にし、`vp run lint`/);
    for (const target of ["./reference/vue-use-v-on-exact.md", "./vue.md"]) {
      assert.ok(text.includes(`](${target})`));
      assert.match(read(`docs/content/ja/rules/${target.slice(2)}`), /^# .+/m);
    }
  }
  assert.match(source, /emit 側も同じ名前に合わせます/);
  assert.match(source, /値の型ではなくディレクティブの書き方を検査する/);
  assert.match(source, /規約を無効にすれば、正しく登録したカスタムディレクティブは有効な Vue/);
});

await test("the actual directive page enrolls both viewports and its natural modified-event occurrences", () => {
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
  const originalSpellings = expected.flatMap((name) =>
    modifiedOnSpellings(read(`docs/content/ja/rules/reference/${name.replaceAll("/", "-")}.md`)),
  );
  assert.deepEqual(spellings, originalSpellings);
  const committed = read("docs/davinci/plan/v-on-corpus/docs--content.tsv")
    .split("\n")
    .filter((line) => line.startsWith(`${generated}\t`))
    .map((line) => line.split("\t")[1]);
  assert.deepEqual(committed, spellings);
  // This proves source ownership/enrollment only. Actual native DOM and public
  // source-bound reader qualification remain separate required gates.
});
