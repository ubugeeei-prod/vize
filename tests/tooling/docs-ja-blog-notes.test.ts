import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { test } from "node:test";
import { collectReviewedTranslations } from "../../docs/scripts/i18n/reviewed.ts";

const root = resolve(import.meta.dirname, "../..");
const require = createRequire(new URL("../../docs/package.json", import.meta.url));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");
const locale = resolve(root, "docs/content/ja");
const paths = [
  "blog/notes/2026-05-16-vapor-mode-and-the-next-vue-compiler-surface.md",
  "blog/notes/2026-05-16-why-musea-and-design-systems-matter-in-the-ai-era.md",
  "blog/index.md",
  "blog/notes/index.md",
];

void test("reviewed Japanese blog pages render and survive translation regeneration", async () => {
  const retained = await collectReviewedTranslations(paths, locale);
  for (const path of paths) {
    const reviewed = readFileSync(resolve(locale, path), "utf8");
    assert.deepEqual(native.transform(reviewed, {}).errors, [], path);
    assert.equal(retained.get(path), reviewed, path);
    assert.ok(reviewed.includes(`<!-- Reviewed translation; source: ${path}`), path);
  }
});
