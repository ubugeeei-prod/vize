import assert from "node:assert/strict";
import { createComponentBridge, type BridgeModule } from "./component-bridge.ts";
import { createMissingSsrEdgeControl, type MissingEdge } from "./missing-ssr-edge-control.ts";
import {
  observeFixturePage,
  observeReversedCssOrder,
  type BrowserObservation,
  type ReversedCssOrderObservation,
} from "./page-observations.ts";
import fs from "node:fs";
import type { IncomingMessage, ServerResponse } from "node:http";
import path from "node:path";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { execFileSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";
import type { BrowserType, Response } from "playwright";
import { sanitizeNuxtDevStylesheetLinks } from "../../../../npm/framework/nuxt/src/dev-html.ts";
import {
  count,
  errorText,
  list,
  readJsonRecord,
  record,
  text,
  type BabelParse,
  type CompilerSfc,
  type FixturePlugin,
} from "./oracle-support.ts";

interface SsrResult {
  readonly html: string;
  readonly warnings: readonly unknown[];
}
interface ModuleNode {
  readonly id: string | null;
  readonly importedModules: ReadonlySet<{ readonly url: string }>;
  readonly ssrTransformResult?: unknown;
  readonly transformResult?: unknown;
}
interface FixtureServer {
  readonly middlewares: {
    use(
      handler: (request: IncomingMessage, response: ServerResponse, next: () => void) => void,
    ): void;
  };
}
interface DevServer {
  listen(): Promise<unknown>;
  close(): Promise<void>;
  ssrLoadModule(url: string): Promise<Readonly<Record<string, unknown>>>;
  readonly httpServer: { address(): { port: number } | string | null } | null;
  readonly environments: {
    readonly ssr: { readonly moduleGraph: { urlToModuleMap: Map<string, ModuleNode> } };
  };
}
type CreateServer = (config: object) => Promise<DevServer>;
type ViteModule = { createServer: CreateServer };
type PluginFactory = { default: (options: object) => object };
interface CapturedResponse {
  readonly order: number;
  readonly url: string;
  readonly status: number;
  readonly statusText: string;
  headers: Record<string, string>;
  readonly request: { method: string; resourceType: string; redirectedFrom: string | null };
  bytes: number | null;
  sha256: string | null;
  text: string | null;
  bodyNullReason: string | null;
}
interface Packet {
  readonly producer: string;
  readonly ssr: SsrResult;
  readonly graph: readonly { url: string; id: string | null; imports: string[] }[];
  readonly styleEdges: readonly string[];
  readonly missingEdges: readonly MissingEdge[];
  readonly bridgeModules: readonly BridgeModule[];
  readonly runtime: {
    mode: string;
    observation: BrowserObservation;
    errors: string[];
    responses: CapturedResponse[];
  }[];
  transport?: { mode: string; originalHtml: string; sanitizedHtml: string }[];
  reversedCssOrder?: ReversedCssOrderObservation;
}

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const directory = path.join(root, "tests/_fixtures/differential/compiler/vuefes-heading-cascade");
const corpus = readJsonRecord(path.join(directory, "corpus.json"));
const corpusFiles = record(corpus.files, "corpus.files");
const corpusOracle = record(corpus.oracle, "corpus.oracle");
const expected = readJsonRecord(path.join(directory, "expected.json"));
const expectedEdges = count(expected.ssrStyleEdges, "expected.ssrStyleEdges");
const desktop = record(expected.desktop, "expected.desktop");
const reversedExpected = record(
  record(expected.negativeControls, "expected.negativeControls").reversedCssOrder,
  "expected.negativeControls.reversedCssOrder",
);
const oracleDir = fileURLToPath(new URL("./fixtures/heading-cascade/", import.meta.url));
const fromOracle = createRequire(path.join(oracleDir, "package.json"));
const fromTests = createRequire(path.join(root, "tests/package.json"));
const load = async (name: string): Promise<unknown> =>
  import(pathToFileURL(fromOracle.resolve(name)).href);
const { createServer } = (await load("vite")) as ViteModule;
const { default: vue } = (await load("@vitejs/plugin-vue")) as PluginFactory;
const compiler: CompilerSfc = fromOracle("@vue/compiler-sfc");
const { chromium }: { chromium: BrowserType } = fromTests("@playwright/test");
const { default: vize } = (await import(
  pathToFileURL(path.join(root, "npm/builder/vite/dist/index.mjs")).href
)) as PluginFactory;
const { parse }: { parse: BabelParse } = fromTests("@babel/parser");
const hash = (data: Uint8Array | string): string => createHash("sha256").update(data).digest("hex");
const output = path.join(root, "target/vuefes-ssr-style-dependencies");
fs.mkdirSync(output, { recursive: true });
const write = (name: string, value: unknown): void =>
  fs.writeFileSync(path.join(output, name), JSON.stringify(value, null, 2) + "\n");
const version = (name: string): unknown =>
  record(fromOracle(`${name}/package.json`), `${name}/package.json`).version;

for (const [name, value] of Object.entries(corpusFiles)) {
  const pin = record(value, `corpus.files.${name}`);
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
  assert.equal(version(name), corpusOracle.vue);
assert.equal(version("vite"), corpusOracle.vite);
assert.equal(version("@vitejs/plugin-vue"), corpusOracle.pluginVue);
const upstream = path.join(root, "tests/_fixtures/_git/vuefes-2025");
assert.equal(
  execFileSync("git", ["rev-parse", "HEAD"], { cwd: upstream, encoding: "utf8" }).trim(),
  record(corpus.upstream, "corpus.upstream").commit,
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
  record(corpusFiles["main.css.txt"], "corpus.files.main.css.txt").sha256,
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
const nativeFile = path.join(nativeDir, bindings[0] ?? "");
const localBinding = record(createRequire(import.meta.url)(nativeFile), "native binding");
const packageBinding = record(
  createRequire(path.join(root, "npm/builder/vite/package.json"))("@vizejs/native"),
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
const receipts: Packet[] = [];
try {
  for (const producer of ["official", "native", "native-missing-ssr-edge"]) {
    const missingEdges: MissingEdge[] = [];
    const bridgeModules: BridgeModule[] = [];
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
    let served: { ssr: SsrResult; sheets: readonly string[] } | undefined;
    let packet: Packet | undefined;
    const transport: FixturePlugin & { configureServer(server: FixtureServer): void } = {
      name: "fixture:whole-ssr-html",
      configureServer(server) {
        server.middlewares.use((request, response, next) => {
          if (!request.url?.startsWith("/fixture?")) return next();
          assert.ok(served && packet, "fixture HTML is served only after SSR");
          const hydrate =
            new URL(request.url, "http://localhost").searchParams.get("mode") === "hydrate";
          response.setHeader("Content-Type", "text/html");
          // Real browser stylesheet requests let Vite insert `direct` itself.
          // Preserve the complete generated dependency query and stylesheet order.
          const originalHtml = `<!doctype html><html><head>${served.sheets.map((url) => `<link rel="stylesheet" href="${`/_nuxt${url}`.replaceAll("&", "&amp;")}">`).join("")}</head><body><div id="app">${hydrate ? served.ssr.html : ""}</div><script type="module" src="/_nuxt/client.mjs"></script></body></html>`;
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
    try {
      await server.listen();
      const { render } = await server.ssrLoadModule("/server.mjs");
      assert.equal(typeof render, "function");
      const rendered = record(await (render as () => Promise<unknown>)(), "SSR render");
      const ssr: SsrResult = {
        html: text(rendered.html, "SSR html"),
        warnings: list(rendered.warnings, "SSR warnings"),
      };
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
      const current: Packet = {
        producer,
        ssr,
        graph,
        styleEdges,
        missingEdges,
        bridgeModules,
        runtime: [],
      };
      packet = current;
      write(`${producer}.json`, current);
      if (producer === "native-missing-ssr-edge") {
        assert.equal(styleEdges.length, 0);
        assert.equal(missingEdges.length, 1);
        assert.throws(() => assert.equal(styleEdges.length, expectedEdges));
        assert.equal(ssr.html, receipts.find((item) => item.producer === "native")?.ssr.html);
        continue;
      }
      assert.equal(styleEdges.length, expectedEdges);
      assert.deepEqual(ssr.warnings, []);
      const sheets = graph
        .filter((item) => /\.(?:css|less|sass|scss|styl|stylus)(?:$|\?)/.test(item.url))
        .map((item) => item.url);
      assert.ok(sheets.some((url) => url.includes("assets/styles/main.css")));
      assert.ok(sheets.some((url) => url.includes("VFHeading.vue")));
      served = { ssr, sheets };
      const address = server.httpServer?.address();
      assert.ok(address && typeof address === "object", "the fixture server listens on TCP");
      for (const mode of ["dom", "hydrate"]) {
        const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
        const page = await context.newPage();
        const errors: string[] = [];
        const responses: CapturedResponse[] = [];
        const responseTasks: Promise<void>[] = [];
        page.on("pageerror", (error) => errors.push(errorText(error)));
        const captureResponse = (response: Response): void => {
          if (!response.url().includes(".css") && !response.url().includes("VFHeading.vue")) return;
          const captured: CapturedResponse = {
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
          responses.push(captured);
          responseTasks.push(
            (async () => {
              try {
                captured.headers = await response.allHeaders();
                const body = await response.body();
                captured.bytes = body.length;
                captured.sha256 = hash(body);
                captured.text = body.toString();
                captured.bodyNullReason = null;
              } catch (error) {
                // Redirects may have no body through Playwright. Retain the full
                // status, headers and explicit API failure instead of losing the
                // packet to an unhandled late rejection. Completeness is checked
                // separately after complete SSR/DOM/hydration comparisons.
                captured.bodyNullReason = errorText(error);
              }
            })(),
          );
        };
        page.on("response", captureResponse);
        const settleResponses = async (): Promise<void> => {
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
          const observation = await page.evaluate(observeFixturePage);
          current.runtime.push({ mode, observation, errors, responses });
          write(`${producer}.json`, current);
          await settleResponses();
          assert.deepEqual(errors, []);
          assert.deepEqual(observation.result.warnings, []);
          assert.equal(observation.headings.length, 4);
          for (const heading of observation.headings) {
            assert.equal(heading.fontSize, desktop.fontSize);
            assert.equal(heading.lineHeight, desktop.lineHeight);
          }
          if (producer === "native" && mode === "hydrate") {
            // An explicit diagnostic counterexample moves the same complete
            // global sheet after scoped sheets, without editing any CSS rule.
            // This is separate from the production-order positive observation.
            const reversed = await page.evaluate(observeReversedCssOrder);
            current.reversedCssOrder = reversed;
            write(`${producer}.json`, current);
            for (const transition of reversed.transitionEvidence)
              assert.equal(
                transition.status,
                "finished",
                "the original finite transition must complete",
              );
            for (const heading of reversed.headings) {
              assert.equal(heading.fontSize, reversedExpected.fontSize);
              assert.equal(heading.lineHeight, reversedExpected.lineHeight);
              assert.notEqual(heading.lineHeight, desktop.lineHeight);
            }
          }
        } finally {
          await settleResponses();
          await context.close();
          page.off("response", captureResponse);
          await settleResponses();
          write(`${producer}.json`, current);
        }
      }
      receipts.push(current);
    } finally {
      if (packet) write(`${producer}.json`, packet);
      await server.close();
    }
  }
  const [official, native] = receipts;
  assert.ok(official && native, "official and native producers both completed");
  assert.equal(native.ssr.html, official.ssr.html, "complete native SSR HTML equals official Vue");
  for (const [index, runtime] of official.runtime.entries())
    assert.equal(
      native.runtime[index]?.observation.innerHtml,
      runtime.observation.innerHtml,
      `complete ${runtime.mode} DOM equals official Vue`,
    );
  for (const packet of [official, native]) {
    for (const transport of packet.transport ?? [])
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
        assert.match(response.headers["content-type"] ?? "", /text\/css/);
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
