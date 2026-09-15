import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/moonbit.ts";

const accessibilityRules = [
  "a11y/alt-text",
  "a11y/anchor-has-content",
  "a11y/anchor-is-valid",
  "a11y/aria-props",
  "a11y/aria-role",
  "a11y/aria-unsupported-elements",
  "a11y/click-events-have-key-events",
  "a11y/form-control-has-label",
  "a11y/heading-has-content",
  "a11y/heading-levels",
  "a11y/iframe-has-title",
  "a11y/img-alt",
  "a11y/interactive-supports-focus",
  "a11y/label-has-for",
  "a11y/landmark-roles",
  "a11y/media-has-caption",
  "a11y/mouse-events-have-key-events",
  "a11y/no-access-key",
  "a11y/no-aria-hidden-on-focusable",
  "a11y/no-autofocus",
  "a11y/no-distracting-elements",
  "a11y/no-i-for-icon",
  "a11y/no-redundant-roles",
  "a11y/no-refer-to-non-existent-id",
  "a11y/no-role-presentation-on-focusable",
  "a11y/no-static-element-interactions",
  "a11y/placeholder-label-option",
  "a11y/role-has-required-aria-props",
  "a11y/tabindex-no-positive",
  "a11y/use-list",
  "vue/use-unique-element-ids",
];

const locales = [
  {
    label: "English",
    paths: [
      path.join(repoRoot, "docs/content/rules/accessibility.md"),
      path.join(repoRoot, "docs/content/rules/accessibility-core.md"),
      path.join(repoRoot, "docs/content/rules/accessibility-structure.md"),
      path.join(repoRoot, "docs/content/rules/accessibility-interactions.md"),
      path.join(repoRoot, "docs/content/rules/accessibility-integrity.md"),
    ],
    severityLabel: "Default severity:",
    presetsLabel: "Presets:",
    optionsLabel: "Options:",
    badLabel: "Bad:",
    goodLabel: "Good:",
    forbiddenHeading: /^## Additional Accessibility Rules$/m,
  },
  {
    label: "Japanese",
    paths: [path.join(repoRoot, "docs/content/ja/rules/accessibility.md")],
    severityLabel: "既定の重大度:",
    presetsLabel: "プリセット:",
    optionsLabel: "オプション:",
    badLabel: "悪い:",
    goodLabel: "良い:",
    forbiddenHeading: /^## 追加のアクセシビリティ ルール$/m,
  },
];

for (const locale of locales) {
  test(`${locale.label} accessibility docs expand every rule with examples`, () => {
    const source = readDocs(locale.paths);
    assert.doesNotMatch(
      source,
      locale.forbiddenHeading,
      "accessibility docs must not group rules into a compact Additional section",
    );

    const documentedRules = [...source.matchAll(/^## `([^`]+)`$/gm)].map((match) => match[1]);
    assert.deepEqual(documentedRules, accessibilityRules);

    for (let index = 0; index < accessibilityRules.length; index += 1) {
      const ruleId = accessibilityRules[index];
      const start = source.indexOf(`## \`${ruleId}\``);
      const nextRule = accessibilityRules[index + 1];
      const end = nextRule === undefined ? source.length : source.indexOf(`## \`${nextRule}\``);
      const section = source.slice(start, end);

      assert.ok(section.includes(locale.severityLabel), `${ruleId} must document severity`);
      assert.ok(section.includes(locale.presetsLabel), `${ruleId} must document preset membership`);
      assert.ok(section.includes(locale.optionsLabel), `${ruleId} must document rule options`);
      assert.ok(section.includes(locale.badLabel), `${ruleId} must include a bad example`);
      assert.ok(section.includes(locale.goodLabel), `${ruleId} must include a good example`);
      const examples = [...section.matchAll(/```vue\n[\s\S]*?\n```/gu)];
      assert.ok(examples.length >= 2, `${ruleId} must include bad and good Vue examples`);
    }
  });
}

function readDocs(paths: string[]): string {
  return paths.map((filePath) => fs.readFileSync(filePath, "utf8")).join("\n");
}
