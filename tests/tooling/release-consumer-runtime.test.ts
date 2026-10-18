import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import YAML from "yaml";
import {
  admitOriginal,
  fixture,
  integrationReference,
  sha256,
} from "../../tools/support/release/consumer_recovery/policy.ts";
import {
  commandPlan,
  verifyConsumer,
} from "../../tools/support/release/consumer_recovery/consumer.ts";
import {
  policyFiles,
  verifyConsumerDirectory,
  type Original,
} from "../../tools/support/release/consumer_recovery/custody.ts";
import {
  assertRecoveryReceiptPolicy,
  legacyPolicyAuthority,
  legacyPolicySha,
  recoveryEvidencePolicy,
  verifyLegacyConsumer,
} from "../../tools/support/release/consumer_recovery/evidence.ts";
import { custodyProbe } from "../../tools/support/release/consumer_recovery/oracles.ts";
import {
  captureCommand,
  effectiveDenoConfig,
  publicEnvironment,
  strictUtf8,
  verifyOriginalLogDirectory,
} from "../../tools/support/release/consumer_recovery/runtime.ts";
const retained = JSON.parse(
  readFileSync(new URL("../_fixtures/release/consumer-recovery.json", import.meta.url), "utf8"),
);
const admit = (packet: typeof retained) =>
  admitOriginal(packet.identity, packet.run, packet.jobs, packet.failed, packet.log, packet.blobs);

test("actual immutable tag Integration-PR annotation keeps its # prefix while parsing one positive identity", () => {
  // Exact field from annotated tag 46536212e8e1a16492f9e196d84908286170d57b.
  const annotation = "Integration-PR: #9197";
  assert.equal(integrationReference(annotation.slice("Integration-PR: ".length), "9197"), 9197);
  assert.equal(annotation, "Integration-PR: #9197", "never rewrite retained annotation bytes");
  for (const value of [
    "9197",
    "#0",
    "#09197",
    "##9197",
    " #9197",
    "#9197\n",
    "#9197 trailing",
    "#9007199254740992",
  ])
    assert.throws(() => integrationReference(value, "9197"));
});

test("recovery policy binds the complete local import and materialized public harness closure", () => {
  const root = new URL("../../", import.meta.url);
  const declared = new Set<string>(policyFiles);
  const seen = new Set<string>();
  function visit(file: string) {
    assert.ok(declared.has(file), `unbound policy dependency: ${file}`);
    if (seen.has(file)) return;
    seen.add(file);
    const source = readFileSync(new URL(file, root), "utf8");
    const imports = [
      ...source.matchAll(
        /^\s*(?:import|export)\s+(?:type\s+)?(?:\{[^}]*\}|[\w* ,]+)\s+from\s*["'](\.[^"']+)["']/gm,
      ),
      ...source.matchAll(/\bimport\(\s*["'](\.[^"']+)["']/g),
    ];
    for (const match of imports)
      visit(path.posix.normalize(path.posix.join(path.posix.dirname(file), match[1])));
  }
  for (const file of policyFiles) if (file.endsWith(".ts")) visit(file);
  visit("tools/support/release/consumer_recovery/main.ts");
  visit("tools/support/release/consumer_recovery/consumer.ts");
  visit("tools/support/release/public_acceptance/accept.ts");
  for (const directory of [
    "tools/support/release/consumer_recovery/",
    "tools/support/release/public_acceptance/",
  ])
    for (const name of readdirSync(new URL(directory, root)).filter((name) => name.endsWith(".ts")))
      assert.ok(declared.has(directory + name), `materialized helper missing: ${name}`);
  assert.ok(declared.has("tools/support/release/jsr/consumer-deno.json"));
  assert.ok(declared.has("tools/support/release/marketplace_query.ts"));
});

