import { requiredEnv } from "../../../../tools/ci/lib/scripts/files.ts";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { spawnSync } from "node:child_process";
const root = requiredEnv("VIZE_DEFAULT_PROP_CAPTURE");
const cases = readdirSync(root, { withFileTypes: true })
  .filter(
    (e) => e.isDirectory() && !["editor-inputs", "imported-runtime-controls"].includes(e.name),
  )
  .map((e) => e.name)
  .sort();
assert.deepEqual(cases, [
  "imported_callable_default_remains_defined_in_all_template_prop_views",
  "undefined_callable_default_keeps_original_cli_conditions_optional",
  "undefined_callable_default_still_rejects_unguarded_template_calls",
]);
assert.deepEqual(JSON.parse(readFileSync(join(root, "editor-diagnostics.json"), "utf8")), []);
assert(existsSync(join(root, "editor-hover.json")));
const editorRuntime = JSON.parse(readFileSync(join(root, "editor-runtime.json"), "utf8"));
const hash = (file: string) => createHash("sha256").update(readFileSync(file)).digest("hex");
assert.equal(
  hash(join(root, "imported-runtime-props.fixture.rs")),
  "73fc6c23f5a91296bb9bce4ef133b5781f27d7b8c0d4f1a5f8d3b10de018aaa8",
);
assert.equal(readFileSync(join(root, "imported-runtime-props.status.txt"), "utf8"), "0\n");
const importedRuntime = JSON.parse(
  readFileSync(join(root, "imported-runtime-props.runtime.json"), "utf8"),
);
assert.equal(importedRuntime.sourceSha, process.env.SOURCE_SHA);
assert.equal(realpathSync(importedRuntime.nativeBinary), realpathSync(editorRuntime.nativeBinary));
const runtimeOutput = readFileSync(join(root, "imported-runtime-props.stdout.txt"), "utf8");
for (const name of [
  "imported_runtime_define_props_reach_script_and_template",
  "type_based_with_defaults_omit_imported_defaults",
  "imported_runtime_props_cache_rebinds_local_define_props_name",
])
  assert(
    runtimeOutput.includes(
      `test batch::type_checker::tests::recent_issues::imported_runtime_props::${name} ... ok`,
    ),
  );
assert(runtimeOutput.includes("test result: ok. 3 passed; 0 failed;"));
for (const name of [
  "imported-runtime-define-props",
  "type-with-defaults-imported-omit",
  "imported-runtime-props-cache-rebind",
])
  assert.deepEqual(
    JSON.parse(
      readFileSync(join(root, "imported-runtime-controls", name, "diagnostics.json"), "utf8"),
    ),
    [],
  );
const files = (dir: string): string[] =>
  readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
    e.isDirectory() ? files(join(dir, e.name)) : [join(dir, e.name)],
  );
const receipts = cases.map((name) => {
  const raw = JSON.parse(readFileSync(join(root, name, "oracle.json"), "utf8"));
  const nativeBinary = realpathSync(raw.nativeBinary);
  assert.equal(realpathSync(editorRuntime.nativeBinary), nativeBinary);
  let packageDir = dirname(nativeBinary);
  while (!existsSync(join(packageDir, "package.json")) && dirname(packageDir) !== packageDir)
    packageDir = dirname(packageDir);
  const packagePath = join(packageDir, "package.json");
  const pkg = JSON.parse(readFileSync(packagePath, "utf8"));
  assert.equal(pkg.name, `@typescript/typescript-${process.platform}-${process.arch}`);
  assert.equal(pkg.version, "7.0.2");
  const version = spawnSync(nativeBinary, ["--version"], { encoding: "utf8" });
  assert.equal(version.status, 0);
  assert.equal(version.stdout.trim(), "Version 7.0.2");
  assert.equal(version.stderr, "");
  return {
    name,
    sourceSha: process.env.SOURCE_SHA,
    cliBinary: realpathSync(raw.cliBinary),
    cliSha256: hash(raw.cliBinary),
    nativeBinary,
    nativeSha256: hash(nativeBinary),
    nativePackageSha256: hash(packagePath),
    nativeVersion: version.stdout.trim(),
  };
});
writeFileSync(
  join(root, "receipts.json"),
  JSON.stringify(
    {
      sourceSha: process.env.SOURCE_SHA,
      matchedCompleteExpectations: true,
      cases: receipts,
      fullInputsAndOutputs: files(root).map((path) => ({
        path: path.slice(root.length + 1),
        sha256: hash(path),
      })),
    },
    null,
    2,
  ) + "\n",
);
(await import("node:child_process")).execFileSync(
  "bash",
  ["tools/support/compat/github/native-lexical-dollar.sh"],
  { stdio: "inherit" },
);
