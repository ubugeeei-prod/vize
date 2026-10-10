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

const vueSources = (text: string) =>
  [...text.matchAll(/```vue[^\n]*\n([\s\S]*?)\n```/g)].map((match) => match[1]);

for (const locale of ["en", "ja", "fr", "pt-BR", "zh-CN"]) {
  await test(`${locale} accessibility navigation targets complete same-page examples`, () => {
    const overview = fs.readFileSync(
      path.join(repoRoot, `docs/content/generated/rules/${locale}/accessibility.md`),
      "utf8",
    );
    const links = [...overview.matchAll(/^\| \[`([^`]+)`\]\(#([^)]*)\)/gm)];
    const headings = [...overview.matchAll(/^### `([^`]+)`$/gm)];
    const expected = [...accessibilityRules].sort((a, b) => a.localeCompare(b));
    assert.deepEqual(
      links.map((row) => row[1]).sort((a, b) => a.localeCompare(b)),
      expected,
    );
    assert.deepEqual(
      headings.map((row) => row[1]).sort((a, b) => a.localeCompare(b)),
      expected,
    );
    assert.doesNotMatch(overview, /\]\(\.\/all\.md#/);
    for (const [_, ruleId, slug] of links) {
      const index = headings.findIndex((heading) => heading[1] === ruleId);
      assert.notEqual(index, -1, `${locale} ${ruleId}: local packet`);
      const packet = overview.slice(headings[index].index, headings[index + 1]?.index);
      const reference = fs.readFileSync(
        path.join(
          repoRoot,
          `docs/content/${locale === "ja" ? "ja/" : ""}rules/reference/${slug}.md`,
        ),
        "utf8",
      );
      assert.ok(reference.includes(`# \`${ruleId}\``));
      for (const kind of ["bad", "good"]) {
        assert.ok(
          overview.includes(`](#${slug}-${kind})`),
          `${locale} ${ruleId}: local ${kind} link`,
        );
        assert.equal(
          packet.split(`<span id="${slug}-${kind}"></span>`).length,
          2,
          `${locale} ${ruleId}: unique ${kind} target`,
        );
      }
      const sources = vueSources(packet);
      assert.ok(sources.length >= 2, `${locale} ${ruleId}: complete Bad/Good Vue witnesses`);
      assert.deepEqual(
        sources,
        vueSources(reference),
        `${locale} ${ruleId}: exact whole copied source`,
      );
      assert.ok(packet.includes(`"${ruleId}":`), `${locale} ${ruleId}: inline rule configuration`);
      const labels =
        locale === "ja"
          ? ["既定の重大度:", "プリセット:", "オプション:", "**悪い**", "**良い**"]
          : locale === "en"
            ? ["Default severity:", "Presets:", "Options:", "**Bad**", "**Good**"]
            : [];
      for (const label of labels)
        assert.ok(packet.includes(label), `${locale} ${ruleId}: ${label}`);
    }
  });
}
