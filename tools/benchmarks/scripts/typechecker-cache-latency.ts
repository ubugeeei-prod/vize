/** Shipping-profile cold samples: setup, oracle, hashing and cleanup are outside the checker window. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { parseArgs } from "node:util";
import {
  prepareColdScale,
  verifyColdDiagnostics,
  coldScaleSupportScriptFiles,
} from "./cold-typechecker-scale.ts";
import { coldScaleCommand } from "./cold-typechecker-scale-command.ts";
import { createColdScaleRuntime, qualifyColdScaleCache } from "./typechecker-cache-runtime.ts";
import { sealFixtureInputs } from "./typechecker-fixture-identity.ts";

const { values } = parseArgs({
  options: {
    provider: { type: "string" },
    binary: { type: "string" },
    out: { type: "string" },
    custody: { type: "string" },
    coverage: { type: "string" },
    sfcs: { type: "string", default: "2500" },
    rounds: { type: "string", default: "5" },
    role: { type: "string" },
    source: { type: "string" },
  },
});
assert.ok(
  values.provider &&
    values.binary &&
    values.out &&
    values.custody &&
    values.coverage &&
    values.role,
);
const provider = fs.realpathSync(values.provider),
  binary = fs.realpathSync(values.binary),
  out = path.resolve(values.out);
const sfcs = Number(values.sfcs),
  rounds = Number(values.rounds);
assert.ok(
  sfcs >= 2500 &&
    Number.isSafeInteger(sfcs) &&
    rounds >= 3 &&
    rounds <= 9 &&
    Number.isSafeInteger(rounds),
);
assert.ok(
  !fs.existsSync(path.join(out, "acceptance.json")),
  "retain previous completed role evidence",
);
fs.mkdirSync(out, { recursive: true });
const hash = (file: string) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const binarySha256 = hash(binary);
const coverage = JSON.parse(fs.readFileSync(values.coverage, "utf8"));
assert.equal(coverage.schema, "vize.source-all-sfc-admission");
assert.equal(coverage.binarySha256, binarySha256);
assert.equal(coverage.passed, true);
assert.equal(coverage.sfcs, sfcs);
assert.equal(coverage.mutated?.actualCount, 2 * sfcs + 1);
assert.equal(coverage.repaired?.actualCount, 0);
const samples = [];
for (let round = 1; round <= rounds; round++) {
  const directory = path.join(out, "sample-" + round);
  assert.ok(!fs.existsSync(directory), "retain previous samples");
  fs.mkdirSync(directory);
  const corpus = prepareColdScale(
    path.join(directory, "project"),
    provider,
    sfcs + coldScaleSupportScriptFiles,
    false,
  );
  const fixture = sealFixtureInputs(path.join(directory, "project"));
  assert.equal(
    fixture.sha256,
    coverage.fixtureIdentity.sha256,
    "the oracle must cover these exact authored inputs and options",
  );
  const { command, backend } = coldScaleCommand({
    provider,
    out: directory,
    binary,
    tool: "cli",
    app: corpus.app,
  });
  const backendSha256 = hash(backend);
  const runtime = createColdScaleRuntime(directory);
  const cache = qualifyColdScaleCache({
    receiptPath: values.custody,
    provider,
    runtime: runtime.evidence,
    executableSha256: binarySha256,
    backendSha256,
    corpusSha256: corpus.corpusSha256,
    fixtureSha256: fixture.sha256,
  });
  const environment: NodeJS.ProcessEnv = { ...process.env, ...command.env, ...runtime.env };
  delete environment.VIZE_LEXICAL_DOLLAR_CAPTURE_DIR;
  delete environment.VIZE_LEXICAL_DOLLAR_CAPTURE_METADATA_ONLY;
  delete environment.VIZE_CACHE_CUSTODY_CAPTURE_DIR;
  const resource = path.join(directory, "resource.txt");
  try {
    const start = performance.now();
    const result = spawnSync(
      "/usr/bin/time",
      ["-v", "-o", resource, command.executable, ...command.args],
      {
        cwd: command.cwd,
        env: environment,
        encoding: "utf8",
        timeout: 600_000,
        maxBuffer: 128 * 1024 * 1024,
      },
    );
    const wallMs = performance.now() - start;
    fs.writeFileSync(path.join(directory, "stdout.json"), result.stdout ?? "");
    fs.writeFileSync(path.join(directory, "stderr.txt"), result.stderr ?? "");
    assert.equal(hash(binary), binarySha256);
    assert.equal(hash(backend), backendSha256);
    assert.equal(sealFixtureInputs(path.join(directory, "project")).sha256, fixture.sha256);
    let correctness: { passed: boolean; error?: string };
    try {
      assert.equal(result.error, undefined);
      assert.equal(result.signal, null);
      assert.equal(result.status, 0);
      verifyColdDiagnostics(result.stdout ?? "", "cli", corpus.files, false);
      correctness = { passed: true };
    } catch (error) {
      correctness = { passed: false, error: String(error) };
    }
    const usage = fs.existsSync(resource) ? fs.readFileSync(resource, "utf8") : "";
    const maxProcessRssKB = usage.match(/Maximum resident set size \(kbytes\):\s*(\d+)/)?.[1];
    const sample = {
      round,
      wallMs,
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      correctness,
      cache,
      corpus,
      fixture,
      command,
      binarySha256,
      backendSha256,
      runtime: runtime.evidence,
      maxIndividualProcessRssBytes: maxProcessRssKB ? Number(maxProcessRssKB) * 1024 : null,
      memoryScope:
        "GNU time maximum individual process/waited child; not simultaneous Rust+Go tree RSS",
      cold: {
        newProcess: true,
        sourceConfigIncremental: false,
        effectiveBackendIncremental: true,
        nativeCache: cache.qualified
          ? "same-byte/config/corpus/context untimed FS custody plus fresh empty per-sample UUID"
          : "unqualified",
        osPageCache: "not flushed",
        daemon: false,
      },
      targetMs: 500,
      meetsTarget: wallMs <= 500 && cache.qualified && correctness.passed,
    };
    fs.writeFileSync(path.join(directory, "measurement.json"), JSON.stringify(sample, null, 2));
    samples.push(sample);
    console.log(
      JSON.stringify({
        round,
        wallMs,
        correctness: correctness.passed,
        cacheQualified: cache.qualified,
      }),
    );
  } finally {
    runtime.cleanup();
  }
}
const walls = samples.map((s) => s.wallMs).sort((a, b) => a - b);
const result = {
  schema: "vize.shipping-profile-cold",
  version: 1,
  role: values.role,
  source: values.source ?? "genuine-published-0.602.0",
  binarySha256,
  profile: values.source ? "source release fat-LTO/cgu1" : "genuine public release",
  sfcs,
  rounds,
  medianMs:
    walls.length % 2
      ? walls[Math.floor(walls.length / 2)]
      : (walls[walls.length / 2 - 1] + walls[walls.length / 2]) / 2,
  maxMs: walls.at(-1),
  passed: samples.every((s) => s.correctness.passed && s.cache.qualified),
  meetsFixture500ms: samples.every((s) => s.meetsTarget),
  primaryGoalQualified: false,
  fullFeatureAccuracyQualified: false,
  admissionScope:
    "supplemental public fixture only; independent full-feature/typed-query and private-primary admission remain required",
  machine: {
    platform: process.platform,
    arch: process.arch,
    cpu: os.cpus()[0]?.model,
    availableCpus: os.availableParallelism(),
  },
  coverage: values.coverage,
  custody: values.custody,
  samples,
  scope:
    "absolute fresh-process shipping-profile baseline; no PR-isolated attribution; OS page cache not flushed",
};
fs.writeFileSync(path.join(out, "acceptance.json"), JSON.stringify(result, null, 2));
console.log(
  JSON.stringify({
    role: result.role,
    medianMs: result.medianMs,
    maxMs: result.maxMs,
    passed: result.passed,
    meetsFixture500ms: result.meetsFixture500ms,
    primaryGoalQualified: result.primaryGoalQualified,
  }),
);
assert.equal(
  result.passed,
  true,
  "valid diagnostics and actual cache custody are required, independently of speed",
);
