// Finite import-address adapter; original helper functions/arguments are forwarded.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import {
  dataUrl,
  fromVaporVue,
  hash,
  type ObjectValue,
} from "./native-original-input-model-vapor-boundary.ts";
type ImportNode = {
  type: string;
  source?: { type: string; value: string; start: number; end: number };
};
export function trimVaporModule(
  code: string,
  runtimeUrl: string,
  hookKey: string,
  attempt: ObjectValue,
) {
  const observerSource =
    `import * as actual from ${JSON.stringify(runtimeUrl)};\n` +
    `export * from ${JSON.stringify(runtimeUrl)};\n` +
    `const hook=globalThis[Symbol.for(${JSON.stringify(hookKey)})];\n` +
    `export function applyTextModel(...args){const id=hook.before(args,actual.currentInstance,actual.getCurrentInstance());` +
    `try{const returned=Reflect.apply(actual.applyTextModel,undefined,args);hook.after(id,returned);return returned;}` +
    `catch(error){hook.failed(id,error);throw error;}}\n`;
  const observerUrl = dataUrl(observerSource);
  const fromCompiler = createRequire(fromVaporVue.resolve("@vue/compiler-sfc/package.json"));
  const parser = fromCompiler("@babel/parser") as {
    parse(source: string, options: { sourceType: "module" }): { program: { body: ImportNode[] } };
  };
  const imports = parser
    .parse(code, { sourceType: "module" })
    .program.body.filter((node) => node.type === "ImportDeclaration");
  const edits = imports.map((node) => {
    assert(node.source && node.source.type === "StringLiteral");
    assert.equal(node.source.value, "vue");
    assert(Number.isInteger(node.source.start) && Number.isInteger(node.source.end));
    return {
      start: node.source.start,
      end: node.source.end,
      replacement: JSON.stringify(observerUrl),
    };
  });
  assert(edits.length > 0);
  let adapted = code;
  for (const edit of edits.toSorted((left, right) => right.start - left.start)) {
    assert(edit.start >= 0 && edit.start < edit.end && edit.end <= code.length);
    adapted = adapted.slice(0, edit.start) + edit.replacement + adapted.slice(edit.end);
  }
  attempt.moduleAddressAdapter = {
    original: code,
    adapted,
    edits,
    observerSource,
    originalSha256: hash(code),
  };
  return adapted;
}
