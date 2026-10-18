// Frozen verifier semantics from reviewed policy d57cf724d2caea6cf47f814250d18801f6b75e15.
// Only the authenticated run below may retain v1 semantics. New evidence is v2.
import assert from "node:assert/strict";
import { lstatSync, readFileSync } from "node:fs";
import path from "node:path";
import { commandPlan, roles, verifyConsumer, type Packet, type Installed } from "./consumer.ts";
import { verifyInstalledArchive, type InstalledArchive } from "./archive.ts";
import { api, command, gitBlob, pages, type Original } from "./custody.ts";
import { custodyProbe } from "./oracles.ts";
import { effectiveDenoConfig, strictUtf8 } from "./runtime.ts";
import {
  admitOriginal,
  assertFresh,
  assertFreshV2,
  fixture,
  failureLogArguments,
  object,
  positive,
  same,
  sha256,
  type Identity,
  type Json,
} from "./policy.ts";

export const legacyPolicySha = "d57cf724d2caea6cf47f814250d18801f6b75e15";
export function recoveryEvidencePolicy(run: Json) {
  if (run.head_sha !== legacyPolicySha) return "current-v2" as const;
  same(String(run.id), "38993331275", "legacy policy belongs only to its original recovery run");
  same(run.run_attempt, 1, "legacy recovery attempt changed");
  return "legacy-v1" as const;
}
/** After Cosign verification, bind the receipt version to authenticated run provenance. */
export function assertRecoveryReceiptPolicy(
  receipt: Json,
  run: Json,
  expected: Identity,
  now = Date.now(),
) {
  same(
    receipt.recovery,
    { run: String(run.id), attempt: run.run_attempt, policySha: run.head_sha },
    "receipt provenance changed",
  );
  if (recoveryEvidencePolicy(run) === "legacy-v1") assertFresh(receipt, expected, now);
  else assertFreshV2(receipt, expected, now);
}
export const legacyPolicyAuthority = [
  {
    file: ".github/workflows/release-consumer-recovery.yml",
    sha256: "b0f1a4ea05af2a8dd133722a52766e99ab504a9ccb0d32b3fbd9e99a31fb316c",
  },
  {
    file: ".github/workflows/release-operator.yml",
    sha256: "78351f5588b23113a3618fdfbb7bcee1276b1098e794c048a35d4d1317183e07",
  },
  {
    file: ".github/workflows/release-public-acceptance.yml",
    sha256: "aea84a3ec9a9b2ad40b3126cd043f4ba5c152b47dff94e1a85c6f6eebf4e322b",
  },
  {
    file: "tools/support/release/consumer_recovery/archive.ts",
    sha256: "7bbbaa92ac98750c955e851049c6410c4d250eafb48c4d2a5ab18ca30dafeea8",
  },
  {
    file: "tools/support/release/consumer_recovery/consumer.ts",
    sha256: "47cecc4686ee78c899a57f2da2ba65669ef9703447d2b0672682a75e945d17c8",
  },
  {
    file: "tools/support/release/consumer_recovery/custody.ts",
    sha256: "09cff1b667e4d4f9b041bb90c4e742807bbf784f1003396d674d431962a887ab",
  },
  {
    file: "tools/support/release/consumer_recovery/main.ts",
    sha256: "bde8c198be809c5088d4d6b4a11e02b4e340f45b9037a61adbccfc4539aa7746",
  },
  {
    file: "tools/support/release/consumer_recovery/oracles.ts",
    sha256: "c901dea9e1283532567b3d7a467a392e5d031eda0eba336c25aeadde0f6857e3",
  },
  {
    file: "tools/support/release/consumer_recovery/policy.ts",
    sha256: "8e68aba4bf13b77f9515c52258e0f0d57bd0eb107406fc31bdc43e758bcde886",
  },
  {
    file: "tools/support/release/marketplace_query.ts",
    sha256: "165a12fdacccecdda60ad8d7647f101c9e4ca87d02b0f3ce1c82db49d29187f5",
  },
  {
    file: "tools/support/release/public_acceptance/github.ts",
    sha256: "9e55b9c53e88b787bc88c6f5c67f61cd063a106f14a61507329ccd99a02c0b94",
  },
  {
    file: "tools/support/release/public_acceptance/http.ts",
    sha256: "6774e058702ae9e025267619804388938ef0e324a8e46af432591cca9dc783fc",
  },
  {
    file: "tools/support/release/public_acceptance/inputs.ts",
    sha256: "0a60afac7fd4d4994b005629ab6778be79dc62bad27fd4d9f552bb7cf4a2e3b5",
  },
  {
    file: "tools/support/release/public_acceptance/installed.ts",
    sha256: "f650601a3187c9058388cd782ed4a0f46ccee0a249805f9d88e30e13e70bf9e6",
  },
  {
    file: "tools/support/release/public_acceptance/plan.ts",
    sha256: "af08da897ebb1113e6c201fb88fbca0ad24bb0e69fe29e703641dfa5761c3ced",
  },
  {
    file: "tools/support/release/public_acceptance/registry.ts",
    sha256: "da1e0405b1418d44774f43fe93e7d2b2e85c59c3bbc370bcabf8913771f1e2da",
  },
  {
    file: "tools/support/release/windows_hover_acceptance.ts",
    sha256: "dee12b2c0bb2eeb5f883a7e00c0346a0c52185ffc1bef9f22c8294bf3f9dd484",
  },
];
export function legacyAuthority(sha: string) {
  same(sha, legacyPolicySha, "unknown legacy verifier policy");
  for (const entry of legacyPolicyAuthority)
    same(sha256(gitBlob(sha, entry.file)), entry.sha256, "frozen legacy authority changed");
  return legacyPolicyAuthority;
}
export function legacyAdmission(original: Original) {
  same(original.schema, "vize-consumer-recovery-original-v1", "original schema mismatch");
  return admitOriginal(
    original.identity,
    original.run,
    original.jobs,
    original.failed,
    original.log,
    original.blobs,
  );
}
function verifyOracleHashes(actual: unknown, policy: Original["policy"], deno: boolean) {
  same(policy.custodyProbeSha256, sha256(custodyProbe), "reviewed custody oracle required");
  same(
    actual,
    {
      "consumer.ts": policy.consumerSha256,
      "custody.ts": policy.custodyProbeSha256,
      ...(deno ? { "deno.json": policy.denoConfigSha256 } : {}),
    },
    "immutable consumer/config/custody oracle changed",
  );
}
export function assertLegacyOracleFiles(
  directory: string,
  policy: Original["policy"],
  deno: boolean,
) {
  const hashes: Record<string, string> = {};
  for (const name of ["consumer.ts", "custody.ts", ...(deno ? ["deno.json"] : [])]) {
    const file = path.join(directory, name);
    assert.ok(
      lstatSync(file).isFile() && !lstatSync(file).isSymbolicLink(),
      "normal oracle file required",
    );
    hashes[name] = sha256(readFileSync(file));
  }
  verifyOracleHashes(hashes, policy, deno);
}

