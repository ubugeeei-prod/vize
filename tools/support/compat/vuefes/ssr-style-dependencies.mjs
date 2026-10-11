import assert from "node:assert/strict";
import { createComponentBridge } from "./component-bridge.mjs";
import { createMissingSsrEdgeControl } from "./missing-ssr-edge-control.mjs";
import { observeReversedCssOrder } from "./reversed-css-order.mjs";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { execFileSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";
import { sanitizeNuxtDevStylesheetLinks } from "../../../../npm/framework/nuxt/src/dev-html.ts";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const directory = path.join(root, "tests/_fixtures/differential/compiler/vuefes-heading-cascade");
const corpus = JSON.parse(fs.readFileSync(path.join(directory, "corpus.json"), "utf8"));
const expected = JSON.parse(fs.readFileSync(path.join(directory, "expected.json"), "utf8"));
const oracleDir = fileURLToPath(new URL("./fixtures/heading-cascade/", import.meta.url));
const fromOracle = createRequire(path.join(oracleDir, "package.json"));
const fromTests = createRequire(path.join(root, "tests/package.json"));
const load = (name) => import(pathToFileURL(fromOracle.resolve(name)).href);
const { createServer } = await load("vite");
const { default: vue } = await load("@vitejs/plugin-vue");
const compiler = fromOracle("@vue/compiler-sfc");
const { chromium } = fromTests("@playwright/test");
const { default: vize } = await import(
  pathToFileURL(path.join(root, "npm/builder/vite/dist/index.mjs"))
);
const { parse } = fromTests("@babel/parser");
const hash = (data) => createHash("sha256").update(data).digest("hex");
const output = path.join(root, "target/vuefes-ssr-style-dependencies");
fs.mkdirSync(output, { recursive: true });
const write = (name, value) =>
  fs.writeFileSync(path.join(output, name), JSON.stringify(value, null, 2) + "\n");

for (const [name, pin] of Object.entries(corpus.files)) {
  const bytes = fs.readFileSync(path.join(directory, name));
  assert.equal(bytes.length, pin.bytes);
  assert.equal(hash(bytes), pin.sha256);
  assert.equal(
    createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex"),
    pin.gitBlob,
  );
}
for (const name of [
  "vue",
  "@vue/compiler-sfc",
  "@vue/compiler-ssr",
  "@vue/runtime-dom",
  "@vue/runtime-core",
  "@vue/shared",
  "@vue/server-renderer",
])
  assert.equal(fromOracle(`${name}/package.json`).version, corpus.oracle.vue);
assert.equal(fromOracle("vite/package.json").version, corpus.oracle.vite);
assert.equal(fromOracle("@vitejs/plugin-vue/package.json").version, corpus.oracle.pluginVue);
const upstream = path.join(root, "tests/_fixtures/_git/vuefes-2025");
assert.equal(
  execFileSync("git", ["rev-parse", "HEAD"], { cwd: upstream, encoding: "utf8" }).trim(),
  corpus.upstream.commit,
);
const project = fs.realpathSync(fs.mkdtempSync(path.join(output, "project-")));
fs.symlinkSync(
  path.join(oracleDir, "node_modules"),
  path.join(project, "node_modules"),
  "junction",
);
fs.writeFileSync(path.join(project, "package.json"), '{"private":true,"type":"module"}\n');
fs.cpSync(path.join(upstream, "app/assets"), path.join(project, "assets"), { recursive: true });
const headingPath = path.join(project, "components/heading/VFHeading.vue");
fs.mkdirSync(path.dirname(headingPath), { recursive: true });
const original = fs.readFileSync(path.join(directory, "VFHeading.vue.txt"));
fs.writeFileSync(headingPath, original);
assert.equal(
  hash(fs.readFileSync(path.join(project, "assets/styles/main.css"))),
  corpus.files["main.css.txt"].sha256,
);
// Only transport entries are authored here. Both original product sources and
// their complete upstream asset dependency closure are copied without edits.
fs.writeFileSync(
  path.join(project, "root.mjs"),
  `
import { h } from 'vue';
import './assets/styles/main.css';
import { Heading } from '#components';
const props = [{}, { id: 'anchor' }, { nth: 1 }, { nth: 3, id: 'anchor-three' }];
export const Root = { render: () => h('main', props.flatMap((props, index) =>
  ['text', 'span'].map(kind => h('section', { 'data-case': index + '-' + kind }, [
    h(Heading, props, { default: () => kind === 'text' ? 'complete slot' : h('span', 'complete slot') })
  ])))) };
`,
);
fs.writeFileSync(
  path.join(project, "server.mjs"),
  `
import { createSSRApp } from 'vue';
import { renderToString } from '@vue/server-renderer';
import { Root } from './root.mjs';
export async function render() {
  const warnings = []; const app = createSSRApp(Root);
  app.config.warnHandler = message => warnings.push(message);
  return { html: await renderToString(app), warnings };
}
`,
);
fs.writeFileSync(
  path.join(project, "client.mjs"),
  `
import { createApp, createSSRApp, nextTick } from 'vue';
import { Root } from './root.mjs';
const warnings = []; const mode = new URL(location.href).searchParams.get('mode');
const app = (mode === 'hydrate' ? createSSRApp : createApp)(Root);
app.config.warnHandler = message => warnings.push(message);
app.mount('#app'); await nextTick();
window.__vuefesFixture = { complete: true, warnings };
`,
);
const nativeDir = path.join(root, "npm/native");
const bindings = fs.readdirSync(nativeDir).filter((name) => /^vize-vitrine\..*\.node$/.test(name));
assert.equal(bindings.length, 1, "a unique freshly built local native binding is required");
const nativeFile = path.join(nativeDir, bindings[0]);
const localBinding = createRequire(import.meta.url)(nativeFile);
const packageBinding = createRequire(path.join(root, "npm/builder/vite/package.json"))(
  "@vizejs/native",
);
assert.equal(
  packageBinding.compileSfc,
  localBinding.compileSfc,
  "the adapter must use this exact local binding",
);
write("identity.json", {
  sourceSha: execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim(),
  corpus,
  native: {
    path: nativeFile,
    bytes: fs.statSync(nativeFile).size,
    sha256: hash(fs.readFileSync(nativeFile)),
  },
  node: process.version,
  oracleLockSha256: hash(fs.readFileSync(path.join(oracleDir, "package-lock.json"))),
  runtimeSha256: hash(fs.readFileSync(fromOracle.resolve("vue/dist/vue.esm-browser.js"))),
});

const browser = await chromium.launch({ headless: true });
const receipts = [];
try {
  for (const producer of ["official", "native", "native-missing-ssr-edge"]) {
    const missingEdges = [];
    const bridgeModules = [];
    const componentBridge = createComponentBridge(producer, {
      entry: path.join(project, "root.mjs"),
      heading: headingPath,
      modules: bridgeModules,
    });
    const componentPlugin =
      producer === "official"
        ? vue({ compiler, features: { componentIdGenerator: () => hash(headingPath).slice(0, 8) } })
        : vize({ precompile: false });
    // This separate negative producer removes only real top-level SSR style
    // edges after native compilation. Original/altered whole modules are kept;
    // it is not presented as the literal-H source producer.
    const negative = createMissingSsrEdgeControl({ producer, missingEdges, parse });
    let servedSsr, servedSheets;
    const transport = {
      name: "fixture:whole-ssr-html",
      configureServer(server) {
        server.middlewares.use((request, response, next) => {
          if (!request.url?.startsWith("/fixture?")) return next();
          const hydrate =
            new URL(request.url, "http://localhost").searchParams.get("mode") === "hydrate";
          response.setHeader("Content-Type", "text/html");
          // Real browser stylesheet requests let Vite insert `direct` itself.
          // Preserve the complete generated dependency query and stylesheet order.
          const originalHtml = `<!doctype html><html><head>${servedSheets.map((url) => `<link rel="stylesheet" href="${`/_nuxt${url}`.replaceAll("&", "&amp;")}">`).join("")}</head><body><div id="app">${hydrate ? servedSsr.html : ""}</div><script type="module" src="/_nuxt/client.mjs"></script></body></html>`;
          const sanitizedHtml =
            producer === "official" ? originalHtml : sanitizeNuxtDevStylesheetLinks(originalHtml);
          packet.transport ??= [];
          packet.transport.push({ mode: hydrate ? "hydrate" : "dom", originalHtml, sanitizedHtml });
          write(`${producer}.json`, packet);
          response.end(sanitizedHtml);
        });
      },
    };
    const server = await createServer({
      root: project,
      base: "/_nuxt/",
      configFile: false,
      plugins: [componentPlugin, componentBridge, negative, transport],
      resolve: { alias: { "~": project } },
      server: { host: "127.0.0.1", port: 0 },
      css: { transformer: "lightningcss", lightningcss: { drafts: { customMedia: true } } },
      optimizeDeps: { noDiscovery: true, include: ["vue"] },
    });
    let packet;
    try {
      await server.listen();
      const { render } = await server.ssrLoadModule("/server.mjs");
      const ssr = await render();
      const graph = [...server.environments.ssr.moduleGraph.urlToModuleMap].map(
        ([url, module]) => ({
          url,
          id: module.id,
          imports: [...module.importedModules].map((item) => item.url),
          transform: module.ssrTransformResult ?? module.transformResult,
        }),
      );
      const heading = graph.find((item) => item.id?.split("?")[0] === headingPath);
      assert.ok(heading, "the original full SFC must be loaded in the real SSR module graph");
      const styleEdges = heading.imports.filter((url) => /[?&]type=style\b/.test(url));
      packet = { producer, ssr, graph, styleEdges, missingEdges, bridgeModules, runtime: [] };
      write(`${producer}.json`, packet);
      if (producer === "native-missing-ssr-edge") {
        assert.equal(styleEdges.length, 0);
        assert.equal(missingEdges.length, 1);
        assert.throws(() => assert.equal(styleEdges.length, expected.ssrStyleEdges));
        assert.equal(ssr.html, receipts.find((item) => item.producer === "native").ssr.html);
        continue;
      }
      assert.equal(styleEdges.length, expected.ssrStyleEdges);
      assert.deepEqual(ssr.warnings, []);
      const sheets = graph
        .filter((item) => /\.(?:css|less|sass|scss|styl|stylus)(?:$|\?)/.test(item.url))
        .map((item) => item.url);
      assert.ok(sheets.some((url) => url.includes("assets/styles/main.css")));
      assert.ok(sheets.some((url) => url.includes("VFHeading.vue")));
      servedSsr = ssr;
      servedSheets = sheets;
      const address = server.httpServer.address();
      for (const mode of ["dom", "hydrate"]) {
        const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
        const page = await context.newPage();
        const errors = [],
          responses = [],
          responseTasks = [];
        page.on("pageerror", (error) => errors.push(String(error.stack ?? error)));
        const captureResponse = (response) => {
          if (!response.url().includes(".css") && !response.url().includes("VFHeading.vue")) return;
          const record = {
            order: responses.length,
            url: response.url(),
            status: response.status(),
            statusText: response.statusText(),
            headers: response.headers(),
            request: {
              method: response.request().method(),
              resourceType: response.request().resourceType(),
              redirectedFrom: response.request().redirectedFrom()?.url() ?? null,
            },
            bytes: null,
            sha256: null,
            text: null,
            bodyNullReason: "capture pending",
          };
          responses.push(record);
          responseTasks.push(
            (async () => {
              try {
                record.headers = await response.allHeaders();
                const body = await response.body();
                record.bytes = body.length;
                record.sha256 = hash(body);
                record.text = body.toString();
                record.bodyNullReason = null;
              } catch (error) {
                // Redirects may have no body through Playwright. Retain the full
                // status, headers and explicit API failure instead of losing the
                // packet to an unhandled late rejection. Completeness is checked
                // separately after complete SSR/DOM/hydration comparisons.
                record.bodyNullReason = String(error.stack ?? error);
              }
            })(),
          );
        };
        page.on("response", captureResponse);
        const settleResponses = async () => {
          let settled = 0;
          while (settled < responseTasks.length) {
            const pending = responseTasks.slice(settled);
            settled = responseTasks.length;
            await Promise.all(pending);
          }
        };
        try {
          await page.goto(`http://127.0.0.1:${address.port}/fixture?mode=${mode}`);
          await page.waitForFunction(() => window.__vuefesFixture?.complete);
          const observation = await page.evaluate(() => ({
            html: document.documentElement.outerHTML,
            innerHtml: document.querySelector("#app").innerHTML,
            result: window.__vuefesFixture,
            headings: [...document.querySelectorAll("h2")].map((heading) => ({
              html: heading.outerHTML,
              fontSize: getComputedStyle(heading).fontSize,
              lineHeight: getComputedStyle(heading).lineHeight,
              height: heading.getBoundingClientRect().height,
            })),
            sheets: [...document.styleSheets].map((sheet) => ({
              href: sheet.href,
              owner: sheet.ownerNode.outerHTML,
              rules: [...sheet.cssRules].map((rule) => rule.cssText),
            })),
          }));
          packet.runtime.push({ mode, observation, errors, responses });
          write(`${producer}.json`, packet);
          await settleResponses();
          assert.deepEqual(errors, []);
          assert.deepEqual(observation.result.warnings, []);
          assert.equal(observation.headings.length, 4);
          for (const heading of observation.headings) {
            assert.equal(heading.fontSize, expected.desktop.fontSize);
            assert.equal(heading.lineHeight, expected.desktop.lineHeight);
          }
          if (producer === "native" && mode === "hydrate") {
            // An explicit diagnostic counterexample moves the same complete
            // global sheet after scoped sheets, without editing any CSS rule.
            // This is separate from the production-order positive observation.
            packet.reversedCssOrder = await page.evaluate(observeReversedCssOrder);
            write(`${producer}.json`, packet);
            for (const transition of packet.reversedCssOrder.transitionEvidence)
              assert.equal(
                transition.status,
                "finished",
                "the original finite transition must complete",
              );
            for (const heading of packet.reversedCssOrder.headings) {
              assert.equal(heading.fontSize, expected.negativeControls.reversedCssOrder.fontSize);
              assert.equal(
                heading.lineHeight,
                expected.negativeControls.reversedCssOrder.lineHeight,
              );
              assert.notEqual(heading.lineHeight, expected.desktop.lineHeight);
            }
          }
        } finally {
          await settleResponses();
          await context.close();
          page.off("response", captureResponse);
          await settleResponses();
          write(`${producer}.json`, packet);
        }
      }
      receipts.push(packet);
    } finally {
      if (packet) write(`${producer}.json`, packet);
      await server.close();
    }
  }
  const [official, native] = receipts;
  assert.equal(native.ssr.html, official.ssr.html, "complete native SSR HTML equals official Vue");
  for (let index = 0; index < official.runtime.length; index++)
    assert.equal(
      native.runtime[index].observation.innerHtml,
      official.runtime[index].observation.innerHtml,
      `complete ${official.runtime[index].mode} DOM equals official Vue`,
    );
  for (const packet of [official, native]) {
    for (const transport of packet.transport)
      assert.equal(
        transport.sanitizedHtml,
        transport.originalHtml,
        "complete generated HTML preserves stylesheet dependencies and order",
      );
    for (const runtime of packet.runtime) {
      assert.deepEqual(
        runtime.responses.filter((response) => response.bodyNullReason !== null),
        [],
        "whole response capture completeness remains a separate gate",
      );
      for (const response of runtime.responses.filter(
        (response) => response.request.resourceType === "stylesheet",
      )) {
        assert.equal(response.status, 200, "actual browser stylesheet requests must succeed");
        assert.match(response.headers["content-type"], /text\/css/);
      }
    }
  }
  write("result.json", {
    status: "passed",
    producers: receipts.map((item) => item.producer),
    completeHtmlEquality: ["ssr", "dom", "hydrate"],
    missingSsrEdgeRejected: true,
  });
  console.log(
    "Full pinned VFHeading/main.css: actual SSR style edge, official/native SSR + DOM + hydration, and missing-edge negative control passed.",
  );
} finally {
  await browser.close();
}
