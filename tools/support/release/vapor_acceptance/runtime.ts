import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import type { Plugin } from "vite";
import { publicNative } from "../public_acceptance/installed.ts";
import { assertTrace, digest, reportedRuntimeEntry, reportedVue } from "./fixtures.ts";
import {
  assertNamespaceTrace,
  campaignDefinition,
  type Binding,
  type PhysicalRow,
} from "./namespace.ts";

const output = path.resolve(process.env.PUBLIC_OUTPUT!);
const preparation = JSON.parse(fs.readFileSync(path.join(output, "preparation.json"), "utf8"));
const campaign = campaignDefinition(preparation.campaign.name);
assert.deepEqual(preparation.campaign, campaign, "reviewed campaign identity and denominator");
const publication = JSON.parse(fs.readFileSync(path.join(output, "publication.json"), "utf8"));
const root = fs.realpathSync(process.cwd());
assert.equal(root, preparation.consumer);
const require = createRequire(path.join(root, "package.json"));
const { native, packages, loaded } = publicNative(preparation.version);
assert.equal(typeof native.compileSfc, "function");
const { createServer, transformWithOxc } = await import("vite");
const pluginEntry = fs.realpathSync(require.resolve("@vizejs/vite-plugin"));
const pluginPackage = packages.find((entry) => entry.name === "@vizejs/vite-plugin");
assert.ok(pluginPackage?.installed && pluginEntry.startsWith(pluginPackage.directory + path.sep));
const pluginSha256 = digest(fs.readFileSync(pluginEntry));
const { default: vize } = (await import(pluginEntry)) as {
  default: (options: { vapor: true }) => Plugin[];
};
const { chromium } = await import("playwright");
const compiler = require("@vue/compiler-sfc");
assert.equal(compiler.version, reportedVue);
assert.equal(require("vue/package.json").version, reportedVue);
assert.equal(require("@vue/runtime-vapor/package.json").version, reportedVue);
const runtimeEntry = require.resolve(reportedRuntimeEntry);
const manifest = fs.readFileSync(path.join(root, "package.json"));
assert.equal(digest(manifest), preparation.manifestSha256);
const lock = JSON.parse(fs.readFileSync(path.join(root, "package-lock.json"), "utf8"));
const runtimes = [
  "vue",
  "@vue/runtime-vapor",
  "@vue/compiler-sfc",
  "@vue/compiler-dom",
  "@vue/runtime-dom",
].map((name) => {
  const entry = lock.packages[`node_modules/${name}`];
  assert.equal(entry.version, reportedVue, name);
  assert.equal(new URL(entry.resolved).origin, "https://registry.npmjs.org");
  assert.match(entry.integrity, /^sha512-[A-Za-z0-9+/]+={0,2}$/u);
  assert.equal(entry.link, undefined);
  return { name, version: entry.version, resolved: entry.resolved, integrity: entry.integrity };
});
for (const entry of preparation.harness)
  assert.equal(digest(fs.readFileSync(path.join(root, entry.file))), entry.sha256, entry.file);
const receipt: Record<string, unknown> = {
  schema: campaign.schema,
  campaign,
  source: preparation.source,
  version: preparation.version,
  reportedVue,
  runtimes,
  runtimeEntrySha256: digest(fs.readFileSync(runtimeEntry)),
  packages,
  loaded,
  plugin: { entry: pluginEntry, sha256: pluginSha256 },
  preparation,
  observations: [],
  errors: [],
  success: false,
};
const persist = () =>
  fs.writeFileSync(path.join(output, "vapor.json"), JSON.stringify(receipt, null, 2) + "\n");
persist();
for (const item of packages) {
  const original = publication.npm.find((entry: { name: string }) => entry.name === item.name);
  assert.ok(original, item.name);
  assert.equal(item.integrity, original.integrity);
  assert.equal(item.resolved, original.tarball);
}
for (const authority of preparation.fixtures.authorities) {
  const suffix = authority.file.slice("tests/_fixtures/differential/compiler/".length);
  assert.equal(digest(fs.readFileSync(path.join(root, "original", suffix))), authority.sha256);
}

