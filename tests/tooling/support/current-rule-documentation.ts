import assert from "node:assert/strict";
import fs from "node:fs";

// Project this finite authored history only for current expectations, never actual output.
const pages = {
  "a11y/interactive-supports-focus": ["accessibility", "a11y-interactive-supports-focus"],
  "a11y/no-aria-hidden-on-focusable": ["accessibility", "a11y-no-aria-hidden-on-focusable"],
  "script/no-async-in-computed": ["type-and-script", "script-no-async-in-computed"],
  "script/define-props-destructuring": ["type-and-script", "script-define-props-destructuring"],
  "script/no-deep-destructure-in-props": ["type-and-script", "script-no-deep-destructure-in-props"],
  "script/no-export-in-script-setup": ["type-and-script", "script-no-export-in-script-setup"],
  "script/no-import-compiler-macros": ["type-and-script", "script-no-import-compiler-macros"],
  "script/no-options-api": ["type-and-script", "script-no-options-api"],
  "script/no-required-prop-with-default": [
    "type-and-script",
    "script-no-required-prop-with-default",
  ],
  "script/no-reserved-keys": ["type-and-script", "script-no-reserved-keys"],
  "script/no-reserved-props": ["type-and-script", "script-no-reserved-props"],
  "script/no-with-defaults": ["type-and-script", "script-no-with-defaults"],
  "script/prefer-import-from-vue": ["type-and-script", "script-prefer-import-from-vue"],
  "script/return-in-emits-validator": ["type-and-script", "script-return-in-emits-validator"],
  "script/valid-define-emits": ["type-and-script", "script-valid-define-emits"],
  "vue/comment-directive": ["vue", null],
  "vue/component-name-in-template-casing": ["vue", "vue-component-name-in-template-casing"],
  "vue/html-self-closing": ["vue", "vue-html-self-closing"],
  "vue/no-boolean-attr-value": ["vue", "vue-no-boolean-attr-value"],
  "vue/no-deprecated-scope-attribute": ["vue", "vue-no-deprecated-scope-attribute"],
  "vue/no-deprecated-slot-scope-attribute": ["vue", "vue-no-deprecated-slot-scope-attribute"],
  "vue/no-deprecated-v-bind-sync": ["vue", "vue-no-deprecated-v-bind-sync"],
  "vue/no-deprecated-v-on-native-modifier": ["vue", "vue-no-deprecated-v-on-native-modifier"],
  "vue/no-unused-properties": ["vue", "vue-no-unused-properties"],
  "vue/no-unused-vars": ["vue", "vue-no-unused-vars"],
  "vue/require-scoped-style": ["vue", "vue-require-scoped-style"],
  "vue/require-v-for-key": ["vue", "vue-require-v-for-key"],
  "vue/sfc-element-order": ["vue", "vue-sfc-element-order"],
  "vue/v-bind-style": ["vue", "vue-v-bind-style"],
  "vue/v-on-style": ["vue", "vue-v-on-style"],
  "vue/v-slot-style": ["vue", "vue-v-slot-style"],
  "nuxt/nuxt-config-keys-order": ["index", "nuxt-nuxt-config-keys-order"],
  "cross-file": ["cross-file", null],
} as const;

export function currentRuleDocumentationMessage(expected: unknown): Record<string, unknown> {
  assert.ok(expected !== null && typeof expected === "object" && !Array.isArray(expected));
  const message = expected as Record<string, unknown>;
  const rule = message.ruleId;
  assert.ok(
    typeof rule === "string" && Object.hasOwn(pages, rule),
    "finite authored documentation rule",
  );
  const [historical, page] = pages[rule as keyof typeof pages];
  assert.equal(
    message.ruleDocsPath,
    `docs/content/rules/${historical}.md`,
    "frozen rule documentation URL",
  );
  // Directive accounting is a synthetic operational finding, not a registered rule.
  const target =
    rule === "vue/comment-directive"
      ? "guide/lint-execution.md"
      : page === null
        ? `rules/${historical}.md`
        : `rules/reference/${page}.md`;
  for (const locale of ["", "ja/"]) {
    assert.ok(
      fs.existsSync(new URL(`../../../docs/content/${locale}${target}`, import.meta.url)),
      "independent committed rule documentation page",
    );
  }
  return { ...message, ruleDocsPath: `docs/content/${target}` };
}

export function currentRuleDocumentationMessages(expected: unknown): Record<string, unknown>[] {
  assert.ok(Array.isArray(expected), "authored diagnostic array");
  return expected.map(currentRuleDocumentationMessage);
}

export function currentRuleDocumentationPacket(expected: unknown): {
  file: string;
  messages: Record<string, unknown>[];
  errorCount: number;
  warningCount: number;
}[] {
  assert.ok(Array.isArray(expected), "authored CLI packet array");
  return expected.map((entry: unknown) => {
    assert.ok(entry !== null && typeof entry === "object" && !Array.isArray(entry));
    const row = entry as Record<string, unknown>;
    assert.deepEqual(
      Object.keys(row).sort(),
      ["errorCount", "file", "messages", "warningCount"],
      "complete authored packet fields",
    );
    assert.ok(typeof row.file === "string");
    assert.ok(typeof row.errorCount === "number" && typeof row.warningCount === "number");
    const messages = currentRuleDocumentationMessages(row.messages);
    assert.equal(
      row.errorCount,
      messages.filter((message) => message.severity === 2).length,
      "authored error counter",
    );
    assert.equal(
      row.warningCount,
      messages.filter((message) => message.severity === 1).length,
      "authored warning counter",
    );
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

export function currentRuleDocumentationPlain(expected: string): string {
  let rule: string | undefined;
  return expected
    .split(/(?<=\n)/u)
    .map((line) => {
      const finding = /^\s+\S+:\d+:\d+ (?:warning|error|info) (\S+) /u.exec(line);
      if (finding) rule = finding[1];
      const prefix = "    Reference: ";
      if (!line.startsWith(prefix)) return line;
      assert.ok(rule, "preceding authored diagnostic rule");
      const ending = line.endsWith("\r\n") ? "\r\n" : line.endsWith("\n") ? "\n" : "";
      const reference = line.slice(prefix.length, ending ? -ending.length : undefined);
      const message = currentRuleDocumentationMessage({ ruleId: rule, ruleDocsPath: reference });
      return prefix + String(message.ruleDocsPath) + ending;
    })
    .join("");
}
