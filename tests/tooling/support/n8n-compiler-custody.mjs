import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  cases,
  driverPaths,
  official,
  read,
  recipe,
  run,
  sha256,
  sourceReceipt,
  validateAuthoredCapture,
  validateCapture,
  validateCurrentParity,
  verifyBaseline,
  verifyInputs,
} from "./n8n-compiler-custody-receipt.mjs";
import { validateOfficial } from "./n8n-compiler-official-validation.mjs";
import { validateCompatibilityNotices } from "./n8n-compiler-compatibility-validation.mjs";
import {
  validateBaselineDiagnostics,
  validateDiagnosticContract,
} from "./n8n-compiler-diagnostic-validation.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const [baseline, repairAnchor, fixtureArg, outputArg, bundleArg] = process.argv.slice(2);
assert.ok(
  baseline && repairAnchor && fixtureArg && outputArg && bundleArg && process.argv.length === 7,
  "usage: n8n-compiler-custody.mjs <capture-only commit> <reviewed repair anchor> <read-only n8n root> <output root> <official bundle>",
);
const fixture = fs.realpathSync(fixtureArg);
function resolvedOutput(argument) {
  let parent = path.resolve(argument);
  const suffix = [];
  while (!fs.existsSync(parent)) {
    suffix.unshift(path.basename(parent));
    parent = path.dirname(parent);
  }
  return path.resolve(fs.realpathSync(parent), ...suffix);
}
const output = resolvedOutput(outputArg);
const bundle = fs.realpathSync(bundleArg);
assert.ok(!output.startsWith(`${fixture}${path.sep}`) && output !== fixture);
fs.mkdirSync(output, { recursive: true });
const write = (name, value) =>
  fs.writeFileSync(path.join(output, name), `${JSON.stringify(value, null, 2)}\n`);
const temporary = fs.mkdtempSync(
  path.join(process.env.RUNNER_TEMP ?? os.tmpdir(), "vize-n8n-custody-"),
);
const before = path.join(temporary, "source");
const targetRoot = path.join(temporary, "cargo-targets");
const manifest = JSON.parse(read(root, cases));
let inputs;
let added = false;

async function capture(state, source) {
  source = fs.realpathSync(source);
  const destination = path.join(output, state);
  const revision = run("git", ["rev-parse", "HEAD"], source);
  assert.match(revision, /^[a-f0-9]{40}$/u);
  // Cargo's shared workspace fingerprints can reuse the other source's executable.
  const target = path.join(targetRoot, "n8n-custody-build", state, revision);
  const invocation = {
    command: "cargo",
    args: [
      ...recipe,
      "--manifest-path",
      path.join(source, "Cargo.toml"),
      "--",
      fixture,
      path.join(destination, "vize"),
    ],
    cwd: source,
  };
  fs.mkdirSync(destination, { recursive: true });
  const log = fs.openSync(path.join(destination, "source-capture.log"), "w");
  try {
    run(invocation.command, invocation.args, invocation.cwd, {
      env: { ...process.env, CARGO_TARGET_DIR: target },
      stdio: ["ignore", log, log],
    });
    run(
      process.execPath,
      [path.join(source, official), fixture, path.join(destination, "official"), bundle],
      source,
      {
        stdio: ["ignore", log, log],
      },
    );
  } finally {
    fs.closeSync(log);
  }
  const receipt = sourceReceipt(source, state, target, inputs, invocation);
  fs.writeFileSync(
    path.join(destination, "source-authentication.json"),
    `${JSON.stringify(receipt, null, 2)}\n`,
  );
  const officialCapture = await validateOfficial(
    path.join(destination, "official"),
    fixture,
    manifest,
  );
  const packets = validateCapture(
    path.join(destination, "vize"),
    fixture,
    manifest,
    officialCapture.languageRecipes,
  );
  const authoredPackets = validateAuthoredCapture(path.join(destination, "vize"));
  validateDiagnosticContract(path.join(destination, "vize"), source);
  const result = {
    ...receipt,
    packets,
    authoredPackets,
    diagnosticContractSha256: sha256(
      read(path.join(destination, "vize"), "diagnostic-contract.json"),
    ),
    officialCapture,
  };
  fs.writeFileSync(
    path.join(destination, "source-receipt.json"),
    `${JSON.stringify(result, null, 2)}\n`,
  );
  return result;
}

