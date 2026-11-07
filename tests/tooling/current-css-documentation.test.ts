import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  currentCssCases,
  currentCssPacket,
  currentCssPlain,
  currentCssReport,
} from "./support/current-css-documentation.ts";
import { currentContentDirectivePacket } from "./support/current-content-directive-documentation.ts";
import { currentBooleanStylePacket } from "./support/current-boolean-style-documentation.ts";
import { loadCases, wholeJson } from "./boolean-attribute-fix-reference.mjs";
import { cliExpected } from "./support/art-lint-public-expectations.mjs";

const oldUrl = "docs/content/rules/musea-and-css.md";
const newUrl = "docs/content/rules/reference/css-no-display-none.md";
interface CssRow {
  id?: string;
  source?: string;
  expectedCli: unknown;
  expectedPlain?: string;
}

function historicalCssBytes(value: unknown): string {
  return JSON.stringify(value)
    .replaceAll(
      '"line":3,"column":23,"endLine":3,"endColumn":30',
      '"line":3,"column":22,"endLine":3,"endColumn":29',
    )
    .replaceAll(
      '"line":2,"column":30,"endLine":2,"endColumn":35',
      '"line":2,"column":29,"endLine":2,"endColumn":34',
    )
    .replaceAll(
      "cases/display-property-before-unicode-selector.vue:3:23 warning css/no-display-none",
      "cases/display-property-before-unicode-selector.vue:3:22 warning css/no-display-none",
    )
    .replaceAll(
      "UnicodeMultipleStyles.vue:2:30 warning css/no-utility-classes",
      "UnicodeMultipleStyles.vue:2:29 warning css/no-utility-classes",
    );
}

await test("the authored compound-ID report keeps its current JSON and projects only legacy plain links", () => {
  const path = new URL(
    "../../crates/vize_patina/tests/fixtures/css-id-selector-ranges/repeated-compound-ids.cli.expected.json",
    import.meta.url,
  );
  const bytes = fs.readFileSync(path);
  assert.equal(bytes.length, 2451);
  assert.equal(
    createHash("sha256").update(bytes).digest("hex"),
    "5cc3f4660cc685b4a25450a08ff8481f2834494a9dfe9194f70f531740c33337",
  );
  const original = JSON.parse(bytes.toString()) as { json: unknown; plain: string };
  const page = "docs/content/rules/reference/css-no-id-selectors.md";
  const expected = { ...original, plain: currentCssPlain(original.plain) };
  assert.equal(expected.json, original.json);
  assert.equal(expected.plain.split(page).length - 1, 4);
  assert.equal(expected.plain.replaceAll(page, oldUrl), original.plain);
  assert.ok(fs.readFileSync(path).equals(bytes));
  assert.throws(() => currentCssPlain(original.plain.replaceAll(oldUrl, page)));
  assert.throws(() =>
    currentCssPlain(original.plain.replaceAll("css/no-id-selectors", "css/foreign")),
  );
});

