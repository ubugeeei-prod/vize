import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
import { collectReviewedTranslations } from "../../docs/scripts/i18n/reviewed.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const require = createRequire(new URL("../../docs/package.json", import.meta.url));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");
const baseline = JSON.parse(
  fs.readFileSync(path.join(root, "tests/tooling/fixtures/docs-ja-wasm.json"), "utf8"),
) as { base: string; file: string; headings: string[]; code: string[]; links: string[] };

test("English and Japanese WASM guides use the public nested SFC compile result", () => {
  for (const locale of ["", "ja/"]) {
    const guide = fs.readFileSync(path.join(root, `docs/content/${locale}guide/wasm.md`), "utf8");

    assert.doesNotMatch(guide, /result\.code\b/);
    assert.doesNotMatch(
      guide,
      /const\s+\{\s*code(?:\s*,\s*errors)?\s*\}\s*=\s*(?:compiler\.)?compileSfc\b/,
    );
    assert.match(guide, /result\.script\.code/);
    assert.match(guide, /result\.template\?\.code/);
    assert.match(guide, /result\.css/);
    assert.match(guide, /result\.errors/);
  }
});

test("Japanese WASM review retains complete examples, API contracts and bookmarks", async () => {
  const japanese = fs.readFileSync(path.join(root, baseline.file), "utf8");
  const english = fs.readFileSync(path.join(root, "docs/content/guide/wasm.md"), "utf8");
  const rendered = native.transform(japanese, {});
  assert.equal(baseline.base, "0cfbdcb98c8df2aff2e526076deb44fb1d9a1ee5");
  assert.deepEqual(rendered.errors, []);
  const code = (source: string) =>
    Array.from(source.matchAll(/```[^\n]*\n([\s\S]*?)\n```/g), (m) => m[1]);
  assert.equal(baseline.code.length, 10);
  assert.deepEqual(code(japanese), baseline.code, "all original recipe bytes");
  assert.deepEqual(code(japanese), code(english), "all complete current English recipes");
  assert.deepEqual(
    Array.from(japanese.matchAll(/\]\(([^)]+)\)/g), (m) => m[1]),
    baseline.links,
    "original destinations",
  );
  const contracts = (source: string) =>
    [
      ...new Set(
        Array.from(
          source.replace(/```[^\n]*\n[\s\S]*?\n```/g, "").matchAll(/`([^`]+)`/g),
          (m) => m[1],
        ),
      ),
    ].toSorted();
  assert.deepEqual(contracts(japanese), contracts(english));
  const ids = new Set(Array.from(rendered.html.matchAll(/\bid="([^"]+)"/g), (m) => m[1]));
  assert.equal(baseline.headings.length, 16);
  for (const id of baseline.headings) assert.ok(ids.has(id), `original #${id}`);
  const retained = await collectReviewedTranslations(
    ["guide/wasm.md"],
    path.join(root, "docs/content/ja"),
  );
  assert.equal(retained.get("guide/wasm.md"), japanese, "complete reviewed regeneration input");
  assert.ok(japanese.includes("<!-- Reviewed translation; source: guide/wasm.md -->"));
});
