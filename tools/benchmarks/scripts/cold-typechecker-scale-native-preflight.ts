/** Untimed native raw-vector custody; no observer enters a timed command. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { classifyRaw, validateNativeArgs } from "./cold-typechecker-scale-native-inputs.ts";

const closure = [
  "cold-typechecker-scale-native-forward.ts",
  "cold-typechecker-scale-native-inputs.ts",
  "cold-typechecker-scale-native-custody.ts",
];
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const read = (directory: string, file: string) =>
  JSON.parse(fs.readFileSync(path.join(directory, file), "utf8")) as Record<string, unknown>;
const write = (file: string, value: unknown) =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });

export function prepareNativeForwarder(output: string) {
  assert.match(process.version, /^v24\./u);
  const root = path.join(output, "native-observer"),
    directory = path.join(root, ".bin");
  fs.mkdirSync(directory, { recursive: true });
  const files = closure.map((file) => {
    const bytes = fs.readFileSync(new URL("./" + file, import.meta.url));
    const destination = path.join(directory, file);
    fs.writeFileSync(destination, bytes, { flag: "wx" });
    assert.equal(fs.realpathSync(destination), destination);
    assert.equal(fs.lstatSync(destination).isSymbolicLink(), false);
    return { file, bytes: bytes.length, sha256: hash(bytes) };
  });
  fs.writeFileSync(path.join(directory, "package.json"), '{"type":"module"}\n', { flag: "wx" });
  const executable = path.join(directory, closure[0]);
  fs.chmodSync(executable, 0o755);
  assert.equal(path.basename(path.dirname(executable)), ".bin");
  assert.notEqual(path.basename(path.dirname(path.dirname(executable))), "node_modules");
  const receipt = {
    executable,
    files,
    node: process.version,
    nodeExecutable: process.execPath,
    nodeSha256: hash(fs.readFileSync(process.execPath)),
    classification:
      "canonical physical .bin parent; not node_modules/.bin, so resolver keeps it and Carton selects async JSON-RPC",
    launch:
      "regular executable TypeScript file; OS shebang selects the installed Node24; qualified observations admit only native CLI/probe/version argv, never an API stream",
  };
  write(path.join(root, "receipt.json"), receipt);
  return receipt;
}

export function readObservedNativeCall(directory: string) {
  const invocation = read(directory, "invocation.json");
  assert.equal(invocation.nodeVersion, process.version);
  assert.equal(invocation.nodeExecutable, fs.realpathSync(process.execPath));
  assert.equal(invocation.nodeSha256, hash(fs.readFileSync(process.execPath)));
  assert(
    fs.existsSync(path.join(directory, "request.json")),
    "unhandled invocation cannot be swallowed by a best-effort outer caller",
  );
  assert(
    fs.existsSync(path.join(directory, "native-replay.status.json")),
    "every attempted invocation needs a complete native observation",
  );
  const request = read(directory, "request.json"),
    status = read(directory, "native-replay.status.json");
  assert.deepEqual(Object.keys(request), [
    "startedNs",
    "executable",
    "imageBefore",
    "argv",
    "cwd",
    "kind",
    "canonicalForwarder",
    "apiClassification",
    "config",
    "buildInfo",
    "buildInfoInitiallyAbsent",
    "environment",
    "scope",
  ]);
  assert.deepEqual(Object.keys(status), ["status", "signal", "error", "invokedPid", "elapsedNs"]);
  assert.equal(request.startedNs, invocation.startedNs);
  assert.deepEqual(request.argv, invocation.argv);
  assert.equal(request.executable, invocation.executable);
  assert.equal(request.imageBefore, invocation.imageBefore);
  assert.equal(request.cwd, invocation.cwd);
  assert.equal(request.canonicalForwarder, invocation.canonicalForwarder);
  assert.equal(typeof request.startedNs, "string");
  assert.match(request.startedNs as string, /^\d+$/u);
  assert.equal(typeof request.cwd, "string");
  assert.equal(typeof request.executable, "string");
  assert.equal(typeof request.canonicalForwarder, "string");
  assert.equal(fs.realpathSync(request.canonicalForwarder as string), request.canonicalForwarder);
  assert.equal(path.basename(path.dirname(request.canonicalForwarder as string)), ".bin");
  assert.notEqual(
    path.basename(path.dirname(path.dirname(request.canonicalForwarder as string))),
    "node_modules",
  );
  assert.equal(
    request.apiClassification,
    "async-json-rpc (canonical physical parent .bin; not node_modules/.bin)",
  );
  assert.equal(status.signal, null);
  assert.equal(status.error, null);
  assert(Number.isSafeInteger(status.invokedPid) && Number(status.invokedPid) > 0);
  assert([0, 1, 2].includes(Number(status.status)));
  const argv = request.argv;
  assert(Array.isArray(argv) && argv.every((value) => typeof value === "string"));
  if (request.kind === "main-cli") {
    assert.equal(typeof request.config, "string");
    validateNativeArgs(argv, request.config as string);
    assert.equal(path.basename(request.config as string), "tsconfig.json");
    assert.equal(request.config, path.join(request.cwd as string, "tsconfig.json"));
    assert.equal(request.buildInfo, argv[4]);
    assert.equal(request.buildInfoInitiallyAbsent, true);
  } else if (request.kind === "option-probe") {
    assert.equal(typeof request.config, "string");
    assert.deepEqual(argv, ["--pretty", "false", "--project", request.config]);
    assert.equal(path.basename(request.config as string), "tsconfig.options.json");
    assert.equal(request.config, path.join(request.cwd as string, "tsconfig.options.json"));
    assert.equal(request.buildInfo, null);
    assert.equal(request.buildInfoInitiallyAbsent, null);
  } else {
    assert.equal(request.kind, "version");
    assert.deepEqual(argv, ["--version"]);
    assert.equal(status.status, 0);
    assert.equal(request.config, null);
    assert.equal(request.buildInfo, null);
  }
  const before = read(directory, "inputs-before.json"),
    after = read(directory, "inputs-after.json");
  assert.deepEqual(
    after,
    before,
    "post-seal failure is not native acceptance even if the caller swallowed it",
  );
  assert.equal(hash(Buffer.from(JSON.stringify(before.rows))), before.sha256);
  assert.equal(read(directory, "image-after.json").sha256, request.imageBefore);
  const stdout = fs.readFileSync(path.join(directory, "native-replay.stdout.bin")),
    stderr = fs.readFileSync(path.join(directory, "native-replay.stderr.bin"));
  const config =
    request.config === null ? null : fs.readFileSync(path.join(directory, "config.bytes"));
  return { request, status, stdout, stderr, config, directory };
}

export function collectNativePreflight(output: string) {
  const names = fs.readdirSync(output);
  assert(names.length >= 1 && names.every((name) => /^call-[A-Za-z0-9]+$/u.test(name)));
  return names
    .map((name) => readObservedNativeCall(path.join(output, name)))
    .sort((a, b) =>
      BigInt(a.request.startedNs as string) < BigInt(b.request.startedNs as string) ? -1 : 1,
    );
}

export function assertSameObservedNative(
  before: ReturnType<typeof readObservedNativeCall>,
  after: typeof before,
) {
  const semantics = (request: Record<string, unknown>) => {
    const { startedNs, ...rest } = request;
    assert.equal(typeof startedNs, "string");
    return rest;
  };
  assert.deepEqual(
    semantics(after.request),
    semantics(before.request),
    "actual native argv/cwd/env/config/route must match without normalization",
  );
  assert.equal(after.status.status, before.status.status);
  assert.deepEqual(after.config, before.config);
  assert(after.stdout.equals(before.stdout), "whole raw native stdout changed");
  assert(after.stderr.equals(before.stderr), "whole raw native stderr changed");
}

export function assertSameSourceProviderRows(
  before: unknown,
  after: unknown,
  project: string,
  provider: string,
) {
  const rows = (value: unknown) => {
    assert(value && typeof value === "object" && !Array.isArray(value));
    const all = (value as Record<string, unknown>).rows;
    assert(Array.isArray(all));
    return all.filter((row: unknown) => {
      assert(row && typeof row === "object" && !Array.isArray(row));
      const file = (row as Record<string, unknown>).path;
      assert.equal(typeof file, "string");
      return [project, provider].some(
        (root) => file === root || (file as string).startsWith(root + path.sep),
      );
    });
  };
  const old = rows(before),
    current = rows(after);
  assert(old.length > 0);
  assert.deepEqual(
    current,
    old,
    "whole authored/project/provider potential input closure must be identical between roles",
  );
}

/** Recreated program files may have new mtimes; their complete paths/content/modes must match. */
export function assertSameMaterializedProgram(before: unknown, after: unknown, root: string) {
  const program = (value: unknown) => {
    assert(value && typeof value === "object");
    const rows = (value as Record<string, unknown>).rows;
    assert(Array.isArray(rows));
    return rows
      .filter((row: Record<string, unknown>) => {
        assert.equal(typeof row.path, "string");
        return row.path === root || (row.path as string).startsWith(root + path.sep);
      })
      .map((row: Record<string, unknown>) => {
        if (row.kind !== "file") return row;
        const { mtimeNs, ...body } = row;
        assert.equal(typeof mtimeNs, "string");
        return body;
      });
  };
  const old = program(before);
  assert(old.length > 0);
  assert.deepEqual(program(after), old, "whole materialized program paths/bodies/modes changed");
  return {
    rows: old.length,
    sha256: hash(Buffer.from(JSON.stringify(old))),
    scope: "whole row equality except recreated file mtimes; original full stat receipts retained",
  };
}

