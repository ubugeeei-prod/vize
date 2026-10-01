import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

type Timing = {
  pid: number;
  production: boolean;
  cache: "built" | "hit" | "bypassed";
  inputMs: number;
  inventoryMs: number;
  hashMs: number;
  abiMs: number;
  buildMs: number;
  bundleSha256: string;
  inputSha256: string | null;
  inputFileCount: number;
};

const directory = resolve(process.argv[2] ?? "target/runtime-bundle-comparison");
mkdirSync(directory, { recursive: true });
const command = [
  "test",
  "--profile",
  "ci",
  "--package",
  "vize_atelier_vapor",
  "--test",
  "davinci_transition_parity",
  "--",
  "controlled_transition_hooks_match_native_retained_and_official_runtime",
  "--exact",
  "--nocapture",
];
const summaries = [];
for (const production of [false, true]) {
  const runtime = production ? "production" : "development";
  const cache = join(directory, `${runtime}-cache`);
  let expectedBundle: string | null = null;
  for (const phase of ["disabled", "cold", "warm"]) {
    if (phase !== "warm") rmSync(cache, { recursive: true, force: true });
    const timings = join(directory, `${runtime}-${phase}.jsonl`);
    rmSync(timings, { force: true });
    const env = {
      ...process.env,
      VIZE_VUE_RUNTIME_PRODUCTION: production ? "1" : "0",
      VIZE_VUE_RUNTIME_BUNDLE_CACHE: phase === "disabled" ? "off" : "on",
      VIZE_VUE_RUNTIME_BUNDLE_DIRECTORY: cache,
      VIZE_VUE_RUNTIME_BUNDLE_TIMINGS: timings,
    };
    const started = performance.now();
    const result = spawnSync("cargo", command, {
      env,
      encoding: "utf8",
      maxBuffer: 8 * 1024 * 1024,
    });
    const elapsedMs = performance.now() - started;
    writeFileSync(
      join(directory, `${runtime}-${phase}.log`),
      `${result.stdout ?? ""}${result.stderr ?? ""}`,
    );
    assert.equal(result.error, undefined, `${runtime}/${phase}: test process failed to start`);
    assert.equal(
      result.status,
      0,
      `${runtime}/${phase}: actual parity comparisons failed; see retained log`,
    );
    assert.match(
      result.stdout,
      /test result: ok\. 1 passed; 0 failed;/,
      "must execute the unchanged parity test",
    );
    const records = readFileSync(timings, "utf8")
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line) as Timing);
    assert.equal(
      records.length,
      27,
      `${runtime}/${phase}: retain all nine scenarios and three lanes`,
    );
    assert.equal(
      new Set(records.map((record) => record.pid)).size,
      27,
      "every comparison owns a fresh process",
    );
    assert.ok(records.every((record) => record.production === production));
    const hashes = new Set(records.map((record) => record.bundleSha256));
    assert.equal(hashes.size, 1, "all comparisons use identical immutable runtime bytes");
    expectedBundle ??= records[0].bundleSha256;
    assert.equal(
      records[0].bundleSha256,
      expectedBundle,
      "reuse must retain disabled-mode bundle bytes",
    );
    if (phase === "disabled")
      assert.ok(records.every((record) => record.cache === "bypassed" && record.buildMs > 0));
    else {
      assert.equal(
        new Set(records.map((record) => record.inputSha256)).size,
        1,
        "every process validates the same complete source identity",
      );
      assert.ok(records.every((record) => record.inputSha256 && record.inputFileCount > 0));
      assert.equal(
        records.filter((record) => record.cache === "built").length,
        phase === "cold" ? 1 : 0,
      );
      assert.equal(
        records.filter((record) => record.cache === "hit").length,
        phase === "cold" ? 26 : 27,
      );
      assert.ok(
        records.filter((record) => record.cache === "hit").every((record) => record.buildMs === 0),
      );
    }
    const summary = {
      runtime,
      phase,
      elapsedMs,
      comparisons: records.length,
      freshProcesses: 27,
      builds: records.filter((record) => record.buildMs > 0).length,
      fingerprintMs: records.reduce((sum, record) => sum + record.inputMs, 0),
      inventoryMs: records.reduce((sum, record) => sum + record.inventoryMs, 0),
      hashMs: records.reduce((sum, record) => sum + record.hashMs, 0),
      abiMs: records.reduce((sum, record) => sum + record.abiMs, 0),
      inputFiles: records[0].inputFileCount,
      bundlingMs: records.reduce((sum, record) => sum + record.buildMs, 0),
      bundleSha256: expectedBundle,
    };
    summaries.push(summary);
    process.stdout.write(`${JSON.stringify(summary)}\n`);
  }
}
writeFileSync(
  join(directory, "summary.json"),
  `${JSON.stringify({ source: process.env.GITHUB_SHA ?? "local", command: ["cargo", ...command], summaries }, null, 2)}\n`,
);
