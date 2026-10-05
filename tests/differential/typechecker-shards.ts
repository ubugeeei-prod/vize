import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { sha256 } from "./harness.mjs";
import { summarizeNativeAcceptance } from "./acceptance-rates.mjs";
import {
  TYPECHECKER_TESTS,
  loadTypecheckerManifest,
  readCapture,
  typecheckerReport,
  validateTypecheckerCapture,
  type Capture,
  type LoadedTypechecker,
} from "./typechecker.ts";

type Receipt = {
  schemaVersion: number;
  workspaceRoot: string;
  sha: string;
  tree: string;
  platform: string;
  arch: string;
  nextestVersion: string;
  rustcVersion: string;
  cargoProfile: string;
  requireTsgo: string | null;
  disableTsgo: string | null;
  nuxtIterations: string;
  archiveSha256: string;
};
type Worker = {
  schema: string;
  version: number;
  shard: number;
  sourceRevision: string;
  sourceTree: string;
  manifestSha256: string;
  receiptBase64: string;
  junitSha256: string;
  tests: string[];
  captures: Capture[];
};

function checkout(repoRoot: string) {
  const git = (...args: string[]) =>
    execFileSync("git", args, {
      cwd: repoRoot,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    }).trim();
  return { sha: git("rev-parse", "HEAD"), tree: git("rev-parse", "HEAD^{tree}") };
}

function receiptIdentity(bytes: Buffer, expected: { sha: string; tree: string }) {
  const receipt: Receipt = JSON.parse(bytes.toString("utf8"));
  assert.equal(receipt.schemaVersion, 3);
  assert.equal(receipt.sha, expected.sha);
  assert.equal(receipt.tree, expected.tree);
  assert.equal(receipt.cargoProfile, "ci");
  assert.equal(receipt.nextestVersion, "0.9.146");
  assert.match(receipt.rustcVersion, /^rustc 1\.98\.0 /);
  assert.equal(receipt.requireTsgo, "1");
  assert.equal(receipt.disableTsgo, null);
  assert.equal(receipt.nuxtIterations, "100");
  assert.match(receipt.archiveSha256, /^[a-f0-9]{64}$/);
  return receipt;
}

// Nextest emits a fixed, local JUnit format. Only this exact binary's registered
// ASCII names are read; failures, skips and duplicate identities are rejected.
export function passedFixtureTests(xml: string) {
  assert(!/<!DOCTYPE|<!ENTITY/.test(xml), "external XML declarations are unsupported");
  const names = new Set<string>();
  for (const match of xml.matchAll(/<testcase\b([^>]*?)(?:\/>|>([\s\S]*?)<\/testcase>)/g)) {
    const attributes = new Map(
      [...match[1].matchAll(/\b([a-zA-Z]+)="([^"]*)"/g)].map((item) => [item[1], item[2]]),
    );
    if (attributes.get("classname") !== "vize_canon::fix_history_diagnostics") continue;
    const name = attributes.get("name");
    assert(
      name && Object.values(TYPECHECKER_TESTS).some((expected) => expected === name),
      "unplanned Canon test identity",
    );
    assert(!names.has(name), "duplicate Canon JUnit identity");
    assert(
      !/<(?:failure|error|skipped)\b/.test(match[2] ?? ""),
      "Canon test did not actually pass",
    );
    names.add(name);
  }
  return [...names];
}

function validateWorker(
  loaded: LoadedTypechecker,
  worker: Worker,
  junit: Buffer,
  expected: { sha: string; tree: string },
) {
  assert.equal(worker.schema, "vize.typechecker-worker-observations");
  assert.equal(worker.version, 1);
  assert(Number.isInteger(worker.shard) && worker.shard >= 1 && worker.shard <= 4);
  assert.equal(worker.sourceRevision, expected.sha);
  assert.equal(worker.sourceTree, expected.tree);
  assert.equal(worker.manifestSha256, loaded.manifestSha256);
  const receipt = Buffer.from(worker.receiptBase64, "base64");
  receiptIdentity(receipt, expected);
  assert.equal(worker.junitSha256, sha256(junit));
  assert.deepEqual(worker.tests, passedFixtureTests(junit.toString("utf8")));
  assert.deepEqual(
    new Set(worker.captures.map((capture) => capture.test)),
    new Set(worker.tests),
    "a missing observation or an observation without a passed test cannot count",
  );
  assert.equal(worker.captures.length, worker.tests.length);
  for (const capture of worker.captures) {
    assert.match(capture.binarySha256, /^[a-f0-9]{64}$/);
    validateTypecheckerCapture(loaded, capture, { receipt, binarySha256: capture.binarySha256 });
  }
  return worker;
}

