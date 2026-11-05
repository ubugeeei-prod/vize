import assert from "node:assert/strict";
import fs from "node:fs";
import { asRecord, asString } from "../../../tools/support/json-value.ts";

// Only these authored histories enter the current CLI expectation readers.
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
  ["css/no-id-selectors", ["musea-and-css", "css-no-id-selectors"]],
  ["css/no-utility-classes", ["musea-and-css", "css-no-utility-classes"]],
  ["html/no-empty-palpable-content", ["html", "html-no-empty-palpable-content"]],
  ["script/no-async-in-computed", ["type-and-script", "script-no-async-in-computed"]],
  ["script/define-props-destructuring", ["type-and-script", "script-define-props-destructuring"]],
  [
    "script/no-deep-destructure-in-props",
    ["type-and-script", "script-no-deep-destructure-in-props"],
  ],
  ["script/no-export-in-script-setup", ["type-and-script", "script-no-export-in-script-setup"]],
  ["script/no-options-api", ["type-and-script", "script-no-options-api"]],
  ["script/no-with-defaults", ["type-and-script", "script-no-with-defaults"]],
  ["script/prefer-import-from-vue", ["type-and-script", "script-prefer-import-from-vue"]],
  ["script/return-in-computed-property", ["type-and-script", "script-return-in-computed-property"]],
  ["type/no-reactivity-loss", ["type-and-script", "type-no-reactivity-loss"]],
  ["vue/component-name-in-template-casing", ["vue", "vue-component-name-in-template-casing"]],
  ["vue/html-self-closing", ["vue", "vue-html-self-closing"]],
  ["vue/no-boolean-attr-value", ["vue", "vue-no-boolean-attr-value"]],
  ["vue/no-deprecated-v-on-number-modifiers", ["vue", "vue-no-deprecated-v-on-number-modifiers"]],
  ["vue/no-deprecated-functional-template", ["vue", "vue-no-deprecated-functional-template"]],
  ["vue/no-unused-properties", ["vue", "vue-no-unused-properties"]],
  ["vue/no-unused-vars", ["vue", "vue-no-unused-vars"]],
  ["vue/require-component-registration", ["vue", "vue-require-component-registration"]],
  ["vue/require-scoped-style", ["vue", "vue-require-scoped-style"]],
  ["vue/sfc-element-order", ["vue", "vue-sfc-element-order"]],
  ["vue/require-v-for-key", ["vue", "vue-require-v-for-key"]],
  ["vue/v-slot-style", ["vue", "vue-v-slot-style"]],
  ["nuxt/nuxt-config-keys-order", ["index", "nuxt-nuxt-config-keys-order"]],
  ["vue/comment-directive", ["vue", ""]],
  ["cross-file", ["cross-file", ""]],
]);

function documentation(rule: string): { before: string; after: string } {
  const page = pages.get(rule);
  assert.ok(page, `finite authored documentation rule: ${rule}`);
  const target =
    rule === "vue/comment-directive"
      ? "guide/lint-execution.md"
      : rule === "cross-file"
        ? "rules/cross-file.md"
        : `rules/reference/${page[1]}.md`;
  for (const locale of ["", "ja/"])
    assert.ok(
      fs.existsSync(new URL(`../../../docs/content/${locale}${target}`, import.meta.url)),
      "independent committed rule documentation page",
    );
  return { before: `docs/content/rules/${page[0]}.md`, after: `docs/content/${target}` };
}

export function currentRuleDocumentationMessage(expected: unknown): Record<string, unknown> {
  const message = asRecord(expected);
  const link = documentation(asString(message.ruleId));
  assert.equal(message.ruleDocsPath, link.before, "frozen rule documentation URL");
  assert.ok(message.severity === 1 || message.severity === 2, "authored diagnostic severity");
  return { ...message, ruleDocsPath: link.after };
}

export function currentRuleDocumentationMessages(expected: unknown): Record<string, unknown>[] {
  assert.ok(Array.isArray(expected), "authored diagnostic array");
  return expected.map(currentRuleDocumentationMessage);
}

export function currentRuleDocumentationPlain(expected: string, rule?: string): string {
  if (rule === undefined) {
    let selected: string | undefined;
    return expected.split(/(?<=\n)/u).map((line) => {
      const finding = /^\s+\S+:\d+:\d+ (?:warning|error|info) (\S+) /u.exec(line);
      if (finding) selected = finding[1];
      const prefix = "    Reference: ";
      if (!line.startsWith(prefix)) return line;
      assert.ok(selected, "preceding authored diagnostic rule");
      const ending = line.endsWith("\r\n") ? "\r\n" : line.endsWith("\n") ? "\n" : "";
      const reference = line.slice(prefix.length, ending ? -ending.length : undefined);
      const link = documentation(selected);
      assert.equal(reference, link.before, "frozen plain documentation URL");
      return prefix + link.after + ending;
    }).join("");
  }
  const link = documentation(rule);
  return expected
    .split("\n")
    .map((line) => {
      if (!line.includes("Reference: ")) return line;
      assert.equal(line, `    Reference: ${link.before}`, "frozen plain documentation URL");
      return `    Reference: ${link.after}`;
    })
    .join("\n");
}

// Clone expected packets only. Actual output, historical packets and all other fields stay intact.
export function currentRuleDocumentationPacket(expected: unknown): {
  file: string;
  messages: Record<string, unknown>[];
  errorCount: number;
  warningCount: number;
}[] {
  assert.ok(Array.isArray(expected), "authored CLI packet array");
  return expected.map((entry: unknown) => {
    const row = asRecord(entry);
    assert.deepEqual(Object.keys(row).sort(), ["errorCount", "file", "messages", "warningCount"]);
    assert.ok(typeof row.file === "string");
    assert.ok(typeof row.errorCount === "number" && typeof row.warningCount === "number");
    assert.ok(Array.isArray(row.messages));
    const messages = row.messages.map(currentRuleDocumentationMessage);
    assert.equal(row.errorCount, messages.filter((message) => message.severity === 2).length);
    assert.equal(row.warningCount, messages.filter((message) => message.severity === 1).length);
    return {
      ...row,
      file: row.file,
      messages,
      errorCount: row.errorCount,
      warningCount: row.warningCount,
    };
  });
}

export function currentRuleDocumentationJson(expected: string): string {
  const packet: unknown = JSON.parse(expected);
  const ending = expected.endsWith("\n") ? "\n" : "";
  assert.equal(JSON.stringify(packet, null, 2) + ending, expected, "canonical authored JSON bytes");
  return JSON.stringify(currentRuleDocumentationPacket(packet), null, 2) + ending;
}
