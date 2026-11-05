import assert from "node:assert/strict";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { currentBooleanStylePacket } from "./support/current-boolean-style-documentation.ts";
import { loadCases, wholeJson } from "./boolean-attribute-fix-reference.mjs";

await test("current boolean/style expectations preserve the whole historical oracle", () => {
  const { cases } = loadCases(fileURLToPath(new URL("../../", import.meta.url)));
  const pages = new Map([
    ["vue/html-self-closing", "vue-html-self-closing"],
    ["vue/component-name-in-template-casing", "vue-component-name-in-template-casing"],
    ["vue/v-slot-style", "vue-v-slot-style"],
    ["vue/no-boolean-attr-value", "vue-no-boolean-attr-value"],
  ]);
  const original = currentBooleanStylePacket(wholeJson(cases[0], false, true));
  const seen = new Set<string>();
  for (const file of original) {
    assert.ok(
      file !== null &&
        typeof file === "object" &&
        "messages" in file &&
        Array.isArray(file.messages),
    );
    for (const message of file.messages) {
      const rule = String(message.ruleId);
      assert.ok(pages.has(rule));
      assert.equal(message.ruleDocsPath, `docs/content/rules/reference/${pages.get(rule)}.md`);
      seen.add(rule);
    }
  }
  assert.equal(seen.size, 4);
  for (const entry of cases) {
    for (const after of [false, true]) {
      const before = wholeJson(entry, after, entry.originalRules);
      const saved = JSON.stringify(before);
      const current = currentBooleanStylePacket(before);
      assert.equal(JSON.stringify(before), saved);
      assert.equal(
        JSON.stringify(current).replaceAll(
          /docs\/content\/rules\/reference\/vue-(?:html-self-closing|component-name-in-template-casing|v-slot-style|no-boolean-attr-value)\.md/gu,
          "docs/content/rules/vue.md",
        ),
        saved,
      );
    }
  }
  for (const ruleDocsPath of [
    undefined,
    "docs/content/rules/html.md",
    "docs/content/rules/reference/vue-no-boolean-attr-value.md",
  ]) {
    assert.throws(
      () =>
        currentBooleanStylePacket([
          { messages: [{ ruleId: "vue/no-boolean-attr-value", ruleDocsPath }] },
        ]),
      /frozen boolean\/style documentation/u,
    );
  }
  assert.throws(
    () =>
      currentBooleanStylePacket([
        { messages: [{ ruleId: "vue/foreign", ruleDocsPath: "docs/content/rules/vue.md" }] },
      ]),
    /authored boolean\/style rule/u,
  );
});
