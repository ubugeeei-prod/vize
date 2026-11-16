import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { availableParallelism } from "node:os";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";
import {
  CLI_ARGS,
  aggregateProfileCounts,
  assertPacketAuthority,
  assertProcess,
  assertProfilePacket,
  cleanEnvironment,
  inventory,
  makeCorpus,
  makeShardPlan,
  median,
  optimizedShardProfile,
  parseResourceUsage,
  sha256,
  shardPacket,
  writeShardProjects,
} from "./patina-project-batch-lib.mjs";

const [baseInput, headInput, baseSha, headSha, outputInput] = process.argv.slice(2);
assert.ok(
  baseInput && headInput && baseSha && headSha && outputInput,
  "usage: patina-project-batch.mjs BASE_BIN HEAD_BIN BASE_SHA HEAD_SHA OUTPUT_DIRECTORY",
);
for (const sha of [baseSha, headSha]) assert.match(sha, /^[a-f0-9]{40}$/u);
assert.notEqual(baseSha, headSha);
assert.equal(
  process.platform,
  "linux",
  "this paired lane requires the Linux GNU time resource observer",
);

const repository = fileURLToPath(new URL("../../../", import.meta.url));
const fixture = path.join(repository, "tools/benchmarks/fixtures/patina-project-batch");
const output = path.resolve(outputInput);
assert.ok(!fs.existsSync(output), "each benchmark must own a fresh evidence directory");
fs.mkdirSync(output, { recursive: true });
const consumer = path.join(output, "consumer");
const frozen = makeCorpus(fixture, consumer);
const shards = makeShardPlan(frozen);
const shardOrder = shards.map((shard) => shard.id);
const shardPlanSha256 = sha256(JSON.stringify(shards));
const shardProjects = writeShardProjects(consumer, frozen, shards);
const historicalPackets = new Map(shards.map((shard) => [shard.id, shardPacket(frozen, shard)]));
const basePackets = new Map();
let baseAggregate;
const referenceShardProfile = optimizedShardProfile(frozen.authority);
const inputPrefixes = [
  "projects",
  "node_modules",
  "package.json",
  "package-lock.json",
  "tsconfig.json",
];

const binaries = { base: fs.realpathSync(baseInput), head: fs.realpathSync(headInput) };
const corsa = fs.realpathSync(
  path.join(
    repository,
    "node_modules/@typescript",
    `typescript-${process.platform}-${process.arch}`,
    "lib/tsc",
  ),
);
const runtimePackage = JSON.parse(
  fs.readFileSync(path.join(path.dirname(corsa), "../package.json"), "utf8"),
);
assert.equal(
  runtimePackage.version,
  "7.0.2",
  "frozen request vectors require the captured checker runtime",
);
const env = cleanEnvironment(process.env, corsa);
const dependencyInstall = spawnSync("npm", ["ci", "--ignore-scripts", "--no-audit", "--no-fund"], {
  cwd: consumer,
  env,
  encoding: "utf8",
  timeout: 120_000,
});
fs.writeFileSync(
  path.join(output, "dependency-install.log"),
  `${dependencyInstall.stdout ?? ""}${dependencyInstall.stderr ?? ""}`,
);
assert.equal(dependencyInstall.error, undefined);
assert.equal(dependencyInstall.status, 0, dependencyInstall.stderr);
assert.equal(
  JSON.parse(fs.readFileSync(path.join(consumer, "node_modules/vue/package.json"), "utf8")).version,
  "3.5.44",
);

