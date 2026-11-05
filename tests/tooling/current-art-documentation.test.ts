import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { cliExpected } from "./support/art-lint-public-expectations.mjs";

await test("Art CLI expectations use only the three independently authored documentation pages", () => {
  const corpus = JSON.parse(
    fs.readFileSync(
      new URL(
        "../_fixtures/differential/linter/art-public-boundaries-7900/cases.json",
        import.meta.url,
      ),
      "utf8",
    ),
  );
  const saved = JSON.stringify(corpus);
  const seen = new Set<string>();
  for (const row of corpus.cases) {
    const source = fs.readFileSync(
      new URL(
        `../_fixtures/differential/linter/art-public-boundaries-7900/${row.file}`,
        import.meta.url,
      ),
      "utf8",
    );
    const packet = cliExpected({ ...row, source });
    for (const message of packet[0].messages) {
      const page = new Map([
        ["vue/no-unused-setup-bindings", "vue-no-unused-setup-bindings"],
        ["vue/require-component-registration", "vue-require-component-registration"],
        ["ssr/no-browser-globals-in-ssr", "ssr-no-browser-globals-in-ssr"],
      ]).get(String(message.ruleId));
      assert.ok(page);
      assert.equal(message.ruleDocsPath, `docs/content/rules/reference/${page}.md`);
      seen.add(message.ruleId);
    }
  }
  assert.equal(seen.size, 3);
  assert.equal(JSON.stringify(corpus), saved);
  const foreign = {
    source: "x",
    filename: "x.art",
    service: {
      error_count: 1,
      warning_count: 0,
      diagnostics: [{ rule_name: "vue/foreign", start: 0, end: 1 }],
    },
  };
  assert.throws(() => cliExpected(foreign), /finite authored Art rule/u);
});
