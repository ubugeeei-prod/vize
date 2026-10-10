import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { test } from "node:test";
import { normalizeMarkdownDocument } from "../../docs/scripts/i18n/markdown.ts";

const root = resolve(import.meta.dirname, "../..");
const require = createRequire(new URL("../../docs/package.json", import.meta.url));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");

function markdownFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = resolve(directory, entry.name);
    return entry.isDirectory() ? markdownFiles(path) : entry.name.endsWith(".md") ? [path] : [];
  });
}

void test("every Japanese page renders authored emphasis without literal delimiters in prose", () => {
  const files = markdownFiles(resolve(root, "docs/content/ja"));
  assert.ok(files.length >= 394, "the full shipped Japanese content is audited");
  for (const file of files) {
    const source = readFileSync(file, "utf8");
    assert.equal(normalizeMarkdownDocument(source), source, `${file}: regeneration remains stable`);
    const rendered = native.transform(source, {});
    assert.deepEqual(rendered.errors, [], file);
    const prose = rendered.html.replace(/<(pre|code|script|style)\b[^>]*>[\s\S]*?<\/\1>/g, "");
    assert.doesNotMatch(
      prose,
      /\*\*/,
      `${file}: strong emphasis must render outside literal source`,
    );
  }
});
