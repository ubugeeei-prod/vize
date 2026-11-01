import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const SOURCE = "c4aa5c7a244e581d7edf30b27ad19e88e09f0f53";
const TREE = "11a6765bc4e7b4f1f4e2e0531e2040b791d5861a";
const WORKFLOW = ".github/workflows/patina-native-boolean-protocol.yml";
const SELF = "tools/support/compat/github/native-linter-history-capture.ts";
const PINS: Record<string, string> = {
  "Cargo.lock": "519fabcc3dd6e10c993829ebeab497c39264bcec68454acb7f0d07f54d5e9104",
  "pnpm-lock.yaml": "bc0ff0b8a3b617d02c86d27540ba3046fe55ef5d41c2e9179dec155caaf5c38c",
  "package.json": "703095a422cfc630a5d29ef5f4a5230b1c853c6013b8e8080809643cc5b4c6a0",
  "tests/_fixtures/differential/linter/manifest.json":
    "345746d75ada639abd110d0ab43665cbe27af0629e230fbdddb1e65acd40d7e4",
  "tests/_fixtures/differential/linter/oracle-index.json":
    "4ca3b59baec2b4377da10d68fb3de000f3d18b753289c616e7f21a7184405bf1",
  "crates/vize_patina/tests/fixtures/lint-history/cases.json":
    "d9a4842eddd84350ce05c3c2cbb973f5a852a67f85d5b255acbb699ebcc2de16",
  "crates/vize_patina/tests/fixtures/next-tick-history/cases.json":
    "96b1745c527e4a65fc0347566d015d5063271e90d7fdc8d21fe160c54856c5c2",
  "crates/vize_patina/tests/fixtures/report-history/cases.json":
    "c10833e985b26e524cc9a0d0e469c3e651850fbe9dd6293d8f2854e692ef5622",
  "tests/_fixtures/differential/lint-static-class/cases.json":
    "b409b01c2e7177419cafbdbbdb6257b11fe32e37e9a46acd9595d09d1f817116",
  "crates/vize_patina/tests/fixtures/component-name-history/cases.json":
    "867a29d7c0a92c17d59fcaa6063be1e3649fb92375024fcd2932a57f27fe062c",
};
const root = fs.realpathSync(process.cwd());
const [phase, argument] = process.argv.slice(2);
assert(argument && path.isAbsolute(argument), "owned absolute evidence directory required");
assert(process.env.RUNNER_TEMP, "runner-owned temporary directory required");
const temporary = fs.realpathSync(process.env.RUNNER_TEMP);
const output = path.resolve(argument);
assert.equal(path.dirname(output), temporary, "evidence must be a direct owned temporary child");
fs.mkdirSync(output, { recursive: true });
assert.equal(fs.realpathSync(output), output);
const sha = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");
const hash = (file: string) => sha(fs.readFileSync(file));
function object(value: unknown): Record<string, unknown> {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
}
function read(file: string) {
  return object(JSON.parse(fs.readFileSync(file, "utf8")));
}
function write(name: string, value: unknown) {
  fs.writeFileSync(path.join(output, name), JSON.stringify(value, null, 2) + "\n");
}
function command(executable: string, argv: string[]) {
  const result = spawnSync(executable, argv, {
    cwd: root,
    timeout: 30_000,
    maxBuffer: 4 * 1024 * 1024,
  });
  const packet = {
    executable,
    argv,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdoutBase64: result.stdout?.toString("base64") ?? "",
    stderrBase64: result.stderr?.toString("base64") ?? "",
  };
  assert.equal(packet.error, null);
  assert.equal(packet.signal, null);
  assert.equal(packet.status, 0, Buffer.from(packet.stderrBase64, "base64").toString());
  return packet;
}
function git(...args: string[]) {
  return Buffer.from(command("git", ["--no-replace-objects", ...args]).stdoutBase64, "base64")
    .toString()
    .trim();
}
function source() {
  const controller = read(path.join(output, "controller.json"));
  assert.equal(process.env.SOURCE_SHA, SOURCE);
  assert.equal(controller.sourceRevision, SOURCE);
  assert.equal(controller.sourceTree, TREE);
  assert.equal(
    process.env.GITHUB_SHA,
    controller.controllerRevision,
    "controller environment changed",
  );
  assert.equal(hash(fileURLToPath(import.meta.url)), controller.scriptSha256);
  assert.equal(hash(path.join(output, "controller-workflow.yml")), controller.workflowSha256);
  assert.equal(git("rev-parse", "HEAD"), SOURCE);
  assert.equal(git("rev-parse", "HEAD^{tree}"), TREE);
  assert.equal(git("diff", "--name-only", "HEAD", "--"), "", "source checkout changed");
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  assert.equal(process.versions.node.split(".")[0], "24");
  assert.equal(process.env.CI, "true");
  assert.equal(process.env.NODE_OPTIONS, "--disable-warning=DEP0040 --disable-warning=DEP0169");
  assert.equal(process.env.GITHUB_RUN_ATTEMPT, "1");
  for (const key of [
    "VIZE_TOOLING_TEST_PLAN",
    "VIZE_TOOLING_TEST_TIER",
    "VIZE_TOOLING_TEST_SHARD",
    "NODE_TEST_NAME_PATTERN",
    "VIZE_TEST_NAME_PATTERN",
    "CORSA_PATH",
  ]) {
    assert.equal(process.env[key], undefined, `unexpected execution filter/provider: ${key}`);
  }
  assert.equal(process.env.VIZE_LINTER_API_EVIDENCE_DIR, "target/differential/linter-api");
  if (process.env.CARGO_TARGET_DIR)
    assert.equal(path.resolve(process.env.CARGO_TARGET_DIR), path.join(root, "target"));
  for (const [file, expected] of Object.entries(PINS))
    assert.equal(hash(path.join(root, file)), expected, file);
  return controller;
}
async function catalog() {
  const api: typeof import("../../../../tests/differential/linter-api.ts") = await import(
    pathToFileURL(path.join(root, "tests/differential/linter-api.ts")).href
  );
  const loaded = api.loadLinterManifest(
    path.join(root, "tests/_fixtures/differential/linter/manifest.json"),
    root,
  );
  assert.equal(loaded.cases.length, 79);
  const inputs = loaded.cases.map(
    (fixture: { id: string; argv: string[]; input: Buffer; expected: Buffer }) => ({
      id: fixture.id,
      argv: fixture.argv,
      inputSha256: sha(fixture.input),
      inputBase64: fixture.input.toString("base64"),
      expectedSha256: sha(fixture.expected),
      expectedBase64: fixture.expected.toString("base64"),
    }),
  );
  return { api, loaded, inputs };
}

