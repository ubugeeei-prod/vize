// Select original compiler and conformance fixtures using TypeScript's own
// syntactic diagnostics before exposing any input to the recovery provider.
import assert from "node:assert/strict";
import { isUtf8 } from "node:buffer";
import { closeSync, openSync, readFileSync, writeSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import type { TypeScriptApi } from "../../../../tests/_helpers/template-alias-typescript-11822.ts";

const ts: TypeScriptApi = createRequire(import.meta.url)("typescript");
const [, , root, manifest, destination] = process.argv;
const options = {
  target: ts.ScriptTarget.Latest,
  jsx: ts.JsxEmit.Preserve,
  module: ts.ModuleKind.ESNext,
  moduleDetection: ts.ModuleDetectionKind.Force,
  strict: true,
  noLib: true,
  noResolve: true,
};
const host = ts.createCompilerHost(options);
const out = openSync(destination, "w");
let parsed = 0;
let broken = 0;
let otherEncoding = 0;
try {
  for (const name of readFileSync(manifest, "utf8").split("\0")) {
    if (!/\.[mc]?tsx?$/u.test(name) || /\.d\.[mc]?ts$/u.test(name)) continue;
    const tsx = name.endsWith(".tsx");
    const filename = tsx ? "/original.tsx" : "/original.ts";
    const bytes = readFileSync(resolve(root, name));
    if (!isUtf8(bytes)) {
      otherEncoding++;
      continue;
    }
    const source = bytes.toString("utf8");
    const unit = ts.createSourceFile(filename, source, options.target, true);
    host.getSourceFile = (path) => (path === filename ? unit : undefined);
    parsed++;
    if (ts.createProgram([filename], options, host).getSyntacticDiagnostics(unit).length === 0)
      continue;
    writeSync(out, JSON.stringify({ name, source, tsx }) + "\n");
    broken++;
  }
} finally {
  closeSync(out);
}
assert.ok(broken > 0 && broken < parsed);
console.log(
  `TypeScript ${ts.version} original UTF-8 fixtures=${parsed} syntactically damaged=${broken} other encodings=${otherEncoding}`,
);