await test("current CSS expectations preserve frozen corpora except documented projections", () => {
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
    const preserved = structuredClone(projected);
    const utf8 = preserved.find(
      (row) => row.id === "utf8-crlf" && row.source?.includes('.日本[data-note="😀"]'),
    );
    if (utf8) {
      const message = (utf8.expectedCli as { messages: Record<string, unknown>[] }[])[0]
        .messages[0];
      Object.assign(message, { column: 22, endColumn: 29 });
    }
    assert.equal(
      JSON.stringify(preserved).replaceAll(newUrl, oldUrl),
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

await test("CSS ID and utility corpora retain frozen reports before current-only projection", () => {
  let count = 0;
  for (const name of ["css-id-selector-ranges", "css-utility-literal-tokens"]) {
    const fixture = new URL(`../../crates/vize_patina/tests/fixtures/${name}/`, import.meta.url);
    const manifest = JSON.parse(fs.readFileSync(new URL("cases.json", fixture), "utf8"));
    for (const entry of manifest.cases) {
      const raw = fs.readFileSync(new URL(entry.cliExpected ?? entry.expected, fixture));
      if (entry.expectedBytes !== undefined) assert.equal(raw.length, entry.expectedBytes);
      if (entry.expectedSha256 !== undefined) {
        assert.equal(createHash("sha256").update(raw).digest("hex"), entry.expectedSha256);
      }
      const original = JSON.parse(raw.toString("utf8"));
      const saved = JSON.stringify(original);
      const authoredCurrent =
        name === "css-id-selector-ranges" && entry.id === "repeated-compound-ids";
      if (authoredCurrent) {
        assert.equal(entry.cliExpected, "repeated-compound-ids.cli.expected.json");
        assert.equal(raw.length, 2451);
        assert.equal(
          createHash("sha256").update(raw).digest("hex"),
          "5cc3f4660cc685b4a25450a08ff8481f2834494a9dfe9194f70f531740c33337",
        );
      }
      const current = authoredCurrent ? original : currentCssReport(original);
      assert.equal(JSON.stringify(original), saved);
      assert.equal(
        historicalCssBytes(current).replaceAll(
          /docs\/content\/rules\/reference\/css-(?:no-id-selectors|no-utility-classes)\.md/gu,
          oldUrl,
        ),
        authoredCurrent
          ? saved.replaceAll("docs/content/rules/reference/css-no-id-selectors.md", oldUrl)
          : saved,
      );
      for (const file of current.json) {
        for (const message of file.messages) {
          const page =
            name === "css-id-selector-ranges" ? "css-no-id-selectors" : "css-no-utility-classes";
          assert.equal(message.ruleDocsPath, `docs/content/rules/reference/${page}.md`);
        }
      }
      count++;
    }
  }
  assert.equal(count, 27);
});

await test("content directive projection retains the complete authored CLI and LSP corpus", () => {
  const root = new URL("../../crates/vize_patina/tests/fixtures/issue-8018/", import.meta.url);
  const source = JSON.parse(fs.readFileSync(new URL("source.json", root), "utf8"));
  const bytes = fs.readFileSync(new URL("references.json", root));
  const pin = source.authored.find((entry: { file: string }) => entry.file === "references.json");
  assert.equal(bytes.length, pin.bytes);
  assert.equal(createHash("sha256").update(bytes).digest("hex"), pin.sha256);
  const reference = JSON.parse(bytes.toString("utf8"));
  const saved = JSON.stringify(reference);
  assert.equal(reference.scenarios.length, 11);
  for (const scenario of reference.scenarios) {
    const current = currentContentDirectivePacket(scenario.expectedCli);
    for (const file of current) {
      assert.ok(
        file !== null &&
          typeof file === "object" &&
          "messages" in file &&
          Array.isArray(file.messages),
      );
      for (const message of file.messages) {
        assert.equal(
          message.ruleDocsPath,
          "docs/content/rules/reference/html-no-empty-palpable-content.md",
        );
      }
    }
    assert.equal(
      JSON.stringify(current).replaceAll(
        "docs/content/rules/reference/html-no-empty-palpable-content.md",
        "docs/content/rules/html.md",
      ),
      JSON.stringify(scenario.expectedCli),
    );
  }
  assert.equal(JSON.stringify(reference), saved);
  for (const ruleDocsPath of [
    undefined,
    "docs/content/rules/vue.md",
    "docs/content/rules/reference/html-no-empty-palpable-content.md",
  ]) {
    assert.throws(
      () =>
        currentContentDirectivePacket([
          { messages: [{ ruleId: "html/no-empty-palpable-content", ruleDocsPath }] },
        ]),
      /frozen content-directive documentation/u,
    );
  }
  assert.throws(
    () =>
      currentContentDirectivePacket([
        { messages: [{ ruleId: "html/foreign", ruleDocsPath: "docs/content/rules/html.md" }] },
      ]),
    /authored content-directive rule/u,
  );
});

await test("current boolean/style expectations preserve the whole historical oracle", () => {
  const { cases } = loadCases(fileURLToPath(new URL("../../", import.meta.url)));
  const pages = new Map([
    ["vue/html-self-closing", "vue-html-self-closing"],
    ["vue/component-name-in-template-casing", "vue-component-name-in-template-casing"],
    ["vue/v-slot-style", "vue-v-slot-style"],
    ["vue/no-boolean-attr-value", "vue-no-boolean-attr-value"],
  ]);
  const original = currentBooleanStylePacket(wholeJson(cases[0], false, true));
  const seen = new Set<string>();
  for (const file of original) {
    assert.ok(
      file !== null &&
        typeof file === "object" &&
        "messages" in file &&
        Array.isArray(file.messages),
    );
    for (const message of file.messages) {
      const rule = String(message.ruleId);
      assert.ok(pages.has(rule));
      assert.equal(message.ruleDocsPath, `docs/content/rules/reference/${pages.get(rule)}.md`);
      seen.add(rule);
    }
  }
  assert.equal(seen.size, 4);
  for (const entry of cases) {
    for (const after of [false, true]) {
      const before = wholeJson(entry, after, entry.originalRules);
      const saved = JSON.stringify(before);
      const current = currentBooleanStylePacket(before);
      assert.equal(JSON.stringify(before), saved);
      assert.equal(
        JSON.stringify(current).replaceAll(
          /docs\/content\/rules\/reference\/vue-(?:html-self-closing|component-name-in-template-casing|v-slot-style|no-boolean-attr-value)\.md/gu,
          "docs/content/rules/vue.md",
        ),
        saved,
      );
    }
  }
  for (const ruleDocsPath of [
    undefined,
    "docs/content/rules/html.md",
    "docs/content/rules/reference/vue-no-boolean-attr-value.md",
  ]) {
    assert.throws(
      () =>
        currentBooleanStylePacket([
          { messages: [{ ruleId: "vue/no-boolean-attr-value", ruleDocsPath }] },
        ]),
      /frozen boolean\/style documentation/u,
    );
  }
  assert.throws(
    () =>
      currentBooleanStylePacket([
        { messages: [{ ruleId: "vue/foreign", ruleDocsPath: "docs/content/rules/vue.md" }] },
      ]),
    /authored boolean\/style rule/u,
  );
});
