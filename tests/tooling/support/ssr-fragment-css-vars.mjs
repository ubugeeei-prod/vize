// #7892: unchanged whole SFC modules, exact rc.10 production SSR and Chromium.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { parse } from "@babel/parser";

const fromTests = createRequire(new URL("../../package.json", import.meta.url));
const vuePackage = fromTests.resolve("vue-ssr-css-vars-oracle/package.json");
const fromVue = createRequire(vuePackage);
const vue = fromTests("vue-ssr-css-vars-oracle");
const compiler = fromVue("@vue/compiler-sfc");
const server = fromVue("@vue/server-renderer");
const version = "3.6.0-rc.10";
assert.equal(process.env.NODE_ENV, "production", "execute actual production Vue builds");
for (const name of ["vue", "@vue/compiler-sfc", "@vue/compiler-ssr", "@vue/server-renderer"])
  assert.equal(name === "vue" ? vue.version : fromVue(`${name}/package.json`).version, version);
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const dataUrl = (code) => `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
const browserRuntime = fs.readFileSync(
  path.join(path.dirname(vuePackage), "dist/vue.runtime.esm-browser.prod.js"),
  "utf8",
);
const runtimeUrl = dataUrl(browserRuntime);

// Only original ImportDeclaration source literals are linked to the exact
// browser runtime. Every compiler statement/expression remains unchanged.
function clientUrl(code) {
  const imports = parse(code, { sourceType: "module" }).program.body.filter(
    (node) => node.type === "ImportDeclaration",
  );
  let linked = code;
  for (const node of imports.reverse()) {
    assert.equal(node.source.value, "vue", "client imports the pinned Vue runtime only");
    linked =
      linked.slice(0, node.source.start) +
      JSON.stringify(runtimeUrl) +
      linked.slice(node.source.end);
  }
  return dataUrl(linked);
}

function reference(source, ssr) {
  const parsed = compiler.parse(source, { filename: "App.vue" });
  assert.deepEqual(parsed.errors, []);
  const descriptor = parsed.descriptor;
  // Vize's production CSS name hashes the filename. Give the independent
  // compiler that same original identifier; no output key is normalized.
  const script = compiler.compileScript(descriptor, {
    id: "App.vue",
    isProd: true,
    genDefaultAs: "__sfc__",
    templateOptions: { ssr },
  });
  const template = compiler.compileTemplate({
    source: descriptor.template.content,
    filename: "App.vue",
    id: "App.vue",
    isProd: true,
    ssr,
    ssrCssVars: descriptor.cssVars,
    compilerOptions: { bindingMetadata: script.bindings },
  });
  assert.deepEqual(template.errors, []);
  assert.deepEqual(template.tips, []);
  const css = compiler.compileStyle({
    source: descriptor.styles[0].content,
    filename: "App.vue",
    id: "data-v-App.vue",
    isProd: true,
  });
  assert.deepEqual(css.errors, []);
  return {
    code: `${script.content}\n${template.code}\n__sfc__.${ssr ? "ssrRender" : "render"} = ${ssr ? "ssrRender" : "render"};\nexport default __sfc__;`,
    css: css.code,
    bindings: script.bindings,
    cssVars: descriptor.cssVars,
  };
}

export async function observeFragmentCss(input) {
  const { chromium } = fromTests("@playwright/test");
  const browser = await chromium.launch();
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-fragment-css-"));
  const rendererPackage = fromVue.resolve("@vue/server-renderer/package.json");
  fs.mkdirSync(path.join(directory, "node_modules", "@vue"), { recursive: true });
  fs.symlinkSync(path.dirname(vuePackage), path.join(directory, "node_modules", "vue"));
  fs.symlinkSync(
    path.dirname(rendererPackage),
    path.join(directory, "node_modules", "@vue", "server-renderer"),
  );
  const cases = [];

  async function render(code, name) {
    const filename = path.join(directory, `${name}.mjs`);
    fs.writeFileSync(filename, code);
    const component = (await import(pathToFileURL(filename).href)).default;
    assert.equal(typeof component.ssrRender, "function", "actual whole SSR component");
    const app = vue.createSSRApp(component);
    app.component("RootLeaf", {
      render() {
        return vue.h("p", null, this.$slots.default?.());
      },
    });
    const diagnostics = [];
    app.config.warnHandler = (message) => diagnostics.push(message);
    app.config.errorHandler = (error) => diagnostics.push(String(error));
    return { html: await server.renderToString(app), diagnostics };
  }

  async function mount(client, html, css, hydrate) {
    const page = await browser.newPage();
    const errors = [];
    page.on("pageerror", (error) => errors.push(error.message));
    try {
      await page.setContent("<!doctype html><html><head></head><body></body></html>");
      const outcome = await page.evaluate(
        async ({ runtimeUrl, client, html, css, hydrate }) => {
          const runtime = await import(runtimeUrl);
          const component = (await import(client)).default;
          const style = document.createElement("style");
          style.textContent = css;
          document.head.append(style);
          const host = document.createElement("section");
          host.innerHTML = html;
          document.body.append(host);
          const original = [...host.querySelectorAll("*")];
          const record = () =>
            [...host.querySelectorAll("*")].map((element) => ({
              tag: element.tagName,
              text: element.textContent,
              style: element.getAttribute("style"),
              color: getComputedStyle(element).color,
            }));
          const before = record();
          const diagnostics = [];
          const app = (hydrate ? runtime.createSSRApp : runtime.createApp)(component);
          app.component("RootLeaf", {
            render() {
              return runtime.h("p", null, this.$slots.default?.());
            },
          });
          app.config.warnHandler = (message) => diagnostics.push(message);
          app.config.errorHandler = (error) => diagnostics.push(String(error));
          app.mount(host);
          await runtime.nextTick();
          const after = record();
          const current = [...host.querySelectorAll("*")];
          const sameNodes =
            hydrate &&
            original.length === current.length &&
            original.every((node, index) => node === current[index]);
          app.unmount();
          await runtime.nextTick();
          const remainingNodes = host.childNodes.length;
          host.remove();
          style.remove();
          return { before, after, sameNodes, diagnostics, remainingNodes };
        },
        { runtimeUrl, client: clientUrl(client), html, css, hydrate },
      );
      assert.deepEqual(errors, [], "actual Chromium execution errors");
      assert.deepEqual(outcome.diagnostics, []);
      assert.equal(outcome.remainingNodes, 0, "whole component unmount");
      if (hydrate) assert.equal(outcome.sameNodes, true, "hydration preserves original nodes");
      return outcome;
    } finally {
      await page.close();
    }
  }

  try {
    for (const [index, fixture] of input.cases.entries()) {
      const stockSsr = reference(fixture.source, true);
      const stockClient = reference(fixture.source, false);
      const expected = await render(stockSsr.code, `stock-${index}`);
      const actual = await render(fixture.ssr.code, `current-${index}`);
      const fallback = await render(fixture.fallback.code, `fallback-${index}`);
      assert.deepEqual(actual, expected, `${fixture.name}: complete raw SSR HTML/diagnostics`);
      assert.deepEqual(fallback, expected, `${fixture.name}: explicit standard SSR fallback`);
      assert.deepEqual(actual.diagnostics, []);
      const properties = [...actual.html.matchAll(/style="([^"]*)"/gu)].filter((match) =>
        /--[^:;]+:red;/u.test(match[1]),
      );
      assert.equal(properties.length, fixture.cssRoots, `${fixture.name}: exact CSS-owning roots`);
      const css = fixture.ssr.css;
      assert.equal(typeof css, "string");
      assert.equal(fixture.client.css, css);
      // Match independently generated property names, preserving all CSS bytes
      // in the capture; differences in compiler whitespace are not CSS parity.
      const variableNames = (text) =>
        [...text.matchAll(/var\((--[^)]+)\)/gu)].map((match) => match[1]);
      assert.deepEqual(variableNames(css), variableNames(stockSsr.css));
      const currentHydration = await mount(fixture.client.code, actual.html, css, true);
      const referenceHydration = await mount(
        stockClient.code,
        expected.html,
        stockClient.css,
        true,
      );
      const currentClient = await mount(fixture.client.code, "", css, false);
      const referenceClient = await mount(stockClient.code, "", stockClient.css, false);
      assert.deepEqual(
        currentHydration,
        referenceHydration,
        `${fixture.name}: actual production hydration`,
      );
      assert.deepEqual(currentClient, referenceClient, `${fixture.name}: actual production client`);
      if (fixture.name === "reported") {
        assert.deepEqual(
          currentHydration.before.map((node) => node.color),
          ["rgb(255, 0, 0)", "rgb(255, 0, 0)"],
        );
      }
      cases.push({
        ...fixture,
        stockSsr,
        stockClient,
        expected,
        actual,
        fallback,
        currentHydration,
        referenceHydration,
        currentClient,
        referenceClient,
        hashes: {
          source: hash(fixture.source),
          ssr: hash(fixture.ssr.code),
          client: hash(fixture.client.code),
          css: hash(css),
        },
      });
    }
    return {
      version,
      production: true,
      browser: browser.version(),
      browserRuntimeSha256: hash(browserRuntime),
      cases,
    };
  } finally {
    await browser.close();
    fs.rmSync(directory, { recursive: true, force: true });
  }
}