/** Called only after the original run, signed source and receipt are authenticated. */
export function verifyLegacyConsumer(value: unknown, expected?: unknown) {
  const packet = object(value);
  same(object(packet.policy).sha, legacyPolicySha, "legacy packet policy changed");
  same(
    packet.run,
    { id: "38993331275", attempt: "1", sha: legacyPolicySha },
    "legacy packet run changed",
  );
  same(object(packet.policy).authority, legacyPolicyAuthority, "legacy packet authority changed");
  verifyFrozenConsumer(value, expected);
}

/** Reject partial packets before the finalizer signs the combined evidence. */
function verifyFrozenConsumer(value: unknown, expected?: unknown) {
  object(value);
  const packet = value as Packet;
  assert.equal(packet.schema, "vize-consumer-recovery-consumer-v1");
  assert.equal(packet.status, "passed");
  assert.ok(Object.hasOwn(roles, packet.role));
  const selected = roles[packet.role];
  assert.deepEqual(packet.runtime, selected);
  if (expected) {
    assert.deepEqual(packet.identity, object(expected).identity);
    assert.deepEqual(packet.policy, object(expected).policy);
  }
  assert.equal(packet.consumerSha256, packet.policy.consumerSha256);
  assert.equal(packet.denoConfigSha256, packet.policy.denoConfigSha256);
  assert.equal(
    packet.consumerSha256,
    "410be5021e5c1bff706191bf68045ddbc2edbaa47b13325ddde3daa22926851c",
  );
  assert.equal(
    packet.denoConfigSha256,
    "7b02ff3799e9e65083d27c1f65260c9a294bb809e79174f828f9bd79148b9700",
  );
  assert.equal(
    packet.policy.workflowSha256,
    "bf5b8d553c05b85afa434a4b4f637a5c5f0b09f9357a6026ab4d0f2014893d81",
  );
  assert.equal(packet.run.sha, packet.policy.sha);
  const platformPath = selected.platform === "win32" ? path.win32 : path.posix;
  assert.ok(platformPath.isAbsolute(packet.project));
  assert.ok(!packet.project.startsWith(packet.checkout + platformPath.sep));
  assert.match(platformPath.basename(packet.nodeExecutable), /^node(?:\.exe)?$/u);
  const version = packet.identity.tag.replace(/^v/u, "");
  const cache = platformPath.join(packet.project, "deno-cache");
  assert.deepEqual(
    packet.commands.map((command) => [command.label, command.command, command.args]),
    commandPlan(packet),
  );
  for (const command of packet.commands) {
    assert.equal(command.status, 0, command.label);
    assert.equal(command.signal, null, command.label);
    assert.equal(typeof command.stdout, "string");
    assert.equal(typeof command.stderr, "string");
    assert.equal(command.cwd, packet.project);
    assert.equal(command.denoDir, selected.runtime === "deno" ? cache : null);
    assert.equal(command.error, undefined);
    verifyOracleHashes(command.oracleBefore, packet.policy, selected.runtime === "deno");
    verifyOracleHashes(command.oracleAfter, packet.policy, selected.runtime === "deno");
  }
  assert.match(packet.commands[0].stdout, new RegExp(`^v${selected.node}\\.`));
  assert.ok(packet.installed);
  assert.equal(packet.installed.native.loaded, true);
  assert.match(packet.installed.native.sha256, /^[a-f0-9]{64}$/u);
  assert.ok(packet.installed.native.path.endsWith(".node"));
  const actual = packet.installed.runtime;
  assert.equal(actual.platform, selected.platform);
  assert.ok(["x64", "arm64"].includes(actual.arch));
  if (selected.runtime === "deno") assert.equal(actual.deno?.deno, "2.9.7");
  else if (selected.runtime === "bun") assert.equal(actual.bun, "1.3.14");
  else assert.match(actual.node, new RegExp(`^v${selected.node}\\.`));
  const provider = `@vizejs/native-${actual.platform === "win32" ? "win32" : actual.platform}-${actual.arch}${actual.platform === "win32" ? "-msvc" : actual.platform === "linux" ? "-gnu" : ""}`;
  assert.deepEqual(
    packet.packages.map((record) => String(record.name)).sort((a, b) => a.localeCompare(b)),
    [
      "vize",
      "@vizejs/native",
      provider,
      ...(selected.runtime === "deno" ? [] : ["@vizejs/vite-plugin"]),
    ].sort((a, b) => a.localeCompare(b)),
  );
  assert.deepEqual(
    packet.installed.packages.map((record) => record.name).sort((a, b) => a.localeCompare(b)),
    packet.packages.map((record) => String(record.name)).sort((a, b) => a.localeCompare(b)),
    "actual installed package observations must be complete",
  );
  for (const record of packet.packages) {
    const installedObservation: Installed["packages"][number] | undefined =
      packet.installed.packages.find((installed) => installed.name === record.name);
    assert.ok(installedObservation, "actual provider observation required");
    for (const key of ["name", "version", "directory", "manifestSha256"] as const)
      assert.equal(
        record[key],
        installedObservation[key],
        `actual installed ${key} differs from archive observation`,
      );
    verifyInstalledArchive({
      ...record,
      native: record.name === provider ? packet.installed.native : undefined,
    } as InstalledArchive);
    assert.equal(record.version, version);
    assert.equal(record.source, packet.identity.head);
    assert.equal(new URL(String(record.tarball)).origin, "https://registry.npmjs.org");
    assert.equal(record.integrity, `sha512-${String(object(record.archive).sha512)}`);
    assert.match(String(object(record.archive).sha256), /^[a-f0-9]{64}$/u);
    assert.ok(
      Date.parse(String(record.published)) <= Date.parse(packet.commands[0].began) - 86_400_000,
    );
    assert.ok(String(record.directory).startsWith(packet.project + platformPath.sep));
  }
  const nativePackage = packet.packages.find((record) => record.name === provider);
  assert.ok(
    packet.installed.native.path.startsWith(String(nativePackage?.directory) + platformPath.sep),
  );
  assert.ok(packet.jsr);
}