test("native and public parents have no job-wide credential; actual child sees only runtime necessities", async (t) => {
  const workflow = YAML.parse(
    readFileSync(
      new URL("../../.github/workflows/release-public-acceptance.yml", import.meta.url),
      "utf8",
    ),
  );
  const job = workflow.jobs["public-consumer"];
  assert.equal(job.env.GH_TOKEN, undefined);
  assert.equal(job.env.GITHUB_TOKEN, undefined);
  const credentialSteps = job.steps.filter(
    (step: { env?: Record<string, string> }) => step.env?.GH_TOKEN,
  );
  assert.deepEqual(
    credentialSteps.map(
      (step: { run: string }) => /accept\.ts (prepare|finish)/.exec(step.run)?.[1],
    ),
    ["prepare", "finish"],
  );
  const parent = {
    ...process.env,
    GH_TOKEN: "marker-gh",
    GITHUB_TOKEN: "marker-github",
    ACTIONS_RUNTIME_TOKEN: "marker-actions",
    ACTIONS_ID_TOKEN_REQUEST_TOKEN: "marker-oidc",
    DENO_AUTH_TOKENS: "marker-deno",
    NODE_OPTIONS: "--require marker-hook",
    PRIVATE_CREDENTIAL: "marker-private",
  };
  const environment = publicEnvironment(parent);
  const directory = mkdtempSync(path.join(os.tmpdir(), "vize-recovery-environment-"));
  t.after(() => rmSync(directory, { recursive: true }));
  const observed = await captureCommand(
    process.execPath,
    [
      "-e",
      "process.stdout.write(JSON.stringify({ runtime: process.version, environment: process.env }))",
    ],
    path.join(directory, "child"),
    { env: environment },
  );
  assert.equal(observed.status, 0);
  assert.equal(observed.signal, null);
  assert.equal(observed.error, undefined);
  const packet = JSON.parse(strictUtf8(observed.stdout));
  assert.equal(packet.runtime, process.version);
  for (const key of [
    "GH_TOKEN",
    "GITHUB_TOKEN",
    "ACTIONS_RUNTIME_TOKEN",
    "ACTIONS_ID_TOKEN_REQUEST_TOKEN",
    "DENO_AUTH_TOKENS",
    "NODE_OPTIONS",
    "PRIVATE_CREDENTIAL",
  ])
    assert.equal(packet.environment[key], undefined, key);
  assert.doesNotMatch(strictUtf8(observed.stdout), /marker-/);
});

test("raw failure custody retains original ANSI and arbitrary bytes before status or decoding gates", async (t) => {
  const raw = Buffer.from(retained.log);
  assert.equal(raw.length, 24145);
  assert.equal(sha256(raw), "df3ec6ea97cd47a655a0fd46528a4e28f24ae3aac6fa256ff8b7e23a8e778860");
  assert.deepEqual(Buffer.from(strictUtf8(raw)), raw);
  const changed = structuredClone(retained);
  changed.log += "unannotated additional error";
  assert.throws(() => admit(changed), /complete original|bytes changed/);
  const directory = mkdtempSync(path.join(os.tmpdir(), "vize-recovery-raw-"));
  t.after(() => rmSync(directory, { recursive: true }));
  const child = await captureCommand(
    process.execPath,
    [
      "-e",
      "process.stdout.write(Buffer.from([0, 128, 255])); process.stderr.write('full stderr'); process.exitCode = 17;",
    ],
    path.join(directory, "negative"),
  );
  assert.equal(child.status, 17);
  assert.equal(child.signal, null);
  assert.deepEqual(
    readFileSync(path.join(directory, "negative/stdout.bin")),
    Buffer.from([0, 128, 255]),
  );
  assert.equal(readFileSync(path.join(directory, "negative/stderr.bin"), "utf8"), "full stderr");
  assert.throws(() => strictUtf8(child.stdout));
});

