import assert from "node:assert/strict";
import fs from "node:fs";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

const fromUi = createRequire(
  process.env.VIZE_TEST_VUE_PACKAGE ?? new URL("../../../npm/ui/package.json", import.meta.url),
);
export const fromVue = createRequire(fromUi.resolve("vue/package.json"));
export const compiler = fromVue("@vue/compiler-sfc");
export const core = fromVue("vue");
export const renderer = fromVue("@vue/server-renderer");
const { parse } = fromVue("@babel/parser");
export const dataUrl = (source: string) =>
  `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;
export const hash = (source: string) => createHash("sha256").update(source).digest("hex");
export function captureHashes(actual: any) {
  return Object.fromEntries(
    ["source", "code", "css", "mapText", "cssMapText"].map((name) => {
      assert.equal(typeof actual[name], "string");
      return [`${name}Sha256`, hash(actual[name])];
    }),
  );
}

export function checkVersions() {
  for (const name of ["vue", "@vue/compiler-sfc", "@vue/compiler-ssr", "@vue/server-renderer"])
    assert.equal(fromVue(`${name}/package.json`).version, "3.5.35");
}
// Only genuine top-level import sources are relinked in this dev execution.
export function relink(code: string, imports: Record<string, string>) {
  let output = "";
  let cursor = 0;
  for (const node of parse(code, { sourceType: "module" }).program.body) {
    if (node.type !== "ImportDeclaration") continue;
    assert(Object.hasOwn(imports, node.source.value), node.source.value);
    output += code.slice(cursor, node.source.start) + JSON.stringify(imports[node.source.value]);
    cursor = node.source.end;
  }
  return output + code.slice(cursor);
}
function runtimeUrl(name: string) {
  const names = Object.keys(fromVue(name)).filter(
    (key) => /^[A-Za-z_$][\w$]*$/.test(key) && key !== "default",
  );
  return dataUrl(
    `import runtime from ${JSON.stringify(pathToFileURL(fromVue.resolve(name)).href)};\n` +
      names.map((key) => `export const ${key} = runtime.${key};`).join("\n"),
  );
}
const imports = {
  vue: runtimeUrl("vue"),
  "@vue/server-renderer": runtimeUrl("@vue/server-renderer"),
  "vue/server-renderer": runtimeUrl("@vue/server-renderer"),
};
export async function loadModule(code: string) {
  return import(dataUrl(relink(code, imports)));
}
export function browserRuntime() {
  return dataUrl(fs.readFileSync(fromVue.resolve("vue/dist/vue.esm-browser.prod.js"), "utf8"));
}
export function stock(fixture: any) {
  const parsed = compiler.parse(fixture.source, {
    filename: fixture.filename,
    sourceMap: true,
    ignoreEmpty: false,
  });
  assert.deepEqual(parsed.errors, []);
  const descriptor = parsed.descriptor;
  assert.equal(descriptor.script, null);
  assert.equal(descriptor.scriptSetup, null);
  assert.deepEqual(descriptor.cssVars, []);
  assert.equal(descriptor.styles.length, 1);
  assert.equal(descriptor.styles[0].scoped, true);
  const common = {
    source: descriptor.template.content,
    filename: fixture.filename,
    id: fixture.scopeId,
    scoped: true,
    inMap: descriptor.template.map,
  };
  const ssr = compiler.compileTemplate({ ...common, ssr: true, ssrCssVars: descriptor.cssVars });
  const client = compiler.compileTemplate(common);
  const metadataOnly = compiler.compileTemplate({
    ...common,
    scoped: false,
    ssr: true,
    ssrCssVars: descriptor.cssVars,
  });
  const css = compiler.compileStyle({
    source: descriptor.styles[0].content,
    filename: fixture.filename,
    id: fixture.scopeId,
    scoped: true,
    trim: false,
    map: descriptor.styles[0].map,
  });
  for (const result of [ssr, client, metadataOnly, css]) assert.deepEqual(result.errors, []);
  const loc = descriptor.styles[0].loc;
  const cssSpan = {
    start: Buffer.byteLength(fixture.source.slice(0, loc.start.offset)),
    end: Buffer.byteLength(fixture.source.slice(0, loc.end.offset)),
  };
  return { ssr, client, metadataOnly, css, originalCss: descriptor.styles[0].content, cssSpan };
}
export async function render(component: any, props: any) {
  const app = core.createSSRApp(component, props);
  const warnings: string[] = [];
  app.config.warnHandler = (message: string) => warnings.push(message);
  const html = await renderer.renderToString(app);
  assert.deepEqual(warnings, []);
  const forbidden = new Proxy(Object.create(null), {
    get(_, name) {
      throw Error(`static SSR read ${String(name)}`);
    },
  });
  const chunks: string[] = [];
  component.ssrRender(forbidden, (chunk: string) => chunks.push(chunk), forbidden, props);
  assert.equal(chunks.join(""), html);
  return html;
}
