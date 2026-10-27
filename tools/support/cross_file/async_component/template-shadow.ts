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
  ): {
    content: string;
  };
}
interface Vue {
  Fragment: unknown;
  createSSRApp(component: unknown): unknown;
  h(component: unknown, props?: unknown, children?: unknown): unknown;
}
interface Renderer {
  renderToString(app: unknown): Promise<string>;
}
type Manifest = { name: string; version: string };
function sha(bytes: string | Buffer): string {
  return createHash("sha256").update(bytes).digest("hex");
}

/** Stock compiler and runtime are a separate authority from the source NAPI image. */
export async function actualTemplateShadows(packageFile: string, expectedVersion = "3.5.43") {
  const providerRequire = createRequire(packageFile);
  const vueManifestFile = providerRequire.resolve("vue/package.json");
  const vueRequire = createRequire(vueManifestFile);
  const compilerManifestFile = vueRequire.resolve("@vue/compiler-sfc/package.json");
  const rendererManifestFile = vueRequire.resolve("@vue/server-renderer/package.json");
  const providers = [vueManifestFile, compilerManifestFile, rendererManifestFile].map((path) => {
    const bytes = readFileSync(path);
    const manifest = JSON.parse(bytes.toString()) as Manifest;
    assert.equal(manifest.version, expectedVersion, manifest.name);
    return { path, name: manifest.name, version: manifest.version, sha256: sha(bytes) };
  });
  assert.deepEqual(
    providers.map((provider) => provider.name),
    ["vue", "@vue/compiler-sfc", "@vue/server-renderer"],
  );
  const compiler = providerRequire("vue/compiler-sfc") as Compiler;
  const vue = providerRequire("vue") as Vue;
  const renderer = vueRequire("@vue/server-renderer") as Renderer;
  const project = pathToFileURL(join(dirname(packageFile), "template-shadow-runtime", "/")).href;
  const header = `<script setup>
import { defineAsyncComponent, h } from "vue";
const Inner = { render() { return h("i", "inner"); } };
const items = [Inner];
const Lazy = defineAsyncComponent(() => import("./Outer.vue"));
const Provider = (_props, { slots }) => slots.default({ Lazy: Inner });
</script>
`;
  const outer =
    '<script setup>const label = "outer";</script><template><b>{{ label }}</b></template>';
  const cases = [
    { id: "root", template: "<Lazy/>", calls: 1, reference: "root" },
    {
      id: "same-element-for",
      template: '<Lazy v-for="Lazy in items"/>',
      calls: 0,
      reference: "loop",
    },
    {
      id: "nested-for",
      template: '<template v-for="Lazy in items"><Lazy/></template>',
      calls: 0,
      reference: "loop",
    },
    {
      id: "slot",
      template: '<Provider v-slot="{ Lazy }"><Lazy/></Provider>',
      calls: 0,
      reference: "slot",
    },
    {
      id: "mixed",
      template: '<Lazy/><template v-for="Lazy in items"><Lazy/></template>',
      calls: 1,
      reference: "mixed",
    },
    { id: "kebab-for", template: '<lazy v-for="Lazy in items"/>', calls: 0, reference: "loop" },
    {
      id: "dynamic-for",
      template: '<component :is="Lazy" v-for="Lazy in items"/>',
      calls: 0,
      reference: "loop",
    },
    {
      id: "hyphenated-for",
      template: '<lazy-link v-for="LazyLink in items"/>',
      calls: 0,
      reference: "loop",
    },
    {
      id: "own-slot",
      template: '<Lazy v-slot="{ Lazy }"><Lazy/></Lazy>',
      calls: 1,
      reference: "root",
    },
  ];
  const sources: Record<string, string> = { "Outer.vue": outer };
  for (const test of cases) {
    const caseHeader =
      test.id === "hyphenated-for"
        ? header.replace("</script>", "const LazyLink = Lazy;\n</script>")
        : header;
    sources[test.id + ".vue"] = caseHeader + "<template>" + test.template + "</template>";
  }
  const compiled: Record<string, string> = {};
  const modules = new Map<string, string>();
  for (const [name, source] of Object.entries(sources)) {
    const filename = new URL(name, project).href;
    const parsed = compiler.parse(source, { filename });
    assert.deepEqual(parsed.errors, [], name);
    const code = compiler.compileScript(parsed.descriptor, {
      id: name,
      inlineTemplate: true,
    }).content;
    compiled[name] = code;
    modules.set(filename, code);
  }
  const loaderCalls = new Map<string, number>();
  const vueEntries = new Set<string>();
  const hooks = registerHooks({
    resolve(specifier, context, nextResolve) {
      if (specifier === "./Outer.vue") {
        assert(context.parentURL && modules.has(context.parentURL));
        loaderCalls.set(context.parentURL, (loaderCalls.get(context.parentURL) ?? 0) + 1);
      }
      const ownUrl = context.parentURL ? new URL(specifier, context.parentURL).href : specifier;
      if (modules.has(ownUrl)) return { url: ownUrl, shortCircuit: true };
      const result = nextResolve(specifier, context);
      if (specifier === "vue" && context.parentURL && modules.has(context.parentURL)) {
        vueEntries.add(result.url);
      }
      return result;
    },
    load(url, context, nextLoad) {
      const code = modules.get(url);
      return code === undefined
        ? nextLoad(url, context)
        : { format: "module", source: code, shortCircuit: true };
    },
  });
  const inner = { render: () => vue.h("i", null, "inner") };
  const outerReference = { render: () => vue.h("b", null, "outer") };
  const provider = {
    setup(_props: unknown, context: { slots: { default(props: { Lazy: unknown }): unknown } }) {
      return () => context.slots.default({ Lazy: inner });
    },
  };
  const loop = () => vue.h(vue.Fragment, null, [vue.h(inner)]);
  const reference = (kind: string) => ({
    render() {
      switch (kind) {
        case "root":
          return vue.h(outerReference);
        case "loop":
          return loop();
        case "slot":
          return vue.h(provider, null, {
            default: (props: { Lazy: unknown }) => [vue.h(props.Lazy)],
          });
        case "mixed":
          return vue.h(vue.Fragment, null, [vue.h(outerReference), loop()]);
        default:
          throw new Error("unknown reference " + kind);
      }
    },
  });
  const packets: Array<{ id: string; html: string; referenceHtml: string; loaderCalls: number }> =
    [];
  try {
    for (const test of cases) {
      const url = new URL(test.id + ".vue", project).href;
      const component = (await import(url)) as { default: unknown };
      const html = await renderer.renderToString(vue.createSSRApp(component.default));
      const referenceHtml = await renderer.renderToString(
        vue.createSSRApp(reference(test.reference)),
      );
      const calls = loaderCalls.get(url) ?? 0;
      packets.push({ id: test.id, html, referenceHtml, loaderCalls: calls });
      assert.equal(html, referenceHtml, test.id);
      assert.equal(calls, test.calls, test.id);
    }
  } finally {
    hooks.deregister();
  }
  assert.equal(vueEntries.size, 1, "one physical Vue entry for every compiled module");
  return {
    schema: "vize.actualVueTemplateShadows.v1",
    claim:
      "stock Vue compiler/runtime semantics; source NAPI and public Vize acceptance are separate",
    node: process.version,
    providers,
    sources,
    compiled,
    vueEntries: [...vueEntries].map((url) => {
      const bytes = readFileSync(new URL(url));
      return { url, bytes: bytes.length, sha256: sha(bytes) };
    }),
    packets,
  };
}
