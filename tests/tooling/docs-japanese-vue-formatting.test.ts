import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { repoRoot } from "./_helpers/tool-commands.ts";
import { CATALOGUE_SOURCES } from "../../docs/scripts/materialize-content.ts";
import { defaultNavigationRenderRoutes } from "../../docs/scripts/navigation-render-routes.ts";
import { createRenderJobs } from "../../docs/scripts/navigation-render-concurrency.ts";
import { verifyRenderedRulePackets } from "../../docs/scripts/rule-render-assertions.ts";

const route = "/ja/rules/vue-formatting";
const expected = [
  "vue/attribute-hyphenation",
  "vue/attribute-order",
  "vue/html-quotes",
  "vue/html-self-closing",
  "vue/mustache-interpolation-spacing",
  "vue/no-boolean-attr-value",
  "vue/no-inline-style",
  "vue/no-multi-spaces",
  "vue/prefer-props-shorthand",
  "vue/prop-name-casing",
  "vue/valid-attribute-name",
].toSorted();
const read = (file: string) => readFileSync(join(repoRoot, file), "utf8");
const fences = (text: string) =>
  [...text.matchAll(/^```(\w+)[^\n]*\n([\s\S]*?)^```[^\n]*$/gm)].map((match) => [
    match[1],
    match[2],
  ]);

await test("Japanese formatting guide preserves eleven complete reference bodies and forty-four examples", () => {
  const source = read("docs/content/generated/rules/ja/vue-formatting.md");
  const index = read("docs/content/ja/rules/vue-formatting.md");
  const headings = [...source.matchAll(/^### `([^`]+)`$/gm)];
  assert.deepEqual(
    headings.map((heading) => heading[1]),
    expected,
  );
  assert.equal(fences(source).length, 44);
  for (const [position, heading] of headings.entries()) {
    const name = heading[1];
    const id = name.replaceAll("/", "-");
    const packet = source.slice(heading.index, headings[position + 1]?.index);
    const reference = read(`docs/content/ja/rules/reference/${id}.md`);
    assert.deepEqual(fences(packet), fences(reference), name);
    assert.deepEqual(fences(packet), fences(read(`docs/content/rules/reference/${id}.md`)), name);
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
    for (const kind of ["bad", "good"]) {
      assert.equal(packet.split(`<span id="${id}-${kind}"></span>`).length, 2);
      assert.ok(source.includes(`](#${id}-${kind})`));
      assert.ok(index.includes(`](https://vizejs.dev${route}.html#${id}-${kind})`));
    }
  }
  for (const text of [index, source]) {
    assert.equal(text.split('<span id="vue-rules-template-formatting"></span>').length, 2);
    assert.match(text, /^# テンプレートの属性と表記を揃える$/m);
    assert.match(text, /Vue 3\.4 以降が対象/);
    assert.match(text, /採用するルールだけを掲載された設定で有効にし、`vp run lint`/);
    for (const target of [
      "./reference/vue-attribute-order.md",
      "./reference/vue-prefer-props-shorthand.md",
      "./options-vue.md",
      "./vue.md",
    ]) {
      assert.ok(text.includes(`](${target})`));
      assert.match(read(`docs/content/ja/rules/${target.slice(2)}`), /^# .+/m);
    }
  }
  assert.match(source, /不正な属性名は、この防御的なルールに届く前に parser\/template で検出/);
});

await test("Japanese formatting guide enrolls real native desktop/mobile packet checks", async () => {
  assert.equal(
    CATALOGUE_SOURCES.filter((file) => file === "generated/rules/ja/vue-formatting.md").length,
    1,
  );
  assert.equal(defaultNavigationRenderRoutes.filter((path) => path === route).length, 1);
  assert.deepEqual(
    createRenderJobs([route]).map((job) => job.device),
    ["desktop", "mobile"],
  );
  const nativeBoundary = new Error("Actual native DOM execution is required");
  let dispatched: { id: string }[] | undefined;
  const page = {
    evaluate: async (_read: unknown, packets: { id: string }[]) => {
      dispatched = packets;
      throw nativeBoundary;
    },
  } as unknown as Parameters<typeof verifyRenderedRulePackets>[0];
  await assert.rejects(verifyRenderedRulePackets(page, route), (error) => error === nativeBoundary);
  assert.deepEqual(
    dispatched?.map(({ id }) => id).toSorted(),
    expected.map((id) => id.replaceAll("/", "-")),
  );
});
