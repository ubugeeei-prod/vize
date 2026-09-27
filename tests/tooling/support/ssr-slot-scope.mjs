// #6898: complete generated SFC modules, real Vue SSR, and raw HTML.
// The loader writes compiler bytes unchanged. Symlinks bind bare imports to
// one existing pinned Vue installation; no authored template or code is rewritten.
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { pathToFileURL, fileURLToPath } from "node:url";
import Forwarder from "../../fixtures/sfc/ssr-slot-scope/forwarder.mjs";

const oracle =
  process.env.VIZE_SLOT_SCOPE_ORACLE ??
  fileURLToPath(new URL("../../../npm/ui/package.json", import.meta.url));
const require = createRequire(oracle);
const vue = require("vue");
const compiler = require("vue/compiler-sfc");
const server = require("vue/server-renderer");
assert.match(vue.version, /^3\.5\./, "the slot scope oracle is Vue 3.5");

export async function observeSlotScopes(input) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-slot-scope-"));
  const vuePackage = require.resolve("vue/package.json");
  const vueRequire = createRequire(vuePackage);
  const rendererPackage = vueRequire.resolve("@vue/server-renderer/package.json");
  fs.mkdirSync(path.join(directory, "node_modules", "@vue"), { recursive: true });
  fs.symlinkSync(path.dirname(vuePackage), path.join(directory, "node_modules", "vue"));
  fs.symlinkSync(
    path.dirname(rendererPackage),
    path.join(directory, "node_modules", "@vue", "server-renderer"),
  );

  async function load(code, name, scopeId, stock) {
    const filename = path.join(directory, `${name}.mjs`);
    fs.writeFileSync(filename, code);
    const module = await import(pathToFileURL(filename).href);
    const component = stock ? { ssrRender: module.ssrRender } : module.default;
    assert.equal(typeof component?.ssrRender, "function", `${name}: actual SSR component`);
    // This is the same component metadata the SFC bundler attaches to __scopeId.
    if (scopeId) component.__scopeId = scopeId;
    return component;
  }

  function stock(source, id, filename) {
    const parsed = compiler.parse(source, { filename });
    assert.deepEqual(parsed.errors, [], `${filename}: reference parse errors`);
    const descriptor = parsed.descriptor;
    const scoped = descriptor.styles.some((style) => style.scoped);
    const result = compiler.compileTemplate({
      source: descriptor.template.content,
      filename,
      id,
      ssr: true,
      scoped,
      slotted: descriptor.slotted,
      ssrCssVars: [],
    });
    assert.deepEqual(result.errors, [], `${filename}: reference compile errors`);
    assert.deepEqual(result.tips, [], `${filename}: reference compile tips`);
    return { code: result.code, scoped, slotted: descriptor.slotted };
  }

  async function render(layout, page, slotMode) {
    const warnings = [];
    layout.components = { Forwarder };
    const app = vue.createSSRApp({
      render: () =>
        vue.h(layout, null, {
          default: () =>
            slotMode === "empty"
              ? []
              : slotMode === "direct"
                ? [
                    vue.h("h1", { class: "title" }, "hello"),
                    ...[1, 2, 3, 4].map((n) => vue.h("p", String(n))),
                  ]
                : vue.h(page),
        }),
    });
    app.config.warnHandler = (message) => warnings.push(message);
    return { html: await server.renderToString(app), warnings };
  }

  try {
    const pageStock = stock(input.pageSource, "page", "Page.vue");
    const vuePage = await load(pageStock.code, "stock-page", "data-v-page", true);
    const currentPage = await load(input.pageCode, "current-page", "data-v-page", false);
    const observations = [];
    for (const [index, fixture] of input.cases.entries()) {
      const reference = stock(fixture.source, "layout", "Layout.vue");
      assert.equal(fixture.hasScoped, reference.scoped, `${fixture.name}: scoped metadata`);
      const scopeId = reference.scoped ? "data-v-layout" : undefined;
      const vueLayout = await load(reference.code, `stock-${index}`, scopeId, true);
      const currentLayout = await load(fixture.code, `current-${index}`, scopeId, false);
      const modes = [];
      for (const slotMode of ["page", "direct", "empty"]) {
        const expected = await render(vueLayout, vuePage, slotMode);
        const current = await render(currentLayout, currentPage, slotMode);
        modes.push({
          slotMode,
          expected,
          current,
          equal: JSON.stringify(current) === JSON.stringify(expected),
        });
      }
      observations.push({
        name: fixture.name,
        slotted: reference.slotted,
        scoped: reference.scoped,
        source: fixture.source,
        referenceCode: reference.code,
        currentCode: fixture.code,
        currentCompileResult: fixture.compileResult,
        modes,
      });
    }
    return {
      vueVersion: vue.version,
      referencePackage: oracle,
      pageReferenceCode: pageStock.code,
      pageCurrentCode: input.pageCode,
      pageCompileResult: input.pageCompileResult,
      observations,
    };
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

if (process.argv[1] && pathToFileURL(path.resolve(process.argv[1])).href === import.meta.url) {
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
  const result = await observeSlotScopes(input);
  process.stdout.write(`${JSON.stringify(result)}\n`);
  if (input.check) {
    for (const fixture of result.observations) {
      for (const mode of fixture.modes) assert.deepEqual(mode.current, mode.expected, fixture.name);
    }
  }
}
