import assert from "node:assert/strict";
import fs from "node:fs";
import { asRecord, asString } from "../../../tools/support/json-value.ts";

// Only these authored histories enter the seven current CLI expectation readers.
const pages = new Map<string, readonly [string, string]>([
  ["a11y/interactive-supports-focus", ["accessibility", "a11y-interactive-supports-focus"]],
  ["a11y/no-aria-hidden-on-focusable", ["accessibility", "a11y-no-aria-hidden-on-focusable"]],
  ["css/no-display-none", ["musea-and-css", "css-no-display-none"]],
  ["script/no-import-compiler-macros", ["type-and-script", "script-no-import-compiler-macros"]],
  [
    "script/no-required-prop-with-default",
    ["type-and-script", "script-no-required-prop-with-default"],
  ],
  ["script/no-reserved-keys", ["type-and-script", "script-no-reserved-keys"]],
  ["script/no-reserved-props", ["type-and-script", "script-no-reserved-props"]],
  ["script/return-in-emits-validator", ["type-and-script", "script-return-in-emits-validator"]],
  ["script/valid-define-emits", ["type-and-script", "script-valid-define-emits"]],
  ["vue/no-deprecated-scope-attribute", ["vue", "vue-no-deprecated-scope-attribute"]],
  ["vue/no-deprecated-slot-scope-attribute", ["vue", "vue-no-deprecated-slot-scope-attribute"]],
  ["vue/no-deprecated-v-bind-sync", ["vue", "vue-no-deprecated-v-bind-sync"]],
  ["vue/no-deprecated-v-on-native-modifier", ["vue", "vue-no-deprecated-v-on-native-modifier"]],
  ["vue/v-bind-style", ["vue", "vue-v-bind-style"]],
  ["vue/v-on-style", ["vue", "vue-v-on-style"]],
]);

export function currentRuleDocumentationMessage(expected: unknown): Record<string, unknown> {
  const message = asRecord(expected);
  const rule = asString(message.ruleId);
  const page = pages.get(rule);
  assert.ok(page, "finite authored documentation rule");
  assert.equal(
    message.ruleDocsPath,
    `docs/content/rules/${page[0]}.md`,
    "frozen rule documentation URL",
  );
  assert.ok(message.severity === 1 || message.severity === 2, "authored diagnostic severity");
  for (const locale of ["", "ja/"])
    assert.ok(
      fs.existsSync(
        new URL(`../../../docs/content/${locale}rules/reference/${page[1]}.md`, import.meta.url),
      ),
      "independent committed rule documentation page",
    );
  return { ...message, ruleDocsPath: `docs/content/rules/reference/${page[1]}.md` };
}

// Clone expected packets only. Actual output, historical packets and all other fields stay intact.
export function currentRuleDocumentationPacket(expected: unknown): Record<string, unknown>[] {
  assert.ok(Array.isArray(expected), "authored CLI packet array");
  return expected.map((entry: unknown) => {
    const row = asRecord(entry);
    assert.deepEqual(Object.keys(row).sort(), ["errorCount", "file", "messages", "warningCount"]);
    assert.equal(typeof row.file, "string");
    assert.ok(Array.isArray(row.messages));
    const messages = row.messages.map(currentRuleDocumentationMessage);
    assert.equal(row.errorCount, messages.filter((message) => message.severity === 2).length);
    assert.equal(row.warningCount, messages.filter((message) => message.severity === 1).length);
    return { ...row, messages };
  });
}
