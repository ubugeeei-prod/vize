import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { hash } from "./authority.ts";
import type { PinnedTypeScriptApi } from "./typescript-api.ts";

/** This inspects the installed real entry; no fixture module augmentation is admitted. */
export function inspectExperimentalExports(project: string, providerRoot: string) {
  const require = createRequire(path.join(project, "package.json"));
  const ts = require("typescript") as PinnedTypeScriptApi;
  assert.equal(ts.version, "5.9.3");
  const entry = path.join(providerRoot, "dist/experimental/index.d.ts");
  const program = ts.createProgram([entry], {
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    target: ts.ScriptTarget.ESNext,
    noEmit: true,
    skipLibCheck: true,
    types: [],
  });
  const checker = program.getTypeChecker();
  const file = program.getSourceFile(entry);
  assert.ok(file && file.isDeclarationFile, "actual experimental declaration entry");
  const module = checker.getSymbolAtLocation(file);
  assert.ok(module, "actual entry has a module symbol");
  const exports = checker
    .getExportsOfModule(module)
    .map((symbol) => {
      const target =
        symbol.flags & ts.SymbolFlags.Alias ? checker.getAliasedSymbol(symbol) : symbol;
      return {
        name: symbol.name,
        flags: symbol.flags,
        targetName: target.name,
        targetFlags: target.flags,
        targetIsTypeAlias: Boolean(target.flags & ts.SymbolFlags.TypeAlias),
        declarations: (target.declarations ?? []).map((declaration) => {
          const source = declaration.getSourceFile();
          return {
            file: path.relative(providerRoot, source.fileName).replaceAll("\\", "/"),
            sha256: hash(fs.readFileSync(source.fileName)),
            start: declaration.getStart(),
            end: declaration.end,
          };
        }),
      };
    })
    .sort((left, right) => left.name.localeCompare(right.name, "en"));
  return {
    typescript: ts.version,
    entry: { path: "dist/experimental/index.d.ts", sha256: hash(fs.readFileSync(entry)) },
    exports,
  };
}