const inputs = inventory(consumer, inputPrefixes);
const binaryHashes = Object.fromEntries(
  Object.entries(binaries).map(([side, binary]) => [side, sha256(fs.readFileSync(binary))]),
);
const runtimeHash = sha256(fs.readFileSync(corsa));
const provenance = {
  baseSha,
  headSha,
  profile: "ci-opt-release-thin-lto-cgu16",
  node: process.version,
  platform: process.platform,
  arch: process.arch,
  availableParallelism: availableParallelism(),
  rayonThreads: "default unpinned",
  binaries,
  binaryHashes,
  checker: { path: corsa, version: runtimePackage.version, sha256: runtimeHash },
  authoritySha256: sha256(fs.readFileSync(path.join(fixture, "authority.json"))),
  inputInventorySha256: sha256(JSON.stringify(inputs)),
  shardPlanSha256,
  shardOrder,
  shardProjects,
};
fs.writeFileSync(path.join(output, "provenance.json"), `${JSON.stringify(provenance, null, 2)}\n`);
fs.writeFileSync(path.join(output, "shard-plan.json"), `${JSON.stringify(shards, null, 2)}\n`);
fs.writeFileSync(path.join(output, "historical-reference.stdout"), frozen.stdout);
fs.writeFileSync(path.join(output, "inputs-before.json"), `${JSON.stringify(inputs, null, 2)}\n`);
fs.writeFileSync(
  path.join(output, "authority.json"),
  fs.readFileSync(path.join(fixture, "authority.json")),
);

const samples = [];

function runShard(side, sampleLabel, shard, { profile = false, seal = false } = {}) {
  const label = `${sampleLabel}.${shard.id}`;
  const profileFile = path.join(output, `${label}.profile.json`);
  const resourceFile = path.join(output, `${label}.resource.txt`);
  const args = [...CLI_ARGS.slice(0, -1), shard.target];
  if (profile) args.push("--profile-json", profileFile);
  const started = performance.now();
  const result = spawnSync(
    "/usr/bin/time",
    ["-v", "-o", resourceFile, "--", binaries[side], ...args],
    {
      cwd: path.join(consumer, shard.directory),
      env,
      timeout: 180_000,
      maxBuffer: 16 * 1024 * 1024,
    },
  );
  const wallMs = performance.now() - started;
  fs.writeFileSync(path.join(output, `${label}.stderr`), result.stderr ?? "");
  try {
    const stdout = Buffer.isBuffer(result.stdout) ? result.stdout : Buffer.from(result.stdout);
    const expected = basePackets.get(shard.id);
    if (seal) {
      assert.equal(side, "base", "only the exact base warmup may seal packet authority");
      assert.equal(expected, undefined, `${shard.id} packet authority was already sealed`);
      assertProcess(result, stdout);
    } else {
      assert.ok(expected, `${shard.id} packet authority must be sealed before comparison`);
      assertProcess(result, expected.stdout);
    }
    const stdoutSha256 = sha256(stdout);
    const rows = JSON.parse(stdout.toString("utf8"));
    const historical = historicalPackets.get(shard.id);
    assert.deepEqual(
      rows.map((row) => row.file),
      historical.rows.map((row) => row.file),
      `${shard.id} must retain every immutable file exactly once and in order`,
    );
    const packet = {
      id: shard.id,
      rows,
      stdout,
      fileCount: rows.length,
      diagnosticCount: rows.reduce((sum, row) => sum + row.messages.length, 0),
      byteLength: stdout.length,
      stdoutSha256,
    };
    if (seal) basePackets.set(shard.id, packet);
    const resources = parseResourceUsage(fs.readFileSync(resourceFile, "utf8"));
    const observation = {
      id: shard.id,
      wallMs,
      ...resources,
      packet: {
        fileCount: packet.fileCount,
        diagnosticCount: packet.diagnosticCount,
        byteLength: packet.byteLength,
        stdoutSha256,
      },
    };
    if (profile) {
      const snapshot = JSON.parse(fs.readFileSync(profileFile, "utf8"));
      const counts = assertProfilePacket(snapshot);
      observation.profile = {
        spanCount: counts.length,
        countsSha256: sha256(JSON.stringify(counts)),
        counts,
      };
    }
    fs.appendFileSync(
      path.join(output, "shard-observations.jsonl"),
      `${JSON.stringify({
        schema: "vize.patina-project-batch-shard-observation",
        version: 1,
        side,
        sample: sampleLabel,
        observation,
      })}\n`,
    );
    return {
      observation,
      packet,
    };
  } catch (error) {
    fs.writeFileSync(path.join(output, `${label}.unexpected.stdout`), result.stdout ?? "");
    throw error;
  }
}

