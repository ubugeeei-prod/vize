import assert from "node:assert/strict";
import { isUtf8 } from "node:buffer";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { MEMORY_INITIALIZATION } from "./fixtures.ts";
import { shippingCargoArtifact } from "../lsp-initial-ready-pair/shipping-adapter.ts";
import { BOUND_PATHS, sdkArtifacts } from "../lsp-initial-ready-pair/readiness-parity.ts";

export const SOURCE_PATHS = [
  ".github/actions/lsp-overlay-custody/source-cli.ts",
  "crates/vize_canon/src/corsa_bridge/bridge/initial_diagnostics.rs",
  "crates/vize_canon/src/corsa_bridge/bridge/initial_diagnostics/tests.rs",
  "crates/vize_canon/src/corsa_bridge/bridge/initial_diagnostics/tests/owned_read.rs",
  "crates/vize_maestro/src/document/store.rs",
  "crates/vize_maestro/src/document/store/closed.rs",
  "crates/vize_maestro/src/document/store/closed/tests.rs",
  "crates/vize_maestro/src/document/store/uri_snapshot.rs",
  "crates/vize_maestro/src/ide/diagnostics/corsa/initial.rs",
  "crates/vize_maestro/src/server/open_document.rs",
  "crates/vize_maestro/src/server/state/config.rs",
  "crates/vize_maestro/src/server/state/features.rs",
  "crates/vize_maestro/src/server/state/project_contexts.rs",
  "crates/vize_maestro/src/server/state/project_contexts/closed.rs",
  "crates/vize_maestro/src/server/state/project_contexts/closed/tests.rs",
  "crates/vize_maestro/src/server/state/project_contexts/registry.rs",
  "crates/vize_maestro/src/server/state/project_contexts/registry/closed_tests.rs",
  "crates/vize_maestro/src/server/state/project_contexts/shutdown.rs",
  "tests/tooling/lsp-overlay-owned-read-roster.test.ts",
] as const;
const SDK_PATHS = [
  "npm/cli/content-mapper-helpers.d.cts",
  "npm/cli/content-mapper-helpers.d.mts",
  "npm/cli/content-mapper-helpers.d.ts",
  "npm/cli/generated/content-mapper-types.d.ts",
  "npm/cli/virtual-ts-globals.d.ts",
  "npm/cli/virtual-ts-patterns.d.ts",
  "npm/cli/virtual-ts-slots.d.ts",
  "npm/cli/virtual-ts.d.cts",
  "npm/cli/virtual-ts.d.mts",
  "npm/cli/virtual-ts.d.ts",
  "npm/cli/package.json",
  "npm/cli/scripts/generate-content-mapper-types.ts",
];
const SHA = /^[a-f0-9]{40}$/;
export type Obj = Record<string, unknown>;
export type Result = { status: number | null; signal: string | null; error: string | null };
export const hash = (bytes: string | Buffer): string =>
  createHash("sha256").update(bytes).digest("hex");
