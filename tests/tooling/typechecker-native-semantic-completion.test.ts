import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { fileURLToPath } from "node:url";
import { sha256 } from "../differential/harness.mjs";
import { loadTypecheckerManifest, TYPECHECKER_TESTS } from "../differential/typechecker.ts";
import { loadNativeSemanticCompletionCases } from "../differential/typechecker-native-semantic-completion.ts";

const root = fileURLToPath(new URL("../_fixtures/differential/typechecker/", import.meta.url));
const originalManifest = JSON.parse(fs.readFileSync(path.join(root, "manifest.json"), "utf8"));
const original = originalManifest.supplementalCases;
const packName = "native-semantic-completion";

function copyFixture(t: TestContext, whole = false) {
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-semantic-contract-"));
  t.after(() => fs.rmSync(temp, { recursive: true, force: true }));
  if (whole) fs.cpSync(root, temp, { recursive: true });
  else fs.cpSync(path.join(root, packName), path.join(temp, packName), { recursive: true });
  return temp;
}

test("the supplement preserves all historical rows and grants no differential lane credit", () => {
  const loaded = loadTypecheckerManifest(path.join(root, "manifest.json"));
  assert.equal(loaded.cases.length, 32);
  assert.equal(
    loaded.cases.reduce((n, fixture) => n + fixture.inputs.length, 0),
    112,
  );
  assert.deepEqual(
    [...new Set(loaded.cases.map((fixture) => fixture.pack))],
    Object.keys(TYPECHECKER_TESTS),
  );
  assert.equal(loaded.supplementalCases.length, 1);
  assert.deepEqual(loaded.supplementalCases[0], original[0]);
  assert.equal(loaded.supplementalCases[0].inputs.length, 8);
  assert.equal(Object.hasOwn(loaded.supplementalCases[0], "adapters"), false);
});

test("missing, duplicate, unknown and false lane registrations refuse", () => {
  for (const value of [undefined, [], [...original, original[0]]]) {
    assert.throws(() => loadNativeSemanticCompletionCases(root, value));
  }
  for (const [key, replacement] of [
    ["pack", "unregistered"],
    ["test", "unregistered_test"],
    ["case", "unregistered"],
    ["id", "typechecker/native-semantic-completion/unregistered"],
    ["state", "draft"],
    ["targets", ["batch"]],
    ["contract", "batch-start-diagnostics-v1"],
    ["adapters", { legacy: "batch-typechecker-fixture-v1", native: "claimed" }],
    ["regressionCommit", "0".repeat(40)],
    ["sourceRevision", "0".repeat(40)],
  ] satisfies [string, unknown][]) {
    const changed = structuredClone(original);
    Object.assign(changed[0], { [key]: replacement });
    assert.throws(() => loadNativeSemanticCompletionCases(root, changed), key);
  }
});

test("the original batch unknown-test refusal remains strict", (t) => {
  const temp = copyFixture(t, true);
  const manifest = structuredClone(originalManifest);
  manifest.cases.push({
    ...manifest.cases[0],
    id: "typechecker/unregistered/new",
    pack: "unregistered",
  });
  fs.writeFileSync(path.join(temp, "manifest.json"), JSON.stringify(manifest));
  assert.throws(
    () => loadTypecheckerManifest(path.join(temp, "manifest.json")),
    /unregistered test binary body/,
  );
});

test("missing, duplicate, reordered and altered physical input registrations refuse", () => {
  const mutate = [
    (inputs: any[]) => inputs.pop(),
    (inputs: any[]) => inputs.push(inputs[0]),
    (inputs: any[]) => inputs.reverse(),
    (inputs: any[]) => {
      inputs[0].file = "Other.vue";
    },
    (inputs: any[]) => {
      inputs[0].path = "native-semantic-completion/original.stdout.json";
    },
    (inputs: any[]) => {
      inputs[0].sha256 = "0".repeat(64);
    },
  ];
  for (const change of mutate) {
    const changed = structuredClone(original);
    change(changed[0].inputs);
    assert.throws(() => loadNativeSemanticCompletionCases(root, changed));
  }
});

test("every complete physical input refuses changed bytes even with a rehashed registration", (t) => {
  const temp = copyFixture(t);
  for (const [index, input] of original[0].inputs.entries()) {
    const file = path.join(temp, input.path);
    const bytes = fs.readFileSync(file);
    fs.writeFileSync(file, Buffer.concat([bytes, Buffer.from("\nchanged source")]));
    assert.throws(
      () => loadNativeSemanticCompletionCases(temp, original),
      /immutable artifact SHA256 mismatch/,
    );
    const changed = structuredClone(original);
    changed[0].inputs[index].sha256 = sha256(fs.readFileSync(file));
    assert.throws(() => loadNativeSemanticCompletionCases(temp, changed));
    fs.writeFileSync(file, bytes);
  }
});

test("full raw stdout, stderr, command and provenance packets cannot be replaced", (t) => {
  const temp = copyFixture(t);
  for (const file of [
    "original.stdout.json",
    "original.stderr.bin",
    "original-command.json",
    "original-provenance.json",
  ]) {
    const target = path.join(temp, packName, file);
    const bytes = fs.readFileSync(target);
    fs.writeFileSync(target, Buffer.concat([bytes, Buffer.from([0xff])]));
    assert.throws(
      () => loadNativeSemanticCompletionCases(temp, original),
      /immutable artifact SHA256 mismatch/,
    );
    fs.writeFileSync(target, bytes);
  }
});

test("rehashing a reduced vector, program, config, source pin or raw output never changes the frozen oracle", (t) => {
  const temp = copyFixture(t);
  const target = path.join(temp, packName, "case.json");
  const pack = JSON.parse(fs.readFileSync(target, "utf8"));
  const changes = [
    (value: any) => value.expectedDiagnostics.pop(),
    (value: any) => value.reportedFiles.pop(),
    (value: any) => value.inputFiles.pop(),
    (value: any) => {
      value.sourceRepairRevision = "0".repeat(40);
    },
    (value: any) => {
      value.sourceRecord = "replacement source";
    },
    (value: any) => {
      value.cliArgv[7] = "replacement-config.json";
    },
    (value: any) => {
      value.expectedExit = 0;
    },
    (value: any) => {
      value.expectedStderr = ["ignored stderr"];
    },
    (value: any) => {
      value.originalOutputs["original.stderr.bin"].sha256 = "0".repeat(64);
    },
  ];
  for (const key of ["file", "line", "column", "code", "severity", "message", "blockType"]) {
    changes.push((value: any) => {
      value.expectedDiagnostics[0][key] = null;
    });
  }
  for (const change of changes) {
    const value = structuredClone(pack);
    change(value);
    fs.writeFileSync(target, JSON.stringify(value));
    const changed = structuredClone(original);
    changed[0].packReference.sha256 = sha256(fs.readFileSync(target));
    assert.throws(
      () => loadNativeSemanticCompletionCases(temp, changed),
      /entire frozen supplemental oracle/,
    );
  }
});
