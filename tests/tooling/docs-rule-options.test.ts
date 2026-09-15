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

  assertRuleOptionsReference(optionsDoc, {
    title: /# Rule Options/,
    unknownFields: /Unknown\s+option fields are rejected/,
    scopedReplacement: /later matching config entries replace the full option object/,
    badLabel: "Bad",
    goodLabel: "Good",
  });
});

test("Japanese rule option docs mirror every typed lint rule option", () => {
  const optionsDoc = fs.readFileSync(
    path.join(repoRoot, "docs/content/ja/rules/options.md"),
    "utf8",
  );

  assertRuleOptionsReference(optionsDoc, {
    title: /# ルール オプション/,
    unknownFields: /未知の option field は拒否/,
    scopedReplacement: /option object 全体を置き換え/,
    badLabel: "悪い",
    goodLabel: "良い",
  });
});

function assertRuleOptionsReference(
  optionsDoc: string,
  labels: {
    title: RegExp;
    unknownFields: RegExp;
    scopedReplacement: RegExp;
    badLabel: string;
    goodLabel: string;
  },
): void {
  assert.match(optionsDoc, labels.title);
  assert.match(optionsDoc, labels.unknownFields);
  assert.match(optionsDoc, labels.scopedReplacement);

  for (const ruleId of configurableRuleOptions) {
    assert.match(
      optionsDoc,
      new RegExp(escapeRegExp(`| \`${ruleId}\` |`)),
      `${ruleId} must be documented in the lint rule option table`,
    );
    const section = sectionFor(optionsDoc, `## \`${ruleId}\``);
    assert.ok(section.includes(labels.badLabel), `${ruleId} must include a bad example`);
    assert.ok(section.includes(labels.goodLabel), `${ruleId} must include a good example`);
    assert.match(section, /```(?:json|ts|vue)[\s\S]*?```/u, `${ruleId} must include code`);
  }
}

test("configuration docs link to the full lint rule option reference", () => {
  const configuration = fs.readFileSync(
    path.join(repoRoot, "docs/content/guide/configuration.md"),
    "utf8",
  );

  assert.match(configuration, /### Lint Rule Options/);
  assert.match(configuration, /\[Rule Options\]\(\.\.\/rules\/options\.md\)/);

  const jaConfiguration = fs.readFileSync(
    path.join(repoRoot, "docs/content/ja/guide/configuration.md"),
    "utf8",
  );

  assert.match(jaConfiguration, /### Lint Rule Options/);
  assert.match(jaConfiguration, /\[ルール オプション\]\(\.\.\/rules\/options\.md\)/);
});

test("rules overview links to lint rule option references", () => {
  const index = fs.readFileSync(path.join(repoRoot, "docs/content/rules/index.md"), "utf8");
  const jaIndex = fs.readFileSync(path.join(repoRoot, "docs/content/ja/rules/index.md"), "utf8");

  assert.match(index, /\[Rule Options\]\(\.\/options\.md\)/);
  assert.match(jaIndex, /\[ルール オプション\]\(\.\/options\.md\)/);
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

function sectionFor(source: string, heading: string): string {
  const start = source.indexOf(heading);
  assert.notEqual(start, -1, `missing ${heading}`);
  const next = source.indexOf("\n## ", start + heading.length);
  return source.slice(start, next === -1 ? source.length : next);
}
