import assert from "node:assert/strict";
import { createRequire, stripTypeScriptTypes } from "node:module";
import { pathToFileURL } from "node:url";
import { scopeId } from "./scoped-oracle.ts";

const require = createRequire(import.meta.url);
const compiler = require("@vue/compiler-sfc");
const sass = require("sass");

// Follow actual Sass anchors through the scoped selector insertion and Vue's
// optional removal of blank lines between top-level rule groups.
export function scopedSass(source: string, expanded: string, filename: string, trim: boolean) {
  const out = compiler.compileStyle({
    filename,
    source,
    id: scopeId,
    scoped: true,
    trim,
    preprocessLang: "scss",
    preprocessCustomRequire: (name: string) => (name === "sass" ? sass : require(name)),
  });
  assert.deepEqual(out.errors, []);
  const unscoped = expanded.split("\n");
  const scoped: string[] = out.code.split("\n");
  const lines: number[] = [];
  let next = 0;
  for (const original of unscoped) {
    if (trim && original === "" && scoped[next] !== "") {
      lines.push(-1);
      continue;
    }
    assert.equal(scoped[next]?.replaceAll(`[${scopeId}]`, ""), original, filename);
    lines.push(next++);
  }
  assert.equal(next, scoped.length);
  return { css: out.code as string, lines, scoped, unscoped };
}

let runtimeUrl: string | undefined;
const data = (source: string) =>
  `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;

async function load(code: string) {
  if (!runtimeUrl) {
    const runtime = require("vue");
    const exported = Object.keys(runtime).filter(
      (name) => /^[a-zA-Z_$][a-zA-Z_$0-9]*$/.test(name) && name !== "default",
    );
    runtimeUrl = data(
      `import runtime from ${JSON.stringify(pathToFileURL(require.resolve("vue")).href)};\n` +
        exported.map((name) => `export const ${name} = runtime.${name};`).join("\n"),
    );
  }
  return import(data(code.replaceAll(/from\s+["']vue["']/g, `from ${JSON.stringify(runtimeUrl)}`)));
}

type RenderDescriptor = {
  template: { content: string };
  script?: { lang?: string };
  scriptSetup?: { lang?: string };
};

export async function compareRender(descriptor: RenderDescriptor, code: string, filename: string) {
  const runtime = require("vue");
  assert.equal(runtime.version, "3.5.35");
  const { renderToString } = createRequire(require.resolve("vue/package.json"))(
    "@vue/server-renderer",
  );
  const script =
    descriptor.script || descriptor.scriptSetup
      ? compiler.compileScript(descriptor, { id: scopeId })
      : undefined;
  const template = compiler.compileTemplate({
    source: descriptor.template.content,
    filename,
    id: scopeId,
    scoped: true,
    compilerOptions: { bindingMetadata: script?.bindings },
  });
  assert.deepEqual(template.errors, []);
  const native = (await load(code)).default;
  assert.equal(native.__scopeId, scopeId);
  const setup = script
    ? (await load(script.lang === "ts" ? stripTypeScriptTypes(script.content) : script.content))
        .default
    : {};
  const reference = { ...setup, __scopeId: scopeId, render: (await load(template.code)).render };
  assert.equal(
    await renderToString(runtime.createSSRApp(native)),
    await renderToString(runtime.createSSRApp(reference)),
    filename,
  );
}
