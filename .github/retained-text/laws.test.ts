import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import {
  base,
  candidate,
  candidatePass,
  cliArtifact,
  exactUnits,
  fixtureRevision,
  inspectMetrics,
  laneCeilings,
  lanes,
  object,
  order,
  postPassed,
  report,
  unitNames,
  verifyGuardRoster,
  type Json,
} from "./laws.ts";

const prefix = "file:///";
function metrics(): Json {
  return {
    schemaVersion: 3,
    commit: candidate,
    fixtureRevision,
    status: "passed",
    failure: null,
    budget: {
      suite: "vben-lsp-incremental",
      scale: 1,
      laneHardTimeoutMs: 60000,
      maxPeakRssMiB: 256,
      maxPeakProcessTreeRssMiB: 512,
      maxProcessTreeSize: 4,
      laneBudgetsMs: Object.fromEntries(lanes.map((name, i) => [name, laneCeilings[i]])),
    },
    timingsMs: Object.fromEntries(lanes.map((name) => [name, 1])),
    corpus: { vueFiles: 500, vueAndTypeScriptFiles: 900, baselineDiagnostics: 0 },
    processTreeSamples: {
      initialize: {
        processes: 1,
        totalKiB: 512,
        members: [
          {
            pid: 42,
            parentPid: 40,
            rssKiB: 512,
            role: "root",
            identity: { birthTicks: "123", executableSha256: "a".repeat(64) },
          },
        ],
      },
    },
    sampledPeakRssKiB: 512,
    sampledPeakProcessTreeRssKiB: 512,
    sampledPeakProcessTreeSize: 1,
  };
}
function artifact(): Json {
  return {
    reason: "compiler-artifact",
    target: { name: "vize", kind: ["bin"] },
    executable: prefix + "target/ci/vize",
    features: ["default", "glyph", "legacy", "maestro"],
    profile: { test: false, opt_level: "0", debuginfo: 0, debug_assertions: true },
  };
}
function cargo(row: Json): string {
  return JSON.stringify(row) + '\n{"reason":"build-finished","success":true}\n';
}
const completed = { status: 0, signal: null, error: null };
const unitOutput = () =>
  "running 7 tests\n" +
  unitNames.map((name) => `test ${name} ... ok\n`).join("") +
  "test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 123 filtered out; finished in 0.01s\n";

