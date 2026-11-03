import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { createRequire } from "node:module";

// These pinned CommonJS oracles do not publish declarations. Keep their API
// boundary narrow while retaining the exact parser and plugin used before.
const require = createRequire(import.meta.url);
const { parseSync } = require("@babel/core") as {
  parseSync(
    this: void,
    source: string,
    options: {
      filename: string;
      sourceType: "module";
      configFile: false;
      babelrc: false;
      plugins: unknown[];
    },
  ): unknown;
};
const tsSyntax: unknown = require("@babel/plugin-syntax-typescript");
import { parse as parseSfc, compileScript } from "vue-computed-inlay-oracle/compiler-sfc";
import { crossMetadata } from "../../docs/scripts/rules/project-metadata.ts";
import { verifyRuleSources } from "../../docs/scripts/rule-render-assertions.ts";
import { ruleGroups } from "../../docs/scripts/rules/rule-groups.ts";
import { ruleHeadings } from "../../docs/scripts/rules/rule-page.ts";
import { defaultNavigationRenderRoutes } from "../../docs/scripts/navigation-render-routes.ts";
import { createRenderJobs } from "../../docs/scripts/navigation-render-concurrency.ts";

const root = resolve(import.meta.dirname, "../..");
const read = (path: string) => readFileSync(resolve(root, path), "utf8");

