// Supplemental genuine component observations; original 84 current outcomes stay mandatory.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { environment7502 } from "./native-attribute-values-7502-loader.ts";
import { primary7502 } from "./native-attribute-values-7502-primary.ts";
import { hash7502 } from "./native-attribute-values-7502-inputs.ts";

const mode = process.env.NODE_ENV;
assert(mode === "development" || mode === "production");
const source = readFileSync(process.argv[2]);
const capture = readFileSync(process.argv[3]);
const fixtures = JSON.parse(source.toString("utf8"));
const rows = JSON.parse(capture.toString("utf8"));
assert.equal(fixtures.positive.length, 16);
assert.equal(rows.length, 16);
const environment = await environment7502(mode);
const descendants = (host: any) => {
  const nodes: any[] = [];
  const walk = (parent: any) => {
    for (const child of parent.childNodes) {
      nodes.push(child);
      walk(child);
    }
  };
  walk(host);
  return nodes;
};
async function mount(loaded: any, configuration: any, html: string | null = null) {
  const host = document.body.appendChild(document.createElement("div"));
  if (html !== null) host.innerHTML = html;
  const original = descendants(host);
  const initialTree = environment.hostTree(host);
  const component = loaded.component;
  component.inheritAttrs = configuration.inheritAttrs;
  const vue = loaded.runtime;
  const vapor = component.__vapor === true;
  const create = vapor
    ? html === null
      ? vue.createVaporApp
      : vue.createVaporSSRApp
    : html === null
      ? vue.createApp
      : vue.createSSRApp;
  const app = create(component, configuration.props);
  assert.deepEqual(app._component, component);
  assert.equal(app._component.render, component.render);
  const diagnostics: string[] = [];
  app.config.warnHandler = (message: string) => diagnostics.push(message);
  app.config.errorHandler = (error: any) => diagnostics.push(String(error));
  const prior = { warn: console.warn, error: console.error };
  console.warn = (...args) => diagnostics.push(args.map(String).join(" "));
  console.error = (...args) => diagnostics.push(args.map(String).join(" "));
  try {
    app.mount(host);
    await vue.nextTick();
    const nodes = descendants(host),
      tree = environment.hostTree(host);
    const retained = original.map((node) => host.contains(node));
    app.unmount();
    await vue.nextTick();
    return {
      nodes,
      trace: {
        initialTree,
        tree,
        diagnostics,
        retained,
        originalNodes: original.length,
        unmounted: environment.hostTree(host),
        renderIdentity: true,
      },
    };
  } finally {
    console.warn = prior.warn;
    console.error = prior.error;
    host.remove();
  }
}
async function render(loaded: any, configuration: any) {
  loaded.component.inheritAttrs = configuration.inheritAttrs;
  const app = loaded.runtime.createSSRApp(loaded.component, configuration.props);
  assert.deepEqual(app._component, loaded.component);
  assert.equal(app._component.ssrRender, loaded.component.ssrRender);
  const diagnostics: string[] = [];
  app.config.warnHandler = (message: string) => diagnostics.push(message);
  app.config.errorHandler = (error: any) => diagnostics.push(String(error));
  const prior = { warn: console.warn, error: console.error };
  console.warn = (...args) => diagnostics.push(args.map(String).join(" "));
  console.error = (...args) => diagnostics.push(args.map(String).join(" "));
  try {
    const html = await loaded.renderer.renderToString(app);
    return { html, tree: environment.htmlTree(html), diagnostics, renderIdentity: true };
  } finally {
    console.warn = prior.warn;
    console.error = prior.error;
  }
}
const configurations = [
  { inheritAttrs: true, props: {} },
  { inheritAttrs: true, props: { class: "caller", title: "inherited", "data-owner": "caller" } },
  { inheritAttrs: false, props: { class: "caller", title: "inherited", "data-owner": "caller" } },
];
const frame: any = {
  schema: "vize.native-vapor-static-class.runtime",
  version: 1,
  mode,
  sourceSha256: hash7502(source),
  captureSha256: hash7502(capture),
  nativeSource: "fresh Rust original-owning template/component capture",
  componentPopulation: 14,
  templatePopulation: 16,
  observations: [],
};
try {
  for (const [index, fixture] of fixtures.positive.entries()) {
    const row = rows[index];
    const observation: any = {
      id: fixture.id,
      source: fixture.source,
      codeSha256: hash7502(row.code),
      componentCodeSha256: row.component.code === null ? null : hash7502(row.component.code),
      primary: null,
      literalTree: environment.htmlTree(fixture.expectedTemplateHtml),
      contexts: [],
      failure: null,
    };
    frame.observations.push(observation);
    try {
      assert.equal(row.id, fixture.id);
      assert.equal(row.source, fixture.source);
      assert.equal(row.code, fixture.expectedCode);
      // All whole pinned official dev/prod modules and raw maps remain independent observations.
      observation.primary = await primary7502(fixture.source);
      if (row.component.code === null) {
        assert(["multiple-roots", "comment-root"].includes(fixture.id));
        assert(row.component.error.includes("ComponentFragmentLifecycle"));
        continue;
      }
      const native = await environment.load(row.component.code, "vapor");
      const dom = await environment.load(
        observation.primary.dom.code,
        "dom",
        observation.primary.dom.helperCode,
      );
      const ssr = await environment.load(
        observation.primary.ssr.code,
        "ssr",
        observation.primary.ssr.helperCode,
      );
      for (const configuration of configurations) {
        const expected = await mount(dom, configuration),
          server = await render(ssr, configuration);
        const mounted = await mount(native, configuration),
          cloned = await mount(native, configuration);
        const hydrated = await mount(native, configuration, server.html);
        const context = {
          configuration,
          officialDom: expected.trace,
          officialSsr: server,
          native: mounted.trace,
          cloned: {
            ...cloned.trace,
            distinctNodes:
              cloned.nodes.length > 0 &&
              cloned.nodes.every((node) => !mounted.nodes.includes(node)),
          },
          hydrated: hydrated.trace,
        };
        observation.contexts.push(context);
      }
    } catch (error) {
      observation.failure = {
        name: error instanceof Error ? error.name : typeof error,
        message: error instanceof Error ? error.message : String(error),
        stack: error instanceof Error ? error.stack : null,
      };
    }
  }
  // Whole observations, including every failed case, are retained before the parent judge asserts.
  process.stdout.write(JSON.stringify(frame));
} finally {
  await environment.window.happyDOM.close();
}
