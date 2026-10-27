import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { appTemplate, counterTemplate, referenceModule } from "./fixtures.ts";
import { hash, json } from "./environment.ts";
import type { RuntimeIdentity, GeneratedFile, TemplateCompiler } from "./types.ts";

export function prepareReference(runtime: RuntimeIdentity, directory: string) {
  const generated: GeneratedFile[] = [];
  for (const phase of ["client", "server"] as const) {
    if (runtime.target === "vapor") {
      const identity = phase === "server" ? runtime.compilerSsr : runtime.compilerVapor;
      const requireCompiler = createRequire(path.join(identity!.root, "package.json"));
      const compiler: TemplateCompiler = requireCompiler(identity!.entry);
      for (const [name, source] of [
        ["counter", counterTemplate],
        ["app", appTemplate],
      ]) {
        const errors: string[] = [],
          warnings: string[] = [];
        const result = compiler.compile(source, {
          mode: "module",
          prefixIdentifiers: true,
          vapor: true,
          comments: true,
          onError: (error) => errors.push(String(error)),
          onWarn: (warning) => warnings.push(String(warning)),
        });
        const packet = { name, phase, source, compiler: identity, result, errors, warnings };
        json(path.join(directory, `reference-${name}-${phase}.json`), packet);
        assert.deepEqual(errors, [], "independent real Vue reference compiles");
        assert.deepEqual(warnings, []);
        assert.equal(typeof result.code, "string");
        const file = path.join(directory, `reference-${name}-${phase}.mjs`);
        fs.writeFileSync(file, result.code);
        generated.push({ file, sha256: hash(fs.readFileSync(file)) });
      }
    }
    const file = path.join(directory, `reference-${phase}.mjs`);
    fs.writeFileSync(file, referenceModule(runtime.target, phase));
    generated.push({ file, sha256: hash(fs.readFileSync(file)) });
  }
  return generated;
}
