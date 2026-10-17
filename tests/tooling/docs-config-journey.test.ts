import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
import { test } from "node:test";
import {
  configRecipe,
  nativeOptions,
  includesOptions,
  retiredPklOptions,
} from "./support/docs-config-semantics.ts";
import { currentConfigExample } from "./support/docs-config-successor.ts";
import { renderReferenceDocs } from "../../npm/ui/scripts/generate-reference-docs.ts";

const root = resolve(import.meta.dirname, "../..");
const require = createRequire(new URL("../../docs/package.json", import.meta.url));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");
const locales = ["", "ja", "fr", "pt-BR", "zh-CN"];
type OriginalDocument = { file: string; headings: string[]; code: string[] };
const original = JSON.parse(
  readFileSync(resolve(import.meta.dirname, "fixtures/docs-config-journey.json"), "utf8"),
) as { base: string; documents: OriginalDocument[] };
const fences = (source: string): string[] =>
  Array.from(source.matchAll(/```[^\n]*\n([\s\S]*?)\n```/g), (match) => match[1] ?? "");
const successor = JSON.parse(
  readFileSync(
    resolve(import.meta.dirname, "fixtures/docs-config-current-successors.json"),
    "utf8",
  ),
) as {
  originalSha256: string;
  recipes: {
    index: number;
    originalCodeSha256: string;
    code: string;
    additionalCode?: string[];
    ownership: string;
  }[];
  workflow: string;
};
let generated: Map<string, string> | undefined;

