import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  admitOriginal,
  annotatedIntegrationPr,
  fixture,
  sha256,
  assertFresh,
  consumerAttempt,
  failureLogArguments,
  object,
  roles,
  recoveryRunIdentity,
  reviewedPolicy,
  type Identity,
  type Json,
} from "../../tools/support/release/consumer_recovery/policy.ts";
import { publicInputs } from "../../tools/support/release/public_acceptance/inputs.ts";
import { verifyGithub } from "../../tools/support/release/public_acceptance/github.ts";
import YAML from "yaml";
import {
  commandPlan,
  verifyConsumer,
} from "../../tools/support/release/consumer_recovery/consumer.ts";
import { policyFiles } from "../../tools/support/release/consumer_recovery/custody.ts";
import {
  captureCommand,
  effectiveDenoConfig,
  publicEnvironment,
  strictUtf8,
} from "../../tools/support/release/consumer_recovery/runtime.ts";
import {
  assertOracleFiles,
  custodyProbe,
  verifyOracleHashes,
} from "../../tools/support/release/consumer_recovery/oracles.ts";

type Snapshot = {
  identity: Identity;
  run: Json;
  jobs: Json[];
  failed: Json;
  log: string;
  blobs: Json;
};
const retained: Snapshot = JSON.parse(
  readFileSync(new URL("../_fixtures/release/consumer-recovery.json", import.meta.url), "utf8"),
);
const admit = (packet: Snapshot) =>
  admitOriginal(packet.identity, packet.run, packet.jobs, packet.failed, packet.log, packet.blobs);

test("complete raw GitHub logs explicitly retain ANSI bytes before fingerprint normalization", () => {
  assert.deepEqual(failureLogArguments(117254411511), [
    "api",
    "--allow-escape-sequences",
    "repos/ubugeeei-prod/vize/actions/jobs/117254411511/logs",
  ]);
  assert.ok(
    retained.log.includes(String.fromCharCode(27)),
    "primary incident retains original escape bytes",
  );
  const before = Buffer.from(retained.log);
  admit(retained);
  assert.deepEqual(
    Buffer.from(retained.log),
    before,
    "fingerprint matching must not rewrite custody bytes",
  );
  for (const id of ["", "1;false", "001", -1]) assert.throws(() => failureLogArguments(id));
});

