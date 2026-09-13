import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/moonbit.ts";

const configurableRuleOptions = [
  "musea/prefer-design-tokens",
  "script/custom-event-name-casing",
  "script/no-restricted-globals",
  "script/no-restricted-members",
  "vue/attribute-hyphenation",
  "vue/component-name-in-template-casing",
  "vue/html-self-closing",
  "vue/no-mutating-props",
  "vue/sfc-element-order",
  "vue/v-on-event-hyphenation",
];

test("rule option docs enumerate every typed lint rule option", () => {
  const optionsDoc = fs.readFileSync(path.join(repoRoot, "docs/content/rules/options.md"), "utf8");

  assert.match(optionsDoc, /# Rule Options/);
  assert.match(optionsDoc, /Unknown\s+option fields are rejected/);
  assert.match(optionsDoc, /later matching config entries replace the full option object/);

  for (const ruleId of configurableRuleOptions) {
    assert.match(
      optionsDoc,
      new RegExp(escapeRegExp(`| \`${ruleId}\` |`)),
      `${ruleId} must be documented in the lint rule option table`,
    );
  }
});

test("configuration docs link to the full lint rule option reference", () => {
  const configuration = fs.readFileSync(
    path.join(repoRoot, "docs/content/guide/configuration.md"),
    "utf8",
  );

  assert.match(configuration, /### Lint Rule Options/);
  assert.match(configuration, /\[Rule Options\]\(\.\.\/rules\/options\.md\)/);
});

test("all rules reference shows which rules accept lint rule options", () => {
  const allRules = fs.readFileSync(path.join(repoRoot, "docs/content/rules/all.md"), "utf8");

  assert.match(
    allRules,
    /\| Rule \| Severity \| Presets \| Fixable \| Options \| Implementation \| Description \|/,
  );
  assert.match(allRules, /rule-option support/);

  for (const ruleId of configurableRuleOptions) {
    const row = allRules.split("\n").find((line) => line.startsWith(`| \`${ruleId}\` |`));
    assert.ok(row, `${ruleId} must appear in all rules`);
    assert.match(
      row,
      /\[`ruleOptions`\]\(\.\/options\.md\)/,
      `${ruleId} must link to lint rule option docs`,
    );
  }

  const a11yRow = allRules.split("\n").find((line) => line.startsWith("| `a11y/img-alt` |"));
  assert.ok(a11yRow, "a sample non-configurable rule must appear in all rules");
  assert.match(a11yRow, /\| No \| \[source\]/);
});

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
