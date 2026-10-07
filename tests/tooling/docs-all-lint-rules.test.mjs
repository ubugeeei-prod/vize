import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { crossMetadata } from "../../docs/scripts/rules/project-metadata.mjs";

const root = resolve(import.meta.dirname, "../..");
const read = (path) => readFileSync(resolve(root, path), "utf8");

function implementations(directory = "crates/vize_patina/src/rules") {
  const found = new Set();
  for (const entry of readdirSync(resolve(root, directory), { withFileTypes: true })) {
    const path = `${directory}/${entry.name}`;
    if (entry.isDirectory() && entry.name !== "snapshots") {
      for (const name of implementations(path)) found.add(name);
    } else if (entry.isFile() && entry.name.endsWith(".rs")) {
      for (const meta of read(path).matchAll(
        /static\s+[A-Z_]*META[A-Z_]*\s*:\s*(?:RuleMeta|ScriptRuleMeta|CssRuleMeta|MuseaRuleMeta)\s*=\s*\w+\s*\{([\s\S]*?)\n\};/g,
      )) {
        const name = meta[1].match(/name:\s*"([^"]+)"/)?.[1];
        if (name) found.add(name);
      }
    }
  }
  return found;
}

await test("the generated bilingual reference covers every implemented rule with real example context", () => {
  const names = [...implementations()].sort((a, b) => a.localeCompare(b));
  assert.equal(names.length, 251);
  for (const locale of ["", "ja/"]) {
    const index = read(`docs/content/${locale}rules/all.md`);
    const links = [...index.matchAll(/^\| \[`([^`]+)`\]\(\.\/reference\/([^)]*)\)/gm)];
    assert.deepEqual(
      links.map((match) => match[1]).sort((a, b) => a.localeCompare(b)),
      names,
    );
    for (const [, name, file] of links) {
      const page = read(`docs/content/${locale}rules/reference/${file}`);
      assert.ok(page.includes(`# \`${name}\``), name);
      for (const label of locale
        ? ["既定の重大度:", "プリセット:", "適用範囲:", "オプション:", "## 悪い", "## 良い"]
        : ["Default severity:", "Presets:", "Applies to:", "Options:", "## Bad", "## Good"]) {
        assert.ok(page.includes(label), `${name}: ${label}`);
      }
      assert.match(page, /@vizejs\/vite-plugin\/vite-plus/);
      assert.match(page, /vp run lint/);
      assert.ok([...page.matchAll(/```(?:vue|ts|html)\n[\s\S]*?\n```/g)].length >= 3, name);
      const en = read(`docs/content/rules/reference/${file}`);
      assert.deepEqual(codeBlocks(page), codeBlocks(en), `${name}: EN/JA code must stay identical`);
    }
  }
});

await test("generation is deterministic without a previously built native binary", () => {
  const result = spawnSync(
    process.execPath,
    ["docs/scripts/generate-patina-rules-page.mjs", "--check"],
    { cwd: root, encoding: "utf8" },
  );
  assert.equal(result.status, 0, result.stderr || result.stdout);
});

await test("migration retains all mapped, divergent and unsupported ESLint identities", () => {
  const inventory = JSON.parse(read("tests/_fixtures/patina-eslint-vue-rule-map.json"));
  for (const locale of ["", "ja/"]) {
    const page = read(`docs/content/${locale}rules/migration.md`);
    const rows = [...page.matchAll(/^\| `([^`]+)` \|/gm)]
      .map((match) => match[1])
      .sort((a, b) => a.localeCompare(b));
    assert.deepEqual(
      rows,
      Object.keys(inventory.entries).sort((a, b) => a.localeCompare(b)),
    );
    assert.match(page, /252/);
    assert.match(page, /123/);
    assert.match(page, /127/);
    assert.match(page, /component-definition-name-casing/);
    assert.match(page, /^- import vue/m);
    assert.match(page, /^\+ import \{ defineConfig \}/m);
  }
});

function codeBlocks(page) {
  return [...page.matchAll(/```\w+\n[\s\S]*?\n```/g)].map((match) => match[0]);
}

await test("project references retain every code and distinguish actual CLI producers", () => {
  const codes = crossMetadata(root);
  assert.equal(codes.length, 60);
  for (const [status, count] of [
    ["cli", 20],
    ["library", 15],
    ["contract", 25],
  ])
    assert.equal(codes.filter((code) => code.status === status).length, count);
  for (const locale of ["", "ja/"]) {
    const index = read(`docs/content/${locale}rules/cross-file.md`);
    const rows = [...index.matchAll(/^\| \[`([^`]+)`\]\(\.\/project\/([^)]*)\)/gm)];
    assert.equal(rows.length, 66);
    for (const [_, id, path] of rows) {
      const page = read(`docs/content/${locale}rules/project/${path}`);
      assert.ok(page.includes(`# \`${id}\``), id);
      for (const label of locale
        ? ["既定の重大度:", "適用範囲:", "オプション:", "## 悪い", "## 良い"]
        : ["Default severity:", "Applies to:", "Options:", "## Bad", "## Good"])
        assert.ok(page.includes(label), `${id}: ${label}`);
      assert.deepEqual(
        codeBlocks(page),
        codeBlocks(read(`docs/content/rules/project/${path}`)),
        id,
      );
    }
  }
});
