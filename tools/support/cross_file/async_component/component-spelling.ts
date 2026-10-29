import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire, registerHooks } from "node:module";
import { dirname, join } from "node:path";
import { pathToFileURL } from "node:url";

interface Compiler {
  parse(source: string, options: { filename: string }): { descriptor: unknown; errors: unknown[] };
  compileScript(
    descriptor: unknown,
    options: { id: string; inlineTemplate: boolean },
  ): { content: string };
}
interface App {
  component(name: string, component: unknown): App;
}
interface Vue {
  createSSRApp(component: unknown): App;
  h(component: unknown, props?: unknown, children?: unknown): unknown;
}
interface Renderer {
  renderToString(app: App): Promise<string>;
}
type Manifest = { name: string; version: string };
const sha = (bytes: string | Buffer): string => createHash("sha256").update(bytes).digest("hex");

/** Independent stock compiler/runtime reference for Vue's binding lookup order. */
export async function actualComponentSpelling(packageFile: string, expectedVersion = "3.5.43") {
  const provider = createRequire(packageFile);
  const vueManifest = provider.resolve("vue/package.json");
  const fromVue = createRequire(vueManifest);
  const providers = [
    vueManifest,
    fromVue.resolve("@vue/compiler-sfc/package.json"),
    fromVue.resolve("@vue/server-renderer/package.json"),
  ].map((path) => {
    const bytes = readFileSync(path);
    const manifest = JSON.parse(bytes.toString()) as Manifest;
    assert.equal(manifest.version, expectedVersion);
    return { path, name: manifest.name, version: manifest.version, sha256: sha(bytes) };
  });
  const compiler = provider("vue/compiler-sfc") as Compiler;
  const vue = provider("vue") as Vue;
  const renderer = fromVue("@vue/server-renderer") as Renderer;
  const both =
    "const childWidget = dac(() => import('./Camel.vue')); const ChildWidget = dac(() => import('./Pascal.vue'));";
  const cases = [
    {
      id: "kebab-prefers-camel",
      declarations: both,
      template: "<child-widget/>",
      label: "camel",
      tag: "b",
      global: "",
      calls: [1, 0, 0],
    },
    {
      id: "raw-pascal",
      declarations: both,
      template: "<ChildWidget/>",
      label: "pascal",
      tag: "b",
      global: "",
      calls: [0, 1, 0],
    },
    {
      id: "raw-camel",
      declarations: both,
      template: "<childWidget/>",
      label: "camel",
      tag: "b",
      global: "",
      calls: [1, 0, 0],
    },
    {
      id: "underscore-is-distinct",
      declarations: "const Child_Widget = dac(() => import('./Local.vue'));",
      template: "<ChildWidget/>",
      label: "global",
      tag: "i",
      global: "ChildWidget",
      calls: [0, 0, 0],
    },
    {
      id: "uppercase-is-distinct",
      declarations: "const lower = dac(() => import('./Local.vue'));",
      template: "<Lower/>",
      label: "global",
      tag: "i",
      global: "Lower",
      calls: [0, 0, 0],
    },
    {
      id: "kebab-uses-pascal",
      declarations: "const ChildWidget = dac(() => import('./Pascal.vue'));",
      template: "<child-widget/>",
      label: "pascal",
      tag: "b",
      global: "",
      calls: [0, 1, 0],
    },
  ];
  cases.push(
    {
      id: "unicode-name-inside-valid-tag",
      declarations: "const CaféWidget = dac(() => import('./Pascal.vue'));",
      template: "<café-widget/>",
      label: "pascal",
      tag: "b",
      global: "",
      calls: [0, 1, 0],
    },
    {
      id: "hyphen-before-unicode-is-preserved",
      declarations: "const CafeÉclair = dac(() => import('./Local.vue'));",
      template: "<cafe-éclair/>",
      label: "global",
      tag: "i",
      global: "cafe-éclair",
      calls: [0, 0, 0],
    },
  );
  const project = pathToFileURL(join(dirname(packageFile), "component-spelling-runtime", "/")).href;
  const sources: Record<string, string> = {};
  for (const [name, label] of [
    ["Camel.vue", "camel"],
    ["Pascal.vue", "pascal"],
    ["Local.vue", "local"],
  ] as const)
    sources[name] =
      `<script setup>const label = "${label}";</script><template><b>{{ label }}</b></template>`;
  for (const test of cases)
    sources[test.id + ".vue"] =
      `<script setup>import { defineAsyncComponent as dac } from "vue";${test.declarations}</script><template>${test.template}</template>`;
  const compiled: Record<string, string> = {};
  const modules = new Map<string, string>();
  for (const [name, source] of Object.entries(sources)) {
    const url = new URL(name, project).href;
    const parsed = compiler.parse(source, { filename: url });
    assert.deepEqual(parsed.errors, [], name);
    const code = compiler.compileScript(parsed.descriptor, {
      id: name,
      inlineTemplate: true,
    }).content;
    compiled[name] = code;
    modules.set(url, code);
  }
  const calls = new Map<string, Record<string, number>>();
  const entries = new Set<string>();
  const names = ["./Camel.vue", "./Pascal.vue", "./Local.vue"];
  const hooks = registerHooks({
    resolve(specifier, context, nextResolve) {
      if (names.includes(specifier) && context.parentURL && modules.has(context.parentURL)) {
        const row = calls.get(context.parentURL) ?? {};
        row[specifier] = (row[specifier] ?? 0) + 1;
        calls.set(context.parentURL, row);
      }
      const url = context.parentURL ? new URL(specifier, context.parentURL).href : specifier;
      if (modules.has(url)) return { url, shortCircuit: true };
      const result = nextResolve(specifier, context);
      if (specifier === "vue" && context.parentURL && modules.has(context.parentURL))
        entries.add(result.url);
      return result;
    },
    load(url, context, nextLoad) {
      const source = modules.get(url);
      return source === undefined
        ? nextLoad(url, context)
        : { format: "module", source, shortCircuit: true };
    },
  });
  const packets = [];
  try {
    for (const test of cases) {
      const url = new URL(test.id + ".vue", project).href;
      const component = (await import(url)) as { default: unknown };
      const app = vue.createSSRApp(component.default);
      if (test.global) app.component(test.global, { render: () => vue.h("i", null, "global") });
      const html = await renderer.renderToString(app);
      const referenceHtml = await renderer.renderToString(
        vue.createSSRApp({ render: () => vue.h(test.tag, null, test.label) }),
      );
      const loaderCalls = names.map((name) => calls.get(url)?.[name] ?? 0);
      packets.push({ id: test.id, html, referenceHtml, loaderCalls });
      assert.equal(html, referenceHtml, test.id);
      assert.deepEqual(loaderCalls, test.calls, test.id);
    }
  } finally {
    hooks.deregister();
  }
  assert.equal(entries.size, 1);
  return {
    schema: "vize.actualVueComponentSpelling.v1",
    claim:
      "stock compiler/runtime reference only; source and public Vize acceptance remain separate",
    node: process.version,
    providers,
    sources,
    compiled,
    vueEntries: [...entries].map((url) => {
      const bytes = readFileSync(new URL(url));
      return { url, bytes: bytes.length, sha256: sha(bytes) };
    }),
    packets,
  };
}
