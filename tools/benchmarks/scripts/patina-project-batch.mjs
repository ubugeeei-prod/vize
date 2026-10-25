import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";
import {
  CLI_ARGS,
  assertProcess,
  assertProfile,
  cleanEnvironment,
  inventory,
  makeCorpus,
  median,
  parseRss,
  sha256,
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
  "this paired lane requires the Linux GNU time RSS observer",
);
const repository = fileURLToPath(new URL("../../../", import.meta.url));
const fixture = path.join(repository, "tools/benchmarks/fixtures/patina-project-batch");
const output = path.resolve(outputInput);
assert.ok(!fs.existsSync(output), "each benchmark must own a fresh evidence directory");
fs.mkdirSync(output, { recursive: true });
const consumer = path.join(output, "consumer");
const frozen = makeCorpus(fixture, consumer);
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
const inputs = inventory(consumer);
const binaryHashes = Object.fromEntries(
  Object.entries(binaries).map(([side, binary]) => [side, sha256(fs.readFileSync(binary))]),
);
const runtimeHash = sha256(fs.readFileSync(corsa));
fs.writeFileSync(
  path.join(output, "provenance.json"),
  `${JSON.stringify(
    {
      baseSha,
      headSha,
      profile: "ci-opt-release-thin-lto-cgu16",
      node: process.version,
      platform: process.platform,
      arch: process.arch,
      binaries,
      binaryHashes,
      checker: { path: corsa, version: runtimePackage.version, sha256: runtimeHash },
      authoritySha256: sha256(fs.readFileSync(path.join(fixture, "authority.json"))),
      inputInventorySha256: sha256(JSON.stringify(inputs)),
    },
    null,
    2,
  )}\n`,
);
fs.writeFileSync(path.join(output, "complete-expected.stdout"), frozen.stdout);
fs.writeFileSync(path.join(output, "inputs-before.json"), `${JSON.stringify(inputs, null, 2)}\n`);
fs.writeFileSync(
  path.join(output, "authority.json"),
  fs.readFileSync(path.join(fixture, "authority.json")),
);
const samples = [];

function run(side, label, { profile = false, rss = false } = {}) {
  const args = [...CLI_ARGS];
  if (profile) args.push("--profile-json", path.join(output, `${label}.profile.json`));
  const rssFile = path.join(output, `${label}.rss.txt`);
  const executable = rss ? "/usr/bin/time" : binaries[side];
  const command = rss ? ["-v", "-o", rssFile, "--", binaries[side], ...args] : args;
  const started = performance.now();
  const result = spawnSync(executable, command, {
    cwd: consumer,
    env,
    timeout: 180_000,
    maxBuffer: 16 * 1024 * 1024,
  });
  const elapsedMs = performance.now() - started;
  fs.writeFileSync(path.join(output, `${label}.stderr`), result.stderr ?? "");
  try {
    const stdoutSha256 = assertProcess(result, frozen.stdout);
    const row = { side, label, elapsedMs, stdoutSha256 };
    if (profile) {
      const snapshot = JSON.parse(
        fs.readFileSync(path.join(output, `${label}.profile.json`), "utf8"),
      );
      assertProfile(snapshot, frozen.authority.requestPhaseCounts[side]);
      row.profile = `${label}.profile.json`;
    }
    if (rss) row.gnuTimeMaximumRssKiB = parseRss(fs.readFileSync(rssFile, "utf8"));
    samples.push(row);
    fs.appendFileSync(path.join(output, "samples.jsonl"), `${JSON.stringify(row)}\n`);
    return row;
  } catch (error) {
    fs.writeFileSync(path.join(output, `${label}.unexpected.stdout`), result.stdout ?? "");
    throw error;
  }
}

