import assert from "node:assert/strict";
import { currentRuleDocumentationMessage } from "./current-rule-documentation.ts";

const pages = {
  "vue/html-self-closing": "vue-html-self-closing",
  "vue/component-name-in-template-casing": "vue-component-name-in-template-casing",
  "vue/v-slot-style": "vue-v-slot-style",
  "vue/no-boolean-attr-value": "vue-no-boolean-attr-value",
} as const;

// The authored #7905 oracle remains historical. Only current source reports use this clone.
export function currentBooleanStylePacket(expected: unknown): unknown[] {
  assert.ok(Array.isArray(expected));
  return expected.map((row: { messages: unknown[] }) => ({
    ...row,
    messages: row.messages.map((entry) => {
      assert.ok(entry !== null && typeof entry === "object");
      const message = entry as Record<string, unknown>;
      const rule = message.ruleId;
      assert.ok(
        typeof rule === "string" && Object.hasOwn(pages, rule),
        "authored boolean/style rule",
      );
      assert.equal(
        message.ruleDocsPath,
        "docs/content/rules/vue.md",
        "frozen boolean/style documentation",
      );
      return currentRuleDocumentationMessage(message);
    }),
  }));
}
