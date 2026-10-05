import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

const root = new URL("../../", import.meta.url);
const corpus = new URL("_fixtures/differential/compiler/", new URL("../", import.meta.url));
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const witnessRoot = new URL("crates/vize_armature/tests/_fixtures/", root);
const witness = JSON.parse(readFileSync(new URL("first_newline_parser_witness.json", witnessRoot)));
for (const reference of [
  ...Object.values(witness.snapshots),
  ...Object.values(witness.historicalBinaries),
])
  assert.equal(hash(readFileSync(new URL(reference.path, witnessRoot))), reference.sha256);
const registry = JSON.parse(readFileSync(new URL("manifest.json", corpus)));
const registration = registry.runtimePacks.find(
  (pack) => pack.path === "first-newline-runtime.manifest.json",
);
assert(registration, "the compiler corpus must register the runtime pack");
const manifestBytes = readFileSync(new URL(registration.path, corpus));
assert.equal(hash(manifestBytes), registration.sha256);
const manifest = JSON.parse(manifestBytes);
const row = manifest.cases.find((entry) => entry.id === "compiler/sfc/pre-textarea-first-newline");
const fixtureRoot = new URL(`${row.inputs.root}/`, corpus);
assert.equal(hash(readFileSync(new URL(row.reference.path, corpus))), row.reference.sha256);
for (const input of row.inputs.files)
  assert.equal(hash(readFileSync(new URL(input.path, fixtureRoot))), input.sha256);
const cases = JSON.parse(readFileSync(new URL("runtime.expected.json", fixtureRoot)));
const original = readFileSync(new URL("App.vue.txt", fixtureRoot), "utf8");

void test("the entire reporter SFC stays pinned independently of the controls", () => {
  assert.equal(
    original,
    '<script setup>\nconst a = "A"\n</script>\n\n<template>\n  <pre>\nline {{ a }}</pre>\n  <textarea>\nvalue</textarea>\n</template>\n',
  );
  assert.equal(cases[0].sourceFile, "App.vue.txt");
});