try {
  inputs = verifyInputs(fixture, manifest);
  const ancestry = verifyBaseline(root, baseline, repairAnchor);
  write("baseline-authentication.json", ancestry);
  run("git", ["worktree", "add", "--detach", before, baseline], root);
  added = true;
  for (const relative of driverPaths) {
    assert.deepEqual(
      read(before, relative),
      read(root, relative),
      `before/after driver differs: ${relative}`,
    );
  }
  const baselineReceipt = await capture("before", before);
  const currentReceipt = await capture("after", root);
  assert.deepEqual(baselineReceipt.inputs, currentReceipt.inputs);
  assert.deepEqual(baselineReceipt.drivers, currentReceipt.drivers);
  assert.deepEqual(baselineReceipt.recipe, currentReceipt.recipe);
  assert.deepEqual(baselineReceipt.officialRecipe, currentReceipt.officialRecipe);
  assert.notEqual(
    baselineReceipt.environment.CARGO_TARGET_DIR,
    currentReceipt.environment.CARGO_TARGET_DIR,
    "before and current source require separate revision-bound Cargo targets",
  );
  assert.equal(
    baselineReceipt.diagnosticContractSha256,
    currentReceipt.diagnosticContractSha256,
    "complete independent diagnostic contract must stay identical before and after",
  );
  verifyInputs(fixture, manifest);
  const custody = {
    schema: "vize.n8n.compiler-before-after",
    version: 1,
    ...ancestry,
    before: baselineReceipt,
    after: currentReceipt,
    qualification:
      "Pending parity and runtime assertions; no successful compiler qualification yet.",
  };
  write("capture.json", custody);
  const diagnosticContract = validateDiagnosticContract(path.join(output, "after/vize"), root);
  const diagnosticComparison = validateBaselineDiagnostics(
    path.join(output, "before/vize"),
    path.join(output, "after/vize"),
    diagnosticContract,
  );
  const compatibilityComparison = await validateCompatibilityNotices(
    path.join(output, "before/vize"),
    path.join(output, "after/vize"),
    path.join(output, "after/official"),
    manifest,
  );
  const comparisons = validateCurrentParity(
    path.join(output, "after/vize"),
    manifest,
    diagnosticContract,
    compatibilityComparison,
  );
  const runtimeLog = fs.openSync(path.join(output, "default-slot-runtime.log"), "w");
  try {
    run(
      process.execPath,
      [
        path.join(root, "tests/tooling/support/n8n-default-slot-loop-runtime.mjs"),
        "--captures",
        path.join(output, "before/vize/n8n-default-slot-loop.json"),
        path.join(output, "after/vize/n8n-default-slot-loop.json"),
        path.join(output, "capture.json"),
        path.join(output, "default-slot-runtime.json"),
      ],
      root,
      { stdio: ["ignore", runtimeLog, runtimeLog] },
    );
  } finally {
    fs.closeSync(runtimeLog);
  }
  write("capture.json", {
    ...custody,
    comparisons,
    diagnosticComparison,
    compatibilityComparison,
    runtimeReceiptSha256: sha256(read(output, "default-slot-runtime.json")),
    qualification:
      "Complete source/official parity and authored runtime passed; full unchanged corpus remains independently required.",
  });
  console.log(
    `Captured both source states for ${manifest.cases.length} unchanged licensed n8n inputs.`,
  );
} catch (error) {
  write("failure.json", { name: error.name, message: error.message, stack: error.stack });
  throw error;
} finally {
  if (added) run("git", ["worktree", "remove", before], root);
  fs.rmSync(temporary, { recursive: true, force: true });
}