function completeDenoPacket() {
  const project = "/fresh",
    output = "/evidence";
  const policy = { sha: "a".repeat(40), ...fixture, custodyProbeSha256: sha256(custodyProbe) };
  const packet: Parameters<typeof commandPlan>[0] = {
    schema: "vize-consumer-recovery-consumer-v2",
    status: "passed",
    role: "deno-linux",
    identity: retained.identity,
    policy,
    consumerSha256: fixture.consumerSha256,
    denoConfigSha256: fixture.denoConfigSha256,
    effectiveDenoConfigSha256: sha256(effectiveDenoConfig),
    run: { sha: policy.sha },
    project,
    checkout: "/checkout",
    output,
    nodeExecutable: "/node/bin/node",
    runtime: { runtime: "deno", node: "24", platform: "linux" },
    commands: [],
    installed: {
      packages: [],
      native: { path: "/fresh/provider/vize.node", loaded: true, sha256: "a".repeat(64) },
      runtime: { node: "v24.14.0", deno: { deno: "2.9.7" }, platform: "linux", arch: "x64" },
    },
    packages: ["vize", "@vizejs/native", "@vizejs/native-linux-x64-gnu"].map((name) => {
      const content = JSON.stringify({ name, version: "0.457.0" });
      const entries = [
        { path: "package.json", sha256: sha256(content), bytes: Buffer.byteLength(content) },
        { path: "vize.node", sha256: "a".repeat(64), bytes: 1 },
      ];
      const integrity = "A".repeat(86) + "==";
      return {
        name,
        version: "0.457.0",
        source: retained.identity.head,
        tarball: "https://registry.npmjs.org/package.tgz",
        integrity: "sha512-" + integrity,
        archive: { sha512: integrity, sha256: "a".repeat(64), bytes: 1 },
        archiveEntries: entries,
        installedFiles: structuredClone(entries),
        archiveManifest: {
          name,
          version: "0.457.0",
          content,
          sha256: sha256(content),
          bytes: Buffer.byteLength(content),
        },
        manifestSha256: sha256(content),
        published: "2026-10-15T00:00:00Z",
        directory: name.includes("native-linux") ? "/fresh/provider" : "/fresh/package",
      };
    }),
    jsr: {},
  };
  packet.installed!.packages = packet.packages.map((record) => ({
    name: String(record.name),
    version: String(record.version),
    directory: String(record.directory),
    manifestSha256: String(record.manifestSha256),
  }));
  const oracle = {
    "consumer.ts": policy.consumerSha256,
    "custody.ts": policy.custodyProbeSha256,
    "deno.json": sha256(effectiveDenoConfig),
  };
  packet.commands = commandPlan(packet).map(([label, command, args]) => ({
    label,
    command,
    args,
    began: "2026-10-17T01:00:00Z",
    ended: "2026-10-17T01:00:01Z",
    cwd: project,
    status: 0,
    signal: null,
    stdout:
      label === "node-version"
        ? "v24.14.0\n"
        : label === "consumer"
          ? "Verified jsr:@vizejs/vize@0.457.0 (no flags)\n"
          : "",
    stderr: "legitimate retained stderr",
    denoDir: "/fresh/deno-cache",
    oracleBefore: { ...oracle },
    oracleAfter: { ...oracle },
  }));
  return packet;
}
test("zero exit cannot replace the exact whole H consumer completion or default-age host-only config", () => {
  const packet = completeDenoPacket();
  verifyConsumer(packet);
  assert.deepEqual(JSON.parse(effectiveDenoConfig), { nodeModulesDir: "none" });
  assert.equal(
    sha256(retained.blobs.denoConfigSha256 as string),
    fixture.denoConfigSha256,
    "retain original #9257 source policy",
  );
  for (const stdout of [
    "",
    "Verified jsr:@vizejs/vize@0.457.0 (no flags)",
    "Verified jsr:@vizejs/vize@0.457.0 (no flags)\nextra\n",
  ]) {
    const bad = structuredClone(packet);
    bad.commands.find((command) => command.label === "consumer")!.stdout = stdout;
    assert.throws(() => verifyConsumer(bad), /final witness/);
  }
  const bad = structuredClone(packet);
  bad.effectiveDenoConfigSha256 = sha256(retained.blobs.denoConfigSha256 as string);
  assert.throws(() => verifyConsumer(bad));
});