// This is the existing separate-template Vue VDOM oracle, on the reporter's release.
async function stock(source: string, filename: string) {
  const parsed = compiler.parse(source, { filename });
  assert.deepEqual(parsed.errors, []);
  const descriptor = parsed.descriptor;
  const isTS = descriptor.scriptSetup?.lang === "ts" || descriptor.script?.lang === "ts";
  const script =
    descriptor.script || descriptor.scriptSetup
      ? compiler.compileScript(
          { ...descriptor, vapor: false },
          { id: "public-reported-vapor", genDefaultAs: "__component" },
        )
      : { content: "const __component = {};", bindings: {} };
  const template = compiler.compileTemplate({
    source: descriptor.template.content,
    filename,
    id: "public-reported-vapor",
    compilerOptions: { bindingMetadata: script.bindings, isTS },
  });
  assert.deepEqual(template.errors, []);
  assert.deepEqual(template.tips, []);
  const code = `${script.content}\n${template.code}\n__component.render = render;\nexport default __component;\n`;
  return isTS
    ? (await transformWithOxc(code, `${filename}.ts`, { lang: "ts", sourcemap: false })).code
    : code;
}

const browser = await chromium.launch({ headless: true });
const servers: Awaited<ReturnType<typeof createServer>>[] = [];
const rows = preparation.fixtures.rows as PhysicalRow[];
assert.equal(rows.length, campaign.rows);
assert.equal(new Set(rows.map((row) => row.name)).size, campaign.rows);
const executedSources: { file: string; sha256: string }[] = [];
try {
  const traces = new Map<string, unknown>();
  for (const backend of ["vapor", "vdom"] as const) {
    const caseRoot = path.join(root, backend);
    fs.mkdirSync(caseRoot);
    for (const [index, row] of rows.entries()) {
      const dir = path.join(caseRoot, String(index));
      fs.mkdirSync(dir);
      const childName = (row.childModule ?? "./Child.vue").slice(2);
      fs.writeFileSync(path.join(dir, "App.vue"), row.app, { flag: "wx" });
      fs.writeFileSync(path.join(dir, childName), row.child, { flag: "wx" });
      if (row.wrapper) fs.writeFileSync(path.join(dir, "Wrapper.vue"), row.wrapper, { flag: "wx" });
      fs.writeFileSync(
        path.join(dir, "fixture.js"),
        `import { reactive } from 'vue';\nexport const state = reactive(${JSON.stringify(row.context)});\nexport const events=[];\nexport const record=value=>events.push(value);\nexport const makeHandler=()=>record;\nexport const handlers={'x;y':record};\n`,
        { flag: "wx" },
      );
      fs.writeFileSync(
        path.join(dir, "main.js"),
        `import * as vue from 'vue';\nimport Component from './${row.wrapper ? "Wrapper" : "App"}.vue';\nimport {state,events} from './fixture.js';\nwindow.__probe={vue,state,events,Component};\n`,
        { flag: "wx" },
      );
      fs.writeFileSync(
        path.join(dir, "index.html"),
        '<html><body><div id="host"></div><script type="module" src="./main.js"></script></body></html>',
        { flag: "wx" },
      );
      for (const file of fs.readdirSync(dir))
        executedSources.push({
          file: path.join(dir, file),
          sha256: digest(fs.readFileSync(path.join(dir, file))),
        });
    }
    const server = await createServer({
      root: caseRoot,
      configFile: false,
      resolve: { alias: { vue: runtimeEntry } },
      plugins:
        backend === "vapor"
          ? [vize({ vapor: true })]
          : [
              {
                name: "independent-reported-vdom",
                enforce: "pre",
                async transform(source, id) {
                  if (!id.endsWith(".vue")) return null;
                  return { code: await stock(source, id), map: null };
                },
              },
            ],
      server: { host: "127.0.0.1", port: 0 },
      logLevel: "warn",
    });
    servers.push(server);
    await server.listen();
    const address = server.httpServer!.address();
    assert.ok(address && typeof address !== "string");
    for (const [index, row] of rows.entries()) {
      const page = await browser.newPage();
      const errors: string[] = [];
      page.on("pageerror", (error) => errors.push(String(error)));
      page.on("console", (message) => {
        if (message.type() === "error") errors.push(message.text());
      });
      let trace, disposal;
      try {
        await page.goto(`http://127.0.0.1:${address.port}/${index}/index.html`);
        await page.waitForFunction(() => !!Reflect.get(window, "__probe"));
        const modules = [
          "App.vue",
          ...(row.app.includes(row.childModule ?? "./Child.vue")
            ? [(row.childModule ?? "./Child.vue").slice(2)]
            : []),
          ...(row.wrapper ? ["Wrapper.vue"] : []),
        ];
        for (const module of modules) {
          const response = await page.request.get(
            `http://127.0.0.1:${address.port}/${index}/${module}`,
          );
          assert.equal(response.status(), 200, "actual transformed module response");
          fs.writeFileSync(
            path.join(output, `${backend}-${index}-${module}.mjs`),
            await response.body(),
            { flag: "wx" },
          );
        }
        trace = await page.evaluate(
          async ({ backend, row, reportedVue }) => {
            const { vue, state, events, Component } = Reflect.get(window, "__probe");
            if (vue.version !== reportedVue) throw new Error("reporter runtime version mismatch");
            const diagnostics: string[] = [];
            const host = document.querySelector("#host")!;
            const app = (backend === "vapor" ? vue.createVaporApp : vue.createApp)(
              Component,
              row.rootProps,
            );
            app.config.warnHandler = (message: string) => diagnostics.push(message);
            app.config.errorHandler = (error: unknown) => diagnostics.push(String(error));
            const observe = (parent: Element): unknown[] => {
              const children: unknown[] = [];
              for (const node of parent.childNodes) {
                if (node.nodeType === Node.TEXT_NODE) {
                  const text = (node as Text).data;
                  if (!text) continue;
                  if (typeof children.at(-1) === "string") children[children.length - 1] += text;
                  else children.push(text);
                } else if (node instanceof Element) {
                  const value: Record<string, unknown> = {
                    tag: node.localName,
                    attributes: Object.fromEntries(
                      [...node.attributes]
                        .map(({ name, value }) => [name, value])
                        .sort(([a], [b]) => a.localeCompare(b)),
                    ),
                    children: observe(node),
                  };
                  if (node instanceof HTMLButtonElement) value.disabled = node.disabled;
                  children.push(value);
                }
              }
              return children;
            };
            const snapshots: {
              tree: unknown[];
              namespaces: string[];
              events: unknown[];
              svg: unknown;
              bindings?: Binding[];
            }[] = [];
            const capture = () => {
              const svg = host.querySelector("svg");
              let svgObservation: unknown = null;
              if (svg) {
                if (
                  !(svg instanceof SVGSVGElement) ||
                  svg.getBoundingClientRect().width !== 64 ||
                  svg.getBoundingClientRect().height !== 64
                )
                  throw new Error("original SVG has no physical 64x64 rendering");
                const shape = svg.querySelector("circle,rect");
                if (
                  !(shape instanceof SVGGraphicsElement) ||
                  shape.getBBox().width !== 32 ||
                  shape.getBBox().height !== 32
                )
                  throw new Error("original SVG shape has no physical 32x32 rendering");
                const computed = getComputedStyle(svg).color;
                if (Object.prototype.toString.call(svg.className) !== "[object SVGAnimatedString]")
                  throw new Error("real SVGAnimatedString ownership required");
                svgObservation = {
                  namespace: svg.namespaceURI,
                  width: svg.getBoundingClientRect().width,
                  height: svg.getBoundingClientRect().height,
                  shapeWidth: shape.getBBox().width,
                  shapeHeight: shape.getBBox().height,
                  computedColor: computed,
                  classNameType: Object.prototype.toString.call(svg.className),
                };
                if (row.name.startsWith("original-") || row.wrapper) {
                  const tier = row.wrapper ? state.tier : row.rootProps!.tier;
                  if (computed !== (tier === 1 ? "rgb(200, 0, 0)" : "rgb(0, 0, 200)"))
                    throw new Error("original dynamic SVG color does not reach computed style");
                }
              }
              snapshots.push({
                tree: observe(host),
                namespaces: [...host.querySelectorAll("*")].map((node) => node.namespaceURI!),
                events: [...events],
                svg: svgObservation,
                ...(row.boundIds
                  ? {
                      bindings:
                        host.childNodes.length === 0
                          ? []
                          : row.boundIds.map((id) => {
                              const node = host.querySelector(`#${id}`);
                              if (!(node instanceof SVGElement || node instanceof HTMLElement))
                                throw new Error(`missing physical binding ${id}`);
                              return {
                                id,
                                namespace: node.namespaceURI!,
                                classNameType: Object.prototype.toString.call(node.className),
                                classAttribute: node.getAttribute("class"),
                                classProperty:
                                  node instanceof SVGElement
                                    ? node.className.baseVal
                                    : node.className,
                                styleAttribute: node.getAttribute("style"),
                                styleColor: node.style.color,
                                computedColor: getComputedStyle(node).color,
                              };
                            }),
                    }
                  : {}),
              });
            };
            try {
              app.mount(host);
              await vue.nextTick();
              capture();
              for (const step of row.steps) {
                if (step.patch) Object.assign(state, step.patch);
                else if (step.click) {
                  const button = host.querySelector(String(step.click)) as HTMLButtonElement | null;
                  if (!button) throw new Error(`missing original event target ${step.click}`);
                  button.click();
                } else throw new Error("unsupported original action");
                await vue.nextTick();
                capture();
              }
              const retained = [...host.children];
              const retainedMarkup = retained.map((node) => node.outerHTML);
              const retainedEvents = JSON.stringify(events);
              app.unmount();
              await vue.nextTick();
              if (host.childNodes.length !== 0)
                throw new Error("unmount must remove all physical nodes including comment anchors");
              const invalidations = row.steps.flatMap((step) => (step.patch ? [step.patch] : []));
              for (const patch of invalidations) {
                Object.assign(state, patch);
                await vue.nextTick();
                if (
                  JSON.stringify(retained.map((node) => node.outerHTML)) !==
                  JSON.stringify(retainedMarkup)
                )
                  throw new Error("unmounted fixture-driven DOM effects remain active");
                if (JSON.stringify(events) !== retainedEvents || host.childNodes.length !== 0)
                  throw new Error("post-unmount fixture invalidation changed the host or events");
              }
              Reflect.set(window, "__publicVaporDisposal", {
                backend,
                hostChildNodes: host.childNodes.length,
                retainedNodes: retained.length,
                fixtureInvalidations: invalidations,
                retainedMarkup,
                afterInvalidation: retained.map((node) => node.outerHTML),
              });
              capture();
              if (diagnostics.length) throw new Error(JSON.stringify(diagnostics));
              return snapshots;
            } finally {
              if (host.childNodes.length) app.unmount();
            }
          },
          { backend, row, reportedVue },
        );
        assert.deepEqual(errors, [], `${row.name}: browser errors`);
        disposal = await page.evaluate(() => Reflect.get(window, "__publicVaporDisposal"));
        assert.equal(disposal.hostChildNodes, 0);
        assert.deepEqual(disposal.afterInvalidation, disposal.retainedMarkup);
        if (campaign.name === "reported") {
          assert.equal(
            row.boundIds,
            undefined,
            "authored namespace rows cannot enter original campaign",
          );
          assertTrace(row, trace);
        } else assertNamespaceTrace(row, trace);
        if (backend === "vapor") traces.set(row.name, trace);
        else
          assert.deepEqual(
            traces.get(row.name),
            trace,
            `${row.name}: complete independent VDOM trace`,
          );
      } finally {
        (receipt.observations as unknown[]).push({
          backend,
          issue: row.issue,
          name: row.name,
          trace,
          disposal,
          errors,
        });
        persist();
        await page.close();
      }
    }
    await server.close();
  }
  assert.equal(
    (receipt.observations as unknown[]).length,
    campaign.observations,
    "complete separate campaign traces with actual disposal receipts",
  );
  for (const entry of preparation.harness)
    assert.equal(digest(fs.readFileSync(path.join(root, entry.file))), entry.sha256);
  for (const entry of executedSources)
    assert.equal(digest(fs.readFileSync(entry.file)), entry.sha256, entry.file);
  for (const authority of preparation.fixtures.authorities) {
    const suffix = authority.file.slice("tests/_fixtures/differential/compiler/".length);
    assert.equal(digest(fs.readFileSync(path.join(root, "original", suffix))), authority.sha256);
  }
  assert.deepEqual(
    publicNative(preparation.version).loaded,
    loaded,
    "same installed provider remains loaded",
  );
  receipt.executedSources = executedSources;
  assert.equal(digest(fs.readFileSync(pluginEntry)), pluginSha256, "same published plugin bytes");
  receipt.lockSha256 = digest(fs.readFileSync(path.join(root, "package-lock.json")));
  fs.copyFileSync(path.join(root, "package-lock.json"), path.join(output, "package-lock.json"));
  receipt.success = true;
  receipt.scope =
    campaign.name === "reported"
      ? "Six unchanged original report controls and two authored live SVG/update/removal controls on installed public Vize and exact reported Vue 3.6.0-rc.10 in real Chromium, with complete independent VDOM traces, physical SVG dimensions/computed color and actual unmount. No SSR, full Vapor parity or n8n umbrella credit."
      : "One separately authored inherited SVG/HTML integration-point case on installed public Vize and exact Vue 3.6.0-rc.10 in real Chromium: complete independent VDOM mount/update/clear/unmount traces, five physical SVGAnimatedString and two HTML class bindings, inline style removal and actual disposal. No original six-report, SSR, full Vapor parity or n8n umbrella credit.";
  persist();
  console.log(
    JSON.stringify({
      source: preparation.source,
      reportedVue,
      ...campaign,
      success: true,
    }),
  );
} catch (error) {
  receipt.failure = { name: error instanceof Error ? error.name : "Error", message: String(error) };
  persist();
  throw error;
} finally {
  await Promise.allSettled(servers.map((server) => server.close()));
  await browser.close();
}
