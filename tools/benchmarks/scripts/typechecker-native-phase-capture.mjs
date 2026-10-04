#!/usr/bin/env node
/** Untimed transparent native-command capture/replay; never used in production. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { performance } from "node:perf_hooks";
import { createNativeGraphArchive } from "./typechecker-native-graph-archive.mjs";
import { splitNativePhaseReport } from "./typechecker-native-phase-report.mjs";
import { replayNativeProfile } from "./typechecker-native-profile-replay.mjs";

function main() {
  const settings = JSON.parse(readFileSync(process.env.NATIVE_PHASE_CONFIG, "utf8"));
  assert([undefined, null, "pprof"].includes(settings.profile), "unsupported native profile mode");
  const args = process.argv.slice(2);
  const projectIndex = args.indexOf("--project");
  if (args.length === 1 && args[0] === "--version") {
    const version = spawnSync(settings.runtime, args, {
      timeout: 300_000,
      maxBuffer: 64 * 1024 * 1024,
    });
    assert.equal(version.error, undefined, version.error?.message);
    assert.equal(version.signal, null);
    process.stdout.write(version.stdout ?? "");
    process.stderr.write(version.stderr ?? "");
    process.exitCode = version.status ?? 1;
    return;
  }
  assert(projectIndex >= 0 && args[projectIndex + 1], "unexpected native invocation/fallback");
  if (settings.profile === "pprof")
    assert.deepEqual(
      args,
      ["--checkers", "1", "--pretty", "false", "--project", args[projectIndex + 1]],
      "profile replay requires the exact known production native argv",
    );
  assert(
    !args.some((arg) =>
      /incremental|tsBuildInfo|extendedDiagnostics|pprofDir|generateTrace/u.test(arg),
    ),
  );
  const config = resolve(args[projectIndex + 1]);
  const stem = join(settings.directory, `${process.pid}`);
  mkdirSync(settings.directory, { recursive: true });
  const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
  const configBytes = readFileSync(config);
  const runtimeSha256 = sha(readFileSync(settings.runtime));
  writeFileSync(`${stem}.tsconfig.json`, configBytes);
  const compilerOptions = JSON.parse(configBytes).compilerOptions;
  const graph = createNativeGraphArchive({
    directory: settings.directory,
    stem,
    cwd: process.cwd(),
    config,
    configBytes,
    runtime: settings.runtime,
    args,
    objectsDirectory: join(dirname(settings.directory), "graph-objects"),
  });
  const runs = {};
  function run(id, extra = []) {
    const startedAt = performance.timeOrigin + performance.now();
    const start = performance.now();
    const result = spawnSync(settings.runtime, [...args, ...extra], {
      cwd: process.cwd(),
      timeout: 300_000,
      maxBuffer: 64 * 1024 * 1024,
    });
    runs[id] = {
      startedAt,
      ms: performance.now() - start,
      status: result.status,
      instrumentationOnly: true,
      contention: "other shard checks or phase replays may be concurrent",
      signal: result.signal,
      error: result.error?.message ?? null,
      command: [settings.runtime, ...args, ...extra],
    };
    writeFileSync(`${stem}.${id}.stdout.txt`, result.stdout ?? "");
    writeFileSync(`${stem}.${id}.stderr.txt`, result.stderr ?? "");
    // Persist failures before asserting so an unsupported phase report is inspectable.
    writeFileSync(`${stem}.partial.json`, `${JSON.stringify({ runs }, null, 2)}\n`);
    assert.equal(result.error, undefined, `${id}: ${result.error?.message}`);
    assert.equal(result.signal, null, `${id}: unexpected signal`);
    return result;
  }
  function utf8(bytes) {
    const text = bytes.toString("utf8");
    assert(Buffer.from(text).equals(bytes), "native report has unsupported non-UTF-8 bytes");
    return text;
  }
  function membership(id) {
    const listed = run(id, ["--listFilesOnly"]);
    assert.equal(listed.status, 0, "native program membership failed");
    assert.equal(listed.stderr.length, 0, "unexpected membership stderr");
    return utf8(listed.stdout)
      .trim()
      .split(/\r?\n/u)
      .filter(Boolean)
      .map((file) => {
        const path = resolve(process.cwd(), file);
        const bytes = readFileSync(path);
        return { path, bytes: bytes.length, sha256: sha(bytes) };
      });
  }
  // This deliberately warms the graph: every wall/phase value is instrumentation
  // only. Hash all private/native dependency bytes before the two diagnostic runs.
  const membersBefore = membership("membership-before");
  graph.captureMembers(membersBefore);
  const baseline = run("baseline");
  const extended = run("extended", ["--extendedDiagnostics"]);
  const phases = splitNativePhaseReport(utf8(extended.stdout));
  assert.equal(extended.status, baseline.status, "phase flag changed native exit status");
  assert(extended.stderr.equals(baseline.stderr), "phase flag changed native stderr");
  assert.equal(
    phases.diagnosticText,
    utf8(baseline.stdout),
    "phase flag changed ordered native diagnostics",
  );
  const nativeProfile =
    settings.profile === "pprof"
      ? replayNativeProfile({
          runtime: settings.runtime,
          args,
          cwd: process.cwd(),
          baseline,
          profileDirectory: join(settings.directory, "pprof-" + process.pid),
        })
      : null;
  const members = membership("membership-after");
  const graphRecord = graph.finish(members);
  const nativeGraphArchive = {
    record: stem + ".graph.json",
    sha256: sha(readFileSync(stem + ".graph.json")),
    memberCount: graphRecord.membersBefore.length,
    graphClosureClaimed: false,
  };
  assert.deepEqual(members, membersBefore, "native program/member bytes changed during replay");
  assert.equal(new Set(members.map((file) => file.path)).size, members.length, "duplicate member");
  assert.equal(
    members.length,
    phases.fields.Files.value,
    "phase and native program membership disagree",
  );
  assert(readFileSync(config).equals(configBytes), "replay changed actual shard config");
  assert.equal(
    sha(readFileSync(settings.runtime)),
    runtimeSha256,
    "native binary changed during replay",
  );
  const record = {
    cwd: process.cwd(),
    config,
    configSha256: sha(configBytes),
    compilerOptions,
    args,
    runtime: settings.runtime,
    runtimeSha256,
    runs,
    phases,
    members,
    membersBefore,
    nativeGraphArchive,
    nativeProfile,
    profileSemantics: nativeProfile
      ? {
          decoding: "not implemented; gzip container validation only",
          cpuSamples: null,
          allocationSamples: null,
          phaseAttribution: null,
        }
      : null,
    nativeGraphBytesUnchanged: true,
    orderedDiagnosticsEqual: true,
    protocol:
      "membership/hash before; original command; phase replay; membership/hash after; all untimed",
  };
  writeFileSync(`${stem}.json`, `${JSON.stringify(record, null, 2)}\n`);
  process.stdout.write(baseline.stdout ?? "");
  process.stderr.write(baseline.stderr ?? "");
  process.exitCode = baseline.status ?? 1;
}
main();
