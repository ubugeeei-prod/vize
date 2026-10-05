import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import { Window } from "happy-dom";

const window = new Window();
for (const name of ["window", "document", "Element", "HTMLElement", "SVGElement", "Node"])
  globalThis[name] = name === "window" ? window : window[name];
const require = createRequire(
  fileURLToPath(new URL("../../../npm/ui/package.json", import.meta.url)),
);
const vue = require("vue");
const compiler = require("vue/compiler-sfc");
const server = require("vue/server-renderer");
const ts = createRequire(import.meta.url)("typescript");
assert.match(vue.version, /^3\.5\./, "the runtime oracle uses the existing pinned Vue");
globalThis.__formatterTemplateOracle = { vue, server };

async function load(code, functionName) {
  const body = ts.transpileModule(code, {
    compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
    reportDiagnostics: true,
  });
  assert.deepEqual(body.diagnostics, []);
  const source = body.outputText.replace(
    /import \{([^}]*)\} from "(vue|@vue\/server-renderer|vue\/server-renderer)";?/g,
    (_, names, from) =>
      `const {${names.replace(/ as /g, ": ")}} = globalThis.__formatterTemplateOracle.${from === "vue" ? "vue" : "server"};`,
  );
  return (await import(`data:text/javascript,${encodeURIComponent(source)}`))[functionName];
}

export async function observeSemantics(source, fixture, whitespace = "condense") {
  const parsed = compiler.parse(source, { filename: fixture.file });
  assert.deepEqual(parsed.errors, [], `${fixture.id}: complete SFC parsing`);
  const options = {
    source: parsed.descriptor.template.content,
    filename: fixture.file,
    id: fixture.id,
    cssVars: [],
    compilerOptions: { whitespace, expressionPlugins: ["typescript"] },
  };
  const dom = compiler.compileTemplate(options);
  const ssr = compiler.compileTemplate({ ...options, ssr: true });
  assert.deepEqual(dom.errors, [], `${fixture.id}: DOM expression parsing`);
  assert.deepEqual(ssr.errors, [], `${fixture.id}: SSR expression parsing`);
  const render = await load(dom.code, "render");
  const ssrRender = await load(ssr.code, "ssrRender");
  const observed = { vue: vue.version, whitespace, dom: dom.code, ssr: ssr.code, states: [] };
  for (const [index, state] of fixture.states.entries()) {
    const data = () => ({ ...state, value: { m: () => 7 }, fallback: { m: () => 9 } });
    const host = document.createElement("div");
    document.body.append(host);
    const app = vue.createApp({ data, render });
    try {
      app.mount(host);
      await vue.nextTick();
      const row = { html: host.innerHTML };
      assert.equal(row.html, fixture.html[index], `${fixture.id}/${index}: original DOM contract`);
      if (fixture.eventHtml) {
        host
          .querySelector("button")
          .dispatchEvent(new window.MouseEvent("click", { bubbles: true }));
        await vue.nextTick();
        row.eventHtml = host.innerHTML;
        assert.equal(
          row.eventHtml,
          fixture.eventHtml[index],
          `${fixture.id}/${index}: event result`,
        );
      }
      row.ssrHtml = await server.renderToString(vue.createSSRApp({ data, ssrRender }));
      observed.states.push(row);
    } finally {
      app.unmount();
      host.remove();
    }
  }
  return observed;
}
