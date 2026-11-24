import { requiredEnv } from "../shared/files.ts";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { spawnSync } from "node:child_process";
const root = requiredEnv("VIZE_TYPECHECK_REGRESSION_CAPTURE");
const cases = readdirSync(root, { withFileTypes: true })
  .filter(
    (e) => e.isDirectory() && !["editor-original-script", "editor-monorepo-alias"].includes(e.name),
  )
  .map((e) => e.name)
  .sort();
assert.deepEqual(cases, [
  "outside_import_keeps_direct_scoped_type_packages_at_the_config_anchor",
  "outside_import_preserves_original_type_libraries_and_real_cli_diagnostics",
  "type_config_anchor_does_not_expose_app_modules_to_sibling_sources",
]);
const sha256 = (file: string) => createHash("sha256").update(readFileSync(file)).digest("hex");
const files = (dir: string): string[] =>
  readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
    e.isDirectory() ? files(join(dir, e.name)) : [join(dir, e.name)],
  );
const receipts = cases.map((name) => {
  const dir = join(root, name);
  const oracle = JSON.parse(readFileSync(join(dir, "oracle.json"), "utf8"));
  const cli = JSON.parse(readFileSync(join(dir, "cli.json"), "utf8"));
  const nativeBinary = realpathSync(oracle.nativeBinary);
  let packageDir = dirname(nativeBinary);
  while (!existsSync(join(packageDir, "package.json")) && dirname(packageDir) !== packageDir)
    packageDir = dirname(packageDir);
  const packagePath = join(packageDir, "package.json");
  const nativePackage = JSON.parse(readFileSync(packagePath, "utf8"));
  assert.equal(nativePackage.name, `@typescript/typescript-${process.platform}-${process.arch}`);
  assert.equal(nativePackage.version, "7.0.2");
  const version = spawnSync(nativeBinary, ["--version"], { encoding: "utf8" });
  assert.equal(version.status, 0);
  assert.equal(version.stdout.trim(), "Version 7.0.2");
  assert.equal(version.stderr, "");
  return {
    name,
    sourceSha: process.env.SOURCE_SHA,
    cliBinary: realpathSync(cli.cliBinary),
    cliSha256: sha256(cli.cliBinary),
    nativeBinary,
    nativeSha256: sha256(nativeBinary),
    nativePackageSha256: sha256(packagePath),
    nativePackage: { name: nativePackage.name, version: nativePackage.version },
    nativeVersion: version.stdout.trim(),
    completeOutputsAndInputs: files(dir).map((path) => ({
      path: path.slice(root.length + 1),
      sha256: sha256(path),
    })),
  };
});
const processReceipt = (
  await import(join(requiredEnv("RUNNER_TEMP"), "typechecker-native-project-retirement.mjs"))
).qualifySourceBoundNativeProjectRetirement({
  capture: root,
  recipePath: join(requiredEnv("RUNNER_TEMP"), "native-source-recipe.json"),
  sourceSha: process.env.SOURCE_SHA,
  nativeBinary: receipts[0].nativeBinary,
});
const editor = JSON.parse(readFileSync(join(root, "editor-original-script/runtime.json"), "utf8"));
const editorFiles = ["editor-original-script", "editor-monorepo-alias"]
  .flatMap((name) => files(join(root, name)))
  .map((path) => ({ path: path.slice(root.length + 1), sha256: sha256(path) }));
writeFileSync(
  join(root, "receipts.json"),
  JSON.stringify(
    {
      sourceSha: process.env.SOURCE_SHA,
      fixtureSha256: sha256(
        "tests/_fixtures/differential/typechecker/outside-import-types/input.json",
      ),
      matchedCompleteExpectations: true,
      nativeProjectRetirement: processReceipt,
      cases: receipts,
      editor: {
        nativeBinary: realpathSync(editor.nativeBinary),
        nativeSha256: sha256(editor.nativeBinary),
        completeOutputsAndInputs: editorFiles,
      },
    },
    null,
    2,
  ) + "\n",
);
