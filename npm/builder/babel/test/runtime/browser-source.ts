import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import ts from "typescript";
import { hash, json } from "./environment.ts";

/** Browser routes keep executable JavaScript; retain both exact source and served bytes. */
export function prepareBrowserSource(file: string, directory: string, name: string): string {
  const source = fs.readFileSync(file, "utf8");
  const result = ts.transpileModule(source, {
    fileName: file,
    reportDiagnostics: true,
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.ESNext,
      verbatimModuleSyntax: true,
      removeComments: false,
    },
  });
  assert.deepEqual(
    result.diagnostics ?? [],
    [],
    "the browser driver must transpile without diagnostics",
  );
  fs.writeFileSync(path.join(directory, name + ".original.ts"), source);
  fs.writeFileSync(path.join(directory, name + ".served.js"), result.outputText);
  json(path.join(directory, name + ".source.json"), {
    scope: "test driver type erasure only; no compiler, native, or runtime qualification credit",
    typescript: ts.version,
    sourceSha256: hash(Buffer.from(source)),
    servedSha256: hash(Buffer.from(result.outputText)),
  });
  return result.outputText;
}
