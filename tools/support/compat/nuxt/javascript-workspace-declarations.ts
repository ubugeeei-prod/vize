// A real JS Vue package publishes the declaration emitted from its unchanged SFC.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { readJson } from "../../json-value.ts";
import type { Command } from "./javascript-workspace-cli.ts";
import { files, save, sha } from "./javascript-workspace-project.ts";
import type { WorkspaceContext } from "./javascript-workspace-project.ts";

export function packageDeclarations(root: string, context: WorkspaceContext, command: Command) {
  const require = createRequire(path.join(root, "tests/package.json"));
  const manifestPath = require.resolve("vue-tsc/package.json");
  const compiler = path.join(path.dirname(manifestPath), "bin/vue-tsc.js");
  const result = command(
    process.execPath,
    [compiler, "--pretty", "false", "-p", "workspace-declarations.tsconfig.json"],
    context.project,
    context.artifacts,
    "stock-package-declarations",
  );
  const generated = path.join(context.project, "generated-types");
  const outputs = fs.existsSync(generated) ? files(generated) : {};
  save(context.artifacts, "package-declarations.json", {
    compiler: fs.realpathSync(compiler),
    manifestPath: fs.realpathSync(manifestPath),
    manifest: readJson(manifestPath),
    configuration: fs.readFileSync(
      path.join(context.project, "workspace-declarations.tsconfig.json"),
      "utf8",
    ),
    outputs,
    result,
  });
  assert.deepEqual(result, { status: 0, stdout: "", stderr: "" });
  assert.deepEqual(Object.keys(outputs), [
    "pricing/src/index.d.ts",
    "pricing/src/label.d.mts",
    "ui/src/BadgeCard.vue.d.ts",
  ]);
  const source = path.join(generated, "ui/src/BadgeCard.vue.d.ts");
  const target = path.join(context.project, "packages/ui/dist/BadgeCard.vue.d.ts");
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.copyFileSync(source, target);
  assert.deepEqual(fs.readFileSync(target), fs.readFileSync(source));
  save(context.artifacts, "published-package-interface.json", {
    source,
    target,
    code: fs.readFileSync(target, "utf8"),
    sha256: sha(fs.readFileSync(target)),
  });
}
