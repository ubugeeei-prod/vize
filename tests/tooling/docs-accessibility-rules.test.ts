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

for (const locale of ["", "ja/"]) {
  test(`${locale || "English"} accessibility navigation targets the complete same-page catalogue`, () => {
    const overview = fs.readFileSync(
      path.join(repoRoot, `docs/content/${locale}rules/accessibility.md`),
      "utf8",
    );
    const links = [...overview.matchAll(/^\| \[`([^`]+)`\]\(\.\/all\.md#([^)]*)\)/gm)];
    assert.deepEqual(
      links.map((row) => row[1]).sort((a, b) => a.localeCompare(b)),
      [...accessibilityRules].sort((a, b) => a.localeCompare(b)),
    );
    for (const [_, ruleId, slug] of links) {
      const file = `${slug}.md`;
      const section = fs.readFileSync(
        path.join(repoRoot, `docs/content/${locale}rules/reference/${file}`),
        "utf8",
      );
      assert.ok(section.includes(`# \`${ruleId}\``));
      assert.ok(overview.includes(`./all.md#${slug}-bad`), `${ruleId}: same-page Bad`);
      assert.ok(overview.includes(`./all.md#${slug}-good`), `${ruleId}: same-page Good`);
      for (const label of locale
        ? ["既定の重大度:", "プリセット:", "オプション:", "## 悪い", "## 良い"]
        : ["Default severity:", "Presets:", "Options:", "## Bad", "## Good"])
        assert.ok(section.includes(label), `${ruleId}: ${label}`);
      assert.ok(
        [...section.matchAll(/```vue\n[\s\S]*?\n```/gu)].length >= 2,
        `${ruleId}: two complete Vue witnesses`,
      );
    }
  });
}