for (const side of ["base", "head"]) run(side, `${side}-warmup`);
const pairs = [];
for (let pair = 0; pair < 5; pair++) {
  const current = {};
  for (const side of pair % 2 === 0 ? ["base", "head"] : ["head", "base"])
    current[side] = run(side, `${side}-pair-${pair}`);
  pairs.push({
    pair,
    baseMs: current.base.elapsedMs,
    headMs: current.head.elapsedMs,
    ratio: current.head.elapsedMs / current.base.elapsedMs,
  });
}
// Profiles and RSS are separate observations, outside every latency sample.
for (let pair = 0; pair < 2; pair++) {
  for (const side of pair % 2 === 0 ? ["base", "head"] : ["head", "base"])
    run(side, `${side}-profile-${pair}`, { profile: true });
}
const rss = Object.fromEntries(
  ["base", "head"].map((side) => [
    side,
    run(side, `${side}-rss`, { rss: true }).gnuTimeMaximumRssKiB,
  ]),
);
const after = inventory(consumer);
fs.writeFileSync(path.join(output, "inputs-after.json"), `${JSON.stringify(after, null, 2)}\n`);
assert.deepEqual(
  after,
  inputs,
  "authored corpus, configuration or installed dependency bytes changed during measurement",
);
for (const side of ["base", "head"])
  assert.equal(sha256(fs.readFileSync(binaries[side])), binaryHashes[side]);
assert.equal(sha256(fs.readFileSync(corsa)), runtimeHash);
const ratio = median(pairs.map((pair) => pair.ratio));
const report = {
  schema: "vize.patina-project-batch-measurement",
  version: 1,
  baseSha,
  headSha,
  profile: "ci-opt-release-thin-lto-cgu16",
  platform: process.platform,
  arch: process.arch,
  node: process.version,
  binaries: Object.fromEntries(
    Object.entries(binaries).map(([side, binary]) => [
      side,
      { binary, sha256: binaryHashes[side] },
    ]),
  ),
  checker: { path: corsa, version: runtimePackage.version, sha256: runtimeHash },
  files: frozen.authority.filesPerCohort * frozen.authority.cohorts,
  diagnostics: frozen.authority.diagnosticsPerCohort * frozen.authority.cohorts,
  completeStdoutSha256: frozen.authority.expandedStdoutSha256,
  authoritySha256: sha256(fs.readFileSync(path.join(fixture, "authority.json"))),
  inputInventorySha256: sha256(JSON.stringify(inputs)),
  pairs,
  samples,
  baseMedianMs: median(pairs.map((pair) => pair.baseMs)),
  headMedianMs: median(pairs.map((pair) => pair.headMs)),
  pairedMedianRatio: ratio,
  changePercent: (ratio - 1) * 100,
  gnuTimeMaximumRssKiB: rss,
  rssScope:
    "Separate GNU time maximum resident set size observation; not a sampled sum of concurrent process RSS.",
  profileScope:
    "Untimed source-stage spans and exact checker RPC phase counts; transport includes IPC/queue/upstream work and is not exclusive worker CPU time.",
  requestPhaseCounts: frozen.authority.requestPhaseCounts,
};
fs.writeFileSync(path.join(output, "measurement.json"), `${JSON.stringify(report, null, 2)}\n`);
fs.writeFileSync(
  path.join(output, "summary.md"),
  `## Type-aware project batch\n\nExact optimized source comparison, ${report.files} files and ${report.diagnostics} complete ordered diagnostics.\n\n| | Base | Head |\n| --- | ---: | ---: |\n| Median latency | ${report.baseMedianMs.toFixed(3)} ms | ${report.headMedianMs.toFixed(3)} ms |\n| Snapshot updates | 1,211 | 2 |\n| GNU time peak RSS | ${rss.base} KiB | ${rss.head} KiB |\n\nFive alternating pairs; paired median change ${report.changePercent.toFixed(2)}%. Profiles and RSS are separate untimed observations. All samples match the frozen complete public output and retain unchanged input/dependency hashes.\n`,
);
console.log(
  JSON.stringify({
    output,
    baseMedianMs: report.baseMedianMs,
    headMedianMs: report.headMedianMs,
    pairedMedianRatio: ratio,
    rss,
  }),
);
assert.ok(
  ratio <= 1.05,
  `type-aware batch latency regressed ${report.changePercent.toFixed(2)}%, budget 5%`,
);
