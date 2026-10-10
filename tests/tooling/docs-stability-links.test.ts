import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { test } from "node:test";

const root = resolve(import.meta.dirname, "../..");
const require = createRequire(new URL("../../docs/package.json", import.meta.url));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");

void test("every Stability locale retains complete external release-policy destinations", () => {
  for (const locale of ["", "ja/", "fr/", "pt-BR/", "zh-CN/"]) {
    const source = readFileSync(resolve(root, `docs/content/${locale}stability.md`), "utf8");
    const rendered = native.transform(source, {});
    assert.deepEqual(rendered.errors, [], locale);
    for (const name of ["production-readiness", "support-policy", "vue-parity-matrix"]) {
      const destination = `https://github.com/ubugeeei-prod/vize/blob/main/docs/release/${name}.md`;
      const anchors = [...rendered.html.matchAll(/<a\b[^>]*href="([^"]+)"[^>]*>/g)].filter(
        (match) => decodeURI(match[1]) === destination,
      );
      assert.equal(anchors.length, 1, `${locale}${name}: original destination remains whole`);
      assert.match(anchors[0][0], /target="_blank"/);
      assert.match(anchors[0][0], /rel="noopener noreferrer"/);
      assert.doesNotMatch(anchors[0][1], /\.md$/, "bare extension must not become a site route");
    }
  }
});