export async function collectLegacyOriginal(
  expected: Identity,
  policySha: string,
): Promise<Original> {
  const policy = {
    sha: policySha,
    authority: legacyAuthority(policySha),
    ...fixture,
    custodyProbeSha256: sha256(custodyProbe),
  };
  const run = api(`actions/runs/${expected.run}`);
  const tagRef = api(`git/ref/tags/${expected.tag}`);
  same(object(tagRef.object).type, "tag", "original annotated tag required");
  const tagObject = api(`git/tags/${String(object(tagRef.object).sha)}`);
  same(object(tagObject.object).sha, expected.head, "original tag target H mismatch");
  const source = api(`pulls/${expected.sourcePr}`);
  const sourceReceipt = {
    number: source.number,
    title: source.title,
    body: source.body,
    draft: source.draft,
    head: {
      sha: object(source.head).sha,
      ref: object(source.head).ref,
      repository: object(object(source.head).repo).full_name,
    },
    base: { ref: object(source.base).ref, repository: object(object(source.base).repo).full_name },
  };
  const jobs = await pages(`actions/runs/${expected.run}/jobs?filter=latest`, "jobs");
  const failures = jobs.filter((job) => job.conclusion === "failure");
  same(failures.length, 1, "one known failed consumer required");
  const failed = api(`actions/jobs/${positive(failures[0].id)}`);
  const logBytes = command("gh", failureLogArguments(failed.id));
  const log = logBytes.toString("utf8");
  assert.ok(Buffer.from(log, "utf8").equals(logBytes), "raw failure log cannot be lossily decoded");
  assert.ok(
    Buffer.byteLength(log) > 0 && Buffer.byteLength(log) <= 16 * 1024 * 1024,
    "bounded complete failed log required",
  );
  const blobs = {
    releaseWorkflowSha256: gitBlob(expected.head, ".github/workflows/release.yml"),
    workflowSha256: gitBlob(expected.head, ".github/workflows/release-jsr.yml"),
    consumerSha256: gitBlob(expected.head, "tools/support/release/jsr/consumer.ts"),
    denoConfigSha256: gitBlob(policySha, "tools/support/release/jsr/consumer-deno.json"),
  };
  const artifacts = await pages(`actions/runs/${expected.run}/artifacts`, "artifacts");
  for (const artifact of artifacts) {
    positive(artifact.id);
    assert.ok(
      typeof artifact.name === "string" && artifact.expired === false,
      "original artifact expired/ambiguous",
    );
    assert.match(
      String(artifact.digest),
      /^sha256:[0-9a-f]{64}$/,
      "original artifact digest required",
    );
  }
  const original: Original = {
    schema: "vize-consumer-recovery-original-v1",
    identity: expected,
    policy,
    run,
    jobs,
    failed,
    log,
    logIdentity: { sha256: sha256(logBytes), bytes: logBytes.byteLength, encoding: "utf8" },
    blobs,
    artifacts,
    pin: { tagRef, tagObject },
    sourceReceipt,
  };
  legacyAdmission(original);
  same(api(`actions/runs/${expected.run}`), run, "R changed during original collection");
  return original;
}

