import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

type Evidence = {
  pid: number;
  production: boolean;
  observer: boolean;
  events: Record<string, number>;
  initialized: number;
  unmounted: number;
  runtimeVersion: string | null;
  appRecords: number;
  diagnostics: unknown[];
  traceSha256: string;
};

const directory = resolve(process.argv[2] ?? "target/mounted-runtime-devtools-comparison");
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
const pids = new Set<number>();
for (const production of [false, true]) {
  const runtime = production ? "production" : "development";
  let baseline: string[] | undefined;
  for (const enabled of [false, true]) {
    const phase = enabled ? "observer" : "baseline";
    const evidence = join(directory, `${runtime}-${phase}.jsonl`);
    rmSync(evidence, { force: true });
    const started = performance.now();
    const result = spawnSync("cargo", command, {
      env: {
        ...process.env,
        VIZE_VUE_RUNTIME_PRODUCTION: production ? "1" : "0",
        VIZE_VUE_RUNTIME_DEVTOOLS_OBSERVER: enabled ? "on" : "off",
        VIZE_VUE_RUNTIME_TRACE_EVIDENCE: evidence,
      },
      encoding: "utf8",
      maxBuffer: 8 * 1024 * 1024,
    });
    const elapsedMs = performance.now() - started;
    writeFileSync(
      join(directory, `${runtime}-${phase}.log`),
      `${result.stdout ?? ""}${result.stderr ?? ""}`,
    );
    assert.equal(result.error, undefined, "the unchanged parity test must start");
    assert.equal(result.status, 0, `${runtime}/${phase}: actual parity failed; see retained log`);
    assert.match(result.stdout, /test result: ok\. 1 passed; 0 failed;/);
    const records = readFileSync(evidence, "utf8")
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line) as Evidence);
    assert.equal(records.length, 27, "retain all nine scenarios and all three backend lanes");
    for (const record of records) {
      assert.equal(pids.has(record.pid), false, "every comparison must own a fresh process");
      pids.add(record.pid);
      assert.equal(record.production, production);
      assert.equal(record.observer, enabled && !production);
      assert.equal(record.initialized, enabled && !production ? 1 : 0);
      assert.equal(record.unmounted, enabled && !production ? 1 : 0);
      assert.equal(record.events["app:init"] ?? 0, enabled && !production ? 1 : 0);
      assert.equal(record.events["app:unmount"] ?? 0, enabled && !production ? 1 : 0);
      assert.equal(record.runtimeVersion, enabled && !production ? "3.6.0-rc.9" : null);
      assert.equal(record.appRecords, 0);
      assert.deepEqual(record.diagnostics, []);
      assert.match(record.traceSha256, /^[a-f0-9]{64}$/u);
    }
    const hashes = records.map(({ traceSha256 }) => traceSha256);
    if (baseline)
      assert.deepEqual(hashes, baseline, "every complete trace must equal its baseline bytes");
    else baseline = hashes;
    const summary = {
      runtime,
      phase,
      elapsedMs,
      comparisons: records.length,
      freshProcesses: 27,
      observedLifecycles: records.filter(({ observer }) => observer).length,
    };
    summaries.push(summary);
    process.stdout.write(`${JSON.stringify(summary)}\n`);
  }
}
assert.equal(pids.size, 108);
writeFileSync(
  join(directory, "summary.json"),
  `${JSON.stringify(
    { source: process.env.GITHUB_SHA ?? "local", command: ["cargo", ...command], summaries },
    null,
    2,
  )}\n`,
);
