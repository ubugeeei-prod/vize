import assert from "node:assert/strict";
import { readFileSync, readdirSync, existsSync } from "node:fs";
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

await test("the generated bilingual reference covers every implemented rule with real example context", () => {
  const names = [...implementations()].sort((a, b) => a.localeCompare(b));
  assert.equal(names.length, 251);
  for (const locale of ["", "ja/"]) {
    const index = read(`docs/content/${locale}rules/all.md`);
    const links = [
      ...index.matchAll(
        /^\| \[`([^`]+)`\]\(https:\/\/vizejs\.dev\/(?:ja\/)?rules\/all\.html#([^)]*)\)/gm,
      ),
    ].filter((match) => names.includes(match[1]));
    assert.deepEqual(
      links.map((match) => match[1]).sort((a, b) => a.localeCompare(b)),
      names,
    );
    for (const [, name, slug] of links) {
      const file = `${slug}.md`;
      const page = read(`docs/content/${locale}rules/reference/${file}`);
      assert.ok(page.includes(`# \`${name}\``), name);
      assert.ok(index.includes(`#${slug}-bad`), `${name}: same-page Bad link`);
      assert.ok(index.includes(`#${slug}-good`), `${name}: same-page Good link`);
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
    assert.match(page, /```ts annotate="remove:1,2;add:3,4,5,6"/);
    assert.match(page, /^ import vue/m);
    assert.match(page, /^ import \{ defineConfig \}/m);
  }
});

function codeBlocks(page: string) {
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
      assert.ok(index.includes(`./project/${path}#${bad}`), `${id}: direct Bad link`);
      assert.ok(index.includes(`./project/${path}#${good}`), `${id}: direct Good link`);
      const slug = path.slice(0, -3);
      assert.ok(all.includes(`#${slug}-bad`), `${id}: same-page Bad link`);
      assert.ok(all.includes(`#${slug}-good`), `${id}: same-page Good link`);
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

await test("both catalogue pages retain all 317 complete reference examples and same-page targets", () => {
  for (const [locale, generated] of [
    ["", "en"],
    ["ja/", "ja"],
  ]) {
    const catalogue = read(`docs/content/generated/rules/${generated}/all.md`);
    const targets = [...catalogue.matchAll(/<span id="([^"\n]+)"><\/span>\n\n### `([^`]+)`/g)];
    assert.equal(targets.length, 317);
    assert.equal(new Set(targets.map((match) => match[1])).size, 317);
    for (let index = 0; index < targets.length; index += 1) {
      const [_, slug, id] = targets[index];
      const section = catalogue.slice(targets[index].index, targets[index + 1]?.index);
      const directories = ["reference", "project"].filter((directory) =>
        existsSync(resolve(root, `docs/content/${locale}rules/${directory}/${slug}.md`)),
      );
      assert.equal(directories.length, 1, `${id}: one source reference`);
      const directory = directories[0];
      const reference = read(`docs/content/${locale}rules/${directory}/${slug}.md`);
      assert.deepEqual(
        codeBlocks(section),
        codeBlocks(reference),
        `${locale}${id}: every complete fenced byte preserved`,
      );
      for (const [kind, label] of [
        ["bad", locale ? "悪い" : "Bad"],
        ["good", locale ? "良い" : "Good"],
      ]) {
        assert.ok(
          section.includes(`<span id="${slug}-${kind}"></span>`),
          `${id}: unique ${kind} target`,
        );
        const rationale = reference.split(`## ${label}\n`)[1].split("```")[0].trim();
        assert.ok(section.includes(rationale), `${id}: whole authored ${kind} explanation/context`);
      }
    }
    const anchors = new Set(
      [...catalogue.matchAll(/<span id="([^"\n]+)"><\/span>/g)].map((match) => match[1]),
    );
    for (const [, anchor] of catalogue.matchAll(/\]\(#([^)]*)\)/g))
      assert.ok(anchors.has(anchor), `${locale}: missing same-page ${anchor}`);
  }
});

await test("all five Vue categories keep every rule and whole Bad/Good source on the current page", () => {
  const names = [...implementations()].filter((name) => name.startsWith("vue/")).sort();
  assert.equal(names.length, 104);
  const sources = (markdown: string) =>
    [...markdown.matchAll(/```(\w+)[^\n]*\n([\s\S]*?)\n```/g)].map((match) => [match[1], match[2]]);
  const legacyAnchors: Record<string, readonly string[]> = {
    "zh-CN/": ["句法与风格规则"],
    "pt-BR/": ["regras-do-vue", "regras-de-sintaxe-e-estilo"],
    "fr/": ["syntaxe-et-règles-de-style"],
  };
  for (const locale of ["", "ja/", "zh-CN/", "pt-BR/", "fr/"]) {
    const category = read(`docs/content/generated/rules/${locale.slice(0, -1) || "en"}/vue.md`);
    for (const anchor of legacyAnchors[locale] ?? [])
      assert.equal(
        category.split(`<span id="${anchor}"></span>`).length,
        2,
        `${locale}: preserve existing category fragment ${anchor}`,
      );
    const targets = [...category.matchAll(/^### `([^`]+)`$/gm)];
    assert.deepEqual(
      targets.map((match) => match[1]).sort(),
      names,
      `${locale}: complete Vue rules`,
    );
    for (let index = 0; index < targets.length; index += 1) {
      const [, name] = targets[index];
      const slug = name.replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase();
      const section = category.slice(targets[index].index, targets[index + 1]?.index);
      const reference = read(
        `docs/content/${locale === "ja/" ? locale : ""}rules/reference/${slug}.md`,
      );
      assert.deepEqual(
        sources(section),
        sources(reference),
        `${locale}${name}: whole copyable bytes`,
      );
      for (const kind of ["bad", "good"]) {
        assert.equal(section.split(`<span id="${slug}-${kind}"></span>`).length, 2);
        assert.ok(category.includes(`](#${slug}-${kind})`), `${locale}${name}: local ${kind} link`);
      }
    }
    assert.match(category, /```vue annotate="remove:/, `${locale}: native removed-line metadata`);
    assert.match(category, /```vue annotate="add:/, `${locale}: native added-line metadata`);
    assert.doesNotMatch(category, /\]\(\.\/all\.md#vue-/, `${locale}: examples stay on this page`);
    assert.doesNotMatch(
      category.split("### `")[0],
      /`&lt;(?:template|component|KeepAlive)&gt;`/,
      `${locale}: inline-code element names retain literal spelling`,
    );
    const noProducer = category.split("### `vue/no-preprocessor-lang`")[1].split("### `")[0];
    assert.match(noProducer, /`no-sfc-finding`/, `${locale}: support boundary remains explicit`);
  }
});
