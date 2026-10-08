import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { parseSync } from "@babel/core";
import tsSyntax from "@babel/plugin-syntax-typescript";
import { parse as parseSfc, compileScript } from "vue-computed-inlay-oracle/compiler-sfc";
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
      const [bad, good] = locale ? ["悪い", "良い"] : ["bad", "good"];
      assert.ok(index.includes(`./reference/${file}#${bad}`), `${name}: direct Bad link`);
      assert.ok(index.includes(`./reference/${file}#${good}`), `${name}: direct Good link`);
      for (const heading of locale ? ["悪い", "良い"] : ["Bad", "Good"]) {
        const rationale = page.split(`## ${heading}\n`)[1].split("```")[0].trim();
        assert.ok(rationale.length > 25, `${name}: explain the authored ${heading} example`);
      }
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

await test("Good scripts and illustrative Bad sources have valid module grammar and unique bindings", () => {
  let scripts = 0;
  for (const { file, language, source: example } of validatedExamples()) {
    const sources =
      language === "ts"
        ? [example]
        : [...example.matchAll(/<script\b[^>]*>([\s\S]*?)<\/script>/g)].map((match) => match[1]);
    for (const source of sources) {
      scripts += 1;
      assert.doesNotThrow(
        () =>
          parseSync(source, {
            filename: file.replace(/\.md$/, ".ts"),
            sourceType: "module",
            configFile: false,
            babelrc: false,
            plugins: [tsSyntax],
          }),
        file,
      );
    }
  }
  assert.ok(scripts > 100, "all authored script examples are parsed");
});

await test("Good Vue scripts and illustrative Bad sources have valid compiler-macro contexts", () => {
  let scripts = 0;
  for (const { file, language, source } of validatedExamples()) {
    if (language !== "vue" || !/<script\b/.test(source)) continue;
    scripts += 1;
    const { descriptor, errors } = parseSfc(source, { filename: file.replace(/\.md$/, ".vue") });
    assert.deepEqual(errors, [], file);
    // This checks the authored example, not Vize compiler output or parity.
    assert.doesNotThrow(() => compileScript(descriptor, { id: file }), file);
  }
  assert.ok(scripts > 90, "all Good SFC script contexts are checked");
});

function validatedExamples() {
  return ["reference", "project"].flatMap((section) =>
    readdirSync(resolve(root, `docs/content/rules/${section}`)).flatMap((file) => {
      const source = read(`docs/content/rules/${section}/${file}`);
      const parts = [source.split("## Good\n")[1]];
      if (source.includes("Example qualification: `illustrative-source-pair`")) {
        // This input explains a retained graph contract, not an emitted CLI finding.
        // Validate its complete shared context and Bad source as well as Good.
        assert.equal(file, "vize-croquis-cf-circular-reactive-dependency.md");
        parts.push(
          source.split("## Bad\n")[1].split("## Good\n")[0],
          source.split("## Shared project files\n")[1].split("## Bad\n")[0],
        );
      }
      return parts.flatMap((part) =>
        [...part.matchAll(/```(vue|ts|html)\n([\s\S]*?)\n```/g)].map((match) => ({
          file: `${section}/${file}`,
          language: match[1],
          source: match[2],
        })),
      );
    }),
  );
}

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
    ["cli", 19],
    ["library", 16],
    ["contract", 25],
  ])
    assert.equal(codes.filter((code) => code.status === status).length, count);
  for (const locale of ["", "ja/"]) {
    const index = read(`docs/content/${locale}rules/cross-file.md`);
    const all = read(`docs/content/${locale}rules/all.md`);
    const rows = [...index.matchAll(/^\| \[`([^`]+)`\]\(\.\/project\/([^)]*)\)/gm)];
    assert.equal(rows.length, 66);
    for (const [_, id, path] of rows) {
      const page = read(`docs/content/${locale}rules/project/${path}`);
      assert.ok(page.includes(`# \`${id}\``), id);
      const [bad, good] = locale ? ["悪い", "良い"] : ["bad", "good"];
      for (const source of [index, all]) {
        assert.ok(source.includes(`./project/${path}#${bad}`), `${id}: direct Bad link`);
        assert.ok(source.includes(`./project/${path}#${good}`), `${id}: direct Good link`);
      }
      for (const label of locale
        ? ["既定の重大度:", "適用範囲:", "オプション:", "## 悪い", "## 良い"]
        : ["Default severity:", "Applies to:", "Options:", "## Bad", "## Good"])
        assert.ok(page.includes(label), `${id}: ${label}`);
      for (const heading of locale ? ["悪い", "良い"] : ["Bad", "Good"])
        assert.match(
          page.split(`## ${heading}\n`)[1],
          /```(?:vue|ts|html)\n/,
          `${id}: concrete ${heading} scenario`,
        );
      assert.deepEqual(
        codeBlocks(page),
        codeBlocks(read(`docs/content/rules/project/${path}`)),
        id,
      );
    }
  }
});
