import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  admitOriginal,
  assertFresh,
  consumerAttempt,
  failureLogArguments,
  object,
  roles,
  recoveryRunIdentity,
  reviewedPolicy,
  sha256,
  type Identity,
  type Json,
} from "../../tools/support/release/consumer_recovery/policy.ts";
import { publicInputs } from "../../tools/support/release/public_acceptance/inputs.ts";
import { verifyGithub } from "../../tools/support/release/public_acceptance/github.ts";
import YAML from "yaml";
import { commandPlan } from "../../tools/support/release/consumer_recovery/consumer.ts";
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
    "deno.json": String(retained.blobs.denoConfigSha256),
    "custody.ts": custodyProbe,
  };
  const policy = {
    consumerSha256: sha256(bytes["consumer.ts"]),
    denoConfigSha256: sha256(bytes["deno.json"]),
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
