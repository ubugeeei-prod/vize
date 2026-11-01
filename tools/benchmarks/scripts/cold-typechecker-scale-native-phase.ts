/** Normal-primary phase observation; no debug output or timed performance claim. */
import assert from "node:assert/strict";
import { spawnSync, type SpawnSyncOptionsWithBufferEncoding } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import {
  recordNativeBeforeInspection,
  retainInputBodies,
  type NativeImage,
  type ReplayOutcome,
} from "./cold-typechecker-scale-native-custody.ts";

export interface InputSnapshot {
  roots: string[];
  rows: Record<string, unknown>[];
  sha256: string;
  files: number;
  bytes: number;
}
const write = (file: string, value: unknown) =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
const inside = (root: string, file: string) => {
  const rel = path.relative(root, file);
  return rel === "" || (rel !== ".." && !rel.startsWith(".." + path.sep) && !path.isAbsolute(rel));
};
const hash = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");
const fileHash = (file: string) => hash(fs.readFileSync(file));
const utf8 = (bytes: Buffer) => new TextDecoder("utf-8", { fatal: true }).decode(bytes);

export function assertPrimaryArgs(args: string[], backend: string) {
  assert.deepEqual(args, [
    "check",
    "--no-config",
    "--format",
    "json",
    "--tsconfig",
    "tsconfig.json",
    "--corsa-path",
    backend,
  ]);
}

/** Refuse inherited semantic/runtime changes BEFORE the existing public-env whitelist. */
export function assertProfileEnvironment(env: NodeJS.ProcessEnv) {
  for (const key of ["GOGC", "GOMEMLIMIT", "NODE_OPTIONS", "NODE_PATH"])
    assert.equal(env[key], undefined, `unexpected ${key}`);
}

export function assertNormalPhase(value: unknown) {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  const report = value as Record<string, unknown>;
  assert.deepEqual(Object.keys(report).sort(), [
    "allocation",
    "budget",
    "command",
    "counters",
    "micro",
    "schema_version",
    "spans",
    "tool",
    "tool_version",
    "truncation",
  ]);
  assert.equal(report.schema_version, 1);
  assert.equal(report.tool, "vize");
  assert.equal(report.command, "check");
  assert.equal(report.allocation, null, "normal phase must not use allocation instrumentation");
  assert.deepEqual(report.truncation, { dropped_spans: 0, dropped_counters: 0 });
  assert(Array.isArray(report.spans) && report.spans.length > 0);
  const commands = report.spans.filter((span) => span.key === "canon.corsa.cli.command");
  assert.equal(commands.length, 1, "native command span must be represented exactly once");
  assert(Number.isSafeInteger(commands[0].count) && commands[0].count > 0);
  assert(Number.isSafeInteger(commands[0].wall_ns.total) && commands[0].wall_ns.total > 0);
  return report;
}

export function captureNormalPhase(
  spec: {
    executable: string;
    args: string[];
    cwd: string;
    env: NodeJS.ProcessEnv;
    out: string;
    images: NativeImage[];
    snapshot: () => InputSnapshot;
  },
  execute = (
    program: string,
    args: string[],
    options: SpawnSyncOptionsWithBufferEncoding,
  ): ReplayOutcome => spawnSync(program, args, options),
) {
  const profileFile = path.join(spec.out, "phase.profile.json");
  assert(!fs.existsSync(profileFile));
  assert.equal(spec.env.VIZE_CACHE_CUSTODY_CAPTURE_DIR, undefined);
  write(path.join(spec.out, "phase-request.json"), {
    executable: spec.executable,
    args: [...spec.args, "--profile-json", profileFile],
    cwd: spec.cwd,
    env: spec.env,
    nativeChildren: "unobserved",
    timingEligible: false,
  });
  const started = process.hrtime.bigint();
  const result = execute(spec.executable, [...spec.args, "--profile-json", profileFile], {
    cwd: spec.cwd,
    env: spec.env,
    maxBuffer: 64 * 1024 * 1024,
    timeout: 600_000,
  });
  return recordNativeBeforeInspection(
    spec.out,
    "phase-vize",
    result,
    (process.hrtime.bigint() - started).toString(),
    () => {
      // Persist the COMPLETE owned tree before process/profile verdicts or any reset.
      const snapshot = spec.snapshot();
      write(path.join(spec.out, "phase-complete-inputs.json"), snapshot);
      retainInputBodies(snapshot, spec.out, spec.images, "phase");
      assert.equal(result.error, undefined);
      assert.equal(result.signal, null);
      assert.equal(result.status, 0);
      const profile = assertNormalPhase(JSON.parse(fs.readFileSync(profileFile, "utf8")));
      write(path.join(spec.out, "phase-observation.json"), {
        scope: "overlapping coarse Rust wall spans; allocation None; not a latency benchmark",
        nativeChildren: "count/PIDs/raw/loaded files unobserved for this normal call",
        executable: spec.executable,
        args: [...spec.args, "--profile-json", profileFile],
        env: spec.env,
        profile,
        completeSnapshot: snapshot.sha256,
      });
      return { result, snapshot };
    },
  );
}