export function compareNativePreflights(
  before: ReturnType<typeof collectNativePreflight>,
  after: typeof before,
  expected: {
    executable: string;
    sha256: string;
    forwarder: string;
    project: string;
    provider: string;
    materializedRoot: string;
    authoredCwd: string;
  },
) {
  assert.equal(before.length, after.length);
  assert(before.length >= 1 && before.length <= 3);
  const rows = before.map((old, index) => {
    const current = after[index];
    const oldInputs = read(old.directory, "inputs-before.json");
    const currentInputs = read(current.directory, "inputs-before.json");
    assertSameSourceProviderRows(
      read(old.directory, "inputs-before.json"),
      read(current.directory, "inputs-before.json"),
      expected.project,
      expected.provider,
    );
    for (const observation of [old, current]) {
      assert.equal(observation.request.executable, expected.executable);
      assert.equal(observation.request.imageBefore, expected.sha256);
      assert.equal(observation.request.canonicalForwarder, expected.forwarder);
      const cwd = observation.request.cwd as string;
      assert(
        observation.request.kind === "version"
          ? cwd === expected.authoredCwd || cwd.startsWith(expected.materializedRoot + path.sep)
          : cwd.startsWith(expected.materializedRoot + path.sep),
      );
      const inputs = read(observation.directory, "inputs-before.json");
      assert.deepEqual(
        inputs.roots,
        observation.request.kind === "version"
          ? [expected.project, expected.provider]
          : [expected.project, expected.provider, cwd],
      );
    }
    const materialized =
      old.request.kind === "version"
        ? null
        : assertSameMaterializedProgram(oldInputs, currentInputs, old.request.cwd as string);
    assertSameObservedNative(old, current);
    const classes =
      old.request.kind === "main-cli"
        ? classifyRaw(old.stdout, old.request.cwd as string, expected.project, expected.provider)
        : null;
    if (classes) {
      write(path.join(old.directory, "classes.json"), classes);
    }
    return {
      kind: old.request.kind,
      status: old.status.status,
      stdoutBytes: old.stdout.length,
      stdoutSha256: hash(old.stdout),
      stderrBytes: old.stderr.length,
      stderrSha256: hash(old.stderr),
      actualPidBefore: old.status.invokedPid,
      actualPidHead: current.status.invokedPid,
      classes: classes?.groups ?? null,
      materialized,
    };
  });
  assert.equal(rows.filter((row) => row.kind === "main-cli").length, 1);
  assert(rows.filter((row) => row.kind === "option-probe").length <= 1);
  assert(rows.filter((row) => row.kind === "version").length <= 1);
  return {
    rows,
    allRawBytesAndStatusesEqual: true,
    zeroApiInvocations: true,
    scope:
      "two untimed normal-image preflights only; all observed raw errors/other lines retained without a historical count/status golden; every timed command uses direct physical native; no native-loaded-file census or native-clean claim",
  };
}
