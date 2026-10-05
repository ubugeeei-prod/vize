import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import {
  type PublishedAuthority,
  verifyPublishedLaunch,
} from "../../tooling/support/lsp/published-launch.ts";
import { finiteCut } from "./warm-type-backed-cut.ts";
import { installPublicAsset, loadSourceReference } from "./warm-type-backed-public-assets.ts";
import { object } from "./warm-type-backed-packets.ts";
import { runSide } from "./warm-type-backed-requests.ts";
import {
  driverRoot,
  inputAuthority,
  runtimeIdentity,
  sha256,
  sourceIdentity,
} from "./warm-type-backed-source.ts";

assert.equal(
  process.platform,
  "linux",
  "the installed original400 replay retains real /proc controls",
);
assert.equal(process.arch, "x64");
assert.ok(process.env.RUNNER_TEMP && process.env.GITHUB_WORKSPACE);
assert.equal(fs.realpathSync(process.env.GITHUB_WORKSPACE), driverRoot);
const output = path.join(process.env.RUNNER_TEMP, "warm-pair");
const read = (name: string) => JSON.parse(fs.readFileSync(path.join(output, name), "utf8"));
const save = (name: string, value: unknown) =>
  fs.writeFileSync(path.join(output, name), `${JSON.stringify(value, null, 2)}\n`);

async function prepare() {
  assert.ok(!fs.existsSync(output), "public replay owns a fresh output directory");
  fs.mkdirSync(output);
  const text = process.env.ORIGINAL_PUBLIC_AUTHORITY ?? "";
  save("public-inputs.json", {
    authorityText: text,
    digest: process.env.ORIGINAL_PUBLIC_AUTHORITY_SHA256,
    driver: sourceIdentity(driverRoot),
  });
  try {
    assert.match(process.env.ORIGINAL_PUBLIC_AUTHORITY_SHA256 ?? "", /^[a-f0-9]{64}$/u);
    assert.equal(sha256(text), process.env.ORIGINAL_PUBLIC_AUTHORITY_SHA256);
    const authority = JSON.parse(text) as PublishedAuthority;
    assert.equal(authority.schema, "vize.original400.published.authority");
    assert.equal(authority.version, 1);
    assert.match(authority.releaseVersion, /^0\.434\.\d+$/u);
    const cut = finiteCut(output);
    assert.ok(cut, "the public replay requires the same literal root-frozen cut");
    assert.deepEqual(authority.cut, { sha: cut.after, tree: cut.tree, manifestSha256: cut.digest });
    for (const entry of [authority.asset, authority.sourceArtifact]) {
      assert.ok(Number.isSafeInteger(entry.id) && entry.id > 0);
      assert.ok(Number.isSafeInteger(entry.size) && entry.size > 0);
      assert.match(entry.sha256, /^[a-f0-9]{64}$/u);
    }
    assert.ok(
      Number.isSafeInteger(authority.sourceArtifact.run) && authority.sourceArtifact.run > 0,
    );
    const driver = sourceIdentity(driverRoot);
    assert.equal(driver.dirty, "");
    assert.equal(driver.revision, process.env.GITHUB_SHA);
    assert.equal(driver.revision, process.env.GITHUB_WORKFLOW_SHA);
    assert.equal(
      authority.sourceArtifact.driverRevision,
      driver.revision,
      "the same reviewed driver owns both executions",
    );
    const referenceRoot = await loadSourceReference(authority, output);
    const reference = JSON.parse(fs.readFileSync(path.join(referenceRoot, "paired.json"), "utf8"));
    const qualified = JSON.parse(
      fs.readFileSync(path.join(referenceRoot, "qualified.json"), "utf8"),
    );
    assert.equal(reference.sourceBinding.authority, "root-frozen-release-cut");
    assert.deepEqual(reference.sourceBinding.driverSource, driver);
    assert.equal(reference.sourceBinding.finiteCut.control, cut.control);
    assert.equal(reference.sourceBinding.finiteCut.after, cut.after);
    assert.equal(reference.sourceBinding.finiteCut.tree, cut.tree);
    assert.equal(reference.sourceBinding.finiteCut.digest, cut.digest);
    assert.equal(reference.after.source.revision, cut.after);
    assert.equal(reference.after.source.tree, cut.tree);
    assert.equal(qualified.wholePacketsEqual, true);
    assert.deepEqual(reference.originals, inputAuthority());
    for (const side of [reference.before, reference.after]) {
      assert.deepEqual(side.failures, []);
      assert.equal(side.rows.length, 74);
      assert.equal(
        side.rows.filter((row: { stage: string }) => /^warm-[1-5]$/u.test(row.stage)).length,
        20,
      );
    }
    assert.equal(
      reference.after.inputs.workspace,
      path.join(output, "workspace"),
      "the original absolute workspace remains unchanged",
    );
    const runtime = runtimeIdentity();
    assert.deepEqual(
      runtime,
      reference.runtime,
      "the complete controlled Linux provider graph stays exact",
    );
    save("public-runtime.json", runtime);
    save("workflow-source.json", {
      authority: "installed-public-release",
      head: cut.after,
      driverSource: driver,
      finiteCut: cut,
      sourceReferenceRoot: referenceRoot,
    });
    fs.writeFileSync(path.join(output, "changed-tree-manifest.json"), cut.manifestBytes);
    const installation = await installPublicAsset(authority, output);
    save("published-installation.json", installation);
    const receiptPath = path.join(output, "published-installation.json");
    const receiptSha256 = sha256(fs.readFileSync(receiptPath));
    verifyPublishedLaunch(receiptPath, receiptSha256);
    save("public-launch.json", {
      receiptPath,
      receiptSha256,
      captureRoot: path.join(output, "public-lsp-sessions"),
      binary: installation.installed.path,
      authority: "published-release",
    });
  } catch (error) {
    save("public-prepare-failure.json", {
      error: error instanceof Error ? error.stack : String(error),
    });
    throw error;
  }
}