if (phase === "controller") {
  assert.equal(process.env.SOURCE_SHA, SOURCE);
  assert.equal(process.env.GITHUB_RUN_ATTEMPT, "1");
  const controllerRevision = git("rev-parse", "HEAD");
  assert.equal(controllerRevision, process.env.GITHUB_SHA);
  assert.notEqual(controllerRevision, SOURCE);
  assert.equal(git("diff", "--name-only", "HEAD", "--"), "");
  assert.equal(git("ls-files", "--error-unmatch", SELF), SELF);
  const script = fs.readFileSync(path.join(root, SELF));
  assert.equal(hash(fileURLToPath(import.meta.url)), sha(script));
  fs.writeFileSync(path.join(output, "controller.ts"), script);
  fs.copyFileSync(path.join(root, WORKFLOW), path.join(output, "controller-workflow.yml"));
  write("controller.json", {
    schema: "vize.native-linter-history-controller",
    version: 1,
    controllerRevision,
    controllerTree: git("rev-parse", "HEAD^{tree}"),
    sourceRevision: SOURCE,
    sourceTree: TREE,
    workflowSha256: hash(path.join(root, WORKFLOW)),
    scriptSha256: sha(script),
    githubRunId: process.env.GITHUB_RUN_ID,
    githubRunAttempt: process.env.GITHUB_RUN_ATTEMPT,
  });
} else if (phase === "source") {
  const controller = source();
  const rustc = command("rustc", ["-vV"]);
  const cargo = command("cargo", ["--version"]);
  assert(Buffer.from(rustc.stdoutBase64, "base64").toString().startsWith("rustc 1.99.1 "));
  assert(Buffer.from(cargo.stdoutBase64, "base64").toString().startsWith("cargo 1.99.1 "));
  for (const file of Object.keys(PINS)) {
    const destination = path.join(output, "inputs", file);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.copyFileSync(path.join(root, file), destination);
  }
  write("source.json", {
    controller,
    sourceRevision: SOURCE,
    sourceTree: TREE,
    pins: PINS,
    node: {
      version: process.version,
      executable: fs.realpathSync(process.execPath),
      sha256: hash(process.execPath),
    },
    rustc,
    cargo,
    platform: process.platform,
    arch: process.arch,
    workflowEnvironmentSha: process.env.GITHUB_SHA,
    identityAuthority: "actual immutable Git HEAD; environment remains controller",
  });
} else if (phase === "cli") {
  source();
  const receiptPath = path.join(root, "target/ci/vize.differential-build.json");
  const receipt = read(receiptPath);
  assert.equal(receipt.schema, "vize.differential.build");
  assert.equal(receipt.version, 1);
  assert.equal(receipt.recipe, "cargo build --profile ci -p vize");
  assert.equal(receipt.sourceRevision, SOURCE);
  assert.equal(receipt.binaryPath, "target/ci/vize");
  const binary = path.join(root, "target/ci/vize");
  assert(fs.lstatSync(binary).isFile() && !fs.lstatSync(binary).isSymbolicLink());
  assert.equal(receipt.binarySha256, hash(binary));
  const version = command(binary, ["--version"]);
  assert.equal(Buffer.from(version.stdoutBase64, "base64").toString().trim(), receipt.cliVersion);
  assert.equal(version.stderrBase64, "");
  const { inputs } = await catalog();
  write("catalog-inputs.json", inputs);
  write("cli-before-test.json", {
    receipt,
    receiptSha256: hash(receiptPath),
    binarySha256: hash(binary),
    inputSetSha256: hash(path.join(output, "catalog-inputs.json")),
    version,
  });
} else if (phase === "verify") {
  source();
  const before = read(path.join(output, "cli-before-test.json"));
  assert.equal(hash(path.join(root, "target/ci/vize")), before.binarySha256);
  assert.equal(
    hash(path.join(root, "target/ci/vize.differential-build.json")),
    before.receiptSha256,
  );
  const directory = path.join(root, "target/differential/linter-api");
  const report = read(path.join(directory, "report.json"));
  const receipt = read(path.join(directory, "build-receipt.json"));
  assert.equal(object(receipt.source).sourceRevision, SOURCE);
  assert.equal(object(receipt.source).sourceTree, TREE);
  assert.equal(receipt.profileName, "ci");
  assert.equal(receipt.offline, false);
  assert.equal(receipt.exitStatus, 0);
  const binary = path.join(directory, "lint_history_observer");
  assert.equal(fs.realpathSync(String(receipt.binaryPath)), fs.realpathSync(binary));
  assert.equal(object(receipt.artifact).sha256, hash(binary));
  assert.deepEqual(report.buildReceipt, receipt);
  const { api, loaded, inputs } = await catalog();
  assert.equal(hash(path.join(output, "catalog-inputs.json")), before.inputSetSha256);
  assert.deepEqual(
    inputs,
    JSON.parse(fs.readFileSync(path.join(output, "catalog-inputs.json"), "utf8")),
  );
  api.validateLinterReport(loaded, report, receipt);
  assert.deepEqual(report.summary, {
    plannedCases: 79,
    legacyMatches: 79,
    legacyFailures: 0,
    baselineDrift: 0,
    nativeUnsupported: 62,
    nativeFailures: 0,
    nativeHandled: 17,
    nativeEquivalent: 17,
    pairedComparisons: 17,
  });
  write("capture.json", {
    schema: "vize.native-linter-history-capture",
    version: 1,
    sourceRevision: SOURCE,
    sourceTree: TREE,
    controller: read(path.join(output, "controller.json")),
    providers: read(path.join(output, "source.json")),
    cli: before,
    observerReceiptSha256: hash(path.join(directory, "build-receipt.json")),
    observerBinarySha256: hash(binary),
    reportSha256: hash(path.join(directory, "report.json")),
    inputSetSha256: hash(path.join(output, "catalog-inputs.json")),
    summary: report.summary,
    publicReleaseAcceptance: false,
    broaderIssueCompletion: false,
  });
} else {
  throw new Error(`unsupported capture phase: ${phase}`);
}