test("installation or invocation side effects and forged retained oracle claims refuse qualification", () => {
  const directory = mkdtempSync(path.join(os.tmpdir(), "vize-consumer-oracle-"));
  const bytes = {
    "consumer.ts": String(retained.blobs.consumerSha256),
    "deno.json": effectiveDenoConfig,
    "custody.ts": custodyProbe,
  };
  const policy = {
    consumerSha256: sha256(bytes["consumer.ts"]),
    denoConfigSha256: fixture.denoConfigSha256,
    custodyProbeSha256: sha256(custodyProbe),
  };
  try {
    for (const [name, source] of Object.entries(bytes))
      writeFileSync(path.join(directory, name), source);
    const before = assertOracleFiles(directory, policy, true);
    assert.doesNotThrow(() => verifyOracleHashes(before, policy, true));
    for (const [name, source] of Object.entries(bytes)) {
      writeFileSync(
        path.join(directory, name),
        source + "\n// installer or consumer changed oracle\n",
      );
      // Packet claims can remain identical to the expected hashes; actual retained bytes still refuse.
      assert.throws(() => assertOracleFiles(directory, policy, true));
      writeFileSync(path.join(directory, name), source);
    }
    assert.throws(() =>
      verifyOracleHashes({ ...before, "consumer.ts": "f".repeat(64) }, policy, true),
    );
    assert.deepEqual(assertOracleFiles(directory, policy, true), before);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("actual retained frozen source and 56 effective jobs admit only the known Deno fixture failure", () => {
  const admitted = admit(retained);
  assert.equal(admitted.failedJob, 117254411511);
  assert.deepEqual(admitted.identity, retained.identity);
  assert.equal(retained.run.conclusion, "failure", "admission must preserve original failure");
});

const mutations: Record<string, (packet: Snapshot) => void> = {
  "foreign R": (packet) => {
    packet.run.id = 1;
  },
  "foreign H": (packet) => {
    packet.run.head_sha = "a".repeat(40);
  },
  "foreign tag": (packet) => {
    packet.identity.tag = "v0.458.0";
  },
  "foreign source PR": (packet) => {
    packet.identity.sourcePr = "999";
  },
  "foreign repository": (packet) => {
    object(packet.run.repository).full_name = "foreign/vize";
  },
  "non-dispatch original": (packet) => {
    packet.run.event = "push";
  },
  "original became successful": (packet) => {
    packet.run.conclusion = "success";
  },
  "original still running": (packet) => {
    packet.run.status = "in_progress";
  },
  "stale original attempt": (packet) => {
    packet.run.run_attempt = 5;
  },
  "missing build": (packet) => {
    packet.jobs.pop();
  },
  "skipped build": (packet) => {
    packet.jobs[0].conclusion = "skipped";
  },
  "failed publication": (packet) => {
    packet.jobs.find((job) => job.name === "Release vize CLI to npm")!.conclusion = "failure";
  },
  "cancelled publication": (packet) => {
    packet.jobs.find((job) => job.name === "Create GitHub Release")!.conclusion = "cancelled";
  },
  "duplicate job": (packet) => {
    packet.jobs[1].id = packet.jobs[0].id;
  },
  "foreign job source": (packet) => {
    packet.jobs[0].head_sha = "b".repeat(40);
  },
  "foreign job R": (packet) => {
    packet.jobs[0].run_id = 1;
  },
  "future job attempt": (packet) => {
    packet.jobs[0].run_attempt = 5;
  },
  "old failed detail": (packet) => {
    packet.failed.run_attempt = 3;
  },
  "replaced failed detail": (packet) => {
    packet.failed.id = 1;
  },
  "other failed step": (packet) => {
    object((packet.failed.steps as unknown[])[3]).conclusion = "failure";
  },
  "different exception": (packet) => {
    packet.log = packet.log.replace(
      "Could not find a matching package",
      "Compile failed for package",
    );
  },
  "other logged error": (packet) => {
    packet.log += "\n##[error]unrelated failure\n";
  },
  "truncated outcome": (packet) => {
    packet.log = packet.log.replace("##[error]Process completed with exit code 1.", "");
  },
  "new frozen workflow": (packet) => {
    packet.blobs.workflowSha256 += "\n";
  },
  "new frozen release graph": (packet) => {
    packet.blobs.releaseWorkflowSha256 += "\n";
  },
  "weakened frozen consumer": (packet) => {
    packet.blobs.consumerSha256 += "\n";
  },
  "unreviewed corrected config": (packet) => {
    packet.blobs.denoConfigSha256 = '{"nodeModulesDir":"auto"}';
  },
};
for (const [label, mutate] of Object.entries(mutations)) {
  test(`recovery admission rejects ${label}`, () => {
    const packet = structuredClone(retained);
    mutate(packet);
    assert.throws(() => admit(packet));
  });
}

function receipt(): Json {
  return {
    schema: "vize-consumer-recovery-v1",
    identity: retained.identity,
    originalConclusion: "failure",
    status: "consumer-requalified",
    completedAt: "2026-10-17T01:00:00Z",
    consumers: roles.map((role) => ({ role })),
  };
}
const now = Date.parse("2026-10-17T02:00:00Z");
test("replacement requires all six distinct consumers and a fresh exact original identity", () => {
  assert.doesNotThrow(() => assertFresh(receipt(), retained.identity, now));
  for (const change of [
    (value: Json) => {
      value.completedAt = "2026-10-15T01:00:00Z";
    },
    (value: Json) => {
      value.completedAt = "2026-10-18T01:00:00Z";
    },
    (value: Json) => {
      value.status = "running";
    },
    (value: Json) => {
      value.originalConclusion = "success";
    },
    (value: Json) => {
      value.identity = { ...retained.identity, run: "1" };
    },
    (value: Json) => {
      value.identity = { ...retained.identity, attempt: 3 };
    },
    (value: Json) => {
      (value.consumers as Json[]).pop();
    },
    (value: Json) => {
      (value.consumers as Json[])[0] = { role: "deno-linux" };
    },
  ]) {
    const packet = receipt();
    change(packet);
    assert.throws(() => assertFresh(packet, retained.identity, now));
  }
});

test("replacement execution refuses foreign workflows, branches, attempts and reused consumer packets", () => {
  const run = {
    id: 123,
    repository: { full_name: "ubugeeei-prod/vize" },
    head_repository: { full_name: "ubugeeei-prod/vize" },
    event: "workflow_dispatch",
    path: ".github/workflows/release-consumer-recovery.yml",
    head_branch: "main",
    status: "completed",
    conclusion: "success",
    head_sha: "a".repeat(40),
    run_attempt: 2,
  };
  assert.doesNotThrow(() => recoveryRunIdentity(run, "123"));
  for (const replacement of [
    { id: 999 },
    { event: "pull_request" },
    { path: ".github/workflows/release.yml" },
    { head_branch: "unreviewed" },
    { status: "in_progress" },
    { conclusion: "failure" },
    { head_sha: "main" },
    { run_attempt: 0 },
    { repository: { full_name: "foreign/vize" } },
    { head_repository: { full_name: "foreign/vize" } },
  ])
    assert.throws(() => recoveryRunIdentity({ ...run, ...replacement }, "123"));
  const packet = { id: "123", attempt: "2", sha: run.head_sha };
  assert.doesNotThrow(() => consumerAttempt(packet, "123", 2, run.head_sha));
  for (const replacement of [{ id: "122" }, { attempt: "1" }, { sha: "b".repeat(40) }])
    assert.throws(() => consumerAttempt({ ...packet, ...replacement }, "123", 2, run.head_sha));
});

test("Unix and Windows replay their actual installed npm entry points and retain all original consumer assertions", () => {
  for (const role of roles) {
    const windows = role === "node24-windows";
    const packet = {
      role,
      identity: retained.identity,
      nodeExecutable: windows ? "C:\\node\\node.exe" : "/node/bin/node",
      project: windows ? "C:\\fresh" : "/fresh",
      output: windows ? "C:\\evidence" : "/evidence",
    } as Parameters<typeof commandPlan>[0];
    const commands = commandPlan(packet);
    if (role.startsWith("node")) {
      assert.equal(
        commands[1][2][0],
        windows
          ? "C:\\node\\node_modules\\npm\\bin\\npm-cli.js"
          : "/node/lib/node_modules/npm/bin/npm-cli.js",
      );
      assert.equal(
        commands[2][2][0],
        windows
          ? "C:\\node\\node_modules\\npm\\bin\\npx-cli.js"
          : "/node/lib/node_modules/npm/bin/npx-cli.js",
      );
    }
    const consumer = commands.find(([label]) => label === "consumer")!;
    assert.ok(consumer[2].includes("consumer.ts"));
    assert.equal(consumer[2].includes("--vite"), role !== "deno-linux");
    if (role === "deno-linux")
      assert.deepEqual(consumer.slice(1), [
        "deno",
        ["run", "--config", "deno.json", "--allow-all", "consumer.ts", "jsr:@vizejs/vize@0.457.0"],
      ]);
  }
});

test("unreviewed, foreign or unsigned replacement source cannot provide policy authority", () => {
  const sha = "a".repeat(40);
  const compare = {
    status: "ahead",
    behind_by: 0,
    base_commit: { sha },
    merge_base_commit: { sha },
  };
  const commit = { sha, commit: { verification: { verified: true, reason: "valid" } } };
  assert.doesNotThrow(() => reviewedPolicy(sha, compare, commit));
  for (const replacement of [
    { status: "diverged" },
    { behind_by: 1 },
    { base_commit: { sha: "b".repeat(40) } },
    { merge_base_commit: { sha: "b".repeat(40) } },
  ])
    assert.throws(() => reviewedPolicy(sha, { ...compare, ...replacement }, commit));
  for (const replacement of [
    { sha: "b".repeat(40) },
    { commit: { verification: { verified: false, reason: "unsigned" } } },
    { commit: { verification: { verified: true, reason: "invalid" } } },
  ])
    assert.throws(() => reviewedPolicy(sha, compare, { ...commit, ...replacement }));
});

test("normal installed acceptance still refuses failed R with no explicit signed recovery", async () => {
  const { identity: source } = retained;
  const sourcePr = {
    number: Number(source.sourcePr),
    head: {
      sha: source.head,
      ref: `release/${source.tag}`,
      repo: { full_name: "ubugeeei-prod/vize" },
    },
    base: { ref: "main", repo: { full_name: "ubugeeei-prod/vize" } },
    merged: false,
    merged_at: null,
    state: "open",
    body: `<!-- vize-release-pin: immutable-v1 -->\n<!-- vize-release-pin-cut: ${source.cut} -->\n<!-- vize-release-pin-head: ${source.head} -->\n<!-- vize-release-integration: 9197 -->`,
  };
  const annotation = `Release ${source.tag}\nRelease-Mode: pinned\nRelease-PR: #${source.sourcePr}\nValidated-run: ${source.run}\nValidated-head: ${source.head}\nValidated-base: ${source.cut}`;
  await assert.rejects(
    verifyGithub(source, async (url) => {
      if (url.includes("/git/ref/tags/"))
        return { ref: `refs/tags/${source.tag}`, object: { type: "tag", sha: "c".repeat(40) } };
      if (url.includes("/git/tags/"))
        return {
          sha: "c".repeat(40),
          tag: source.tag,
          object: { type: "commit", sha: source.head },
          message: annotation,
        };
      if (url.includes("/pulls/")) return sourcePr;
      if (url.endsWith(`/actions/runs/${source.run}`)) return retained.run;
      throw new Error(`Unexpected request ${url}`);
    }),
    /terminal own-repository exact-H/,
  );
});

test("recovery is an explicit read-only official operation; ordinary resume remains unchanged", () => {
  const file = (name: string) =>
    readFileSync(new URL(`../../.github/workflows/${name}`, import.meta.url), "utf8");
  const recovery = YAML.parse(file("release-consumer-recovery.yml"));
  assert.deepEqual(Object.keys(recovery.jobs).sort(), [
    "consumer",
    "contracts",
    "finalize",
    "prepare",
  ]);
  assert.equal(recovery.jobs.prepare.needs, "contracts");
  for (const job of Object.values(recovery.jobs)) {
    for (const step of object(job).steps as Record<string, unknown>[]) {
      if (String(step.uses).startsWith("actions/upload-artifact@"))
        assert.ok(
          String(object(step.with).name).includes("${{ github.run_attempt }}"),
          "every producer isolates its actual attempt",
        );
    }
  }
  assert.ok(
    recovery.jobs.finalize.steps.some(
      (step: { with?: { pattern?: string } }) =>
        step.with?.pattern === "consumer-recovery-*-${{ github.run_attempt }}",
    ),
  );
  assert.equal(recovery.jobs.consumer.strategy.matrix.include.length, 6);
  assert.equal(recovery.jobs.prepare.if, "github.ref == 'refs/heads/main'");
  const operator = YAML.parse(file("release-operator.yml"));
  assert.ok(operator.on.workflow_dispatch.inputs.operation.options.includes("recover-consumer"));
  assert.equal(operator.jobs.operator.environment, "release-operator");
  assert.match(file("release-operator.yml"), /vp run release --resume "\$RELEASE_SOURCE_PR" --pin/);
  const inputs = {
    PUBLIC_CUT: retained.identity.cut,
    PUBLIC_HEAD: retained.identity.head,
    PUBLIC_TAG: retained.identity.tag,
    PUBLIC_SOURCE_PR: retained.identity.sourcePr,
    PUBLIC_RELEASE_RUN: retained.identity.run,
  };
  assert.ok(!Object.hasOwn(publicInputs(inputs), "recoveryRun"));
  assert.throws(() => publicInputs({ ...inputs, PUBLIC_CONSUMER_RECOVERY_RUN: "not-a-run" }));
  assert.equal(publicInputs({ ...inputs, PUBLIC_CONSUMER_RECOVERY_RUN: "123" }).recoveryRun, "123");
});

test("actual immutable tag Integration-PR annotation keeps its # prefix while parsing one positive identity", () => {
  // Exact field from annotated tag 46536212e8e1a16492f9e196d84908286170d57b.
  const annotation = "Integration-PR: #9197";
  assert.equal(annotatedIntegrationPr(annotation.slice("Integration-PR: ".length)), 9197);
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
    assert.throws(() => annotatedIntegrationPr(value));
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
    schema: "vize-consumer-recovery-consumer-v1",
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