export function verifyTypecheckerWorker({
  repoRoot,
  evidenceDir,
  receiptPath,
  junitPath,
  shard,
}: {
  repoRoot: string;
  evidenceDir: string;
  receiptPath: string;
  junitPath: string;
  shard: number;
}) {
  const loaded = loadTypecheckerManifest(
    path.join(repoRoot, "tests/_fixtures/differential/typechecker/manifest.json"),
  );
  const expected = checkout(repoRoot);
  const receipt = fs.readFileSync(receiptPath);
  const identity = receiptIdentity(receipt, expected);
  const junit = fs.readFileSync(junitPath);
  const tests = passedFixtureTests(junit.toString("utf8"));
  fs.mkdirSync(evidenceDir, { recursive: true });
  const captures = fs
    .readdirSync(evidenceDir)
    .filter((file) => file.endsWith(".json") && file !== "worker.json")
    .sort()
    .map((file) => readCapture(path.join(evidenceDir, file)));
  for (const capture of captures) {
    const binary = fs.realpathSync(capture.binaryPath);
    const relative = path.relative(fs.realpathSync(identity.workspaceRoot), binary);
    assert(relative && !relative.startsWith("..") && !path.isAbsolute(relative));
    assert.match(
      relative.replaceAll("\\", "/"),
      /^target\/ci\/deps\/fix_history_diagnostics-[a-f0-9]+(?:\.exe)?$/,
    );
    validateTypecheckerCapture(loaded, capture, {
      receipt,
      binarySha256: sha256(fs.readFileSync(binary)),
    });
  }
  const worker: Worker = {
    schema: "vize.typechecker-worker-observations",
    version: 1,
    shard,
    sourceRevision: expected.sha,
    sourceTree: expected.tree,
    manifestSha256: loaded.manifestSha256,
    receiptBase64: receipt.toString("base64"),
    junitSha256: sha256(junit),
    tests,
    captures,
  };
  validateWorker(loaded, worker, junit, expected);
  fs.writeFileSync(path.join(evidenceDir, "worker.json"), `${JSON.stringify(worker, null, 2)}\n`);
  return worker;
}

export function aggregateTypecheckerWorkers({
  repoRoot,
  artifactRoot,
  outputDir,
}: {
  repoRoot: string;
  artifactRoot: string;
  outputDir: string;
}) {
  for (const name of ["report.json", "acceptance.json"]) {
    fs.rmSync(path.join(outputDir, name), { force: true });
  }
  const loaded = loadTypecheckerManifest(
    path.join(repoRoot, "tests/_fixtures/differential/typechecker/manifest.json"),
  );
  const expected = checkout(repoRoot);
  const workers = fs
    .readdirSync(artifactRoot)
    .sort()
    .map((directory) => {
      const identity = /^rust-test-shard-([1-4])-[1-9]\d*-[1-9]\d*$/.exec(directory);
      assert(identity, "an official Rust worker artifact directory is required");
      const root = path.join(artifactRoot, directory);
      const worker: Worker = JSON.parse(
        fs.readFileSync(path.join(root, "typechecker-fixtures/worker.json"), "utf8"),
      );
      assert.equal(
        worker.shard,
        Number(identity[1]),
        "worker packet must match its artifact shard",
      );
      return validateWorker(
        loaded,
        worker,
        fs.readFileSync(path.join(root, "junit.xml")),
        expected,
      );
    });
  assert.equal(workers.length, 4, "all four complete worker artifacts are required");
  assert.deepEqual(new Set(workers.map((worker) => worker.shard)), new Set([1, 2, 3, 4]));
  assert.equal(
    new Set(workers.map((worker) => worker.receiptBase64)).size,
    1,
    "different build archives",
  );
  const captures = workers.flatMap((worker) => worker.captures);
  assert.equal(
    new Set(captures.map((capture) => capture.binarySha256)).size,
    1,
    "different test executables",
  );
  const report = typecheckerReport(loaded, captures, expected.sha);
  const acceptance = summarizeNativeAcceptance(loaded, report, {
    sourceRevision: expected.sha,
    buildReceiptSha256: captures[0].archiveReceiptSha256,
    requiredStages: ["parse", "facts", "emit", "check"],
  });
  assert.equal(report.summary.legacyMatches, loaded.cases.length);
  assert.equal(acceptance.total.nativeHandled, 0);
  assert.equal(acceptance.total.nativeEquivalent, 0);
  fs.mkdirSync(outputDir, { recursive: true });
  for (const [name, value] of Object.entries({ report, acceptance })) {
    fs.writeFileSync(path.join(outputDir, `${name}.json`), `${JSON.stringify(value, null, 2)}\n`);
  }
  return { report, acceptance };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [operation, ...args] = process.argv.slice(2);
  if (operation === "worker" && args.length === 4) {
    const worker = verifyTypecheckerWorker({
      repoRoot: process.cwd(),
      evidenceDir: path.resolve(args[0]),
      receiptPath: path.resolve(args[1]),
      junitPath: path.resolve(args[2]),
      shard: Number(args[3]),
    });
    console.log(
      JSON.stringify({
        shard: worker.shard,
        executedBodies: worker.tests.length,
        projects: worker.captures.reduce((n, capture) => n + capture.cases.length, 0),
      }),
    );
  } else if (operation === "aggregate" && args.length === 2) {
    const result = aggregateTypecheckerWorkers({
      repoRoot: process.cwd(),
      artifactRoot: path.resolve(args[0]),
      outputDir: path.resolve(args[1]),
    });
    console.log(JSON.stringify(result.report.summary));
  } else {
    throw new Error(
      "usage: typechecker-shards.ts worker CAPTURES RECEIPT JUNIT SHARD | aggregate ARTIFACTS OUTPUT",
    );
  }
}
