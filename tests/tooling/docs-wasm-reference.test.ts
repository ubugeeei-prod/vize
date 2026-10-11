import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const file = (path: string) => readFileSync(new URL(`../../${path}`, import.meta.url), "utf8");
const fixture = JSON.parse(file("tests/tooling/fixtures/docs-japanese-wasm.json"));

test("WASM locale guidance matches lint and compiler option boundaries", () => {
  const inventory = file("crates/vize_vitrine/src/wasm/options.rs");
  assert.doesNotMatch(inventory, /=>\s*\("locale"/);
  const declaration = file("npm/wasm/index.d.ts").match(
    /export interface CompilerOptions \{([\s\S]*?)\n\}/,
  )?.[1];
  assert.ok(declaration);
  assert.doesNotMatch(declaration, /\blocale\??:/);
  assert.match(file("npm/wasm/lint-format.d.ts"), /locale\?: "en" \| "ja" \| "zh"/);
  assert.match(
    file("npm/wasm/index.js"),
    /compileSfc\(source, options = \{\}\)[\s\S]*?return wasmCompileSfc\(source, options\)/,
  );
  assert.match(file("crates/vize_vitrine/src/wasm/lint.rs"), /JsValue::from_str\("locale"\)/);
  for (const prefix of ["", "ja/"]) {
    const guide = file(`docs/content/${prefix}guide/wasm.md`);
    assert.match(guide, /`lintSfc`[^\n]*`locale`|`locale`[^\n]*`lintSfc`/);
    assert.match(guide, /`CompilerOptions`[^\n]*`locale`/);
    assert.doesNotMatch(
      guide,
      /All WASM APIs that produce diagnostics|診断 .*を生成するすべての WASM API/,
    );
    assert.doesNotMatch(guide, /1\.5 MB/);
    assert.ok(guide.includes("gzip -c npm/wasm/vize_vitrine_bg.wasm | wc -c"));
  }
});

test("WASM factual review retains every full published example and compiler contract", () => {
  const source = file("docs/content/guide/wasm.md"),
    translation = file("docs/content/ja/guide/wasm.md");
  const recipes = (text: string) => [...text.matchAll(/```[^\n]*\n[\s\S]*?```/g)].map((m) => m[0]);
  assert.deepEqual(recipes(translation), fixture.blocks, "all original recipe bytes");
  assert.deepEqual(recipes(translation), recipes(source), "all complete current English recipes");
  for (const block of fixture.blocks)
    assert.ok(translation.includes(block), `missing published example: ${block}`);
  const inline = (text: string) =>
    new Set(
      [...text.replace(/```[^\n]+\n[\s\S]*?```/g, "").matchAll(/`([^`\n]+)`/g)].map((m) => m[1]),
    );
  for (const code of inline(source))
    assert.ok(inline(translation).has(code), `missing compiler contract: ${code}`);
});
