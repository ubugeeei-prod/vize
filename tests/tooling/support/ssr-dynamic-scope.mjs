import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire, registerHooks } from "node:module";
import { pathToFileURL } from "node:url";

const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const require = createRequire(fromUi.resolve("vue/package.json"));
assert.equal(require("vue/package.json").version, "3.5.35");
assert.equal(require("@vue/server-renderer/package.json").version, "3.5.35");
const { createSSRApp, h } = require("vue");
const { renderToString } = require("@vue/server-renderer");
const input = JSON.parse(readFileSync(0, "utf8"));
const moduleUrl = new URL("./__vuefes_original_heading__.vue", import.meta.url).href;
const vueUrl = pathToFileURL(require.resolve("vue/dist/vue.cjs.js")).href;
const rendererUrl = pathToFileURL(require.resolve("@vue/server-renderer")).href;
registerHooks({
  resolve(specifier, context, next) {
    if (specifier === moduleUrl) return { url: moduleUrl, shortCircuit: true };
    if (specifier === "vue") return { url: vueUrl, shortCircuit: true };
    if (specifier === "@vue/server-renderer" || specifier === "vue/server-renderer")
      return { url: rendererUrl, shortCircuit: true };
    return next(specifier, context);
  },
  load(url, context, next) {
    if (url === moduleUrl)
      return { format: "module-typescript", source: input.compileResult.code, shortCircuit: true };
    return next(url, context);
  },
});
const { default: Heading } = await import(moduleUrl);
// This is the existing public Vite assembly contract: the raw SFC compiler
// emits the render module and the adapter attaches its retained scoped ID.
// The hosted original graph independently records this same assignment.
assert.equal(input.hasScoped, true);
assert.equal(Heading.__scopeId, undefined);
Heading.__scopeId = input.scopeId;
// The complete original transport workload: four authored props, each with
// text and span slots. No case, attribute, marker or diagnostic is filtered.
const props = [{}, { id: "anchor" }, { nth: 1 }, { nth: 3, id: "anchor-three" }];
const Root = {
  render: () =>
    h(
      "main",
      props.flatMap((props, index) =>
        ["text", "span"].map((kind) =>
          h("section", { "data-case": index + "-" + kind }, [
            h(Heading, props, {
              default: () => (kind === "text" ? "complete slot" : h("span", "complete slot")),
            }),
          ]),
        ),
      ),
    ),
};
const warnings = [];
const app = createSSRApp(Root);
app.config.warnHandler = (message) => warnings.push(message);
const ssr = { html: await renderToString(app), warnings };
assert.deepEqual(ssr, input.expected);
process.stdout.write(JSON.stringify({ runtime: "3.5.35", cases: 8, ssr }) + "\n");