async function measure() {
  const binding = read("workflow-source.json");
  assert.equal(binding.authority, "installed-public-release");
  assert.deepEqual(sourceIdentity(driverRoot), binding.driverSource);
  const launch = read("public-launch.json");
  const installation = verifyPublishedLaunch(launch.receiptPath, launch.receiptSha256);
  const runtime = runtimeIdentity();
  assert.deepEqual(runtime, read("public-runtime.json"));
  const reference = JSON.parse(
    fs.readFileSync(path.join(binding.sourceReferenceRoot, "paired.json"), "utf8"),
  );
  const side = await runSide(
    driverRoot,
    path.join(output, "workspace"),
    runtime,
    path.join(output, "public"),
    launch,
  );
  const packet = {
    sourceBinding: binding,
    installation,
    runtime,
    originals: inputAuthority(),
    side,
    authority:
      "installed GitHub Linux-x64 release CLI; same original400 and controlled Linux provider graph",
    comparison:
      "all74 complete measured params/response envelopes and20 warm rows against the literal-cut source artifact; setup/shutdown and all89+89 frames retained separately",
    timingScope:
      "public release-profile observations only; no ratio against ci-profile source measurements or general10x claim",
    limits:
      "phase markers do not prove complete native diagnostics; process lives nonrunning may include unreaped zombies; startup/config/peak/RSS and every failed outcome remain retained",
  };
  save("public.json", packet);
  assert.deepEqual(
    side.failures,
    [],
    "every installed-public control and complete packet is required",
  );
  assert.deepEqual(side.inputs.source, reference.after.inputs.source);
  assert.equal(side.inputs.sourceSha256, reference.after.inputs.sourceSha256);
  assert.deepEqual(side.initialization, reference.after.initialization);
  assert.equal(
    object(object(side.initialization).serverInfo).version,
    installation.receipt.authority.releaseVersion,
  );
  const observation = object(object(side.wire).observation);
  assert.equal(observation.launchAuthority, "published-release");
  assert.ok(
    !Object.hasOwn(observation, "buildReceipt"),
    "a public binary has no forged Cargo receipt",
  );
  assert.deepEqual(observation.binary, installation.expected);
  assert.equal(sha256(fs.readFileSync(installation.binary)), installation.expected.binarySha256);
  const whole = (rows: Array<Record<string, unknown>>) =>
    rows.map((row) => ({
      stage: row.stage,
      name: row.name,
      method: row.method,
      params: row.comparableParams,
      requestId: row.requestId,
      response: row.comparableResponse,
    }));
  assert.deepEqual(whole(side.rows), whole(reference.after.rows));
  assert.equal(side.rows.length, 74);
  assert.equal(side.rows.filter((row) => /^warm-[1-5]$/u.test(String(row.stage))).length, 20);
  save("public-qualified.json", {
    installed: installation.expected,
    literalCut: binding.finiteCut.after,
    wholePacketsEqual: true,
    warmRequests: 20,
    observations: side.rows.filter((row) => /^warm-[1-5]$/u.test(String(row.stage))),
    scope: packet.timingScope,
  });
}

if (process.argv[2] === "prepare") await prepare();
else if (process.argv[2] === "measure") await measure();
else throw new Error("usage: warm-type-backed-public.ts prepare|measure");
