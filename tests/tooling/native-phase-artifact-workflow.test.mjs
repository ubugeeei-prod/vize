import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";
import { rawPaths } from "../../tools/support/compat/github/native-phase-artifact-files.mjs";

const workflow = parse(
  readFileSync(
    new URL("../../.github/workflows/typechecker-native-phases.yml", import.meta.url),
    "utf8",
  ),
);
const job = workflow.jobs["native-phases"];

void test("packing retains the complete original raw eligibility and failed-pack fallback", () => {
  assert.deepEqual(rawPaths, [
    "outside-import-types/",
    "typechecker-native-source-custody.json",
    "native-source-recipe*",
    "reference-path-module/",
    "typechecker-native-phases.json",
    "typechecker-native-phases.json.samples/provenance.json",
    "template-dollar-emit/",
    "config-scoped-vue-helpers/",
    "tsconfig-types-extends/",
    "jsx-slot-parameter-types/",
    "canon-selected-alias/",
    "lsp-passive-3952/",
    "unchecked-index-access/",
    "unknown-template-options/",
    "template-emit-ts40-projection/",
    "typechecker-native-phases.json.samples/failure.json",
    "typechecker-native-phases.json.samples/inputs/",
    "typechecker-native-phases.json.samples/raw/",
    "typechecker-native-phases.json.samples/native/",
    "typechecker-native-phases.json.samples/projections/",
    "typechecker-native-phases.json.samples/virtual-ts/",
    "undefined-default-props/",
  ]);
  const index = job.steps.findIndex((step) => step.id === "native-artifact-pack");
  const pack = job.steps[index];
  const upload = job.steps[index + 1];
  assert.equal(index, job.steps.length - 2, "all original qualification steps precede packing");
  assert.equal(pack.if, "always()");
  assert.match(
    pack.run,
    /native-phase-artifact\.mjs" pack "\$RUNNER_TEMP" "\$RUNNER_TEMP\/native-phase-artifact"/,
  );
  assert.equal(upload.if, "always()");
  assert.equal(upload.uses, "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a");
  assert.equal(upload.with.name, "typechecker-native-phases");
  assert.equal(upload.with["include-hidden-files"], true);
  assert.equal(upload.with["retention-days"], 30);
  assert.equal(upload.with["if-no-files-found"], "error");
  assert.equal(
    upload.with.path,
    "${{ steps.native-artifact-pack.outcome == 'success' && format('{0}/native-phase-artifact', runner.temp) || steps.native-artifact-pack.outputs.raw_paths }}",
  );
  assert.equal(
    upload.with["compression-level"],
    "${{ steps.native-artifact-pack.outcome == 'success' && '0' || '6' }}",
  );
});

void test("a real failed pack exports all original paths without disguising the failure", (t) => {
  const root = mkdtempSync(join(tmpdir(), "vize-native-empty-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const output = join(root, "github-output");
  const script = fileURLToPath(
    new URL("../../tools/support/compat/github/native-phase-artifact.mjs", import.meta.url),
  );
  const result = spawnSync(process.execPath, [script, "pack", root, join(root, "envelope")], {
    encoding: "utf8",
    env: {
      ...process.env,
      SOURCE_SHA: "1".repeat(40),
      DRIVER_SHA: "2".repeat(40),
      GITHUB_OUTPUT: output,
    },
  });
  assert.notEqual(result.status, 0, "failed reconstruction/packing remains a failed check");
  const emitted = readFileSync(output, "utf8");
  assert.equal(
    emitted,
    `raw_paths<<NATIVE_RAW_PATHS\n${rawPaths.map((path) => `${root}/${path}`).join("\n")}\nNATIVE_RAW_PATHS\n`,
  );
});