test("predeclared serial order is the literal two-role B/C/C/B sequence", () => {
  assert.deepEqual(order, ["B", "C", "C", "B"]);
  assert.notEqual(base, candidate);
});
test("complete fourteen-lane/member receipts are retained without changing original results", () => {
  assert.equal(inspectMetrics(metrics(), candidate).status, "passed");
  const failed = metrics();
  failed.status = "failed";
  failed.failure = "original tree budget failure";
  assert.equal(inspectMetrics(failed, candidate).status, "failed");
  assert.throws(() => candidatePass(completed, failed));
  assert.throws(() => candidatePass({ ...completed, status: 1 }, metrics()));
  assert.throws(() => candidatePass({ ...completed, signal: "SIGTERM" }, metrics()));
  for (const [field, value] of [
    ["sampledPeakRssKiB", 256 * 1024 + 1],
    ["sampledPeakProcessTreeRssKiB", 512 * 1024 + 1],
    ["sampledPeakProcessTreeSize", 5],
  ] as const) {
    const over = metrics();
    over[field] = value;
    assert.throws(() => candidatePass(completed, over));
  }
  const late = metrics();
  object(late.timingsMs).cancellationConverge = 6001;
  assert.throws(() => candidatePass(completed, late));
});
test("missing, reordered, extra lanes and budget/source/corpus mutations refuse", () => {
  const mutations: ((data: Json) => void)[] = [
    (data) => {
      delete object(data.timingsMs).hover;
    },
    (data) => {
      data.timingsMs = Object.fromEntries([...lanes].reverse().map((name) => [name, 1]));
    },
    (data) => {
      object(data.timingsMs).invented = 1;
    },
    (data) => {
      object(data.budget).scale = 2;
    },
    (data) => {
      object(data.budget).maxPeakProcessTreeRssMiB = 768;
    },
    (data) => {
      object(data.budget).maxProcessTreeSize = 5;
    },
    (data) => {
      data.commit = base;
    },
    (data) => {
      data.fixtureRevision = base;
    },
    (data) => {
      object(data.corpus).vueFiles = 499;
    },
    (data) => {
      object(data.timingsMs).hover = Number.NaN;
    },
    (data) => {
      object(object(data.processTreeSamples).initialize).totalKiB = 1;
    },
    (data) => {
      object(object(data.processTreeSamples).initialize).members = [];
    },
  ];
  for (const change of mutations) {
    const data = metrics();
    change(data);
    assert.throws(() => inspectMetrics(data, candidate));
  }
});
test("candidate-only seven controls require every exact FQN and sole no-ignore success", () => {
  exactUnits(unitOutput());
  for (const output of [
    "running 0 tests\n",
    unitOutput().replace(unitNames[0], "unrelated"),
    unitOutput().replace("0 ignored", "1 ignored"),
    unitOutput() + `test ${unitNames[0]} ... ok\n`,
    unitOutput().replace("7 passed", "6 passed"),
    unitOutput().replace("running 7 tests", "running 0 tests"),
  ]) {
    assert.throws(() => exactUnits(output));
  }
});
test("Cargo JSON distinguishes first legacy-ci CLI from profiling/library contexts", () => {
  cliArtifact(cargo(artifact()));
  const profiled = artifact();
  profiled.features = [...(profiled.features as string[]), "profiling"];
  assert.throws(() => cliArtifact(cargo(profiled)));
  const unit = artifact();
  object(unit.profile).test = true;
  assert.throws(() => cliArtifact(cargo(unit)));
  assert.throws(() => cliArtifact(cargo(artifact()) + JSON.stringify(artifact())));
  assert.throws(() => cliArtifact(cargo(artifact()).replace('"success":true', '"success":false')));
});
test("frozen post-measurement command roster preserves all original ordered commands", () => {
  const original =
    process.env.RETAINED_ORIGINAL_WORKFLOW ??
    fileURLToPath(
      new URL("../../baseline/.github/workflows/lsp-config-diagnostics.yml", import.meta.url),
    );
  const raw = fs.readFileSync(original, "utf8");
  const manifest = JSON.parse(fs.readFileSync(new URL("./manifest.json", import.meta.url), "utf8"));
  assert.equal(createHash("sha256").update(raw).digest("hex"), manifest.originalWorkflowSha256);
  const guards = JSON.parse(
    fs.readFileSync(new URL("./post-guards.json", import.meta.url), "utf8"),
  );
  assert.equal(guards.commands.length, 16);
  let previous = -1;
  for (const row of guards.commands as { argv: string[]; retain?: string }[]) {
    const index = raw.indexOf(row.argv.join(" "));
    assert.ok(index > previous, row.argv.join(" "));
    previous = index;
    if (row.retain) assert.ok(raw.includes(row.retain));
  }
  assert.equal(
    guards.commands.filter((row: { argv: string[] }) => row.argv.includes("profiling")).length,
    1,
  );
  assert.equal(
    manifest.sources.filter((row: { B: string | null; C: string | null }) => row.B !== row.C)
      .length,
    6,
  );
  verifyGuardRoster(guards, raw);
  for (const change of [
    (rows: Json[]) => {
      rows.pop();
    },
    (rows: Json[]) => {
      rows.push({ ...rows[0] });
    },
    (rows: Json[]) => {
      rows[1].argv = ["cargo", "test"];
    },
    (rows: Json[]) => {
      rows[3].requireRust = false;
    },
    (rows: Json[]) => {
      rows[15].retain = "--retain-completion-producer";
    },
  ]) {
    const mutant = structuredClone(guards);
    change(mutant.commands);
    assert.throws(() => verifyGuardRoster(mutant, raw));
  }
  const results = guards.commands.map((row: Json) => ({ label: row.label, passed: true }));
  const labels = results.map((row: Json) => String(row.label));
  assert.equal(postPassed(results, labels), true);
  results[15].passed = false;
  assert.equal(postPassed(results, labels), false);
  assert.throws(() => postPassed(results.slice(0, -1), labels));
});
test("report retains four independent failed/passed statuses and all fourteen raw lane columns", () => {
  const data = metrics();
  data.status = "failed";
  const text = report(
    order.map((role, i) => ({
      label: `${i + 1}-${role}`,
      role,
      metrics: i === 0 ? data : metrics(),
    })),
    false,
  );
  assert.ok(text.includes("1-B | failed"));
  assert.ok(text.includes("2-C | passed"));
  for (const lane of lanes) assert.ok(text.includes(`| ${lane} |`));
  assert.ok(text.includes("Campaign admission: false"));
});
