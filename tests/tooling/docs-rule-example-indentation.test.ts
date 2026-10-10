import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { test } from "node:test";
import { ruleExamples } from "../../docs/scripts/rules/examples.ts";
import {
  indentationRuleMetadata,
  ruleIndentationExamples,
} from "../../docs/scripts/rules/indentation-examples.ts";

const root = resolve(import.meta.dirname, "../..");
for (const fixture of ruleIndentationExamples) {
  await test(`${fixture.name}: whole original authored Bad/Good indentation`, () => {
    const source = readFileSync(resolve(root, fixture.sourcePath));
    assert.equal(createHash("sha256").update(source).digest("hex"), fixture.sourceSha256);
    assert.deepEqual(ruleExamples(root, indentationRuleMetadata(fixture)), fixture.expected);
  });
}

for (const language of ["vue", "ts", "html", "css"]) {
  await test(`${language}: preserve intentional Bad whitespace through wrappers`, () => {
    const directory = mkdtempSync(resolve(tmpdir(), "vize-rule-doc-indent-"));
    // Uneven leading spaces, a tab, a whitespace-only line and trailing spaces
    // are authored violations, not transport or a request to format the snippet.
    const bad = "  first {\n\tsecond\n     third  \n \n  }";
    const good = "first {\n  second\n}";
    const docs = [
      "//!### Bad",
      `//! \`\`\`${language}`,
      ...bad.split("\n").map((line) => `//! ${line}`),
      "//! ```",
      "//!",
      "//! ### Good",
      `//! \`\`\`${language}`,
      ...good.split("\n").map((line) => `//! ${line}`),
      "//! ```",
    ].join("\n");
    try {
      writeFileSync(resolve(directory, "authored.rs"), docs);
      const metadata = {
        ...indentationRuleMetadata(ruleIndentationExamples[0]),
        name: language === "html" ? "petite-vue/authored-indent" : "authored/indent",
        implementationPath: "authored.rs",
      };
      const wrap = (source: string) =>
        language === "ts"
          ? `<script setup lang="ts">\n${source}\n</script>`
          : language === "css"
            ? `<style scoped>\n${source}\n</style>`
            : language === "html"
              ? `<!doctype html>\n<html><body>\n${source}\n<script src="https://unpkg.com/petite-vue" init></script>\n</body></html>`
              : `<template>\n${source}\n</template>`;
      assert.deepEqual(ruleExamples(directory, metadata), {
        bad: { language: language === "html" ? "html" : "vue", source: wrap(bad) },
        good: { language: language === "html" ? "html" : "vue", source: wrap(good) },
        evidence: "authored.rs",
      });
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  });
}
