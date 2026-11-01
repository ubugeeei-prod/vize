import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const selectionKeys = [
  "PATH",
  "CARGO",
  "RUSTC",
  "RUSTUP_TOOLCHAIN",
  "RUSTUP_HOME",
  "CARGO_HOME",
  "RUSTC_WRAPPER",
  "RUSTC_WORKSPACE_WRAPPER",
  "CARGO_BUILD_RUSTC",
  "CARGO_BUILD_RUSTC_WRAPPER",
  "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
  "LD_PRELOAD",
  "LD_AUDIT",
] as const;
const wrapperKeys = [
  "RUSTC_WRAPPER",
  "RUSTC_WORKSPACE_WRAPPER",
  "CARGO_BUILD_RUSTC_WRAPPER",
  "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
  "LD_PRELOAD",
  "LD_AUDIT",
] as const;
const hash = (body: Uint8Array) => createHash("sha256").update(body).digest("hex");
const bytes = (body: Buffer) => ({ base64: body.toString("base64"), sha256: hash(body) });
function record(value: unknown): asserts value is Record<string, unknown> {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
}
function statFields(stat: fs.BigIntStats) {
  return {
    dev: String(stat.dev),
    ino: String(stat.ino),
    mode: String(stat.mode),
    size: String(stat.size),
    mtimeNs: String(stat.mtimeNs),
    ctimeNs: String(stat.ctimeNs),
  };
}

/** Linux capture only: retain path selection, every symlink, and actual executable bytes. */
export function nativeHistoryToolImage(file: string, cwd: string) {
  const selected = path.resolve(cwd, file);
  const links: { path: string; target: string; stat: ReturnType<typeof statFields> }[] = [];
  let prefix = path.parse(selected).root;
  for (const part of selected.slice(prefix.length).split(path.sep)) {
    prefix = path.join(prefix, part);
    const stat = fs.lstatSync(prefix, { bigint: true });
    if (stat.isSymbolicLink())
      links.push({ path: prefix, target: fs.readlinkSync(prefix), stat: statFields(stat) });
  }
  const realpath = fs.realpathSync(selected);
  const stat = fs.statSync(realpath, { bigint: true });
  assert(stat.isFile() && (stat.mode & 0o111n) !== 0n, "toolchain image is not executable");
  const body = fs.readFileSync(realpath);
  assert(
    body.subarray(0, 4).equals(Buffer.from([0x7f, 0x45, 0x4c, 0x46])),
    "capture requires a real Linux ELF toolchain image, not an unqualified wrapper",
  );
  assert.equal(BigInt(body.length), stat.size);
  return { path: selected, realpath, links, stat: statFields(stat), sha256: hash(body) };
}
type Image = ReturnType<typeof nativeHistoryToolImage>;
function executable(command: string, cwd: string, environment: NodeJS.ProcessEnv) {
  if (command.includes(path.sep)) return path.resolve(cwd, command);
  for (const directory of (environment.PATH ?? "").split(path.delimiter)) {
    const candidate = path.resolve(cwd, directory, command);
    try {
      fs.accessSync(candidate, fs.constants.X_OK);
      if (fs.statSync(candidate).isFile()) return candidate;
    } catch {}
  }
  throw new Error(`toolchain command is absent from the actual PATH: ${command}`);
}
function processPacket(command: string[], cwd: string, environment: NodeJS.ProcessEnv) {
  const result = spawnSync(command[0], command.slice(1), {
    cwd,
    env: environment,
    maxBuffer: 1024 * 1024,
  });
  return {
    command,
    cwd,
    stdout: bytes(result.stdout ?? Buffer.alloc(0)),
    stderr: bytes(result.stderr ?? Buffer.alloc(0)),
    exitStatus: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
  };
}
type Packet = ReturnType<typeof processPacket>;
function successful(packet: Packet) {
  for (const stream of ["stdout", "stderr"] as const) {
    record(packet[stream]);
    assert.equal(typeof packet[stream].base64, "string");
    assert.deepEqual(packet[stream], bytes(Buffer.from(packet[stream].base64, "base64")));
  }
  assert.equal(packet.exitStatus, 0);
  assert.equal(packet.signal, null);
  assert.equal(packet.error, null);
  return Buffer.from(packet.stdout.base64, "base64").toString();
}
function environmentSelection(environment: NodeJS.ProcessEnv) {
  return Object.fromEntries(selectionKeys.map((key) => [key, environment[key] ?? null]));
}
function write(nativeDir: string, name: string, value: unknown) {
  const directory = path.join(nativeDir, ".artifacts/native/formatter-history/toolchain");
  fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(path.join(directory, name), `${JSON.stringify(value)}\n`);
}
function version(image: Image, args: string[], cwd: string, environment: NodeJS.ProcessEnv) {
  return processPacket([image.realpath, ...args], cwd, environment);
}
export interface NativeHistoryToolchain {
  schema: "vize.native-history-real-toolchain";
  version: 1;
  cwd: string;
  ambient: ReturnType<typeof environmentSelection>;
  rustup: Image;
  requested: { rustc: Image; cargo: Image };
  selection: { rustc: Packet; cargo: Packet };
  before: { rustc: Image; cargo: Image };
  buildEnvironment: { CARGO: string; RUSTC: string; PATH: string };
  versions: { rustc: Packet; cargo: Packet };
}

