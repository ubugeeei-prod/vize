import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { spawnSync } from "node:child_process";
const root = process.env.VIZE_TEMPLATE_EMIT_CAPTURE;
const cases = readdirSync(root, { withFileTypes: true })
  .filter((e) => e.isDirectory() && !["editor-inputs", "editor-oracle"].includes(e.name))
  .map((e) => e.name)
  .sort();
assert.deepEqual(cases, [
  "absent_macro_retains_the_public_instance_emit_contract",
  "controls__declared_emit_and_model_update_keep_both_typed_event_contracts",
  "controls__generic_setup_emit_preserves_its_local_type_parameter",
  "controls__optional_model_update_preserves_undefined_and_declared_event_payloads",
  "controls__strict_generated_template_context_uses_the_same_declared_emit_vector",
  "original_unassigned_macro_rejects_complete_event_and_payload_vector",
  "renamed_macro_result_and_local_call_signature_keep_payload_types",
  "runtime_array_macro_keeps_event_names_without_inventing_payload_constraints",
  "shadowed_macro_retains_the_public_instance_emit_contract",
  "unassigned_macro_accepts_declared_events_and_numeric_payloads",
]);
assert.equal(JSON.parse(readFileSync(join(root, "editor-diagnostics.json"), "utf8")).length, 2);
assert(existsSync(join(root, "editor-hover.json")));
assert(existsSync(join(root, "editor-payload-hover.json")));
assert(existsSync(join(root, "editor-oracle", "hover.json")));
assert(existsSync(join(root, "editor-oracle", "valid-hover.json")));
assert(existsSync(join(root, "editor-oracle", "payload-hover.json")));
assert.equal(
  JSON.parse(readFileSync(join(root, "editor-oracle", "diagnostics.json"), "utf8")).length,
  2,
);
const editorRuntime = JSON.parse(readFileSync(join(root, "editor-runtime.json"), "utf8"));
const oracleRuntime = JSON.parse(readFileSync(join(root, "editor-oracle", "runtime.json"), "utf8"));
const hash = (file) => createHash("sha256").update(readFileSync(file)).digest("hex");
const files = (dir) =>
  readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
    e.isDirectory() ? files(join(dir, e.name)) : [join(dir, e.name)],
  );
const receipts = cases.map((name) => {
  const raw = JSON.parse(readFileSync(join(root, name, "oracle.json"), "utf8"));
  const nativeBinary = realpathSync(raw.nativeBinary);
  assert.equal(realpathSync(editorRuntime.nativeBinary), nativeBinary);
  assert.equal(realpathSync(oracleRuntime.nativeBinary), nativeBinary);
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