export function resetOwnedPhaseRuntime(out: string, snapshot: InputSnapshot) {
  const runtime = path.join(out, "runtime");
  for (const name of ["xdg", "tmp"]) {
    const root = path.join(runtime, name);
    assert(snapshot.roots.includes(root));
    assert.equal(fs.realpathSync(root), root);
    assert(fs.lstatSync(root).isDirectory() && !fs.lstatSync(root).isSymbolicLink());
    fs.rmSync(root, { recursive: true });
    fs.mkdirSync(root);
    assert.deepEqual(fs.readdirSync(root), []);
  }
  write(path.join(out, "phase-runtime-reset.json"), {
    snapshot: snapshot.sha256,
    roots: [path.join(runtime, "xdg"), path.join(runtime, "tmp")],
    empty: true,
    osPageCache: "not flushed",
    discardedInputBodies: false,
  });
}

export function assertPhaseProgram(
  phase: InputSnapshot,
  current: InputSnapshot,
  runtime: string,
  buildInfo: string,
  initialBuildInfo: unknown,
) {
  for (const snapshot of [phase, current]) {
    assert.deepEqual(Object.keys(snapshot).sort(), ["bytes", "files", "roots", "rows", "sha256"]);
    assert.equal(
      snapshot.sha256,
      createHash("sha256").update(JSON.stringify(snapshot.rows)).digest("hex"),
    );
    const files = snapshot.rows.filter((row) => row.kind === "file");
    assert.equal(snapshot.files, files.length);
    assert.equal(
      snapshot.bytes,
      files.reduce((sum, row) => sum + Number(row.bytes), 0),
    );
  }
  assert.equal(phase.roots.length, 4);
  assert.deepEqual(phase.roots.slice(2), [path.join(runtime, "xdg"), path.join(runtime, "tmp")]);
  assert.deepEqual(current.roots, phase.roots, "no program/root subset may be selected");
  const rows = phase.rows.filter((row) => row.path === buildInfo);
  assert(rows.length <= 1);
  const info = rows[0];
  if (info) assert.equal(info.kind, "file");
  assert.deepEqual(
    initialBuildInfo,
    info ? { exists: true, sha256: info.sha256, bytes: info.bytes } : { exists: false },
  );
  const project = (snapshot: InputSnapshot) => {
    const seen = new Set<string>();
    return snapshot.rows
      .filter((row) => row.path !== buildInfo)
      .map((row) => {
        assert.equal(typeof row.path, "string");
        assert(!seen.has(row.path as string), "duplicate physical/lexical input row");
        seen.add(row.path as string);
        if (row.kind === "file" && inside(runtime, row.path as string)) {
          const { mtimeNs, ...recreated } = row;
          assert.equal(typeof mtimeNs, "string");
          return recreated;
        }
        return row;
      });
  };
  assert.deepEqual(
    project(current),
    project(phase),
    "complete program equality; only recreated runtime file mtimeNs and owned build info differ",
  );
}

export function assertPrimaryOutput(before: ReplayOutcome, after: ReplayOutcome) {
  assert.equal(before.error, undefined);
  assert.equal(after.error, undefined);
  assert.equal(before.signal, null);
  assert.equal(after.signal, null);
  assert.equal(after.status, before.status);
  for (const stream of ["stdout", "stderr"] as const)
    assert(
      (before[stream] ?? Buffer.alloc(0)).equals(after[stream] ?? Buffer.alloc(0)),
      `whole primary ${stream} mismatch`,
    );
}

/** Entire physical input closure plus exact lexical symlink edges; only one output is excluded. */
export function sealInputs(roots: string[], buildInfo: string | null) {
  const canonicalRoots = roots.map((root) => fs.realpathSync(root));
  const visited = new Set<string>(),
    rows: Record<string, unknown>[] = [];
  const visit = (file: string) => {
    const stat = fs.lstatSync(file, { bigint: true });
    const physical = fs.realpathSync(file);
    assert(
      canonicalRoots.some((root) => inside(root, physical)),
      "input symlink escaped sealed roots",
    );
    if (stat.isSymbolicLink()) {
      rows.push({
        path: file,
        kind: "symlink",
        target: utf8(fs.readlinkSync(file, { encoding: "buffer" })),
        physical,
      });
      if (visited.has(physical)) return;
      visit(physical);
      return;
    }
    if (visited.has(physical)) return;
    visited.add(physical);
    if (stat.isDirectory()) {
      rows.push({ path: physical, kind: "directory", mode: Number(stat.mode) });
      for (const name of fs.readdirSync(physical, { encoding: "buffer" }).sort(Buffer.compare)) {
        visit(path.join(physical, utf8(name)));
      }
      return;
    }
    assert(stat.isFile(), "unmodeled input file kind");
    if (buildInfo !== null && physical === buildInfo) return;
    rows.push({
      path: physical,
      kind: "file",
      bytes: Number(stat.size),
      mtimeNs: stat.mtimeNs.toString(),
      mode: Number(stat.mode),
      sha256: fileHash(physical),
    });
  };
  for (const root of canonicalRoots) visit(root);
  rows.sort((a, b) => JSON.stringify(a).localeCompare(JSON.stringify(b)));
  return {
    roots: canonicalRoots,
    rows,
    sha256: hash(JSON.stringify(rows)),
    files: rows.filter((row) => row.kind === "file").length,
    bytes: rows
      .filter((row) => row.kind === "file")
      .reduce((sum, row) => sum + Number(row.bytes), 0),
  };
}