/** Select in the producer's build cwd, then use those same real binaries for NAPI and metadata. */
export function prepareNativeHistoryToolchain(nativeDir: string, environment: NodeJS.ProcessEnv) {
  const cwd = fs.realpathSync(nativeDir);
  const ambient = environmentSelection(environment);
  const journal: Record<string, unknown> = { cwd, ambient };
  const persist = () => write(nativeDir, "selection.json", journal);
  persist();
  try {
    for (const key of wrapperKeys)
      assert(!environment[key], `unqualified toolchain wrapper: ${key}`);
    const rustup = nativeHistoryToolImage(executable("rustup", cwd, environment), cwd);
    journal.rustup = rustup;
    const requested = {
      rustc: nativeHistoryToolImage(
        executable(environment.RUSTC ?? "rustc", cwd, environment),
        cwd,
      ),
      cargo: nativeHistoryToolImage(
        executable(environment.CARGO ?? "cargo", cwd, environment),
        cwd,
      ),
    };
    journal.requested = requested;
    const selection = {
      rustc: processPacket([rustup.realpath, "which", "rustc"], cwd, environment),
      cargo: processPacket([rustup.realpath, "which", "cargo"], cwd, environment),
    };
    journal.selection = selection;
    persist();
    function selected(packet: Packet) {
      const output = successful(packet);
      const file = output.trim();
      assert(path.isAbsolute(file) && output === `${file}\n`, "ambiguous rustup selection");
      return nativeHistoryToolImage(file, cwd);
    }
    const before = { rustc: selected(selection.rustc), cargo: selected(selection.cargo) };
    journal.before = before;
    assert.equal(path.basename(before.rustc.realpath), "rustc");
    assert.equal(path.basename(before.cargo.realpath), "cargo");
    assert.equal(path.dirname(before.rustc.realpath), path.dirname(before.cargo.realpath));
    for (const tool of ["rustc", "cargo"] as const)
      assert(
        (requested[tool].sha256 === rustup.sha256 &&
          path.basename(requested[tool].path) === tool) ||
          requested[tool].realpath === before[tool].realpath,
        `ambient ${tool} is neither the selected real image nor the authenticated rustup proxy`,
      );
    if (environment.CARGO_BUILD_RUSTC)
      assert.equal(
        fs.realpathSync(executable(environment.CARGO_BUILD_RUSTC, cwd, environment)),
        before.rustc.realpath,
        "unqualified CARGO_BUILD_RUSTC override",
      );
    const buildEnvironment = {
      CARGO: before.cargo.realpath,
      RUSTC: before.rustc.realpath,
      PATH: `${path.dirname(before.cargo.realpath)}${path.delimiter}${environment.PATH ?? ""}`,
    };
    const pinned = { ...environment, ...buildEnvironment };
    const versions = {
      rustc: version(before.rustc, ["-Vv"], cwd, pinned),
      cargo: version(before.cargo, ["-V"], cwd, pinned),
    };
    journal.buildEnvironment = buildEnvironment;
    journal.versions = versions;
    persist();
    successful(versions.rustc);
    successful(versions.cargo);
    assert.deepEqual(nativeHistoryToolImage(rustup.path, cwd), rustup);
    for (const tool of ["rustc", "cargo"] as const)
      assert.deepEqual(nativeHistoryToolImage(before[tool].path, cwd), before[tool]);
    const provider: NativeHistoryToolchain = {
      schema: "vize.native-history-real-toolchain",
      version: 1,
      cwd,
      ambient,
      rustup,
      requested,
      selection,
      before,
      buildEnvironment,
      versions,
    };
    journal.qualified = true;
    persist();
    return { provider, environment: pinned };
  } catch (error) {
    journal.error =
      error instanceof Error ? { message: error.message, stack: error.stack } : String(error);
    persist();
    throw error;
  }
}

