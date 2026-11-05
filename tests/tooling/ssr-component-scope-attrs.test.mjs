import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { Window } from "happy-dom";

const capture = process.env.VIZE_SCOPE_ATTR_CAPTURE;
const dataUrl = (source) => `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;

await test(
  "SSR component scope stays outside forwarded attrs in SSR, hydration and CSR",
  { skip: !capture && "complete source graphs execute in the scoped CSS action" },
  async () => {
    const fromUi = createRequire(
      process.env.VIZE_SCOPE_ATTR_VUE_PACKAGE ??
        new URL("../../npm/ui/package.json", import.meta.url),
    );
    const fromVue = createRequire(fromUi.resolve("vue/package.json"));
    const vue = fromUi("vue");
    const server = fromVue("vue/server-renderer");
    const compiler = fromVue("vue/compiler-sfc");
    assert.equal(vue.version, "3.5.35");
    assert.equal(compiler.version, vue.version);
    globalThis.__scopeAttrRuntime = { vue, server };
    const runtime = Object.fromEntries(
      [
        ["vue", "vue", vue],
        ["@vue/server-renderer", "server", server],
        ["vue/server-renderer", "server", server],
      ].map(([name, binding, exports]) => [
        name,
        dataUrl(
          Object.keys(exports)
            .filter((key) => /^[A-Za-z_$][\w$]*$/.test(key) && key !== "default")
            .map((key) => `export const ${key} = globalThis.__scopeAttrRuntime.${binding}.${key};`)
            .join("\n"),
        ),
      ]),
    );
    const cases = JSON.parse(readFileSync(capture, "utf8"));
    assert.deepEqual(
      cases.map((entry) => entry.name),
      ["reported", "no-forwarded-attrs", "dynamic-root", "inherited-attrs"],
    );
    const window = new Window();
    const tree = (html) => {
      const root = window.document.createElement("div");
      root.innerHTML = html;
      const visit = (node) =>
        node.nodeType === 1
          ? {
              tag: node.localName,
              attrs: [...node.attributes]
                .map(({ name, value }) => [name, value])
                .sort(([left], [right]) => left.localeCompare(right)),
              children: [...node.childNodes].map(visit),
            }
          : { type: node.nodeType, text: node.textContent };
      return [...root.childNodes].map(visit);
    };

    function componentCode(code, target, scopeId) {
      const declaration = code.includes("export default ")
        ? code.replace("export default ", "const __scope_component = ")
        : `${code}\nconst __scope_component = { ${target === "ssr" ? "ssrRender" : "render"} };`;
      return `${declaration}\n${scopeId ? `__scope_component.__scopeId = ${JSON.stringify(scopeId)};` : ""}\nexport default __scope_component;`;
    }
    function graph(modules, target, resources = runtime) {
      const urls = {};
      for (const entry of modules) {
        let code = componentCode(entry.code, target, entry.scopeId);
        code = code.replace(/from (["'])([^"']+)\1/g, (_, _quote, name) => {
          const url = resources[name] ?? urls[name];
          assert.equal(typeof url, "string", `complete dependency ${name}`);
          return `from ${JSON.stringify(url)}`;
        });
        urls[`./${entry.name}.vue`] = dataUrl(code);
      }
      return urls["./App.vue"];
    }
    function stock(entry, target) {
      const parsed = compiler.parse(entry.source, { filename: `${entry.name}.vue` });
      assert.deepEqual(parsed.errors, []);
      const descriptor = parsed.descriptor;
      const script = descriptor.scriptSetup
        ? compiler.compileScript(descriptor, {
            id: entry.scopeId ?? entry.name,
            isProd: true,
            genDefaultAs: "__sfc__",
          })
        : { content: "const __sfc__ = {};", bindings: {} };
      const template = compiler.compileTemplate({
        source: descriptor.template.content,
        filename: `${entry.name}.vue`,
        id: entry.scopeId ?? entry.name,
        scoped: descriptor.styles.some((style) => style.scoped),
        ssr: target === "ssr",
        ssrCssVars: descriptor.cssVars,
        isProd: true,
        compilerOptions: { bindingMetadata: script.bindings },
      });
      assert.deepEqual(template.errors, []);
      const renderer = target === "ssr" ? "ssrRender" : "render";
      return {
        name: entry.name,
        source: entry.source,
        scopeId: entry.scopeId,
        code: `${script.content}\n${template.code}\n__sfc__.${renderer} = ${renderer};\nexport default __sfc__;`,
      };
    }
    async function render(modules) {
      const { default: App } = await import(graph(modules, "ssr"));
      const app = vue.createSSRApp(App);
      const warnings = [];
      app.config.warnHandler = (message) => warnings.push(message);
      const html = await server.renderToString(app);
      assert.deepEqual(warnings, []);
      return html;
    }

    const fromTests = createRequire(new URL("../package.json", import.meta.url));
    const { chromium } = fromTests("@playwright/test");
    const browser = await chromium.launch();
    const runtimeSource = readFileSync(fromVue.resolve("vue/dist/vue.esm-browser.prod.js"), "utf8");
    const observations = [];
    try {
      for (const entry of cases) {
        assert.deepEqual(
          entry.modules.map((module) => module.name),
          ["Leaf", "Frame", "Panel", "App"],
        );
        for (const module of entry.modules) assert.deepEqual(module.selected, module.legacy);
        const officialSsr = entry.modules.map((module) => stock(module, "ssr"));
        const officialClient = entry.modules.map((module) => stock(module, "client"));
        const referenceHtml = await render(officialSsr);
        const client = entry.modules.map((module) => ({ ...module, code: module.client.code }));
        const css = entry.modules.map((module) => module.selected.css ?? "").join("\n");
        const lanes = {};
        for (const lane of ["selected", "legacy"]) {
          const modules = entry.modules.map((module) => ({ ...module, code: module[lane].code }));
          const html = await render(modules);
          assert.deepEqual(
            tree(html),
            tree(referenceHtml),
            `${entry.name}/${lane}: entire server tree`,
          );
          const page = await browser.newPage();
          const errors = [];
          page.on("pageerror", (error) => errors.push(error.message));
          const result = await page.evaluate(
            async ({ modules, officialClient, html, referenceHtml, runtimeSource, css }) => {
              const runtimeUrl = URL.createObjectURL(
                new Blob([runtimeSource], { type: "text/javascript" }),
              );
              const vue = await import(runtimeUrl);
              const visit = (node) =>
                node.nodeType === 1
                  ? {
                      tag: node.localName,
                      attrs: [...node.attributes]
                        .map(({ name, value }) => [name, value])
                        .sort(([left], [right]) => left.localeCompare(right)),
                      children: [...node.childNodes].map(visit),
                    }
                  : { type: node.nodeType, text: node.textContent };
              const load = async (graph) => {
                const urls = {};
                for (const entry of graph) {
                  let code = entry.code.includes("export default ")
                    ? entry.code.replace("export default ", "const __scope_component = ")
                    : `${entry.code}\nconst __scope_component = { render };`;
                  code += `\n${entry.scopeId ? `__scope_component.__scopeId = ${JSON.stringify(entry.scopeId)};` : ""}\nexport default __scope_component;`;
                  code = code.replace(
                    /from (["'])([^"']+)\1/g,
                    (_, _quote, name) =>
                      `from ${JSON.stringify(name === "vue" ? runtimeUrl : urls[name])}`,
                  );
                  urls[`./${entry.name}.vue`] = URL.createObjectURL(
                    new Blob([code], { type: "text/javascript" }),
                  );
                }
                return (await import(urls["./App.vue"])).default;
              };
              const style = document.createElement("style");
              style.textContent = css;
              document.head.append(style);
              const run = async (graph, ssrHtml) => {
                const host = document.createElement("div");
                host.innerHTML = ssrHtml ?? "";
                document.body.append(host);
                const original = [...host.querySelectorAll("*")];
                const warnings = [];
                const originalWarn = console.warn,
                  originalError = console.error;
                console.warn = (...args) => warnings.push(args.map(String).join(" "));
                console.error = console.warn;
                const app = (ssrHtml === null ? vue.createApp : vue.createSSRApp)(
                  await load(graph),
                );
                app.config.warnHandler = (message) => warnings.push(message);
                try {
                  app.mount(host);
                  await vue.nextTick();
                  const section = host.querySelector("section");
                  const after = [...host.querySelectorAll("*")];
                  return {
                    html: host.innerHTML,
                    tree: [...host.childNodes].map(visit),
                    warnings,
                    reused:
                      ssrHtml === null
                        ? null
                        : after.map((node, index) => node === original[index]),
                    sectionAttrs: [...section.attributes]
                      .map(({ name, value }) => [name, value])
                      .sort(([left], [right]) => left.localeCompare(right)),
                    appRuleMatches: section.matches(".leaf[data-v-app]"),
                    fontStyle: getComputedStyle(section).fontStyle,
                  };
                } finally {
                  console.warn = originalWarn;
                  console.error = originalError;
                  app.unmount();
                  host.remove();
                }
              };
              return {
                nativeHydrated: await run(modules, html),
                officialHydrated: await run(officialClient, referenceHtml),
                nativeCsr: await run(modules, null),
                officialCsr: await run(officialClient, null),
              };
            },
            { modules: client, officialClient, html, referenceHtml, runtimeSource, css },
          );
          assert.deepEqual(errors, []);
          for (const mode of ["Hydrated", "Csr"]) {
            const actual = result[`native${mode}`],
              reference = result[`official${mode}`];
            assert.deepEqual(
              actual.tree,
              reference.tree,
              `${entry.name}/${lane}/${mode}: complete DOM`,
            );
            assert.deepEqual(actual.warnings, []);
            assert.deepEqual(reference.warnings, []);
            assert.deepEqual(actual.sectionAttrs, [
              ["class", entry.name === "no-forwarded-attrs" ? "leaf" : "leaf from-app"],
              ["data-v-panel", ""],
            ]);
            assert.equal(actual.appRuleMatches, false);
            assert.equal(actual.fontStyle, "normal");
          }
          assert.deepEqual(result.nativeHydrated.tree, tree(html));
          assert.deepEqual(
            result.nativeHydrated.reused,
            result.nativeHydrated.reused.map(() => true),
          );
          lanes[lane] = { html, ...result };
          await page.close();
        }
        // Reintroducing the old props merge is a negative control on the whole graph.
        let mutation = null;
        if (entry.name === "reported") {
          const modules = entry.modules.map((module) => ({
            ...module,
            code: module.selected.code,
          }));
          const app = modules.find((module) => module.name === "App");
          const before = app.code;
          app.code = app.code.replace(
            '_mergeProps({ class: "from-app" }, _attrs)',
            '_mergeProps(_mergeProps({ class: "from-app" }, { "data-v-app": "" }), _attrs)',
          );
          assert.notEqual(app.code, before, "negative control restores the complete old merge");
          const html = await render(modules);
          assert.notDeepEqual(tree(html), tree(referenceHtml));
          const host = window.document.createElement("div");
          host.innerHTML = html;
          assert.deepEqual(
            [...host.querySelector("section").attributes]
              .map(({ name, value }) => [name, value])
              .sort(([left], [right]) => left.localeCompare(right)),
            [
              ["class", "leaf from-app"],
              ["data-v-app", ""],
              ["data-v-panel", ""],
            ],
          );
          mutation = { html, tree: tree(html) };
        }
        observations.push({ name: entry.name, referenceHtml, lanes, mutation });
      }
    } finally {
      await browser.close();
      await window.happyDOM.close();
    }
    const destination = process.env.VIZE_SCOPE_ATTR_RUNTIME_CAPTURE;
    assert.equal(typeof destination, "string", "retain complete browser and server observations");
    writeFileSync(destination, JSON.stringify({ vueVersion: vue.version, observations }, null, 2));
  },
);