function implementations(directory = "crates/vize_patina/src/rules") {
  const found = new Set<string>();
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

const locales = ["en", "ja", "zh-CN", "pt-BR", "fr"] as const;
const prefix = (locale: string) => (locale === "en" ? "" : `${locale}/`);
const slug = (id: string) => id.replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase();
const facts: Record<(typeof locales)[number], string> = {
  en: "| Default severity | Presets | Automatic fix | Options |",
  ja: "| 既定の重大度 | プリセット | 自動修正 | オプション |",
  "zh-CN": "| 默认严重程度 | 预设 | 自动修复 | 选项 |",
  "pt-BR": "| Severidade padrão | Predefinições | Correção automática | Opções |",
  fr: "| Gravité par défaut | Préréglages | Correction automatique | Options |",
};

await test("every implemented rule has one readable page per locale with the executed examples inline", () => {
  const names = [...implementations()].sort((a, b) => a.localeCompare(b));
  assert.equal(names.length, 267);
  for (const locale of locales) {
    const { bad, good } = ruleHeadings[locale];
    for (const name of names) {
      const file = `${slug(name)}.md`;
      const page = read(`docs/content/${prefix(locale)}rules/reference/${file}`);
      assert.ok(page.includes(`# \`${name}\``), `${locale} ${name}: identity`);
      assert.ok(page.includes(facts[locale]), `${locale} ${name}: facts table`);
      // Bad, then Good, then configuration: the reader sees the examples first.
      const order = [
        page.indexOf(`## ${bad}\n`),
        page.indexOf(`## ${good}\n`),
        page.lastIndexOf("\n```ts\nimport { defineConfig }"),
      ];
      assert.ok(
        order[0] > 0 && order[0] < order[1] && order[1] < order[2],
        `${locale} ${name}: order`,
      );
      for (const heading of [bad, good]) {
        const rationale = page.split(`## ${heading}\n`)[1].split("```")[0].trim();
        assert.ok(rationale.length > 10, `${locale} ${name}: explain the ${heading} example`);
      }
      assert.doesNotMatch(
        page,
        /\]\(#(?:bad|good|悪い|良い)\)/,
        `${locale} ${name}: no jump links`,
      );
      assert.match(page, /@vizejs\/vite-plugin\/vite-plus/);
      assert.match(page, /vp run lint/);
      const en = read(`docs/content/rules/reference/${file}`);
      assert.deepEqual(codeBlocks(page), codeBlocks(en), `${locale} ${name}: same code as English`);
    }
  }
  const unsupported = read("docs/content/rules/reference/vue-no-preprocessor-lang.md");
  assert.doesNotMatch(unsupported, /`no-sfc-finding`/, "executable SFC support");
  assert.match(
    unsupported,
    /```vue annotate="remove:2"\n<template><p>Notice<\/p><\/template>\n<style lang="scss">/,
  );
  assert.match(unsupported, /```vue annotate="add:2"\n/);
});

await test("generation is deterministic without a previously built native binary", () => {
  const result = spawnSync(
    process.execPath,
    ["docs/scripts/generate-patina-rules-page.ts", "--check"],
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
      const parts = [source.split("## Good\n")[1].split("\n## ")[0]];
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
        [...part.matchAll(/```(vue|ts|html)(?: [^\n]*)?\n([\s\S]*?)\n```/g)].map((match) => ({
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
    assert.match(page, /111/);
    assert.match(page, /139/);
    assert.match(page, /component-definition-name-casing/);
    assert.match(page, /```ts annotate="remove:1,2;add:3,4,5,6"/);
    assert.match(page, /^ import vue/m);
    assert.match(page, /^ import \{ defineConfig \}/m);
  }
});

function codeBlocks(page: string) {
  return [...page.matchAll(/```(\w+)[^\n]*\n([\s\S]*?)\n```/g)].map((match) => [
    match[1],
    match[2],
  ]);
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
  const english = read("docs/content/rules/cross-file.md");
  for (const status of ["Experimental; not reported by the CLI yet", "Planned; not reported yet"])
    assert.ok(english.includes(status), status);
  for (const locale of locales) {
    const index = read(`docs/content/${prefix(locale)}rules/cross-file.md`);
    const rows = [
      ...index.matchAll(/^\| <span id="[^"]+"><\/span>\[`([^`]+)`\]\(\.\/project\/([^)]+)\.md\)/gm),
    ];
    assert.equal(rows.length, 66, locale);
    for (const [, id, file] of rows) {
      const page = read(`docs/content/${prefix(locale)}rules/project/${file}.md`);
      assert.ok(page.includes(`# \`${id}\``), id);
      for (const heading of [ruleHeadings[locale].bad, ruleHeadings[locale].good])
        assert.match(
          page.split(`## ${heading}\n`)[1],
          /```(?:vue|ts|html)\n/,
          `${locale} ${id}: concrete ${heading} scenario`,
        );
      assert.deepEqual(
        codeBlocks(page),
        codeBlocks(read(`docs/content/rules/project/${file}.md`)),
        id,
      );
    }
  }
});

await test("every locale's rule indexes list each rule once and link to its page", () => {
  for (const locale of locales)
    for (const page of ["all", ...ruleGroups, "cross-file"])
      verifyRuleSources(`/${prefix(locale)}rules/${page}`);
});

await test("every supported All Rules index receives native desktop and mobile reader coverage", () => {
  const jobs = createRenderJobs(defaultNavigationRenderRoutes);
  for (const prefix of ["", "/ja", "/zh-CN", "/pt-BR", "/fr"]) {
    for (const device of ["desktop", "mobile"]) {
      assert.equal(
        jobs.filter((job) => job.route === `${prefix}/rules/all` && job.device === device).length,
        1,
        `${prefix || "en"}: one complete native ${device} index review`,
      );
    }
  }
});

await test("every documented native rule uses the Rust-checked preset membership", () => {
  const snapshot: Record<string, string[]> = JSON.parse(
    read(
      "crates/vize_patina/src/preset/snapshots/vize_patina__preset__tests__lint_preset_rule_membership.snap",
    ).split(/\r?\n---\r?\n/, 2)[1],
  );
  const names = [...implementations()];
  for (const [preset, rules] of Object.entries(snapshot)) {
    if (preset === "opt_in") continue;
    for (const rule of rules)
      assert.ok(names.includes(rule), `${preset}: ${rule} has an implementation`);
  }
  for (const locale of locales) {
    for (const name of names) {
      const page = read(`docs/content/${prefix(locale)}rules/reference/${slug(name)}.md`);
      const row = page.split("\n").find((line) => /^\| `(?:error|warning)` \|/.test(line));
      assert.ok(row, `${locale} ${name}: explicit severity`);
      const documented = [...row.split(" | ")[1].matchAll(/`([^`]+)`/g)]
        .map((match) => match[1])
        .sort();
      const actual = Object.entries(snapshot)
        .filter(([preset, rules]) => preset !== "opt_in" && rules.includes(name))
        .map(([preset]) => preset.replaceAll("_", "-"))
        .sort();
      assert.deepEqual(documented, actual, `${locale} ${name}: native preset membership`);
    }
  }
});