void test(
  "source-built CLI whole SFCs mount and hydrate in Chromium like pinned Vue",
  { skip: !process.env.VIZE_FIRST_NEWLINE_BIN, timeout: 180_000 },
  async () => {
    const fromUi = createRequire(new URL("npm/ui/package.json", root));
    const fromVue = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
    const compiler = fromVue("vue/compiler-sfc");
    const vue = fromVue("vue");
    const server = fromVue("vue/server-renderer");
    assert.equal(compiler.version, "3.6.0-rc.9");
    assert.equal(vue.version, compiler.version);
    const { transformSync } = fromUi("@babel/core");
    const { chromium } = createRequire(new URL("tests/package.json", root))("@playwright/test");
    const { compileFirstNewlineVaporReference } =
      await import("./support/first-newline-vapor-oracle.mjs");
    const runtimeUrl = dataUrl(
      readFileSync(
        fromUi.resolve("vue-vapor-runtime/dist/vue.runtime-with-vapor.esm-browser.js"),
        "utf8",
      ),
    );
    const browser = await chromium.launch();
    const directory = mkdtempSync(join(tmpdir(), "vize-first-newline-"));
    const observations = [];
    const failures = [];
    let sequence = 0;
    function relink(code, imports) {
      return transformSync(code, {
        configFile: false,
        babelrc: false,
        plugins: [
          () => ({
            visitor: {
              ImportDeclaration(path) {
                assert(imports.has(path.node.source.value), path.node.source.value);
                path.node.source.value = imports.get(path.node.source.value);
              },
            },
          }),
        ],
      }).code;
    }
    const serverImports = new Map([
      ["vue", pathToFileURL(fromVue.resolve("vue")).href],
      ["@vue/server-renderer", pathToFileURL(fromVue.resolve("vue/server-renderer")).href],
      ["vue/server-renderer", pathToFileURL(fromVue.resolve("vue/server-renderer")).href],
    ]);
    async function serverHtml(code) {
      const component = (await import(dataUrl(relink(code, serverImports) + `\n// ${sequence++}`)))
        .default;
      const app = vue.createSSRApp(component);
      const diagnostics = [];
      app.config.warnHandler = (message) => diagnostics.push(message);
      app.config.errorHandler = (error) => diagnostics.push(String(error));
      const html = await server.renderToString(app);
      assert.deepEqual(diagnostics, []);
      return html;
    }
    function stock(source, ssr) {
      const parsed = compiler.parse(source, {
        filename: "App.vue",
        templateParseOptions: { comments: false },
      });
      assert.deepEqual(parsed.errors, []);
      assert.equal(parsed.descriptor.source, source);
      return compiler.compileScript(parsed.descriptor, {
        id: "first-newline",
        inlineTemplate: true,
        templateOptions: { ssr, compilerOptions: { comments: false } },
      }).content;
    }
    async function observe(code, vapor, html = null, helper = null) {
      const imports = new Map([
        ["vue", runtimeUrl],
        ["vue/vapor", runtimeUrl],
      ]);
      if (helper) imports.set("\0plugin-vue:export-helper", dataUrl(helper));
      const page = await browser.newPage();
      const errors = [];
      page.on("pageerror", (error) => errors.push(error.message));
      try {
        await page.setContent('<!doctype html><div id="app"></div>');
        const result = await page.evaluate(
          async ({ runtimeUrl, componentUrl, vapor, html }) => {
            const runtime = await import(runtimeUrl);
            const component = (await import(componentUrl)).default;
            const host = document.querySelector("#app");
            if (html !== null) host.innerHTML = html;
            const before = [...host.querySelectorAll("pre,textarea")];
            const factory = vapor
              ? html === null
                ? runtime.createVaporApp
                : runtime.createVaporSSRApp
              : html === null
                ? runtime.createApp
                : runtime.createSSRApp;
            const app = factory(component);
            const diagnostics = [];
            app.config.warnHandler = (message) => diagnostics.push(message);
            app.config.errorHandler = (error) => diagnostics.push(String(error));
            app.mount(host);
            await runtime.nextTick();
            const elements = [...host.querySelectorAll("pre,textarea")];
            const observed = {
              pre: [...host.querySelectorAll("pre")].map((node) => node.textContent),
              textarea: [...host.querySelectorAll("textarea")].map((node) => node.value),
              html: host.innerHTML,
              diagnostics,
              retained: html === null || before.every((node, index) => node === elements[index]),
            };
            app.unmount();
            return observed;
          },
          { runtimeUrl, componentUrl: dataUrl(relink(code, imports)), vapor, html },
        );
        assert.deepEqual(errors, []);
        return result;
      } finally {
        await page.close();
      }
    }
    try {
      for (const fixture of cases) {
        const source = fixture.sourceFile
          ? readFileSync(new URL(fixture.sourceFile, fixtureRoot), "utf8")
          : `<script setup>\nconst a = "A"\n</script>\n<template>${fixture.template}</template>\n`;
        try {
          const input = join(directory, "App.vue");
          writeFileSync(input, source);
          const modules = {};
          for (const { mode, flags } of [
            { mode: "dom", flags: [] },
            { mode: "ssr", flags: ["--ssr"] },
            { mode: "vapor", flags: ["--vapor"] },
            { mode: "vaporSsr", flags: ["--vapor", "--ssr"] },
          ]) {
            const output = join(directory, mode);
            execFileSync(
              process.env.VIZE_FIRST_NEWLINE_BIN,
              ["build", "--no-config", "-o", output, ...flags, input],
              { cwd: directory, encoding: "utf8", stdio: "pipe" },
            );
            modules[mode] = readFileSync(join(output, "App.js"), "utf8");
          }
          const referenceDom = stock(source, false);
          const referenceSsr = await serverHtml(stock(source, true));
          const referenceVapor = await compileFirstNewlineVaporReference(
            source,
            "/first-newline/App.vue",
          );
          const dom = await observe(modules.dom, false);
          const expectedDom = await observe(referenceDom, false);
          assert.deepEqual(dom, expectedDom, `${fixture.id}: complete VDOM`);
          assert.deepEqual(dom.pre, fixture.pre, `${fixture.id}: only first pre newline`);
          assert.deepEqual(dom.textarea, fixture.textarea, `${fixture.id}: live textarea value`);
          const vapor = await observe(modules.vapor, true);
          const expectedVapor = await observe(
            referenceVapor.code,
            true,
            null,
            referenceVapor.helperCode,
          );
          assert.deepEqual(vapor.pre, expectedVapor.pre, `${fixture.id}: Vapor pre`);
          assert.deepEqual(vapor.textarea, expectedVapor.textarea, `${fixture.id}: Vapor textarea`);
          assert.deepEqual(vapor.diagnostics, []);
          const ssr = await serverHtml(modules.ssr);
          const vaporSsr = await serverHtml(modules.vaporSsr);
          assert.equal(ssr, referenceSsr, `${fixture.id}: complete SSR HTML`);
          assert.equal(
            vaporSsr,
            referenceVapor.serverHtml,
            `${fixture.id}: complete Vapor-requested SSR HTML`,
          );
          const hydrated = await observe(modules.dom, false, ssr);
          const expectedHydrated = await observe(referenceDom, false, referenceSsr);
          assert.deepEqual(
            hydrated,
            expectedHydrated,
            `${fixture.id}: actual HTML-parser hydration`,
          );
          let vaporHydrated = null;
          if (
            fixture.id === "reporter" ||
            fixture.id === "textarea-trailing-space-after-interpolation"
          ) {
            assert.deepEqual(hydrated.diagnostics, []);
            assert(hydrated.retained);
            vaporHydrated = await observe(modules.vapor, true, vaporSsr);
            referenceVapor.hydrated = await observe(
              referenceVapor.code,
              true,
              referenceVapor.serverHtml,
              referenceVapor.helperCode,
            );
            assert.deepEqual(vaporHydrated, referenceVapor.hydrated);
            assert.deepEqual(vaporHydrated.diagnostics, []);
            assert(vaporHydrated.retained);
            assert.deepEqual(vaporHydrated.pre, fixture.pre);
            assert.deepEqual(vaporHydrated.textarea, fixture.textarea);
          }
          observations.push({
            id: fixture.id,
            sourceSha256: hash(source),
            modules,
            reference: { dom: referenceDom, ssr: referenceSsr, vapor: referenceVapor },
            dom,
            vapor,
            ssr,
            vaporSsr,
            hydrated,
            vaporHydrated,
          });
        } catch (error) {
          failures.push({ id: fixture.id, message: error.message, stack: error.stack });
          observations.push({ id: fixture.id, sourceSha256: hash(source), error: error.message });
        }
      }
      if (process.env.VIZE_FIRST_NEWLINE_CAPTURE)
        writeFileSync(
          process.env.VIZE_FIRST_NEWLINE_CAPTURE,
          JSON.stringify(observations, null, 2),
        );
      assert.deepEqual(failures, [], "every whole-SFC runtime control must qualify");
    } finally {
      rmSync(directory, { recursive: true, force: true });
      await browser.close();
    }
  },
);

function dataUrl(code) {
  return `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
}