/** Check the signed packet against the actual retained command bytes and effective config. */
export function verifyConsumerDirectory(
  directory: string,
  original: Pick<Original, "identity" | "policy">,
) {
  const packet = object(JSON.parse(readFileSync(path.join(directory, "consumer.json"), "utf8")));
  verifyConsumer(packet, original);
  for (const value of packet.commands as unknown[]) {
    const record = object(value),
      label = String(record.label);
    assert.match(label, /^[a-z-]+$/);
    const raw = path.join(directory, `${label}-raw`);
    const stdout = readFileSync(path.join(raw, "stdout.bin")),
      stderr = readFileSync(path.join(raw, "stderr.bin"));
    same(strictUtf8(stdout), record.stdout, "complete command stdout changed");
    same(strictUtf8(stderr), record.stderr, "complete command stderr changed");
    const process = object(JSON.parse(readFileSync(path.join(raw, "process.json"), "utf8")));
    for (const key of ["command", "args", "cwd", "status", "signal", "error"])
      same(process[key], record[key], `raw command ${key} changed`);
    same(
      process.stdout,
      { bytes: stdout.length, sha256: sha256(stdout) },
      "raw stdout digest changed",
    );
    same(
      process.stderr,
      { bytes: stderr.length, sha256: sha256(stderr) },
      "raw stderr digest changed",
    );
  }
  if (packet.role === "deno-linux")
    same(
      readFileSync(path.join(directory, "deno.json"), "utf8"),
      effectiveDenoConfig,
      "effective Deno config changed",
    );
  return packet;
}