let observation = 0;
/** Every consumer keeps image equality and FULL version checks, using producer-bound absolute paths. */
export function validateNativeHistoryToolchain(nativeDir: string, value: unknown, phase: string) {
  record(value);
  assert.deepEqual(Object.keys(value), [
    "schema",
    "version",
    "cwd",
    "ambient",
    "rustup",
    "requested",
    "selection",
    "before",
    "buildEnvironment",
    "versions",
  ]);
  assert.equal(value.schema, "vize.native-history-real-toolchain");
  assert.equal(value.version, 1);
  assert.equal(value.cwd, fs.realpathSync(nativeDir));
  record(value.before);
  record(value.buildEnvironment);
  record(value.versions);
  record(value.ambient);
  record(value.rustup);
  record(value.requested);
  record(value.selection);
  assert.deepEqual(Object.keys(value.ambient), selectionKeys);
  for (const key of selectionKeys)
    assert(value.ambient[key] === null || typeof value.ambient[key] === "string");
  for (const key of wrapperKeys) {
    assert(!value.ambient[key], `unqualified producer wrapper: ${key}`);
    assert(!process.env[key], `unqualified observer wrapper: ${key}`);
  }
  const cwd = value.cwd as string;
  const observed: Record<string, unknown> = { phase, pid: process.pid, cwd };
  const ordinal = ++observation;
  const persist = () => write(nativeDir, `observation-${process.pid}-${ordinal}.json`, observed);
  try {
    assert(typeof value.rustup.path === "string");
    assert.deepEqual(nativeHistoryToolImage(value.rustup.path, cwd), value.rustup);
    for (const tool of ["rustc", "cargo"] as const) {
      record(value.requested[tool]);
      const requested: Record<string, unknown> = value.requested[tool];
      assert(typeof requested.path === "string");
      assert.deepEqual(nativeHistoryToolImage(requested.path, cwd), requested);
    }
    const actual: Record<string, Image> = {};
    const versions: Record<string, Packet> = {};
    for (const tool of ["rustc", "cargo"] as const) {
      record(value.before[tool]);
      const expected: Record<string, unknown> = value.before[tool];
      assert(typeof expected.path === "string");
      actual[tool] = nativeHistoryToolImage(expected.path, cwd);
      observed.images = actual;
      persist();
      assert.deepEqual(actual[tool], expected, `physical ${tool} changed since producer selection`);
      assert.equal(
        value.buildEnvironment[tool === "cargo" ? "CARGO" : "RUSTC"],
        actual[tool].realpath,
      );
      record(value.selection[tool]);
      const selection = value.selection[tool] as unknown as Packet;
      assert.deepEqual(selection.command, [value.rustup.realpath, "which", tool]);
      assert.equal(selection.cwd, cwd);
      assert.equal(successful(selection), `${actual[tool].path}\n`);
      const requested = value.requested[tool] as Image;
      assert(
        (requested.sha256 === value.rustup.sha256 && path.basename(requested.path) === tool) ||
          requested.realpath === actual[tool].realpath,
      );
      record(value.versions[tool]);
      const oldVersion = value.versions[tool] as unknown as Packet;
      assert.deepEqual(oldVersion.command, [
        actual[tool].realpath,
        tool === "rustc" ? "-Vv" : "-V",
      ]);
      assert.equal(oldVersion.cwd, cwd);
      const environment = { ...process.env, ...value.buildEnvironment } as NodeJS.ProcessEnv;
      versions[tool] = version(actual[tool], tool === "rustc" ? ["-Vv"] : ["-V"], cwd, environment);
      observed.versions = versions;
      persist();
      assert.equal(
        successful(versions[tool]),
        successful(oldVersion),
        `full ${tool} version changed`,
      );
      assert.deepEqual(nativeHistoryToolImage(expected.path, cwd), expected);
    }
    assert.equal(path.dirname(actual.rustc.realpath), path.dirname(actual.cargo.realpath));
    assert.deepEqual(value.buildEnvironment, {
      CARGO: actual.cargo.realpath,
      RUSTC: actual.rustc.realpath,
      PATH: `${path.dirname(actual.cargo.realpath)}${path.delimiter}${value.ambient.PATH ?? ""}`,
    });
    observed.qualified = true;
    persist();
    return {
      rustc: successful(versions.rustc),
      cargo: successful(versions.cargo),
      observation: { phase, pid: process.pid, cwd, images: actual, versions, qualified: true },
    };
  } catch (error) {
    observed.error =
      error instanceof Error ? { message: error.message, stack: error.stack } : String(error);
    persist();
    throw error;
  }
}