export const record = (value: unknown): Obj => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Obj;
};
export const array = (value: unknown): unknown[] => {
  assert.ok(Array.isArray(value));
  return value;
};
export function json(file: string): unknown {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}
export function save(file: string, value: unknown): void {
  fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`, { flag: "wx", mode: 0o600 });
}
export function fileHash(file: string): string {
  assert.ok(fs.lstatSync(file).isFile());
  return hash(fs.readFileSync(file));
}
export function copyOwned(source: string, destination: string): void {
  if (fs.existsSync(destination)) assert.ok(fs.lstatSync(destination).isFile());
  fs.copyFileSync(source, destination);
  fs.chmodSync(destination, 0o700);
}
export function git(repo: string, ...args: string[]): string {
  const result = spawnSync("git", args, {
    cwd: repo,
    encoding: "utf8",
    maxBuffer: 8 * 1024 * 1024,
  });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}
export function pathsBetween(repo: string, left: string, right: string): string[] {
  return git(repo, "diff", "--name-only", left, right).split("\n").filter(Boolean);
}
export function parentFromCommit(raw: string): string {
  const header = raw.split("\n\n", 1)[0]!;
  const parents = header
    .split("\n")
    .filter((line) => line.startsWith("parent "))
    .map((line) => line.slice(7));
  assert.equal(parents.length, 1, "finite source must have one immutable parent");
  assert.match(parents[0]!, SHA);
  return parents[0]!;
}
export function immutableParent(repo: string, source: string): string {
  return parentFromCommit(git(repo, "cat-file", "-p", source));
}
export function clean(repo: string, source: string): void {
  assert.equal(git(repo, "rev-parse", "HEAD"), source);
  assert.equal(git(repo, "status", "--porcelain=v1", "--untracked-files=no"), "");
}
export function snapshot(repo: string): Obj {
  const tree = git(repo, "ls-tree", "-r", "-z", "--format=%(objectmode) %(path)", "HEAD");
  const files = tree
    .split("\0")
    .filter((entry) => /^100(644|755) /.test(entry))
    .map((entry) => entry.slice(7))
    .filter(
      (name) =>
        [
          "tests/_helpers/",
          "tests/tooling/support/",
          "tests/differential/",
          "tests/performance/support/",
          "tests/_fixtures/performance/lsp-dual-server/",
          "tests/tooling/lsp-dual-server-",
        ].some((prefix) => name.startsWith(prefix)) ||
        [
          "tests/performance/lsp-dual-server-benchmark.ts",
          "tests/package.json",
          "Cargo.toml",
          "Cargo.lock",
          "rust-toolchain.toml",
          "package.json",
          "pnpm-lock.yaml",
          "pnpm-workspace.yaml",
          ".cargo/config.toml",
          ...SDK_PATHS,
          ...SOURCE_PATHS,
          ...BOUND_PATHS,
        ].includes(name),
    );
  return {
    source: git(repo, "rev-parse", "HEAD"),
    tree: git(repo, "rev-parse", "HEAD^{tree}"),
    trackedTreeSha256: hash(tree),
    files: files.map((name) => ({ path: name, sha256: fileHash(path.join(repo, name)) })),
  };
}
export function run(
  repo: string,
  dir: string,
  label: string,
  executable: string,
  args: string[],
  env: NodeJS.ProcessEnv,
): Result {
  const stdout = fs.openSync(path.join(dir, `${label}.stdout`), "wx", 0o600);
  const stderr = fs.openSync(path.join(dir, `${label}.stderr`), "wx", 0o600);
  const startNs = process.hrtime.bigint();
  let result: Result;
  try {
    const child = spawnSync(executable, args, {
      cwd: repo,
      env,
      stdio: ["ignore", stdout, stderr],
    });
    result = { status: child.status, signal: child.signal, error: child.error?.message ?? null };
  } finally {
    fs.closeSync(stdout);
    fs.closeSync(stderr);
  }
  save(path.join(dir, `${label}.exit.json`), {
    executable,
    args,
    cwd: repo,
    startNs: String(startNs),
    endNs: String(process.hrtime.bigint()),
    ...result,
  });
  return result;
}
export function succeeded(result: Result): void {
  assert.deepEqual(result, { status: 0, signal: null, error: null });
}
export function regularIdentity(file: string): Obj {
  const stat = fs.statSync(file, { bigint: true });
  assert.ok(stat.isFile());
  return {
    realpath: fs.realpathSync(file),
    bytes: Number(stat.size),
    dev: String(stat.dev),
    ino: String(stat.ino),
    mtimeNs: String(stat.mtimeNs),
    ctimeNs: String(stat.ctimeNs),
    sha256: fileHash(fs.realpathSync(file)),
  };
}

export function cargoArtifact(file: string, executable: string): Obj {
  const rows = fs
    .readFileSync(file, "utf8")
    .split("\n")
    .filter(Boolean)
    .map((line) => record(JSON.parse(line)));
  const bins = rows.filter(
    (row) =>
      row.reason === "compiler-artifact" &&
      record(row.target).name === "vize" &&
      row.executable === executable,
  );
  assert.equal(bins.length, 1, "one actual Cargo CLI artifact is required");
  const bin = bins[0]!;
  shippingCargoArtifact(bin);
  const maestro = rows.filter(
    (row) => row.reason === "compiler-artifact" && record(row.target).name === "vize_maestro",
  );
  assert.ok(maestro.length > 0);
  assert.ok(maestro.every((row) => array(row.features).includes("native")));
  assert.ok(
    maestro.every((row) =>
      array(row.features).every(
        (feature) => feature !== "profiling" && feature !== "micro-profiling",
      ),
    ),
  );
  assert.ok(rows.some((row) => row.reason === "build-finished" && row.success === true));
  return { cli: bin, maestro, sdkArtifacts: sdkArtifacts(rows) };
}

export function resolveProgram(repo: string, name: string): string {
  for (const directory of (process.env.PATH ?? "").split(path.delimiter)) {
    const candidate = path.resolve(directory || repo, name);
    try {
      fs.accessSync(candidate, fs.constants.X_OK);
      if (fs.statSync(candidate).isFile()) return fs.realpathSync(candidate);
    } catch {
      /* Continue the same PATH search used for the child command. */
    }
  }
  throw new Error(`missing executable: ${name}`);
}

export function selectedTool(repo: string, dir: string, name: "rustc" | "cargo"): string {
  succeeded(run(repo, dir, `selected-${name}`, "rustup", ["which", name], process.env));
  const output = fs.readFileSync(path.join(dir, `selected-${name}.stdout`), "utf8");
  const selected = output.trimEnd();
  assert.ok(path.isAbsolute(selected) && !/[\r\n]/.test(selected));
  assert.equal(output, `${selected}\n`);
  return fs.realpathSync(selected);
}

export function checkAmbientOverrides(environment: NodeJS.ProcessEnv): void {
  for (const name of Object.keys(environment)) {
    if (
      name.startsWith("LSP_DUAL_") ||
      [
        "VIZE_LSP_BIN",
        "VIZE_LSP_CURRENT_BASELINE",
        "VIZE_LSP_BUILD_RECIPE",
        "VIZE_LSP_REQUIRE_SOURCE_BUILD",
        "CORSA_PATH",
        "CORSA_EXECUTABLE",
        "VIZE_CORSA_PATH",
        "TSGO_PATH",
        "TSGO_EXECUTABLE",
        "VIZE_TSGO_PATH",
        "TSRS_PATH",
        "TSRS_EXECUTABLE",
        "VIZE_TSRS_PATH",
        "VIZE_TRACE_EDITOR_PREPARATION",
        "VIZE_TEST_TSGO_PATH",
      ].includes(name)
    )
      assert.equal(environment[name], undefined, `explicit ambient ${name} refused`);
  }
}

/** Require complete finite coverage after the workload has saved every raw result. */
export function checkOutcome(
  value: unknown,
  mode: "memory",
  source: string,
  attempt: number,
): void {
  const row = record(value);
  assert.equal(row.schemaVersion, 1);
  assert.equal(row.source, source);
  assert.equal(row.mode, mode);
  assert.equal(row.attempt, attempt);
  assert.equal(row.complete, true);
  assert.deepEqual(row.errors, []);
  const rows = array(row.rows);
  const captures = array(row.rawCaptures);
  assert.equal(row.workloadVersion, 2);
  assert.equal(mode, "memory");
  assert.equal(rows.length, 1);
  assert.equal(captures.length, 1);
  const memory = record(rows[0]);
  assert.equal(memory.fixture, "distinct-configs");
  assert.equal(memory.workloadVersion, 2);
  assert.deepEqual(memory.initializationOptions, MEMORY_INITIALIZATION);
  const roots = array(memory.workspaceRoots);
  assert.equal(roots.length, 8);
  assert.equal(new Set(roots).size, 8);
  assert.ok(roots.every((root) => typeof root === "string" && path.isAbsolute(root)));
  assert.deepEqual(memory.errors, []);
  assert.equal(record(memory.resources).failure, null);
  assert.equal(record(memory.resources).quotaDroppedRows, 0);
  assert.deepEqual(
    array(memory.milestones).map((value) => record(value).milestone),
    [1, 4, 8, "closed", "reopened8"],
  );
  const names = new Set<string>();
  const digest = /^[a-f0-9]{64}$/;
  for (const value of captures) {
    const capture = record(value);
    assert.equal(capture.complete, true);
    assert.equal(capture.sourceRevision, source);
    assert.match(String(capture.binarySha256), digest);
    assert.match(String(capture.observationSha256), digest);
    assert.equal(typeof capture.directory, "string");
    const directory = capture.directory as string;
    assert.ok(
      directory.length > 0 &&
        !path.isAbsolute(directory) &&
        !directory.includes("\\") &&
        !/[\0\r\n]/.test(directory),
    );
    assert.ok(directory.split("/").every((part) => part !== "" && part !== "." && part !== ".."));
    assert.ok(!names.has(directory));
    names.add(directory);
    assert.deepEqual(capture.process, { exitStatus: 0, signal: null, error: null });
    const streams = record(capture.streams);
    assert.deepEqual(Object.keys(streams).sort(), ["client", "server", "stderr"]);
    for (const name of ["client", "server", "stderr"]) {
      const stream = record(streams[name]);
      assert.ok(
        Number.isSafeInteger(stream.bytes) &&
          Number(stream.bytes) >= 0 &&
          Number(stream.bytes) <= 16 * 1024 * 1024,
      );
      assert.match(String(stream.sha256), digest);
      if (name !== "stderr") {
        assert.ok(Number(stream.bytes) > 0);
        assert.equal(stream.framing, "complete");
      }
    }
  }
}
export function verifyCaptures(output: string, value: unknown, image: string): void {
  const outcome = record(value);
  const root = fs.realpathSync(output);
  for (const value of array(outcome.rawCaptures)) {
    const capture = record(value);
    assert.equal(capture.binarySha256, image);
    const directory = path.join(root, String(capture.directory));
    assert.ok(fs.lstatSync(directory).isDirectory() && !fs.lstatSync(directory).isSymbolicLink());
    assert.equal(
      fs.realpathSync(directory),
      directory,
      "raw capture path must not traverse a symlink",
    );
    const observationPath = path.join(directory, "observation.json");
    const observationStat = fs.lstatSync(observationPath);
    assert.ok(
      observationStat.isFile() &&
        !observationStat.isSymbolicLink() &&
        observationStat.size <= 4 * 1024 * 1024,
    );
    assert.equal(fileHash(observationPath), capture.observationSha256);
    const observation = record(json(observationPath));
    assert.equal(observation.state, "process-closed");
    assert.equal(observation.sourceRevision, outcome.source);
    assert.equal(record(observation.binary).binarySha256, image);
    assert.equal(record(observation.binary).sourceRevision, outcome.source);
    assert.equal(record(observation.buildReceipt).sourceRevision, outcome.source);
    assert.equal(record(observation.buildReceipt).binarySha256, image);
    assert.equal(record(observation.buildReceipt).recipe, "cargo build --release -p vize");
    assert.deepEqual(observation.process, capture.process);
    for (const name of ["client", "server", "stderr"]) {
      const file = path.join(directory, `${name}.bin`);
      const metadata = record(record(capture.streams)[name]);
      const stat = fs.lstatSync(file);
      assert.ok(stat.isFile() && !stat.isSymbolicLink() && stat.size <= 16 * 1024 * 1024);
      const bytes = fs.readFileSync(file);
      assert.equal(bytes.length, metadata.bytes);
      assert.equal(hash(bytes), metadata.sha256);
      const observed = record(record(observation.streams)[name]);
      assert.equal(observed.truncated, false);
      assert.equal(observed.observedBytes, bytes.length);
      assert.equal(observed.capturedBytes, bytes.length);
      assert.equal(observed.sha256, metadata.sha256);
      if (name !== "stderr") {
        assert.equal(record(observed.framing).state, "complete");
        completeFrames(bytes);
        if (name === "client")
          checkMemoryInitialization(
            bytes,
            array(record(array(outcome.rows)[0]).workspaceRoots) as string[],
          );
      }
    }
  }
}

/** Independent framing check after raw sessions have closed; outside latency windows. */
export function completeFrames(bytes: Buffer, visit?: (message: Obj) => void): number {
  let offset = 0;
  let frames = 0;
  while (offset < bytes.length) {
    const headerEnd = bytes.indexOf("\r\n\r\n", offset);
    assert.ok(
      headerEnd >= offset && headerEnd - offset <= 8192,
      "complete bounded LSP header required",
    );
    const headers = bytes.subarray(offset, headerEnd).toString("ascii").split("\r\n");
    const sizes = headers
      .map((line) => /^Content-Length:\s*(\d+)$/i.exec(line))
      .filter((row) => row !== null);
    assert.equal(sizes.length, 1);
    const size = Number(sizes[0]![1]);
    assert.ok(Number.isSafeInteger(size) && size > 0 && size <= 16 * 1024 * 1024);
    const end = headerEnd + 4 + size;
    assert.ok(end <= bytes.length, "truncated LSP body refused");
    const body = bytes.subarray(headerEnd + 4, end);
    assert.ok(isUtf8(body));
    const message = record(JSON.parse(body.toString("utf8")));
    assert.equal(message.jsonrpc, "2.0");
    visit?.(message);
    offset = end;
    frames++;
  }
  assert.ok(frames > 0, "empty raw protocol stream refused");
  return frames;
}

/** Bind the actual sent initialize packet, rather than trusting an outcome label. */
export function checkMemoryInitialization(bytes: Buffer, roots: readonly string[]): void {
  const messages: Obj[] = [];
  completeFrames(bytes, (message) => {
    if (message.method === "initialize") messages.push(message);
  });
  assert.equal(messages.length, 1);
  const params = record(messages[0]!.params);
  assert.equal(params.rootUri, null);
  assert.deepEqual(params.initializationOptions, MEMORY_INITIALIZATION);
  assert.deepEqual(
    params.workspaceFolders,
    roots.map((root) => ({ uri: pathToFileURL(root).href, name: path.basename(root) })),
  );
}
