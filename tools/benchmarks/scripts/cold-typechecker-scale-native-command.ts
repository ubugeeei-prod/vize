/** One opt-in Vize custody capture and exact materialized native command replay. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { coldScalePacketView } from "./cold-typechecker-scale-packet.ts";
import {
  retainInputBodies,
  recordReplayBeforeInspection,
  type NativeImage,
} from "./cold-typechecker-scale-native-custody.ts";

export interface CaptureSpec {
  sourceSha: string;
  sourceTree: string;
  buildReceipt: string;
  providerLockSha256: string;
  vizeExecutable: string;
  vizeArgs: string[];
  backend: string;
  backendSha256: string;
  project: string;
  provider: string;
  cwd: string;
  out: string;
  expectedFiles: string[];
  metadataOnlyImages: NativeImage[];
}
type Row = Record<string, unknown>;
const object = (value: unknown): Row => {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Row;
};
const hash = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");
const fileHash = (file: string) => hash(fs.readFileSync(file));
const inside = (root: string, file: string) => {
  const rel = path.relative(root, file);
  return rel === "" || (rel !== ".." && !rel.startsWith(".." + path.sep) && !path.isAbsolute(rel));
};
const write = (file: string, value: unknown) =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
const rawWrite = (file: string, bytes: Buffer) => fs.writeFileSync(file, bytes, { flag: "wx" });
const utf8 = (bytes: Buffer) => new TextDecoder("utf-8", { fatal: true }).decode(bytes);

/** Finite flags from the existing Rust command; no checker-width/option substitution. */
export function validateNativeArgs(args: unknown, config: string): string[] {
  assert(Array.isArray(args) && args.every((arg) => typeof arg === "string"));
  assert.equal(args.length, 9);
  assert.deepEqual(args.slice(0, 4), ["--checkers", "1", "--incremental", "--tsBuildInfoFile"]);
  assert.deepEqual(args.slice(5), ["--pretty", "false", "--project", config]);
  assert(path.isAbsolute(args[4]));
  return args;
}

export function assertAuthoredPacket(
  packet: unknown,
  project: string,
  cwd: string,
  expectedFiles: string[],
) {
  const authored = object(packet);
  assert.equal(authored.errorCount, 0);
  assert.equal(authored.warningCount, 0);
  assert.equal(authored.fileCount, expectedFiles.length);
  assert(Array.isArray(authored.files));
  authored.files.forEach((row) => assert.deepEqual(object(row).diagnostics, []));
  const entries = expectedFiles.map((file) => {
    assert(path.isAbsolute(file) && path.normalize(file) === file && inside(project, file));
    return { path: path.relative(project, file) };
  });
  const view = coldScalePacketView(authored, { root: project, app: cwd, entries });
  assert.deepEqual(
    view.files.map((file) => file.file).toSorted(),
    entries.map((entry) => entry.path).toSorted(),
  );
  return view;
}

/** Preserve every line. Diagnostic roles are physical path classes, never mapper reasons. */
export function classifyRaw(bytes: Buffer, cwd: string, project: string, provider: string) {
  const parts = utf8(bytes).split("\n");
  const rows = parts.map((part, index) => {
    const hasLF = index < parts.length - 1;
    const raw = part + (hasLF ? "\n" : "");
    const line = hasLF && part.endsWith("\r") ? part.slice(0, -1) : part;
    const diagnostic = /^(.*)\((\d+),(\d+)\): (error|warning|info) TS(\d+): (.*)$/u.exec(line);
    const global = /^(error|warning|info) TS(\d+): (.*)$/u.exec(line);
    if (!diagnostic)
      return { index, raw, kind: global ? "global" : "other", code: global?.[2] ?? null };
    const file = path.resolve(cwd, diagnostic[1]);
    let physical: string | null = null;
    try {
      physical = fs.realpathSync(file);
    } catch {
      /* Explicit unresolved role below. */
    }
    return {
      index,
      raw,
      kind: "positioned",
      code: diagnostic[5],
      file,
      physical,
      role:
        physical === null
          ? "unresolved"
          : inside(provider, physical)
            ? "provider-tree"
            : inside(project, physical)
              ? "authored-tree"
              : inside(cwd, physical)
                ? "materialized"
                : "outside-sealed-roots",
      line: Number(diagnostic[2]),
      column: Number(diagnostic[3]),
    };
  });
  const groups = new Map<string, number>();
  for (const row of rows)
    if (row.code !== null) {
      const key = JSON.stringify([row.code, "role" in row ? row.role : row.kind]);
      groups.set(key, (groups.get(key) ?? 0) + 1);
    }
  return { rows, groups: [...groups].sort(([a], [b]) => a.localeCompare(b)) };
}

