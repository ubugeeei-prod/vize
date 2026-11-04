import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { currentCssCases, currentCssPacket } from "./support/current-css-documentation.ts";
import { cliExpected } from "./support/art-lint-public-expectations.mjs";

const oldUrl = "docs/content/rules/musea-and-css.md";
const newUrl = "docs/content/rules/reference/css-no-display-none.md";
interface CssRow {
  expectedCli: unknown;
  expectedPlain?: string;
}

await test("current CSS expectations preserve complete frozen corpora except documentation fields", () => {
  let count = 0;
  for (const name of [
    "issue-7976",
    "issue-7976-global",
    "issue-7976-compound",
    "issue-7976-universal",
  ]) {
    const raw = fs.readFileSync(
      new URL(`../../crates/vize_patina/tests/fixtures/${name}/cases.json`, import.meta.url),
      "utf8",
    );
    const { cases } = JSON.parse(raw) as { cases: CssRow[] };
    const saved = JSON.stringify(cases);
    const projected = currentCssCases(cases);
    for (const row of projected) {
      for (const file of row.expectedCli as { messages: { ruleDocsPath: string }[] }[]) {
        for (const message of file.messages) assert.equal(message.ruleDocsPath, newUrl);
      }
      if (row.expectedPlain?.includes("Reference:")) {
        assert.ok(row.expectedPlain.includes(newUrl));
        assert.ok(!row.expectedPlain.includes(oldUrl));
      }
    }
    assert.equal(JSON.stringify(cases), saved, "frozen input remains untouched");
    assert.equal(
      JSON.stringify(projected).replaceAll(newUrl, oldUrl),
      saved,
      "every non-URL byte is preserved",
    );
    count += cases.length;
  }
  assert.equal(count, 225);
});

await test("CSS projection rejects foreign rules and invalid historical documentation", () => {
  const message = { ruleId: "css/no-display-none", ruleDocsPath: oldUrl };
  for (const ruleDocsPath of [undefined, newUrl, "docs/content/rules/vue.md"]) {
    assert.throws(
      () => currentCssPacket([{ messages: [{ ...message, ruleDocsPath }] }]),
      /frozen CSS documentation URL/u,
    );
  }
  assert.throws(
    () => currentCssPacket([{ messages: [{ ...message, ruleId: "css/foreign" }] }]),
    /finite historical CSS rule/u,
  );
  assert.throws(
    () => currentCssCases([{ expectedCli: [], expectedPlain: `    Reference: ${oldUrl}\n` }]),
    /preceding authored CSS finding/u,
  );
});

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
      const page = {
        "vue/no-unused-setup-bindings": "vue-no-unused-setup-bindings",
        "vue/require-component-registration": "vue-require-component-registration",
        "ssr/no-browser-globals-in-ssr": "ssr-no-browser-globals-in-ssr",
      }[message.ruleId as string];
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
