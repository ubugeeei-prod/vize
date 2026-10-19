import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";

const require = createRequire(new URL("../../docs/package.json", import.meta.url));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");

test("in-page fragment links of the practices guide reach existing anchors", () => {
  for (const locale of ["", "ja/"]) {
    const source = readFileSync(
      new URL(
        `../../docs/content/${locale}architecture/language-engineering-practices.md`,
        import.meta.url,
      ),
      "utf8",
    );
    const rendered = native.transform(source, {});
    assert.deepEqual(rendered.errors, [], locale);
    const ids = new Set(
      Array.from(rendered.html.matchAll(/\bid="([^"]+)"/g), (match: RegExpMatchArray) => match[1]),
    );
    const prose = source.replace(/^```[\s\S]*?^```/gm, "").replace(/`[^`\n]*`/g, "");
    for (const [, fragment] of prose.matchAll(/\]\(#([^)\s]+)\)/g))
      assert.ok(ids.has(fragment), `${locale}language-engineering-practices.md: #${fragment}`);
  }
});
