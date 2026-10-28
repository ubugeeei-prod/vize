/** Calibrate observable hardware signals before admitting Vize instruction counts. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { parseArgs } from "node:util";
import { fileURLToPath, pathToFileURL } from "node:url";
import { captureSourceOperationPerf, sourcePerfEnvironment } from "./source-operation-perf.ts";
import { coreEvents, cacheEvents } from "./source-operation-perf-evidence.ts";

type Control = { name: string; group: "core" | "cache"; bytes: number };
const controls: Control[] = [
  { name: "baseline", group: "core", bytes: 64 },
  { name: "instructions", group: "core", bytes: 64 },
  { name: "branches", group: "core", bytes: 64 },
  { name: "cache-setup-small", group: "cache", bytes: 64 * 1024 },
  { name: "cache-small", group: "cache", bytes: 64 * 1024 },
  { name: "cache-setup-large", group: "cache", bytes: 64 * 1024 * 1024 },
  { name: "cache-large", group: "cache", bytes: 64 * 1024 * 1024 },
];

const digest = (file: string) => createHash("sha256").update(readFileSync(file)).digest("hex");

function buildCarrier(binary: string, out: string, compiler: string) {
  assert.ok(
    !existsSync(binary),
    "carrier must be a fresh compiler output, not an arbitrary supplied binary",
  );
  const source = fileURLToPath(new URL("../src/pmu_calibration.rs", import.meta.url));
  const sourceSha256 = digest(source);
  const args = ["--edition=2024", "-O", "-g", source, "-o", binary];
  const version = spawnSync(compiler, ["-Vv"], { encoding: "utf8", timeout: 30_000 });
  assert.equal(version.status, 0, "compiler identity probe failed");
  mkdirSync(dirname(binary), { recursive: true });
  const build = spawnSync(compiler, args, {
    encoding: "utf8",
    timeout: 120_000,
    killSignal: "SIGKILL",
    maxBuffer: 8 * 1024 * 1024,
  });
  writeFileSync(join(out, "build.stdout.txt"), build.stdout ?? "");
  writeFileSync(join(out, "build.stderr.txt"), build.stderr ?? "");
  assert.equal(build.status, 0, "carrier compilation failed; inspect build.stderr.txt");
  assert.equal(digest(source), sourceSha256, "carrier source changed while compiling");
  assert.ok(existsSync(binary) && statSync(binary).size > 0, "compiler did not emit a carrier");
  const receipt = {
    qualified: true,
    method: "fresh-source-compiler-output",
    source,
    sourceSha256,
    binary,
    binarySha256: digest(binary),
    binaryBytes: statSync(binary).size,
    compiler: { executable: compiler, args, version: version.stdout.trim() },
    scope:
      "carrier compilation outside all perf windows; compiler identity retained, not a Vize speed measurement",
  };
  writeFileSync(join(out, "build.json"), JSON.stringify(receipt, null, 2) + "\n");
  return receipt;
}

export function calibrateSourcePmu(options: {
  binary: string;
  perf: string;
  out: string;
  rounds?: number;
  iterations?: number;
  /** Test-only compiler injection; the workflow uses pinned rustc. */
  rustcExecutable?: string;
}) {
  const { binary, perf, out } = options;
  const rounds = options.rounds ?? 3;
  const iterations = options.iterations ?? 200_000;
  assert.ok(Number.isSafeInteger(rounds) && rounds >= 1 && rounds <= 10);
  assert.ok(Number.isSafeInteger(iterations) && iterations >= 1 && iterations <= 100_000_000);
  assert.ok(!existsSync(out), "refuse to overwrite PMU calibration");
  mkdirSync(out, { recursive: true });
  const environment = sourcePerfEnvironment();
  const build = buildCarrier(binary, out, options.rustcExecutable ?? "rustc");
  const { sourceSha256, binarySha256 } = build;
  const rows: {
    name: string;
    group: "core" | "cache";
    round: number;
    correct: boolean;
    reason: string | null;
    collection: string;
    schedulingQualified: boolean;
    plausible: boolean;
    counters: { event: string; value: number | null }[];
    checksum: string | null;
    receipt: string;
  }[] = [];
  for (let round = 1; round <= rounds; round++) {
    for (const control of controls) {
      const directory = `${control.name}-${round}`;
      const receipt = captureSourceOperationPerf({
        mode: "stat",
        perfExecutable: perf,
        eventGroup: control.group,
        outputDir: join(out, directory),
        command: {
          executable: binary,
          args: [control.name, String(iterations), String(control.bytes)],
          cwd: out,
          env: {},
        },
        identity: {
          calibration: true,
          sourceSha256,
          binarySha256,
          buildMethod: build.method,
          control: control.name,
          round,
          iterations,
          workingSetBytes: control.bytes,
          scope: "original Rust single process including its setup; no Node driver inside perf",
          simulation: false,
        },
      });
      let correct = false,
        checksum: string | null = null,
        reason: string | null = null;
      try {
        assert.equal(receipt.perf.status, 0);
        assert.equal(receipt.binarySha256, binarySha256, "control used a different carrier");
        assert.equal(receipt.binarySha256After, binarySha256, "control replaced its carrier");
        const packet = JSON.parse(readFileSync(join(out, directory, "stdout.txt"), "utf8"));
        assert.equal(packet.schema, "vize-pmu-calibration");
        assert.equal(packet.completed, true);
        assert.equal(packet.mode, control.name);
        assert.equal(packet.iterations, iterations);
        assert.equal(packet.bytes, control.bytes);
        if (control.group === "cache")
          assert.equal(
            packet.dependentLoads,
            control.name.startsWith("cache-setup") ? 0 : iterations,
          );
        assert.match(packet.checksum, /^[a-f0-9]{16}$/u);
        checksum = packet.checksum;
        correct = true;
      } catch (error) {
        reason = String(error);
      }
      rows.push({
        name: control.name,
        group: control.group,
        round,
        correct,
        checksum,
        reason,
        collection: receipt.collection,
        schedulingQualified: receipt.schedulingQualified,
        plausible: receipt.counterSanity.plausible,
        counters: receipt.counters,
        receipt: directory + "/receipt.json",
      });
    }
  }
  const median = (name: string, event: string) => {
    const values = rows
      .filter((row) => row.name === name)
      .map((row) => row.counters.find((counter) => counter.event === event)?.value)
      .filter((value): value is number => typeof value === "number")
      .sort((a, b) => a - b);
    return values.length === rounds ? values[Math.floor(values.length / 2)]! : null;
  };
  const groups = {
    core: { qualified: false, reasons: [] as string[] },
    cache: { qualified: false, reasons: [] as string[] },
  };
  for (const group of ["core", "cache"] as const) {
    const selected = rows.filter((row) => row.group === group);
    if (selected.some((row) => !row.correct))
      groups[group].reasons.push(
        "a calibration command did not complete its exact workload contract",
      );
    if (selected.some((row) => row.collection !== "collected" || !row.plausible))
      groups[group].reasons.push("missing or implausible hardware events");
    if (selected.some((row) => !row.schedulingQualified))
      groups[group].reasons.push(
        "multiplexed or unknown scheduling; strict counters require at least 99.9 percent running",
      );
    for (const name of new Set(selected.map((row) => row.name))) {
      if (
        new Set(selected.filter((row) => row.name === name).map((row) => row.checksum)).size !== 1
      )
        groups[group].reasons.push("workload checksum changed between fresh processes");
    }
    for (const event of group === "core" ? coreEvents : cacheEvents) {
      if (
        selected.some(
          (row) => row.counters.find((counter) => counter.event === event)?.value == null,
        )
      )
        groups[group].reasons.push("an event is unavailable: " + event);
    }
  }
  if (digest(binary) !== binarySha256 || digest(build.source) !== sourceSha256)
    for (const group of Object.values(groups))
      group.reasons.push("carrier source or binary changed after build");
  const cacheDelta = (size: "small" | "large", event: string) => {
    const workload = median("cache-" + size, event),
      setup = median("cache-setup-" + size, event);
    return workload === null || setup === null ? null : workload - setup;
  };
  const signals = {
    baselineInstructions: median("baseline", "instructions"),
    busyInstructions: median("instructions", "instructions"),
    baselineBranches: median("baseline", "branches"),
    busyBranches: median("branches", "branches"),
    smallCacheReferences: cacheDelta("small", "cache-references"),
    largeCacheReferences: cacheDelta("large", "cache-references"),
    smallCacheMisses: cacheDelta("small", "cache-misses"),
    largeCacheMisses: cacheDelta("large", "cache-misses"),
  };
  const hasTwofoldSignal = (busy: number | null, baseline: number | null) =>
    busy !== null && baseline !== null && busy > 2 * baseline;
  if (!hasTwofoldSignal(signals.busyInstructions, signals.baselineInstructions))
    groups.core.reasons.push("instruction positive control did not exceed baseline by twofold");
  if (!hasTwofoldSignal(signals.busyBranches, signals.baselineBranches))
    groups.core.reasons.push("branch positive control did not exceed baseline by twofold");
  if (
    !(
      signals.largeCacheReferences !== null &&
      signals.smallCacheReferences !== null &&
      signals.largeCacheMisses !== null &&
      signals.smallCacheMisses !== null &&
      signals.smallCacheReferences >= 0 &&
      signals.smallCacheMisses >= 0 &&
      signals.largeCacheReferences > signals.smallCacheReferences &&
      signals.largeCacheMisses > signals.smallCacheMisses
    )
  )
    groups.cache.reasons.push(
      "larger dependent loads did not exceed same-size setup-adjusted cache-event signal",
    );
  for (const group of Object.values(groups)) group.qualified = group.reasons.length === 0;
  const result = {
    schemaVersion: 1,
    environmentFingerprint: environment.fingerprint,
    environment,
    groups,
    qualified: groups.core.qualified && groups.cache.qualified,
    sourceSha256,
    binarySha256,
    build,
    signals,
    controls: rows,
    scope:
      "hardware-counter admission only; these controls are not Vize speed measurements or simulated Callgrind Ir",
    laws: {
      minimumRunningPercent: 99.9,
      minimumInstructionRatio: 2,
      minimumBranchRatio: 2,
      cacheSignal:
        "median(setup+dependent-loads) minus median(same-size setup-only); larger working set must add more references/misses; generic CPU-specific events, not exact per-load counts",
    },
  };
  writeFileSync(join(out, "calibration.json"), JSON.stringify(result, null, 2) + "\n");
  console.log(
    JSON.stringify({
      qualified: result.qualified,
      groups,
      environmentFingerprint: environment.fingerprint,
    }),
  );
  return result;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      binary: { type: "string" },
      perf: { type: "string" },
      out: { type: "string" },
      rounds: { type: "string", default: "3" },
      iterations: { type: "string", default: "200000" },
    },
  });
  assert.ok(values.binary && values.perf && values.out);
  calibrateSourcePmu({
    binary: resolve(values.binary),
    perf: resolve(values.perf),
    out: resolve(values.out),
    rounds: Number(values.rounds),
    iterations: Number(values.iterations),
  });
}