test(
  "timeout owns descendant pipes and retains an explicit failed process packet",
  { skip: process.platform === "win32" },
  async (t) => {
    const directory = mkdtempSync(path.join(os.tmpdir(), "vize-recovery-timeout-"));
    t.after(() => rmSync(directory, { recursive: true }));
    const began = performance.now();
    const observed = await captureCommand(
      process.execPath,
      [
        "-e",
        `
    require("node:child_process").spawn(process.execPath, ["-e", "setInterval(()=>{},1000)"], { stdio: "inherit" });
    process.stdout.write("retained-before-timeout");
    setInterval(()=>{},1000);
  `,
      ],
      path.join(directory, "capture"),
      { timeoutMs: 1_000 },
    );
    assert.ok(performance.now() - began < 5_000, "inherited pipes must not own an unbounded wait");
    assert.match(observed.error ?? "", /timed out/);
    assert.equal(strictUtf8(observed.stdout), "retained-before-timeout");
    assert.deepEqual(readFileSync(path.join(directory, "capture/stdout.bin")), observed.stdout);
    assert.deepEqual(
      JSON.parse(readFileSync(path.join(directory, "capture/process.json"), "utf8")),
      observed.custody,
    );
  },
);

test("retained raw bytes and process status cannot diverge from a consumer packet", (t) => {
  const directory = mkdtempSync(path.join(os.tmpdir(), "vize-recovery-raw-packet-"));
  t.after(() => rmSync(directory, { recursive: true }));
  const packet = completeDenoPacket();
  writeFileSync(path.join(directory, "consumer.json"), JSON.stringify(packet));
  writeFileSync(path.join(directory, "deno.json"), effectiveDenoConfig);
  for (const record of packet.commands) {
    const raw = path.join(directory, `${record.label}-raw`);
    mkdirSync(raw);
    writeFileSync(path.join(raw, "stdout.bin"), record.stdout);
    writeFileSync(path.join(raw, "stderr.bin"), record.stderr);
    writeFileSync(
      path.join(raw, "process.json"),
      JSON.stringify({
        command: record.command,
        args: record.args,
        cwd: record.cwd,
        status: record.status,
        signal: record.signal,
        stdout: { bytes: Buffer.byteLength(record.stdout), sha256: sha256(record.stdout) },
        stderr: { bytes: Buffer.byteLength(record.stderr), sha256: sha256(record.stderr) },
      }),
    );
  }
  const original = { identity: packet.identity, policy: packet.policy };
  verifyConsumerDirectory(directory, original);
  const raw = path.join(directory, "consumer-raw");
  writeFileSync(path.join(raw, "stdout.bin"), "forged");
  assert.throws(() => verifyConsumerDirectory(directory, original), /stdout changed/);
  writeFileSync(
    path.join(raw, "stdout.bin"),
    packet.commands.find((entry) => entry.label === "consumer")!.stdout,
  );
  const processFile = path.join(raw, "process.json");
  const process = JSON.parse(readFileSync(processFile, "utf8"));
  writeFileSync(processFile, JSON.stringify({ ...process, status: 17 }));
  assert.throws(() => verifyConsumerDirectory(directory, original), /status changed/);
});

