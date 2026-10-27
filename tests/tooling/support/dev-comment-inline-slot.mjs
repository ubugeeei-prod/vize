import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { Window } from "happy-dom";
import { historicalSetupCode } from "./script-setup-default-expose.ts";
const window = new Window();
for (const key of [
  "window",
  "document",
  "Element",
  "HTMLElement",
  "SVGElement",
  "Node",
  "Text",
  "Comment",
])
  globalThis[key] = key === "window" ? window : window[key];
const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const vue = fromUi("vue");
const server = fromUi("vue/server-renderer");
assert.equal(process.env.NODE_ENV, "development");
assert.match(vue.version, /^3\.5\./);
globalThis.__inlineSlot = { vue, server };
const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
const corpus = new URL(
  "../../_fixtures/differential/compiler/dev-comment-hydration/",
  import.meta.url,
);
const manifest = JSON.parse(readFileSync(new URL("cases.json", corpus)));
assert.equal(
  createHash("sha256").update(input.source).digest("hex"),
  manifest.files.find((file) => file.path === input.fixture).sha256,
);
async function load(code) {
  const body = code.replace(
    /import \{([^}]*)\} from ["'](vue|@vue\/server-renderer|vue\/server-renderer)["'];?/g,
    (_, names, source) =>
      `const {${names.replace(/ as /g, ": ")}}=globalThis.__inlineSlot.${source === "vue" ? "vue" : "server"};`,
  );
  return (await import(`data:text/javascript;base64,${Buffer.from(body).toString("base64")}`))
    .default;
}
const Group = {
  render() {
    return vue.h("section", this.$slots.default());
  },
};
function app(component) {
  const app = vue.createSSRApp(component);
  app.component("Group", Group);
  return app;
}
const html = await server.renderToString(app(await load(input.ssr.code)));
const expected = manifest.inlineSlots.find((fixture) => fixture.fixture === input.fixture);
assert.equal(
  createHash("sha256").update(historicalSetupCode(input.ssr.code, input.source)).digest("hex"),
  expected.ssrCodeSha256,
);
assert.equal(html, expected.html);
const root = document.createElement("div");
root.innerHTML = html;
document.body.append(root);
const warnings = [];
const instance = app(await load(input.dom.code));
instance.config.warnHandler = (m) => warnings.push(m);
instance.mount(root);
await vue.nextTick();
assert.equal(root.textContent, "a  b Session");
assert.deepEqual(warnings, []);
process.stdout.write(JSON.stringify({ html, hydrated: root.innerHTML, warnings }));
