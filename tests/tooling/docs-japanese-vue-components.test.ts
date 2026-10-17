import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/tool-commands.ts";
import { CATALOGUE_SOURCES } from "../../docs/scripts/materialize-content.ts";
import { defaultNavigationRenderRoutes } from "../../docs/scripts/navigation-render-routes.ts";
import { createRenderJobs } from "../../docs/scripts/navigation-render-concurrency.ts";
import { verifyRenderedRulePackets } from "../../docs/scripts/rule-render-assertions.ts";

const route = "/ja/rules/vue-components";
const expected = [
  "vue/no-mutating-props",
  "vue/no-unused-components",
  "vue/no-unused-properties",
  "vue/require-component-is",
  "vue/component-definition-name-casing",
  "vue/component-name-in-template-casing",
  "vue/multi-word-component-names",
  "vue/no-non-component-keep-alive-child",
  "vue/no-reserved-component-names",
  "vue/require-component-registration",
].toSorted();
const read = (file: string) => readFileSync(join(repoRoot, file), "utf8");
const fences = (text: string) =>
  [...text.matchAll(/^```(\w+)[^\n]*\n([\s\S]*?)^```[^\n]*$/gm)].map((match) => [
    match[1],
    match[2],
  ]);
const urls = (text: string) =>
  [...text.replace(/^```[^\n]*\n[\s\S]*?^```[^\n]*$/gm, "").matchAll(/\[[^\]]*\]\(([^)]+)\)/g)].map(
    (match) => match[1],
  );

await test("Japanese components preserve ten whole reference bodies, defaults and all forty code blocks", () => {
  const source = read("docs/content/generated/rules/ja/vue-components.md");
  const index = read("docs/content/ja/rules/vue-components.md");
  const headings = [...source.matchAll(/^### `([^`]+)`$/gm)];
  assert.deepEqual(
    headings.map((match) => match[1]),
    expected,
  );
  assert.equal(fences(source).length, 40);
  const allRows = read("docs/content/rules/all.md").split("\n");
  for (const [position, heading] of headings.entries()) {
    const name = heading[1];
    const id = name.replaceAll("/", "-");
    const packet = source.slice(heading.index, headings[position + 1]?.index);
    const reference = read(`docs/content/ja/rules/reference/${id}.md`);
    assert.equal(fences(reference).length, 4);
    assert.deepEqual(fences(packet), fences(reference), `${name}: whole original copied code`);
    assert.deepEqual(fences(packet), fences(read(`docs/content/rules/reference/${id}.md`)));
    // Invert only the finite structural changes of an inline packet. Preserve
    // every internal Markdown byte, including hard breaks, filenames and prose.
    const restored = packet
      .replace(/^### /, "# ")
      .replaceAll(`<span id="${id}-bad"></span>\n\n`, "")
      .replaceAll(`<span id="${id}-good"></span>\n\n`, "")
      .replace(/^\*\*(設定（Vite\+）|悪い|良い)\*\*$/gm, "## $1")
      .replace(/^(```(?:ts|sh|vue)) annotate="(?:add|remove):\d+(?:,\d+)*"$/gm, "$1")
      .replaceAll(`](#${id}-bad)`, "](#悪い)")
      .replaceAll(`](#${id}-good)`, "](#良い)")
      .replaceAll("](options.md)", "](../options.md)")
      .replaceAll("](all.md)", "](../all.md)")
      .trim();
    assert.equal(restored, reference.replace(/^---\n[^\n]+\n---\n/, "").trim(), name);
    assert.deepEqual(
      urls(packet),
      urls(reference).map((target) =>
        target === "#悪い"
          ? `#${id}-bad`
          : target === "#良い"
            ? `#${id}-good`
            : target === "../options.md"
              ? "options.md"
              : target === "../all.md"
                ? "all.md"
                : target,
      ),
    );
    const implementation = reference.match(/\/blob\/main\/([^#]+)#L\d+/)?.[1];
    assert.ok(implementation);
    const metadata = read(implementation).match(
      /static META: RuleMeta = RuleMeta \{([\s\S]*?)\n\};/,
    )?.[1];
    assert.ok(metadata);
    assert.ok(metadata.includes(`name: "${name}"`));
    const severity = metadata
      .match(/default_severity: Severity::(Error|Warning)/)?.[1]
      ?.toLowerCase();
    assert.ok(severity);
    assert.ok(packet.includes(`既定の重大度: \`${severity}\``));
    assert.equal(
      packet.includes("自動修正: 対応している診断で利用できます"),
      metadata.includes("fixable: true"),
    );
    const row = allRows.find((line) => line.startsWith(`| [\`${name}\`](`));
    assert.ok(row);
    const cells = row.split(" | ");
    const severityColumn = cells.findIndex((cell) => /^`(?:error|warning)`$/.test(cell));
    assert.ok(severityColumn >= 0);
    assert.equal(reference.match(/^プリセット: (.+)$/m)?.[1].trim(), cells[severityColumn + 1]);
    for (const kind of ["bad", "good"]) {
      assert.equal(packet.split(`<span id="${id}-${kind}"></span>`).length, 2);
      assert.ok(source.includes(`](#${id}-${kind})`));
      assert.ok(index.includes(`](https://vizejs.dev${route}.html#${id}-${kind})`));
    }
  }
  for (const text of [index, source]) {
    assert.equal(text.split('<span id="vue-rules-components-and-props"></span>').length, 2);
    assert.equal([...text.matchAll(/^# コンポーネントと props のルール$/gm)].length, 1);
    assert.match(text, /親から受け取った値を書き換える代わりに、更新をイベントで親へ通知する/);
    assert.match(text, /掲載された設定でルールを有効にし、`vp run lint` を実行してください。/);
    for (const target of ["./reference/vue-no-mutating-props.md", "./vue.md"]) {
      assert.ok(text.includes(`](${target})`));
      assert.match(read(`docs/content/ja/rules/${target.slice(2)}`), /^# .+/m);
    }
  }
  for (const filename of ["myComponent.vue", "MyComponent.vue", "Item.vue", "TodoItem.vue"])
    assert.ok(source.split("\n").includes(`\`${filename}\``));
  assert.match(source, /子要素のタグ名は対象ではありません/);
  assert.match(source, /オプションだけではルールは有効になりません/);
  assert.match(source, /import や登録そのものは行いません/);
  assert.match(source, /標準要素がキャッシュされるという意味ではありません/);
});

await test("Japanese components enroll native packet execution without claiming source-only DOM success", async () => {
  assert.equal(
    CATALOGUE_SOURCES.filter((source) => source === "generated/rules/ja/vue-components.md").length,
    1,
  );
  assert.equal(defaultNavigationRenderRoutes.filter((source) => source === route).length, 1);
  assert.deepEqual(
    createRenderJobs([route]).map((job) => job.device),
    ["desktop", "mobile"],
  );
  const nativeBoundary = new Error("Actual native DOM execution is required");
  let dispatched: { id: string }[] | undefined;
  const page = {
    evaluate: async (_read: unknown, packet: { id: string }[]) => {
      dispatched = packet;
      throw nativeBoundary;
    },
  } as unknown as Parameters<typeof verifyRenderedRulePackets>[0];
  await assert.rejects(verifyRenderedRulePackets(page, route), (error) => error === nativeBoundary);
  assert.deepEqual(
    dispatched?.map(({ id }) => id).toSorted(),
    expected.map((id) => id.replaceAll("/", "-")),
  );
});