/** Entire physical input closure plus exact lexical symlink edges; only one output is excluded. */
function seal(roots: string[], buildInfo: string | null) {
  const canonicalRoots = roots.map((root) => fs.realpathSync(root));
  const visited = new Set<string>(),
    rows: Row[] = [];
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

/** Invoked only by the explicitly selected hosted attribution mode. */
export function captureThenReplay(spec: CaptureSpec) {
  assert.equal(process.platform, "linux");
  assert.match(spec.sourceSha, /^[a-f0-9]{40}$/u);
  assert.match(spec.sourceTree, /^[a-f0-9]{40}$/u);
  for (const file of [
    spec.vizeExecutable,
    spec.backend,
    spec.project,
    spec.provider,
    spec.cwd,
    spec.out,
    spec.buildReceipt,
  ]) {
    assert(path.isAbsolute(file), "all invocation/input/output paths must be absolute");
  }
  assert.equal(fs.realpathSync(path.dirname(spec.out)), path.dirname(spec.out));
  assert(!fs.existsSync(spec.out), "refuse evidence replacement");
  const project = fs.realpathSync(spec.project),
    provider = fs.realpathSync(spec.provider);
  const backend = fs.realpathSync(spec.backend),
    vize = fs.realpathSync(spec.vizeExecutable);
  const buildBytes = fs.readFileSync(spec.buildReceipt),
    build = object(JSON.parse(utf8(buildBytes)));
  assert.equal(build.sourceSha, spec.sourceSha);
  assert.equal(build.treeSha, spec.sourceTree);
  assert.equal(build.binarySha256, fileHash(vize));
  assert.equal(build.observerSourceSha, spec.sourceSha);
  const features = object(build.cargoArtifact).features;
  assert(
    Array.isArray(features) &&
      features.includes("cache-custody") &&
      !features.includes("profiling"),
  );
  assert.equal(fileHash(path.join(provider, "package-lock.json")), spec.providerLockSha256);
  assert(path.isAbsolute(spec.cwd) && inside(project, spec.cwd));
  assert(!inside(project, spec.out) && !inside(provider, spec.out));
  assert.equal(fileHash(backend), spec.backendSha256);
  assert.equal(
    spec.metadataOnlyImages.filter(
      (image) => image.path === backend && image.sha256 === spec.backendSha256,
    ).length,
    1,
    "the invoked physical backend must be explicitly metadata-only",
  );
  assert.deepEqual(
    [...fs.readFileSync(backend).subarray(0, 4)],
    [127, 69, 76, 70],
    "require physical ELF, not proxy",
  );
  assert.equal(spec.vizeArgs.filter((arg) => arg === "--corsa-path").length, 1);
  assert.equal(spec.vizeArgs[spec.vizeArgs.indexOf("--corsa-path") + 1], spec.backend);
  fs.mkdirSync(spec.out);
  rawWrite(path.join(spec.out, "build-receipt.json"), buildBytes);
  const runtime = path.join(spec.out, "runtime"),
    custody = path.join(spec.out, "custody");
  fs.mkdirSync(runtime);
  fs.mkdirSync(custody);
  const xdg = path.join(runtime, "xdg"),
    tmp = path.join(runtime, "tmp");
  fs.mkdirSync(xdg);
  fs.mkdirSync(tmp);
  assert.deepEqual(fs.readdirSync(xdg), []);
  // Keep system HOME unchanged. Both processes inherit this exact public-only map.
  const env: NodeJS.ProcessEnv = {};
  for (const key of ["PATH", "HOME", "USER", "LOGNAME", "LANG", "LC_ALL", "TZ"]) {
    if (process.env[key] !== undefined) env[key] = process.env[key];
  }
  Object.assign(env, {
    NO_COLOR: "1",
    GOMAXPROCS: "8",
    RAYON_NUM_THREADS: "8",
    XDG_CACHE_HOME: xdg,
    TMPDIR: tmp,
    VIZE_CACHE_CUSTODY_CAPTURE_DIR: custody,
  });
  write(path.join(spec.out, "request.json"), {
    spec,
    env,
    xdgStartedEmpty: true,
    osPageCache: "not flushed; replay is OS-cache-warm",
    nativePidAtFirstCall: null,
  });
  const sourceBefore = seal([project, provider], null);
  write(path.join(spec.out, "source-before.json"), sourceBefore);
  retainInputBodies(sourceBefore, spec.out, spec.metadataOnlyImages, "source");
  const vizeHash = fileHash(vize),
    capture = spawnSync(vize, spec.vizeArgs, {
      cwd: spec.cwd,
      env,
      maxBuffer: 64 * 1024 * 1024,
      timeout: 600_000,
    });
  rawWrite(path.join(spec.out, "vize.stdout.bin"), capture.stdout ?? Buffer.alloc(0));
  rawWrite(path.join(spec.out, "vize.stderr.bin"), capture.stderr ?? Buffer.alloc(0));
  write(path.join(spec.out, "vize.status.json"), {
    status: capture.status,
    signal: capture.signal,
    error: capture.error?.message ?? null,
  });
  assert.equal(capture.error, undefined);
  assert.equal(capture.signal, null);
  assert.equal(capture.status, 0);
  const owners = fs.readdirSync(custody);
  assert.equal(owners.length, 1);
  const directory = path.join(custody, owners[0]),
    members = fs.readdirSync(directory).toSorted();
  assert.deepEqual(members, [
    "cache-000-invoked-process.completed.json",
    "cache-000-invoked-process.requested.json",
  ]);
  const receiptFile = path.join(directory, members[0]);
  const receipt = object(JSON.parse(fs.readFileSync(receiptFile, "utf8")));
  assert.equal(receipt.schema, "vize.cache-runtime-custody");
  assert.equal(receipt.version, 2);
  assert.equal(receipt.metadataOnly, true);
  assert.equal(receipt.timingEligible, false);
  assert.equal(receipt.route, "invoked-cli-process");
  assert.equal(receipt.stage, "completed");
  assert.equal(receipt.compiledSourceRevision, spec.sourceSha);
  assert.deepEqual(receipt.errors, []);
  assert.equal(receipt.completion, "INVOKED_PROCESS_REAPED");
  assert.equal(receipt.resolvedInvokedProgram, backend);
  assert.equal(receipt.invokedProgramSha256, spec.backendSha256);
  assert.equal(receipt.invokedProgram, spec.backend);
  assert.equal(typeof receipt.requestedConfigPath, "string");
  assert.equal(typeof receipt.virtualRoot, "string");
  assert.equal(receipt.actualCwd, receipt.virtualRoot);
  const config = receipt.requestedConfigPath as string,
    cwd = receipt.actualCwd as string;
  assert.equal(fs.realpathSync(cwd), cwd);
  assert(inside(xdg, cwd));
  assert(inside(cwd, config));
  const args = validateNativeArgs(receipt.actualArgs, config),
    buildInfo = path.resolve(args[4]);
  assert(inside(cwd, buildInfo));
  assert.equal(fs.realpathSync(path.dirname(buildInfo)), path.dirname(buildInfo));
  assert.equal(fileHash(config), receipt.requestedConfigSha256);
  assert.equal(fs.readFileSync(config, "utf8"), receipt.requestedConfigBytes);
  const bytes = (value: unknown): Buffer => {
    assert(
      Array.isArray(value) &&
        value.every((byte) => Number.isInteger(byte) && byte >= 0 && byte <= 255),
    );
    return Buffer.from(value);
  };
  const stdout = bytes(receipt.rawStdoutBytes),
    stderr = bytes(receipt.rawStderrBytes);
  rawWrite(path.join(spec.out, "native-initial.stdout.bin"), stdout);
  rawWrite(path.join(spec.out, "native-initial.stderr.bin"), stderr);
  rawWrite(path.join(spec.out, "materialized-config.json"), fs.readFileSync(config));
  const retainBuildInfo = (role: string) => {
    if (!fs.existsSync(buildInfo)) return { exists: false };
    assert(fs.lstatSync(buildInfo).isFile() && !fs.lstatSync(buildInfo).isSymbolicLink());
    const record = {
      exists: true,
      sha256: fileHash(buildInfo),
      bytes: fs.statSync(buildInfo).size,
    };
    fs.renameSync(buildInfo, path.join(spec.out, role + ".tsbuildinfo"));
    return record;
  };
  const initialBuildInfo = retainBuildInfo("initial");
  assert(!fs.existsSync(buildInfo));
  assert.deepEqual(seal([project, provider], null), sourceBefore);
  const inputBefore = seal([project, provider, cwd], buildInfo);
  write(path.join(spec.out, "materialized-before.json"), inputBefore);
  retainInputBodies(inputBefore, spec.out, spec.metadataOnlyImages, "materialized");
  write(path.join(spec.out, "native-initial.classes.json"), {
    stdout: classifyRaw(stdout, cwd, project, provider),
    stderr: classifyRaw(stderr, cwd, project, provider),
  });
  assertAuthoredPacket(JSON.parse(utf8(capture.stdout)), project, spec.cwd, spec.expectedFiles);
  const started = process.hrtime.bigint();
  const replay = spawnSync(receipt.invokedProgram as string, args, {
    cwd,
    env,
    maxBuffer: 64 * 1024 * 1024,
    timeout: 600_000,
  });
  const elapsedNs = (process.hrtime.bigint() - started).toString();
  return recordReplayBeforeInspection(spec.out, replay, elapsedNs, (status) => {
    const replayBuildInfo = retainBuildInfo("replay");
    const inputAfter = seal([project, provider, cwd], buildInfo);
    write(path.join(spec.out, "materialized-after.json"), inputAfter);
    write(path.join(spec.out, "native-replay.input-custody.json"), {
      initialBuildInfo,
      replayBuildInfo,
    });
    assert.equal(replay.error, undefined);
    assert.equal(replay.signal, null);
    assert.equal(replay.status, receipt.exitCode);
    assert.deepEqual(replay.stdout, stdout);
    assert.deepEqual(replay.stderr, stderr);
    assert.deepEqual(inputAfter, inputBefore);
    assert.equal(fileHash(backend), spec.backendSha256);
    assert.equal(fileHash(vize), vizeHash);
    write(path.join(spec.out, "same-command-replay.json"), {
      ...status,
      parity: true,
      scope:
        "one fresh native process, identical captured program/argv/cwd/environment and sealed post-Vize materialized bytes",
      initialNativePreFileVector: "unobserved; pre-config is observed, tree is sealed after Vize",
      loadedFileVector: "unobserved",
      normalizedOptions: "unobserved",
      osPageCache: "warm, not flushed",
      timingClaim: "one native replay observation; no p50/p95 or shipping Vize improvement claim",
      originalRunExternalCodeComparison: "not attempted; no cross-route equal-work claim",
    });
  });
}