function runSample(side, label, { profile = false, seal = false } = {}) {
  const started = performance.now();
  const runs = shards.map((shard) => runShard(side, label, shard, { profile, seal }));
  const aggregate = assertPacketAuthority(
    shards,
    shards.map((shard) => basePackets.get(shard.id)),
    runs.map((run) => run.packet),
  );
  if (seal) {
    baseAggregate = aggregate;
    const shardPackets = shards.map((shard) => {
      const packet = basePackets.get(shard.id);
      return {
        id: packet.id,
        fileCount: packet.fileCount,
        diagnosticCount: packet.diagnosticCount,
        byteLength: packet.byteLength,
        stdoutSha256: packet.stdoutSha256,
      };
    });
    fs.writeFileSync(path.join(output, "complete-base.stdout"), aggregate.stdout);
    fs.writeFileSync(
      path.join(output, "base-packet-authority.json"),
      `${JSON.stringify(
        {
          schema: "vize.patina-project-batch-run-authority",
          version: 1,
          baseSha,
          shardPlanSha256,
          shardPackets,
          aggregate: {
            fileCount: aggregate.fileCount,
            diagnosticCount: aggregate.diagnosticCount,
            byteLength: aggregate.byteLength,
            stdoutSha256: aggregate.stdoutSha256,
          },
          historicalReference: {
            diagnosticCount: frozen.authority.diagnosticsPerCohort * frozen.authority.cohorts,
            byteLength: frozen.stdout.length,
            stdoutSha256: frozen.authority.expandedStdoutSha256,
            matchesBase: aggregate.stdout.equals(frozen.stdout),
          },
        },
        null,
        2,
      )}\n`,
    );
  }
  const row = {
    side,
    label,
    shardOrder,
    wallMs: runs.reduce((sum, run) => sum + run.observation.wallMs, 0),
    orchestratorWallMs: performance.now() - started,
    cpuMs: runs.reduce((sum, run) => sum + run.observation.cpuMs, 0),
    userCpuMs: runs.reduce((sum, run) => sum + run.observation.userCpuMs, 0),
    systemCpuMs: runs.reduce((sum, run) => sum + run.observation.systemCpuMs, 0),
    peakRssKiB: Math.max(...runs.map((run) => run.observation.maximumRssKiB)),
    packet: {
      fileCount: aggregate.fileCount,
      diagnosticCount: aggregate.diagnosticCount,
      byteLength: aggregate.byteLength,
      stdoutSha256: aggregate.stdoutSha256,
    },
    shards: runs.map((run) => run.observation),
  };
  if (profile) {
    const vectors = runs.map((run) => run.observation.profile.counts);
    for (const vector of vectors.slice(1))
      assert.deepEqual(
        vector,
        vectors[0],
        "identical frozen cohorts must produce identical checker request vectors",
      );
    const counts = aggregateProfileCounts(vectors);
    row.profile = {
      perShardSpanCount: vectors[0].length,
      perShardCountsSha256: sha256(JSON.stringify(vectors[0])),
      perShardCounts: vectors[0],
      spanCount: counts.length,
      countsSha256: sha256(JSON.stringify(counts)),
      counts,
    };
  }
  samples.push(row);
  fs.appendFileSync(path.join(output, "samples.jsonl"), `${JSON.stringify(row)}\n`);
  return row;
}

runSample("base", "base-warmup", { seal: true });
runSample("head", "head-warmup");
const pairs = [];
const pairedSamples = { base: [], head: [] };
for (let pair = 0; pair < 5; pair++) {
  const current = {};
  for (const side of pair % 2 === 0 ? ["base", "head"] : ["head", "base"]) {
    current[side] = runSample(side, `${side}-pair-${pair}`);
    pairedSamples[side].push(current[side]);
  }
  pairs.push({
    pair,
    baseWallMs: current.base.wallMs,
    headWallMs: current.head.wallMs,
    wallRatio: current.head.wallMs / current.base.wallMs,
    baseCpuMs: current.base.cpuMs,
    headCpuMs: current.head.cpuMs,
    cpuRatio: current.head.cpuMs / current.base.cpuMs,
    basePeakRssKiB: current.base.peakRssKiB,
    headPeakRssKiB: current.head.peakRssKiB,
  });
}

