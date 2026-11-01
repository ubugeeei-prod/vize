// Finite actual ImportDeclaration address adapters; source bytes otherwise remain exact.
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import {
  dataUrl,
  fromModelVue,
  hash,
  type openModelDom,
} from "./native-original-input-model-sfc-dom-environment.ts";
import {
  fromVaporVue,
  object,
  type ObjectValue,
} from "./native-original-input-model-vapor-boundary.ts";
type Dom = Awaited<ReturnType<typeof openModelDom>>;
type ImportNode = {
  type: string;
  source?: { type: string; value: string; start: number; end: number };
  specifiers?: { type: string; imported?: { type: string; name?: string } }[];
};

export function checkboxModule(code: string, runtime: Dom["runtime"], identity: string) {
  const compiler = createRequire(fromModelVue.resolve("@vue/compiler-sfc/package.json"));
  const provider: unknown = compiler("@babel/parser");
  assert(provider && typeof provider === "object");
  assert.equal(typeof Reflect.get(provider, "parse"), "function");
  const parser = provider as {
    parse(source: string, options: { sourceType: "module" }): { program: { body: ImportNode[] } };
  };
  const imports = parser
    .parse(code, { sourceType: "module" })
    .program.body.filter((node) => node.type === "ImportDeclaration");
  assert(imports.length > 0);
  const names = new Set<string>();
  for (const node of imports) {
    assert.equal(node.source?.value, "vue");
    for (const specifier of node.specifiers ?? []) {
      assert.equal(specifier.type, "ImportSpecifier");
      assert.equal(specifier.imported?.type, "Identifier");
      const name = specifier.imported?.name;
      assert(name && /^[A-Za-z_$][\w$]*$/.test(name));
      assert(
        [
          "vModelCheckbox",
          "withDirectives",
          "openBlock",
          "createElementBlock",
          "createElementVNode",
          "defineComponent",
          "unref",
          "isRef",
        ].includes(name),
        name,
      );
      assert(Object.hasOwn(runtime, name), name);
      const value: unknown = Reflect.get(runtime, name);
      assert(
        name === "vModelCheckbox"
          ? value && typeof value === "object"
          : typeof value === "function",
      );
      names.add(name);
    }
  }
  assert(names.has("vModelCheckbox"));
  const runtimePath = fromModelVue.resolve("vue");
  const packagePath = fromModelVue.resolve("vue/package.json");
  const metadata: unknown = JSON.parse(fs.readFileSync(packagePath, "utf8"));
  assert(metadata && typeof metadata === "object" && Reflect.get(metadata, "version") === "3.5.35");
  const runtimeDomPath = fromModelVue.resolve("@vue/runtime-dom");
  const observerSource =
    `import runtime from ${JSON.stringify(pathToFileURL(runtimePath).href)};\n` +
    [...names].map((name) => `export const ${name}=runtime.${name};`).join("\n");
  const observerUrl = dataUrl(observerSource) + "#" + encodeURIComponent(identity);
  const edits = imports.map((node) => {
    assert(node.source?.type === "StringLiteral");
    const { start, end } = node.source;
    assert(
      Number.isSafeInteger(start) &&
        Number.isSafeInteger(end) &&
        start >= 0 &&
        start < end &&
        end <= code.length,
    );
    return { start, end, replacement: JSON.stringify(observerUrl) };
  });
  let adapted = code,
    next = code.length;
  for (const edit of edits.toSorted((a, b) => b.start - a.start)) {
    assert(edit.end <= next);
    adapted = adapted.slice(0, edit.start) + edit.replacement + adapted.slice(edit.end);
    next = edit.start;
  }
  return {
    original: code,
    adapted,
    edits,
    observerSource,
    observerUrl,
    moduleUrl: dataUrl(adapted) + "#" + encodeURIComponent(identity),
    runtimePath,
    packagePath,
    metadata,
    runtimeDomPath,
    runtimeDomSha256: hash(fs.readFileSync(runtimeDomPath, "utf8")),
    runtimeSha256: hash(fs.readFileSync(runtimePath, "utf8")),
    originalSha256: hash(code),
  };
}

export function checkboxVaporModule(
  code: string,
  runtimeUrl: string,
  hookKey: string,
  actual: ObjectValue,
) {
  const fromCompiler = createRequire(fromVaporVue.resolve("@vue/compiler-sfc/package.json"));
  const provider: unknown = fromCompiler("@babel/parser");
  object(provider);
  assert.equal(typeof provider.parse, "function");
  const parser = provider as {
    parse(source: string, options: { sourceType: "module" }): { program: { body: ImportNode[] } };
  };
  const imports = parser
    .parse(code, { sourceType: "module" })
    .program.body.filter((node) => node.type === "ImportDeclaration");
  assert(imports.length > 0);
  const names = new Set<string>();
  for (const node of imports) {
    assert.equal(node.source?.value, "vue");
    for (const specifier of node.specifiers ?? []) {
      assert.equal(specifier.type, "ImportSpecifier");
      assert.equal(specifier.imported?.type, "Identifier");
      const name = specifier.imported?.name;
      assert(name && /^[A-Za-z_$][\w$]*$/.test(name));
      assert(
        [
          "applyCheckboxModel",
          "unref",
          "isRef",
          "template",
          "defineComponent",
          "defineVaporComponent",
        ].includes(name),
        name,
      );
      assert(Object.hasOwn(actual, name), name);
      assert.equal(typeof actual[name], "function", name);
      names.add(name);
    }
  }
  assert(names.has("applyCheckboxModel"));
  const observerSource =
    `import * as actual from ${JSON.stringify(runtimeUrl)};\n` +
    [...names]
      .filter((name) => name !== "applyCheckboxModel")
      .map((name) => `export {${name}} from ${JSON.stringify(runtimeUrl)};`)
      .join("\n") +
    `\nconst hook=globalThis[Symbol.for(${JSON.stringify(hookKey)})];\n` +
    `export function applyCheckboxModel(...args){const id=hook.before(args,actual.currentInstance,actual.getCurrentInstance());` +
    `let returned;try{returned=Reflect.apply(actual.applyCheckboxModel,undefined,args);}` +
    `catch(error){try{hook.failed(id,error);}finally{throw error;}}` +
    `hook.after(id,returned);return returned;}\n`;
  const observerUrl = dataUrl(observerSource);
  const edits = imports.map((node) => {
    assert(node.source?.type === "StringLiteral");
    const { start, end } = node.source;
    assert(
      Number.isSafeInteger(start) &&
        Number.isSafeInteger(end) &&
        start >= 0 &&
        start < end &&
        end <= code.length,
    );
    return { start, end, replacement: JSON.stringify(observerUrl) };
  });
  let adapted = code,
    next = code.length;
  for (const edit of edits.toSorted((a, b) => b.start - a.start)) {
    assert(edit.end <= next);
    adapted = adapted.slice(0, edit.start) + edit.replacement + adapted.slice(edit.end);
    next = edit.start;
  }
  return {
    original: code,
    adapted,
    edits,
    observerSource,
    observerUrl,
    originalSha256: hash(code),
    moduleUrl: dataUrl(adapted) + "#" + encodeURIComponent(hookKey),
  };
}