test("actual frozen authorized v1 receipt and whole Deno packet retain their original semantics", () => {
  const frozen = JSON.parse(
    readFileSync(
      new URL("../_fixtures/release/consumer-recovery-legacy-v1.json", import.meta.url),
      "utf8",
    ),
  );
  for (const key of ["receipt", "deno"] as const) {
    assert.equal(Buffer.byteLength(frozen[`${key}Raw`]), frozen.provenance[key].bytes);
    assert.equal(sha256(frozen[`${key}Raw`]), frozen.provenance[key].sha256);
  }
  const receipt = JSON.parse(frozen.receiptRaw),
    deno = JSON.parse(frozen.denoRaw);
  const run = { id: 38993331275, run_attempt: 1, head_sha: legacyPolicySha };
  const now = Date.parse(receipt.completedAt);
  assert.equal(recoveryEvidencePolicy(run), "legacy-v1");
  assertRecoveryReceiptPolicy(receipt, run, receipt.identity, now);
  assert.deepEqual(receipt.authority, legacyPolicyAuthority);
  const packetFile = receipt.files.find(
    (entry: { file: string }) => entry.file === "consumers/deno-linux/consumer.json",
  );
  assert.equal(packetFile.sha256, sha256(frozen.denoRaw));
  assert.equal(packetFile.bytes, Buffer.byteLength(frozen.denoRaw));
  verifyLegacyConsumer(deno, { identity: receipt.identity, policy: deno.policy });
  assert.equal(
    deno.effectiveDenoConfigSha256,
    undefined,
    "historical v1 never earns the new policy",
  );
  assert.throws(() => verifyConsumer(deno));
  assert.throws(() =>
    assertRecoveryReceiptPolicy(receipt, { ...run, id: 38993331276 }, receipt.identity, now),
  );
  assert.throws(() =>
    assertRecoveryReceiptPolicy(receipt, { ...run, run_attempt: 2 }, receipt.identity, now),
  );
  assert.throws(() =>
    assertRecoveryReceiptPolicy(
      receipt,
      { ...run, head_sha: "a".repeat(40) },
      receipt.identity,
      now,
    ),
  );
  const changed = structuredClone(receipt);
  changed.recovery.policySha = "a".repeat(40);
  assert.throws(() => assertRecoveryReceiptPolicy(changed, run, receipt.identity, now));
  const altered = structuredClone(deno);
  altered.policy.authority[0].sha256 = "a".repeat(64);
  assert.throws(() => verifyLegacyConsumer(altered));
});

test("new provenance cannot downgrade v2 by relabeling or removing raw/config evidence", (t) => {
  const packet = completeDenoPacket();
  const downgraded = structuredClone(packet);
  downgraded.schema = "vize-consumer-recovery-consumer-v1";
  assert.throws(() => verifyConsumer(downgraded));
  assert.throws(() => verifyLegacyConsumer(downgraded), /legacy packet policy/);
  const incomplete = structuredClone(packet);
  delete (incomplete as Partial<typeof incomplete>).effectiveDenoConfigSha256;
  assert.throws(() => verifyConsumer(incomplete));
  const directory = mkdtempSync(path.join(os.tmpdir(), "vize-recovery-missing-raw-"));
  t.after(() => rmSync(directory, { recursive: true }));
  writeFileSync(path.join(directory, "consumer.json"), JSON.stringify(packet));
  assert.throws(
    () =>
      verifyConsumerDirectory(directory, {
        identity: packet.identity,
        policy: packet.policy,
      }),
    /ENOENT/,
  );
});

test("original failure transport cannot be resealed with mismatched raw streams", (t) => {
  const directory = mkdtempSync(path.join(os.tmpdir(), "vize-recovery-original-raw-"));
  t.after(() => rmSync(directory, { recursive: true }));
  const raw = path.join(directory, "original-log-capture");
  mkdirSync(raw);
  writeFileSync(path.join(raw, "stdout.bin"), retained.log);
  writeFileSync(path.join(raw, "stderr.bin"), "");
  const logCustody = {
    command: "gh",
    args: ["api", "original-job/logs"],
    status: 0,
    signal: null,
    stdout: { bytes: Buffer.byteLength(retained.log), sha256: sha256(retained.log) },
    stderr: { bytes: 0, sha256: sha256("") },
  };
  writeFileSync(
    path.join(raw, "process.json"),
    JSON.stringify({ ...logCustody, cwd: "/transport" }),
  );
  const original = { ...retained, logCustody } as Original;
  verifyOriginalLogDirectory(directory, "original-log-capture", original);
  writeFileSync(path.join(raw, "stderr.bin"), "unrecorded transport output");
  assert.throws(
    () => verifyOriginalLogDirectory(directory, "original-log-capture", original),
    /stderr digest/,
  );
});
