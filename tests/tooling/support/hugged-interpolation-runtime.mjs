import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { compileFunction } from "node:vm";
import { Window } from "happy-dom";

const window = new Window();
for (const name of ["window", "document", "Element", "HTMLElement", "SVGElement", "Node"])
  globalThis[name] = name === "window" ? window : window[name];
const require = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const vue = require("vue");
const server = require("vue/server-renderer");
assert.equal(vue.version, "3.5.35");

function load(code, name) {
  const body = code
    .replace(
      /import \{([^}]*)\} from "(vue|vue\/server-renderer|@vue\/server-renderer)";?/g,
      (_, names, from) =>
        `const {${names.replace(/ as /g, ": ")}} = ${from === "vue" ? "Vue" : "Server"};`,
    )
    .replace(`export function ${name}`, `function ${name}`);
  assert.doesNotMatch(body, /\b(?:import|export)\b/);
  return compileFunction(`${body}\nreturn ${name};`, ["Vue", "Server"])(vue, server);
}

// Consume the two exact complete compiler modules; no code-normalized comparison.
export async function observeHuggedRuntime(compiled, reference, observed, persist) {
  const render = load(compiled.dom.code, "render");
  const ssrRender = load(compiled.ssr.code, "ssrRender");
  for (const control of reference.states) {
    const row = { id: control.id, domHtml: null, ssrHtml: null, error: null, warnings: [] };
    observed.push(row);
    persist();
    const methods = {
      veryLongFunctionName(left, right) {
        return `${left}|${right}`;
      },
    };
    const data = () => structuredClone(control.state);
    const host = document.createElement("div");
    document.body.append(host);
    const app = vue.createApp({ data, methods, render });
    app.config.warnHandler = (message) => row.warnings.push(message);
    try {
      app.mount(host);
      await vue.nextTick();
      row.domHtml = host.innerHTML;
      persist();
      const ssrApp = vue.createSSRApp({ data, methods, ssrRender });
      ssrApp.config.warnHandler = (message) => row.warnings.push(message);
      row.ssrHtml = await server.renderToString(ssrApp);
      persist();
      assert.equal(row.domHtml, control.domHtml, `${control.id}: complete mounted DOM`);
      assert.equal(row.ssrHtml, control.ssrHtml, `${control.id}: complete SSR HTML`);
      assert.deepEqual(row.warnings, []);
    } catch (error) {
      row.error = { message: error.message, stack: error.stack };
      persist();
      throw error;
    } finally {
      try {
        app.unmount();
      } finally {
        host.remove();
        persist();
      }
    }
  }
}
