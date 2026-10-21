import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import { test } from "node:test";
import { inlineReference } from "../../docs/scripts/rules/inline-reference.ts";

const root = resolve(import.meta.dirname, "../..");
const alias = '<span id="設定できる-id-現在の-sfc-検出なし"></span>';
const codeBlocks = (source: string) =>
  [...source.matchAll(/^```[^\n]*\n([\s\S]*?)^```$/gm)].map((match) => match[1]);

test("combined Vue examples exclude standalone configuration bookmarks and retain every source", () => {
  const directory = resolve(root, "docs/content/ja/rules/reference");
  const files = readdirSync(directory).filter((file) => file.startsWith("vue-"));
  assert.equal(files.length, 108);
  let standaloneAliases = 0;
  for (const file of files) {
    const source = readFileSync(resolve(directory, file), "utf8");
    const id = source.match(/^# `([^`]+)`/m)?.[1];
    assert(id);
    standaloneAliases += source.split(alias).length - 1;
    const combined = inlineReference(source, id);
    assert(!combined.includes(alias), `${id}: bookmark belongs to its standalone page`);
    assert.deepEqual(codeBlocks(combined), codeBlocks(source), `${id}: complete original fences`);
    const slug = file.replace(/\.md$/, "");
    for (const section of ["bad", "good"])
      assert.equal(combined.split(`<span id="${slug}-${section}"></span>`).length, 2);
  }
  assert.equal(standaloneAliases, 1, "the remaining unsupported bookmark stays authored");
  const category = readFileSync(resolve(root, "docs/content/generated/rules/ja/vue.md"), "utf8");
  assert(
    !category.includes(alias),
    "the complete generated category has no repeated standalone ID",
  );
});

test("literal bookmark markup inside a complete fenced example is preserved", () => {
  const reference = [
    "---",
    "title: fenced bookmark",
    "---",
    "",
    "# `vue/example`",
    "",
    alias,
    "",
    "## 設定できる ID（現在の SFC lint では診断されません）",
    "",
    "```html",
    alias,
    "```",
    "",
    "## 悪い",
    "",
    "```vue",
    "<template>bad</template>",
    "```",
    "",
    "## 良い",
    "",
    "```vue",
    "<template>good</template>",
    "```",
    "",
  ].join("\n");
  const combined = inlineReference(reference, "vue/example");
  assert.deepEqual(codeBlocks(combined), codeBlocks(reference));
  const outsideCode = combined.replace(/^```[^\n]*\n[\s\S]*?^```$/gm, "");
  assert(!outsideCode.includes(alias), "only the ineligible prose bookmark is removed");
  assert.equal(combined.split(alias).length, 2, "the literal source example remains complete");
});
