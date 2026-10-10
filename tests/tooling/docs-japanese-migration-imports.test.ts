import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const fixture = JSON.parse(
  readFileSync(new URL("./fixtures/docs-japanese-migration-imports.json", import.meta.url), "utf8"),
);
for (const page of fixture.pages) {
  test(`Japanese ${page.name} retains full published examples and source contracts`, () => {
    const english = readFileSync(
      new URL(`../../docs/content/guide/${page.name}.md`, import.meta.url),
      "utf8",
    );
    const japanese = readFileSync(
      new URL(`../../docs/content/ja/guide/${page.name}.md`, import.meta.url),
      "utf8",
    );
    for (const block of page.blocks)
      assert.ok(japanese.includes(block), `missing published example: ${block}`);
    assert.match(japanese, /Reviewed translation;[^\n]+scope: complete document/);
    const inline = (text: string) =>
      new Set(
        [...text.replace(/```[^\n]+\n[\s\S]*?```/g, "").matchAll(/`([^`\n]+)`/g)].map((m) => m[1]),
      );
    const translated = inline(japanese);
    for (const code of inline(english))
      assert.ok(translated.has(code), `missing source contract: ${code}`);
    if (page.name === "troubleshooting")
      assert.doesNotMatch(japanese, /癖は必要|空の要素 .*があり/);
  });
}