function sourceFor(file: string): string | undefined {
  const absolute = resolve(root, file);
  if (existsSync(absolute)) return readFileSync(absolute, "utf8");
  generated ??= renderReferenceDocs();
  return generated.get(file.replace(/^docs\/content\//u, ""));
}

function render(file: string): { html: string; source: string } {
  const source = sourceFor(file);
  assert.ok(source !== undefined, `${file}: source or actual generated reference exists`);
  const result = native.transform(source, {});
  assert.deepEqual(result.errors, [], file);
  return { source, html: result.html as string };
}

void test("configuration reorganization retains every original native heading target", () => {
  assert.equal(original.base, "fe3934d8a24c7d2fe0d26b2ca095d61c90f49b1f");
  assert.equal(original.documents.length, 15);
  let total = 0;
  for (const document of original.documents) {
    const { html } = render(document.file);
    const ids = Array.from(html.matchAll(/\bid="([^"]+)"/g), (match) => match[1]);
    for (const id of document.headings) {
      assert.equal(
        ids.filter((value) => value === id).length,
        1,
        `${document.file}: original #${id} occurs once`,
      );
      total += 1;
    }
  }
  assert.equal(total, 172);
});

void test("configuration references preserve every complete archived recipe through explicit successors", () => {
  assert.equal(
    createHash("sha256")
      .update(readFileSync(resolve(import.meta.dirname, "fixtures/docs-config-journey.json")))
      .digest("hex"),
    successor.originalSha256,
  );
  assert.deepEqual(
    successor.recipes.map((recipe) => recipe.index),
    Array.from({ length: 14 }, (_, i) => i),
  );
  let count = 0;
  for (const document of original.documents.filter((item) => item.code.length > 0)) {
    const directory = dirname(document.file);
    const retained = ["configuration-reference", "compiler-configuration-reference"].flatMap(
      (name) => fences(render(`${directory}/${name}.md`).source),
    );
    assert.equal(document.code.length, 14, document.file);
    for (const [index, code] of document.code.entries()) {
      const current = successor.recipes[index];
      assert.equal(createHash("sha256").update(code).digest("hex"), current.originalCodeSha256);
      assert.ok(current.ownership.length > 0);
      for (const packet of [current.code, ...(current.additionalCode ?? [])]) {
        assert.ok(
          retained.includes(packet),
          `${document.file}: complete successor for original example ${index}`,
        );
      }
      count += 1;
    }
  }
  assert.equal(count, 42);
});

void test("every locale publishes the same complete Vite setup and native config recipes", () => {
  const recipes = fences(render("docs/content/getting-started.md").source);
  const configuration = fences(render("docs/content/guide/configuration.md").source);
  for (const locale of locales) {
    const prefix = `docs/content/${locale ? `${locale}/` : ""}`;
    const startup = render(`${prefix}getting-started.md`);
    const config = render(`${prefix}guide/configuration.md`);
    assert.deepEqual(fences(startup.source), recipes, locale);
    assert.deepEqual(fences(config.source), configuration, locale);
    assert.match(config.source, /`tsconfig\.json`/);
    assert.match(config.source, /`vueCompilerOptions`/);
    assert.match(config.source, /defineConfig\(vpConfig\)\.withVue/);
    assert.match(config.source, /`--tsconfig`/);
    assert.doesNotMatch(config.html, /<strong>\s*<code>vize\.config/);
    const startupNote = startup.source.match(/^> \[!NOTE\]\n(> .+)$/m)?.[1];
    assert.ok(startupNote, `${locale}: publication scope stays visible`);
    for (const option of [
      "`vue:`",
      "`vite.config.*`",
      "`tsconfig.json`",
      "`vueCompilerOptions`",
      "`vize.config.*`",
      "`vize:`",
    ]) {
      assert.ok(startupNote.includes(option), `${locale}: startup migration explains ${option}`);
    }
    assert.ok(config.source.includes("> [!NOTE]"), `${locale}: migration scope stays visible`);
    assert.match(config.source, /`vize\.config\.\*`/);
    for (const document of [startup, config]) {
      assert.doesNotMatch(
        document.source,
        /0\.457\.0/,
        `${locale}: configuration migration must not claim the released setup baseline`,
      );
      assert.doesNotMatch(
        document.source,
        /next release|次(?:の|回)リリース|prochaine version|próxima versão|下一个版本/u,
        `${locale}: configuration instructions remain concrete`,
      );
    }
  }
});

void test("localized configuration reading paths resolve through native rendered targets", () => {
  const rendered = new Map<string, ReturnType<typeof render>>();
  for (const locale of locales) {
    const prefix = `docs/content/${locale ? `${locale}/` : ""}`;
    for (const name of [
      "getting-started.md",
      "guide/configuration.md",
      "guide/configuration-reference.md",
      "guide/compiler-configuration-reference.md",
    ]) {
      const file = `${prefix}${name}`;
      const document = render(file);
      for (const [, href, fragment] of document.html.matchAll(/\bhref="([^"#]*\.md)(#[^"]*)?"/g)) {
        if (!href || /^(?:https?:|\/\/)/u.test(href)) continue;
        const target = href.startsWith("/")
          ? resolve(root, "docs/content", `.${href}`)
          : resolve(root, dirname(file), href);
        const relative = target.slice(`${root}/`.length);
        const chosen = relative;
        assert.ok(sourceFor(chosen) !== undefined, `${file}: ${href}`);
        if (!rendered.has(chosen)) rendered.set(chosen, render(chosen));
        if (fragment) {
          const id = decodeURIComponent(fragment.slice(1));
          const html = rendered.get(chosen)?.html;
          assert.ok(html?.includes(`id="${id}"`), `${file}: ${href}${fragment}`);
        }
      }
    }
  }
  assert.ok(rendered.size >= 30, "the complete localized reading paths were visited");
});

void test("library and Musea shared examples keep the same native scope in every locale", () => {
  const shared = (source: string, section: string) =>
    fences(source).find((code) => code.includes(`    ${section}: {`));
  const musea = shared(render("docs/content/guide/configuration-reference.md").source, "musea");
  const lib = shared(render("docs/content/guide/lib-pull.md").source, "lib");
  assert.ok(musea && lib, "complete authoritative English examples exist");
  for (const locale of locales) {
    const prefix = `docs/content/${locale ? `${locale}/` : ""}guide/`;
    const museaFile = locale ? "musea.md" : "configuration-reference.md";
    assert.equal(shared(render(`${prefix}${museaFile}`).source, "musea"), musea, locale);
    const library = render(`${prefix}lib-pull.md`);
    assert.equal(shared(library.source, "lib"), lib, locale);
    assert.match(library.source, /`vite\.config\.mjs`/);
    assert.match(library.source, /`vue\.lib`/);
    assert.match(library.source, /`lib`/);
    assert.doesNotMatch(library.source, /writes?[^\n]*`vize\.config/);
  }
});

void test("the archived dedicated workflow has a complete canonical successor in both references", () => {
  const retained = JSON.parse(
    readFileSync(
      resolve(import.meta.dirname, "fixtures/docs-workflow-dedicated-config.json"),
      "utf8",
    ),
  ) as { base: string; code: string; codeSha256: string };
  assert.equal(retained.base, "878d7833c2a12391bb4111c6894dbe1d2c11fda3");
  assert.equal(createHash("sha256").update(retained.code).digest("hex"), retained.codeSha256);
  assert.equal(retained.code.split("\n").length, 18);
  for (const locale of ["", "ja/"]) {
    const reference = render(`docs/content/${locale}guide/configuration-reference.md`);
    assert.equal(fences(reference.source).filter((code) => code === successor.workflow).length, 1);
    const workflow = render(`docs/content/${locale}guide/workflows.md`);
    assert.ok(fences(workflow.source).some((code) => code.includes("// vite.config.ts")));
  }
});

void test("canonical recipe successors retain complete option values and actual package ownership", () => {
  const archived = original.documents.find((document) => document.code.length > 0)!.code;
  const envs = [
    { command: "serve", mode: "development", isSsrBuild: false },
    { command: "build", mode: "production", isSsrBuild: true },
  ];
  for (const env of envs) {
    assert.deepEqual(
      configRecipe(successor.recipes[0].code, env).vue,
      nativeOptions(configRecipe(archived[0], env)),
      `dynamic config: ${env.command}/${env.mode}/SSR=${env.isSsrBuild}`,
    );
    for (const index of [4, 6, 9, 10, 11, 12]) {
      const current = configRecipe(successor.recipes[index].code, env);
      const before = nativeOptions(configRecipe(archived[index], env));
      includesOptions(index === 4 ? current : current.vue, before, `recipe ${index}`);
    }
    // The gallery setting keeps its plugin owner; discovery/VRT stay shared.
    const gallery = configRecipe(successor.recipes[13].code, env);
    const beforeGallery = configRecipe(archived[13], env).musea as Record<string, unknown>;
    const { storybookCompat, ...shared } = beforeGallery;
    assert.deepEqual(gallery.vue, { musea: shared });
    assert.deepEqual(gallery.plugins, [{ musea: { storybookCompat } }]);
    const currentWorkflow = configRecipe(successor.workflow, env).vue;
    const oldWorkflow = JSON.parse(
      readFileSync(
        resolve(import.meta.dirname, "fixtures/docs-workflow-dedicated-config.json"),
        "utf8",
      ),
    ) as { code: string };
    assert.deepEqual(currentWorkflow, configRecipe(oldWorkflow.code, env));
  }
  const env = envs[0];
  const legacyProjects = configRecipe(archived[1], env);
  const projects = [successor.recipes[1].code, ...successor.recipes[1].additionalCode!];
  const entries = legacyProjects.entries as Record<string, unknown>[];
  for (const [index, source] of projects.entries()) {
    const { basePath, ...settings } = entries[index];
    assert.ok(source.startsWith(`// ${String(basePath)}/vite.config.ts`));
    assert.deepEqual(configRecipe(source, env).vue, {
      ...settings,
      formatter: { ...(legacyProjects.formatter as object), ...(settings.formatter as object) },
    });
  }
  includesOptions(
    configRecipe(successor.recipes[3].code, env).vue,
    nativeOptions(configRecipe(archived[3], env)),
    "retired JSON values",
  );
  const { entries: pklEntries, ...pklShared } = retiredPklOptions(archived[2]);
  includesOptions(
    configRecipe(successor.recipes[2].code, env).vue,
    pklShared,
    "retired PKL values",
  );
  const [{ basePath, ...pklWeb }] = pklEntries as Record<string, unknown>[];
  assert.ok(
    successor.recipes[2].additionalCode![0].startsWith(`// ${String(basePath)}/vite.config.ts`),
  );
  includesOptions(configRecipe(successor.recipes[2].additionalCode![0], env).vue, pklWeb);
  for (const index of [5, 7]) assert.equal(successor.recipes[index].code, archived[index]);
  assert.equal(
    successor.recipes[8].code.split("\n").slice(1).join("\n"),
    archived[8].split("\n").slice(1).join("\n"),
  );
});

void test("config succession preserves Nuxt module and Oxlint settings namespaces", () => {
  for (const source of [
    'export default defineNuxtConfig({\n  modules: ["@vizejs/nuxt"],\n  vize: { compiler: true },\n});',
    'export default defineConfig({\n  settings: {\n    vize: { nativePath: "./native.node" },\n  },\n});',
    "export default {\n  vize: { compiler: true },\n};",
  ])
    assert.equal(currentConfigExample(source), source);
});
