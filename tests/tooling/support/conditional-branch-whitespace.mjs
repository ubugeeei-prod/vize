// Source-built Rust tests supply complete DOM and SSR modules on stdin.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import { Window } from "happy-dom";

const corpusRoot = new URL("../../_fixtures/differential/compiler/", import.meta.url);
const manifest = JSON.parse(
  readFileSync(new URL("conditional-branch-whitespace.manifest.json", corpusRoot)),
);
const pinned = manifest.cases[0];
for (const artifact of [
  ...pinned.inputs.files.map((input) => ({
    ...input,
    path: `${pinned.inputs.root}/${input.path}`,
  })),
  pinned.reference,
]) {
  const bytes = readFileSync(new URL(artifact.path, corpusRoot));
  assert.equal(
    createHash("sha256").update(bytes).digest("hex"),
    artifact.sha256,
    `the complete reporter corpus must keep its pinned bytes: ${artifact.path}`,
  );
}

assert.equal(process.env.NODE_ENV, "production", "the blank Nuxt page occurs in production");
const window = new Window();
for (const name of ["window", "document", "Element", "HTMLElement", "SVGElement", "Node"])
  globalThis[name] = name === "window" ? window : window[name];
const require = createRequire(
  fileURLToPath(new URL("../../../npm/ui/package.json", import.meta.url)),
);
const vue = require("vue");
const { compileTemplate } = require("vue/compiler-sfc");
const server = require("vue/server-renderer");
assert.match(vue.version, /^3\.5\./, "the runtime oracle must use pinned stable Vue");
globalThis.__conditionalWhitespaceVue = { vue, server };

async function load(code, functionName) {
  const body = code.replace(
    /import \{([^}]*)\} from "(vue|@vue\/server-renderer|vue\/server-renderer)";?/g,
    (_, names, source) =>
      `const {${names.replace(/ as /g, ": ")}} = globalThis.__conditionalWhitespaceVue.${source === "vue" ? "vue" : "server"};`,
  );
  const source = body.includes(`export function ${functionName}`)
    ? body
    : `${body}\nexport { ${functionName} };`;
  return (await import(`data:text/javascript,${encodeURIComponent(source)}`))[functionName];
}

const branch = (name, props = []) => ({
  props,
  render: () => vue.h("section", { "data-branch": name }, name),
});
const components = {
  ErrorComponent: branch("error", ["error"]),
  IslandRenderer: branch("island", ["context"]),
  AppComponent: { render: () => vue.h("div", { id: "layout" }, vue.h("h1", "Hello 1")) },
  B: { render: () => vue.h("b", "B") },
  C: { render: () => vue.h("u", "C") },
};
function component(render, state, ssr = false) {
  return {
    components,
    [ssr ? "ssrRender" : "render"]: render,
    data: () => ({
      a: false,
      b: false,
      end: "end",
      abortRender: false,
      error: null,
      islandContext: null,
      ...state,
      SingleRenderer: state.singleRenderer ? vue.markRaw(branch("single")) : null,
    }),
    methods: { onResolve() {} },
  };
}
async function mount(render, state) {
  const host = document.createElement("div");
  document.body.append(host);
  const app = vue.createApp(component(render, state));
  try {
    app.mount(host);
    await vue.nextTick();
    return host.innerHTML;
  } finally {
    app.unmount();
    host.remove();
  }
}
const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const cases = JSON.parse(Buffer.concat(chunks).toString("utf8"));
for (const fixture of cases) {
  const common = {
    source: fixture.template,
    filename: `${fixture.name}.vue`,
    id: fixture.name,
    cssVars: [],
    compilerOptions: { whitespace: fixture.whitespace },
  };
  const upstreamDom = compileTemplate(common);
  const upstreamSsr = compileTemplate({ ...common, ssr: true });
  assert.deepEqual(upstreamDom.errors, []);
  assert.deepEqual(upstreamSsr.errors, []);
  const dom = await load(fixture.dom, "render");
  const oracleDom = await load(upstreamDom.code, "render");
  const ssr = await load(fixture.ssr, "ssrRender");
  const oracleSsr = await load(upstreamSsr.code, "ssrRender");
  for (const [index, state] of fixture.states.entries()) {
    const label = `${fixture.whitespace}/${fixture.name}/${index}`;
    const oracleHtml = await mount(oracleDom, state.data);
    assert.equal(oracleHtml, state.html, `${label}: independent expected DOM`);
    const html = await mount(dom, state.data);
    assert.equal(html, oracleHtml, `${label}: actual production DOM`);
    const oracleSsrHtml = await server.renderToString(
      vue.createSSRApp(component(oracleSsr, state.data, true)),
    );
    const ssrHtml = await server.renderToString(vue.createSSRApp(component(ssr, state.data, true)));
    assert.equal(ssrHtml, oracleSsrHtml, `${label}: complete SSR HTML`);
    console.log(JSON.stringify({ label, vue: vue.version, html, ssrHtml }));
  }
}
