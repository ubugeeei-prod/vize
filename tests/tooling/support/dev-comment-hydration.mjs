// #8652: complete native development SFC modules, SSR HTML and hydrated photo/text pairs.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { historicalSetupCode } from "./script-setup-default-expose.ts";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { Window } from "happy-dom";

assert.equal(process.env.NODE_ENV, "development");
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
assert.match(vue.version, /^3\.5\./, "use the declared stable Vue runtime");
globalThis.__devCommentRuntime = { vue, server };
const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const cases = JSON.parse(Buffer.concat(chunks).toString("utf8"));
const corpus = new URL(
  "../../_fixtures/differential/compiler/dev-comment-hydration/",
  import.meta.url,
);
const manifest = JSON.parse(readFileSync(new URL("cases.json", corpus)));
for (const file of manifest.files)
  assert.equal(
    createHash("sha256")
      .update(readFileSync(new URL(file.path, corpus)))
      .digest("hex"),
    file.sha256,
    file.path,
  );
const expectedSsr = JSON.parse(readFileSync(new URL("ssr.expected.json", corpus)));
assert.equal(cases.length, expectedSsr.length);
const expectedPairs = [
  ["/session.png", "Session"],
  ["/lightning.png", "Lightning Talk"],
];
const pairs = (root) =>
  [...root.querySelectorAll("article")].map((article) => [
    article.querySelector("img").getAttribute("src"),
    article.querySelector("p").textContent,
  ]);
async function load(code) {
  const body = code.replace(
    /import \{([^}]*)\} from ["'](vue|@vue\/server-renderer|vue\/server-renderer)["'];?/g,
    (_, names, source) =>
      `const {${names.replace(/ as /g, ": ")}} = globalThis.__devCommentRuntime.${source === "vue" ? "vue" : "server"};`,
  );
  return (await import(`data:text/javascript;base64,${Buffer.from(body).toString("base64")}`))
    .default;
}
const observations = [];
const Group = {
  render() {
    return vue.h("section", this.$slots.default());
  },
};
function app(component) {
  const instance = vue.createSSRApp(component);
  instance.component("Group", Group);
  return instance;
}
for (const [index, fixture] of cases.entries()) {
  const dom = await load(fixture.dom.code);
  const ssr = await load(fixture.ssr.code);
  const html = await server.renderToString(app(ssr));
  assert.equal(fixture.comment, expectedSsr[index].comment);
  assert.equal(
    createHash("sha256")
      .update(historicalSetupCode(fixture.ssr.code, fixture.source))
      .digest("hex"),
    expectedSsr[index].ssrCodeSha256,
    "existing SSR module bytes",
  );
  assert.equal(html, expectedSsr[index].html, "existing SSR HTML bytes");
  const root = document.createElement("div");
  root.innerHTML = html;
  document.body.appendChild(root);
  assert.deepEqual(pairs(root), expectedPairs, `${fixture.comment}: original SSR pairs`);
  const annotation = /^(eslint|oxlint)-(disable(?:-line|-next-line)?|enable)\s/.test(
    fixture.comment,
  );
  assert.equal(html.includes(`<!-- ${fixture.comment} -->`), !annotation);
  const warnings = [];
  const hydratedApp = app(dom);
  hydratedApp.config.warnHandler = (message) => warnings.push(message);
  hydratedApp.mount(root);
  await vue.nextTick();
  assert.deepEqual(pairs(root), expectedPairs, `${fixture.comment}: hydrated pairs`);
  assert.deepEqual(warnings, [], `${fixture.comment}: hydration warnings`);
  assert.equal(root.innerHTML.includes(`<!-- ${fixture.comment} -->`), !annotation);
  observations.push({
    comment: fixture.comment,
    html,
    hydrated: root.innerHTML,
    pairs: pairs(root),
    warnings,
  });
  hydratedApp.unmount();
  root.remove();
}
process.stdout.write(JSON.stringify(observations));
