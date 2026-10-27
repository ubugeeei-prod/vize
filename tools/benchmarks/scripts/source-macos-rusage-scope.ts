/** Establish /usr/bin/time -l retired-instruction scope with original Rust controls. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parseArgs } from "node:util";
import { sourcePerfEnvironment } from "./source-operation-perf.ts";

export function retiredInstructions(stderr: string) {
  const raw = /^\s*(\d+)\s+instructions retired\s*$/m.exec(stderr)?.[1] ?? null;
  const count = raw === null ? null : Number(raw);
  return { raw, count: count !== null && Number.isSafeInteger(count) ? count : null };
}

export function inferInstructionScope(medians: Record<string, number | null>) {
  const { baseline, instructions, threads, child } = medians;
  const singleDelta = baseline !== null && instructions !== null ? instructions - baseline : null;
  const workerRatio =
    singleDelta && singleDelta > 0 && threads !== null && baseline !== null
      ? (threads - baseline) / singleDelta
      : null;
  const childRatio =
    singleDelta && singleDelta > 0 && child !== null && baseline !== null
      ? (child - baseline) / singleDelta
      : null;
  const scope = (ratio: number | null, expected: number) =>
    ratio === null
      ? "unavailable"
      : ratio >= expected * 0.75 && ratio <= expected * 1.25
        ? "included"
        : ratio >= 0 && ratio < 0.25
          ? "excluded"
          : "inconclusive";
  return {
    singleDelta,
    workerRatio,
    childRatio,
    workerThreads: scope(workerRatio, 2),
    waitedRustChild: scope(childRatio, 1),
    unvalidated: [
      "Node wrappers",
      "Go child processes",
      "unwaited or detached children",
      "product source-function attribution",
    ],
  };
}

export function captureMacInstructionScope(options: {
  out: string;
  rustc: string;
  rounds?: number;
  iterations?: number;
}) {
  assert.equal(process.platform, "darwin");
  const { out, rustc } = options;
  const rounds = options.rounds ?? 3,
    iterations = options.iterations ?? 200_000;
  assert.ok(Number.isSafeInteger(rounds) && rounds >= 1 && rounds <= 10);
  assert.ok(Number.isSafeInteger(iterations) && iterations >= 1 && iterations <= 100_000_000);
  assert.ok(!existsSync(out), "refuse to overwrite scope evidence");
  mkdirSync(out, { recursive: true });
  const source = fileURLToPath(new URL("../src/pmu_calibration.rs", import.meta.url));
  const binary = join(out, "pmu-calibration");
  const sha = (path: string) => createHash("sha256").update(readFileSync(path)).digest("hex");
  const sourceSha256 = sha(source);
  writeFileSync(join(out, "carrier.rs"), readFileSync(source));
  const args = [source, "-O", "-g", "--edition", "2024", "-o", binary];
  const version = spawnSync(rustc, ["--version", "--verbose"], { encoding: "utf8" });
  const built = spawnSync(rustc, args, { encoding: "utf8", timeout: 30_000 });
  writeFileSync(join(out, "compiler.stdout.txt"), built.stdout ?? "");
  writeFileSync(join(out, "compiler.stderr.txt"), built.stderr ?? "");
  assert.equal(built.status, 0, built.stderr);
  assert.equal(sha(source), sourceSha256, "carrier source must remain unchanged during build");
  const binarySha256 = sha(binary);
  const rows: {
    mode: string;
    round: number;
    instructions: number | null;
    packet: Record<string, unknown>;
    childPacket: unknown;
    stdout: string;
    stderr: string;
  }[] = [];
  const modes = ["baseline", "instructions", "threads", "child"];
  for (let round = 1; round <= rounds; round++) {
    for (const mode of round % 2 === 0 ? [...modes].reverse() : modes) {
      const command = ["-l", binary, mode, String(iterations), "64"];
      const executed = spawnSync("/usr/bin/time", command, {
        encoding: "utf8",
        timeout: 10_000,
        env: { ...process.env, LC_ALL: "C" },
      });
      const prefix = mode + "-" + round;
      writeFileSync(join(out, prefix + ".stdout.txt"), executed.stdout ?? "");
      writeFileSync(join(out, prefix + ".stderr.txt"), executed.stderr ?? "");
      assert.equal(executed.status, 0, executed.stderr);
      const packet = JSON.parse(executed.stdout);
      assert.equal(packet.schema, "vize-pmu-calibration");
      assert.equal(packet.completed, true);
      assert.equal(packet.mode, mode);
      assert.equal(packet.iterations, iterations);
      assert.equal(packet.bytes, 64);
      let childPacket: unknown = null;
      if (mode === "child") {
        assert.equal(packet.childExit, 0);
        assert.ok(Number.isSafeInteger(packet.childPid) && packet.childPid > 0);
        const rawChild = /^child_packet=(.*)$/m.exec(executed.stderr)?.[1];
        assert.ok(rawChild);
        childPacket = JSON.parse(rawChild);
        assert.equal((childPacket as Record<string, unknown>).mode, "instructions");
      }
      if (mode === "threads") assert.equal(packet.workerChecksums.length, 2);
      rows.push({
        mode,
        round,
        instructions: retiredInstructions(executed.stderr).count,
        packet,
        childPacket,
        stdout: prefix + ".stdout.txt",
        stderr: prefix + ".stderr.txt",
      });
    }
  }
  const referenceChecksums = new Set(
    rows.filter((row) => row.mode === "instructions").map((row) => row.packet.checksum),
  );
  assert.equal(referenceChecksums.size, 1, "single-process work must be deterministic");
  const checksum = [...referenceChecksums][0];
  for (const row of rows) {
    if (row.mode === "threads") assert.deepEqual(row.packet.workerChecksums, [checksum, checksum]);
    if (row.mode === "child") {
      assert.equal((row.childPacket as Record<string, unknown>).checksum, checksum);
      assert.equal((row.childPacket as Record<string, unknown>).completed, true);
      assert.equal((row.childPacket as Record<string, unknown>).iterations, iterations);
    }
  }
  const medians: Record<string, number | null> = {};
  for (const mode of modes) {
    const values = rows
      .filter((row) => row.mode === mode)
      .map((row) => row.instructions)
      .filter((value): value is number => value !== null)
      .sort((a, b) => a - b);
    medians[mode] = values.length === rounds ? values[Math.floor(values.length / 2)]! : null;
  }
  const result = {
    schemaVersion: 1,
    metric: "macOS time-l retired instructions",
    productSpeedClaim: false,
    environment: sourcePerfEnvironment(),
    compiler: { executable: rustc, args, version: version.stdout },
    sourceSha256,
    binarySha256,
    binarySha256After: sha(binary),
    rounds,
    iterations,
    rows,
    medians,
    inference: inferInstructionScope(medians),
    scope:
      "control-based inference for this time binary, OS and native Rust carrier only; driver/build excluded",
    simulation: false,
  };
  writeFileSync(join(out, "scope.json"), JSON.stringify(result, null, 2) + "\n");
  console.log(JSON.stringify({ medians, inference: result.inference, sourceSha256, binarySha256 }));
  return result;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      out: { type: "string" },
      rustc: { type: "string" },
      rounds: { type: "string", default: "3" },
      iterations: { type: "string", default: "200000" },
    },
  });
  assert.ok(values.out && values.rustc);
  captureMacInstructionScope({
    out: resolve(values.out),
    rustc: resolve(values.rustc),
    rounds: Number(values.rounds),
    iterations: Number(values.iterations),
  });
}
