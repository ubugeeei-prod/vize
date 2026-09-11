import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/moonbit.ts";

const promotedRules = [
  "a11y/mouse-events-have-key-events",
  "a11y/no-i-for-icon",
  "a11y/no-refer-to-non-existent-id",
];

const locales = [
  {
    label: "English",
    path: path.join(repoRoot, "docs/content/rules/accessibility.md"),
    additionalHeading: "## Additional Accessibility Rules",
    clarification: /The split here is documentation detail only/,
  },
  {
    label: "Japanese",
    path: path.join(repoRoot, "docs/content/ja/rules/accessibility.md"),
    additionalHeading: "## 追加のアクセシビリティ ルール",
    clarification: /ドキュメントの詳しさだけの違い/,
  },
];

for (const locale of locales) {
  test(`${locale.label} accessibility docs distinguish expanded examples from compact rule listings`, () => {
    const source = fs.readFileSync(locale.path, "utf8");
    const additionalIndex = source.indexOf(locale.additionalHeading);
    assert.ok(additionalIndex > 0, "accessibility docs must keep an Additional section");

    assert.match(
      source.slice(additionalIndex),
      locale.clarification,
      "Additional rule placement must not imply a different implementation category",
    );

    for (const ruleId of promotedRules) {
      const heading = `## \`${ruleId}\``;
      const headingIndex = source.indexOf(heading);
      assert.ok(headingIndex > 0, `${ruleId} must have an expanded documentation section`);
      assert.ok(headingIndex < additionalIndex, `${ruleId} must appear before Additional`);
      assert.equal(
        source.slice(additionalIndex + locale.additionalHeading.length).includes(`\`${ruleId}\``),
        false,
        `${ruleId} must not also stay in the compact-only Additional list`,
      );
    }
  });
}