// Profiles are separate observations, outside every latency sample.
const profileSamples = Object.fromEntries(
  ["base", "head"].map((side) => [side, runSample(side, `${side}-profile`, { profile: true })]),
);

const after = inventory(consumer, inputPrefixes);
fs.writeFileSync(path.join(output, "inputs-after.json"), `${JSON.stringify(after, null, 2)}\n`);
assert.deepEqual(
  after,
  inputs,
  "authored corpus, shard projects, configuration or installed dependency bytes changed",
);
for (const side of ["base", "head"])
  assert.equal(sha256(fs.readFileSync(binaries[side])), binaryHashes[side]);
assert.equal(sha256(fs.readFileSync(corsa)), runtimeHash);

const wallRatio = median(pairs.map((pair) => pair.wallRatio));
const cpuRatio = median(pairs.map((pair) => pair.cpuRatio));
const rssRatio = median(pairs.map((pair) => pair.headPeakRssKiB / pair.basePeakRssKiB));
assert.ok(baseAggregate, "exact base packet authority must be sealed");
const basePacketSummaries = shards.map((shard) => {
  const packet = basePackets.get(shard.id);
  return {
    id: packet.id,
    fileCount: packet.fileCount,
    diagnosticCount: packet.diagnosticCount,
    byteLength: packet.byteLength,
    stdoutSha256: packet.stdoutSha256,
  };
});
const report = {
  schema: "vize.patina-project-batch-measurement",
  version: 3,
  baseSha,
  headSha,
  profile: "ci-opt-release-thin-lto-cgu16",
  platform: process.platform,
  arch: process.arch,
  availableParallelism: availableParallelism(),
  rayonThreads: "default unpinned",
  node: process.version,
  binaries: Object.fromEntries(
    Object.entries(binaries).map(([side, binary]) => [
      side,
      { binary, sha256: binaryHashes[side] },
    ]),
  ),
  checker: { path: corsa, version: runtimePackage.version, sha256: runtimeHash },
  files: frozen.authority.filesPerCohort * frozen.authority.cohorts,
  diagnostics: baseAggregate.diagnosticCount,
  completeStdoutSha256: baseAggregate.stdoutSha256,
  fixtureAuthoritySha256: provenance.authoritySha256,
  inputInventorySha256: provenance.inputInventorySha256,
  shardPlanSha256,
  shardOrder,
  shardPackets: basePacketSummaries,
  historicalReference: {
    diagnostics: frozen.authority.diagnosticsPerCohort * frozen.authority.cohorts,
    byteLength: frozen.stdout.length,
    stdoutSha256: frozen.authority.expandedStdoutSha256,
    matchesBase: baseAggregate.stdout.equals(frozen.stdout),
  },
  pairs,
  samples,
  baseMedianMs: median(pairs.map((pair) => pair.baseWallMs)),
  headMedianMs: median(pairs.map((pair) => pair.headWallMs)),
  pairedMedianRatio: wallRatio,
  changePercent: (wallRatio - 1) * 100,
  cpu: {
    baseMedianMs: median(pairs.map((pair) => pair.baseCpuMs)),
    headMedianMs: median(pairs.map((pair) => pair.headCpuMs)),
    pairedMedianRatio: cpuRatio,
  },
  peakRssKiB: {
    baseMedian: median(pairs.map((pair) => pair.basePeakRssKiB)),
    headMedian: median(pairs.map((pair) => pair.headPeakRssKiB)),
    pairedMedianRatio: rssRatio,
  },
  shardMedians: shards.map((shard) => ({
    id: shard.id,
    baseWallMs: median(
      pairedSamples.base.map(
        (sample) => sample.shards.find((entry) => entry.id === shard.id).wallMs,
      ),
    ),
    headWallMs: median(
      pairedSamples.head.map(
        (sample) => sample.shards.find((entry) => entry.id === shard.id).wallMs,
      ),
    ),
    baseCpuMs: median(
      pairedSamples.base.map(
        (sample) => sample.shards.find((entry) => entry.id === shard.id).cpuMs,
      ),
    ),
    headCpuMs: median(
      pairedSamples.head.map(
        (sample) => sample.shards.find((entry) => entry.id === shard.id).cpuMs,
      ),
    ),
    basePeakRssKiB: median(
      pairedSamples.base.map(
        (sample) => sample.shards.find((entry) => entry.id === shard.id).maximumRssKiB,
      ),
    ),
    headPeakRssKiB: median(
      pairedSamples.head.map(
        (sample) => sample.shards.find((entry) => entry.id === shard.id).maximumRssKiB,
      ),
    ),
  })),
  requestPhaseCounts: {
    optimizedReferencePerShard: referenceShardProfile,
    observed: Object.fromEntries(
      Object.entries(profileSamples).map(([side, sample]) => [side, sample.profile.counts]),
    ),
  },
  resourceScope:
    "Each shard is an isolated sequential GNU-time observation. Sample wall and CPU are sums; sample peak RSS is the maximum single-shard resident set, never a sum.",
  profileScope:
    "Untimed source-stage spans and exact checker RPC phase counts per shard and aggregate; transport includes IPC/queue/upstream work and is not exclusive worker CPU time.",
};
fs.writeFileSync(path.join(output, "measurement.json"), `${JSON.stringify(report, null, 2)}\n`);
fs.writeFileSync(
  path.join(output, "summary.md"),
  `## Type-aware project batch\n\nExact optimized source comparison, ${shards.length} deterministic shards, ${report.files} files and ${report.diagnostics} complete ordered diagnostics. The exact base warmup sealed aggregate packet ${report.completeStdoutSha256}; every later base/head sample matched its complete bytes, digest and count. Historical reference ${report.historicalReference.stdoutSha256} ${report.historicalReference.matchesBase ? "matches" : "does not match"} this base and is retained only as drift evidence.\n\n| Metric | Base | Head |\n| --- | ---: | ---: |\n| Median summed shard wall | ${report.baseMedianMs.toFixed(3)} ms | ${report.headMedianMs.toFixed(3)} ms |\n| Median summed shard CPU | ${report.cpu.baseMedianMs.toFixed(3)} ms | ${report.cpu.headMedianMs.toFixed(3)} ms |\n| Median peak shard RSS | ${report.peakRssKiB.baseMedian} KiB | ${report.peakRssKiB.headMedian} KiB |\n\nFive alternating pairs; paired median changes: wall ${report.changePercent.toFixed(2)}%, CPU ${((report.cpu.pairedMedianRatio - 1) * 100).toFixed(2)}%, peak RSS ${((report.peakRssKiB.pairedMedianRatio - 1) * 100).toFixed(2)}%. Every base/head sample traversed ${shardOrder.join(", ")} in the same order. Profiles are separate untimed observations with exact per-shard and aggregate request vectors.\n`,
);
console.log(
  JSON.stringify({
    output,
    baseMedianMs: report.baseMedianMs,
    headMedianMs: report.headMedianMs,
    pairedMedianRatio: wallRatio,
    cpu: report.cpu,
    peakRssKiB: report.peakRssKiB,
    completeStdoutSha256: report.completeStdoutSha256,
    historicalReference: report.historicalReference,
    shardPlanSha256,
  }),
);
assert.ok(
  wallRatio <= 1.05,
  `type-aware batch latency regressed ${report.changePercent.toFixed(2)}%, budget 5%`,
);
assert.ok(
  cpuRatio <= 1.05,
  `type-aware batch CPU regressed ${((cpuRatio - 1) * 100).toFixed(2)}%, budget 5%`,
);
assert.ok(
  rssRatio <= 1.05,
  `type-aware batch peak RSS regressed ${((rssRatio - 1) * 100).toFixed(2)}%, budget 5%`,
);
