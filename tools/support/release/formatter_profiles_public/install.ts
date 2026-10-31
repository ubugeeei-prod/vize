import assert from "node:assert/strict";
import { createRequire } from "node:module";
import {
  existsSync,
  lstatSync,
  readFileSync,
  realpathSync,
  rmSync,
  rmdirSync,
  mkdirSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { asArray, asRecord, asString, readJsonRecord, type JsonRecord } from "../../json-value.ts";
import { npmArgv, Recorder, success, type Packet } from "../../config/windows_consumer/common.ts";
import { digest, inside, inventory, type Entry } from "../editor_vite_stub/filesystem.ts";
import { hook, observed, verifyJournal } from "../editor_vite_stub/custody.ts";
import {
  assertLockedPackage,
  validateNativeReceipt,
} from "../constructor_original27_public/authority.ts";
import type { Authority, OriginalContract } from "./authority.ts";
import { auditCache, captureCache } from "./cache.ts";

/** This child observes only modules resolved from the fresh registry installation. */
export function probe(rootArg: string, output: string): void {
  const root = realpathSync(rootArg),
    fromApp = createRequire(join(root, "package.json"));
  const entry = realpathSync(fromApp.resolve("vize")),
    cliRoot = resolve(dirname(entry), "..");
  const manifest = readJsonRecord(join(cliRoot, "package.json"));
  assert.equal(manifest.name, "vize");
  assert.equal(realpathSync(resolve(cliRoot, asString(manifest.main))), entry);
  const cli = realpathSync(resolve(cliRoot, asString(asRecord(manifest.bin).vize)));
  const fromCli = createRequire(entry),
    nativeEntry = realpathSync(fromCli.resolve("@vizejs/native"));
  const fromNative = createRequire(nativeEntry),
    native = asRecord(fromCli("@vizejs/native"));
  const token = native.getNativeBuildToken;
  assert.equal(typeof token, "function");
  const buildToken: unknown = Reflect.apply(token as (...args: unknown[]) => unknown, native, []);
  const helper = asRecord(fromNative("./native-binding-identity.js"));
  assert.equal(typeof helper.getBindingIdentity, "function");
  const binding = asRecord(
    Reflect.apply(helper.getBindingIdentity as (...args: unknown[]) => unknown, helper, [native]),
  );
  assert.equal(binding.kind, "native");
  assert.equal(binding.token, buildToken);
  const platform = "@vizejs/native-linux-x64-gnu";
  const binary = realpathSync(fromNative.resolve(platform));
  const packageRoots = [cliRoot, dirname(nativeEntry), dirname(binary)].map((path) =>
    realpathSync(path),
  );
  for (const file of [entry, cli, nativeEntry, binary]) {
    assert(inside(root, file) && statSync(file).isFile(), "installed provider ownership: " + file);
  }
  for (const path of packageRoots) assert(inside(root, path));
  assert.equal(fromNative.cache[binary]?.loaded, true);
  const versions = packageRoots.map((path) => {
    const value = readJsonRecord(join(path, "package.json"));
    return {
      name: asString(value.name),
      version: asString(value.version),
      path,
      manifest: digest(join(path, "package.json")),
    };
  });
  writeFileSync(
    output,
    JSON.stringify(
      {
        node: { path: process.execPath, version: process.version, ...digest(process.execPath) },
        platform: process.platform,
        arch: process.arch,
        targetPlatform: "linux-x64",
        cli,
        cliEntry: entry,
        cliWrapper: digest(cli),
        cliModule: digest(entry),
        nativeEntry,
        nativeWrapper: digest(nativeEntry),
        binary: { path: binary, ...digest(binary) },
        identity: binding,
        packageRoots,
        versions,
      },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
}

export function packagePayload(rows: Entry[], roots: string[]): Entry[] {
  return rows.filter((row) =>
    roots.some((root) => row.path === root || row.path.startsWith(root + "/")),
  );
}
export function expectStatus(packet: Packet, expected: number): void {
  assert.equal(packet.error, null, packet.label);
  assert.equal(packet.signal, null, packet.label);
  assert.equal(packet.status, expected, packet.label + "\n" + packet.stdout + packet.stderr);
}

export type Context = {
  root: string;
  version: string;
  cli: string;
  identity: JsonRecord;
  original: OriginalContract;
  record: Recorder;
  payloadRoots: string[];
  payload: Entry[];
};
export function preservePayload(context: Context, label: string): Entry[] {
  const rows = inventory(join(context.root, "node_modules"));
  context.record.json(label + ".modules.json", rows);
  assert.deepEqual(
    packagePayload(rows, context.payloadRoots),
    context.payload,
    "published package bytes and links must remain exact",
  );
  return rows;
}

export function assertVersionUnchanged(
  before: { project: Entry[]; modules: Entry[] },
  after: { project: Entry[]; modules: Entry[] },
): void {
  assert.deepEqual(after, before, "--version must preserve the whole project and installation");
}

export function assertAuthoredUnchanged(before: Entry[], after: Entry[], writable: string[]): void {
  // CLI config-cache effects are retained separately; published packages have
  // their own exact payload guard. This is only the authored-file boundary.
  const protectedRows = (rows: Entry[]) =>
    rows.filter(
      (row) =>
        row.path !== "node_modules" &&
        !row.path.startsWith("node_modules/") &&
        !writable.includes(row.path),
    );
  assert.deepEqual(
    protectedRows(after),
    protectedRows(before),
    "unselected authored inputs must remain exact",
  );
  for (const file of writable) {
    assert.equal(before.find((row) => row.path === file)?.kind, "file");
    assert.equal(after.find((row) => row.path === file)?.kind, "file");
  }
}

/** Reset only already-verified owned fixture files between original cases. */
export function resetAuthoredCase(context: Context, label: string): void {
  const names = [
    "Original雪🌸.js",
    "Original雪🌸.ts",
    "UserCard.vue",
    "imports.ts",
    "config.json",
    "eval-count.txt",
    "App.vue",
    "vite.config.mjs",
    "nested/UserCard.vue",
    "nested/Ignored.vue",
  ];
  const deleted: { name: string; rawBase64: string }[] = [];
  for (const name of names) {
    const path = join(context.root, name);
    if (!existsSync(path)) continue;
    assert(lstatSync(path).isFile() && !lstatSync(path).isSymbolicLink());
    deleted.push({ name, rawBase64: readFileSync(path).toString("base64") });
    context.record.json(label + ".setup-reset.json", { names, deleted });
    rmSync(path);
  }
  const nested = join(context.root, "nested");
  if (existsSync(nested)) rmdirSync(nested);
  context.record.json(label + ".setup-reset.json", {
    names,
    deleted,
    scope:
      "only controlled fixture files, after their previous output/foreign-file guards; installation untouched",
  });
}

/** Config-cache effects remain visible in complete module snapshots. */
export function cli(
  context: Context,
  label: string,
  args: string[],
  expected: number,
  files: string[] = ["vite.config.mjs"],
): Packet {
  const authoredBefore = inventory(context.root);
  context.record.json(label + ".project-before.json", authoredBefore);
  const capture = (phase: string) =>
    context.record.json(
      label + ".authored-" + phase + ".json",
      files.map((name) => {
        const path = resolve(context.root, name);
        assert(inside(context.root, path));
        return { name, rawBase64: existsSync(path) ? readFileSync(path).toString("base64") : null };
      }),
    );
  capture("before");
  const before = preservePayload(context, label + "-before");
  const cacheBefore = captureCache(
    context.root,
    context.version,
    before,
    context.record,
    label + "-before",
  );
  const custody = observed(context.root, context.identity, context.record, label);
  const packet = context.record.run(
    label,
    [process.execPath, "--require", hook, context.cli, "fmt", ...args],
    context.root,
    custody.env,
  );
  capture("after");
  const authoredAfter = inventory(context.root);
  context.record.json(label + ".project-after.json", authoredAfter);
  const after = preservePayload(context, label + "-after");
  const cacheAfter = captureCache(
    context.root,
    context.version,
    after,
    context.record,
    label + "-after",
  );
  context.record.json(label + ".filesystem.json", {
    before,
    after,
    scope:
      "complete retained module vectors; all rows enforced with only source-owned validated config-cache transitions",
  });
  const explicit =
    args.includes("--no-config") ||
    args.includes("config.json") ||
    args.some((flag) => flag === "--semi=invalid") ||
    (args.includes("--no-semi") &&
      args.some((flag) => flag === "--semi" || flag === "--semi=false"))
      ? null
      : args.includes("--config");
  auditCache(cacheBefore, cacheAfter, explicit, context.record, label);
  expectStatus(packet, expected);
  const writable =
    expected === 0 && args.includes("--write")
      ? files.filter(
          (file) =>
            !["vite.config.mjs", "config.json", "package.json", "package-lock.json"].includes(file),
        )
      : [];
  assertAuthoredUnchanged(authoredBefore, authoredAfter, writable);
  // The successful fmt path returns to Node. Rust error/check exits bypass its exit event.
  verifyJournal(custody.journal, context.identity, expected === 0 ? 0 : null);
  return packet;
}

export function config(root: string, formatter: JsonRecord): Buffer {
  // The native fixture writer uses serde_json's compact sorted-map carrier.
  const sorted = JSON.parse(
    JSON.stringify(formatter, (_key, value: unknown) =>
      value && typeof value === "object" && !Array.isArray(value)
        ? Object.fromEntries(Object.entries(value).sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)))
        : value,
    ),
  ) as unknown;
  const bytes = Buffer.from(
    "export default { vue: " + JSON.stringify({ formatter: sorted }) + " };\n",
  );
  writeFileSync(join(root, "vite.config.mjs"), bytes);
  return bytes;
}

export function install(
  root: string,
  authority: Authority,
  original: OriginalContract,
  record: Recorder,
): Context {
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  assert.equal(process.version, "v24.14.0");
  mkdirSync(root, { recursive: false });
  writeFileSync(
    join(root, "package.json"),
    JSON.stringify(
      { private: true, type: "module", dependencies: { vize: authority.version } },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
  success(
    record.run(
      "public-install",
      npmArgv(
        "install",
        "--prefix",
        root,
        "--ignore-scripts",
        "--no-audit",
        "--no-fund",
        "--registry=https://registry.npmjs.org",
      ),
      root,
      process.env,
      600_000,
    ),
  );
  const installedModules = inventory(join(root, "node_modules"));
  record.json("installed-before-identity.modules.json", installedModules);
  const identityPath = join(record.root, "installed-identity.json");
  success(
    record.run(
      "installed-identity",
      [process.execPath, fileURLToPath(import.meta.url), "--probe", root, identityPath],
      root,
    ),
  );
  const identityModules = inventory(join(root, "node_modules"));
  record.json("installed-after-identity.modules.json", identityModules);
  assert.deepEqual(
    identityModules,
    installedModules,
    "identity observation must preserve the entire fresh installation",
  );
  const identity = readJsonRecord(identityPath);
  assert.equal(identity.platform, "linux");
  assert.equal(identity.arch, "x64");
  const versions = identity.versions;
  assert(Array.isArray(versions));
  assert.deepEqual(
    versions.map((value: unknown) => asRecord(value).name),
    ["vize", "@vizejs/native", "@vizejs/native-linux-x64-gnu"],
  );
  for (const value of versions) assert.equal(asRecord(value).version, authority.version);
  const binary = asRecord(identity.binary),
    binding = asRecord(identity.identity);
  validateNativeReceipt(
    authority.capsule,
    {
      target: "linux-x64",
      bytes: Number(binary.bytes),
      sha256: asString(binary.sha256),
      token: asString(binding.token),
    },
    authority.version,
    authority.candidate,
    authority.run,
  );
  const lock = readJsonRecord(join(root, "package-lock.json"));
  for (const name of ["vize", "@vizejs/native", "@vizejs/native-linux-x64-gnu"]) {
    assertLockedPackage(lock, authority.capsule, name, authority.version);
    const result = record.run(
      "registry-" + name.replaceAll(/[@/]/gu, "-"),
      npmArgv(
        "view",
        name + "@" + authority.version,
        "dist",
        "--json",
        "--registry=https://registry.npmjs.org",
      ),
      root,
    );
    success(result);
    const dist = asRecord(JSON.parse(result.stdout) as unknown),
      expected = asRecord(asRecord(authority.capsule.npmSRI)[name]);
    assert.equal(dist.integrity, expected.integrity);
    assert.equal(dist.tarball, expected.tarball);
  }
  record.json("installed-package-lock.json", lock);
  record.json("installed-package.json", readJsonRecord(join(root, "package.json")));
  const payloadRoots = asArray(identity.packageRoots).map((value) =>
    relative(join(root, "node_modules"), asString(value)).replaceAll("\\", "/"),
  );
  assert(payloadRoots.every((path) => path && !path.startsWith("..") && !path.startsWith("/")));
  const modules = inventory(join(root, "node_modules"));
  record.json("installed-modules.json", modules);
  assert.deepEqual(
    modules,
    installedModules,
    "registry metadata reads must preserve the entire fresh installation",
  );
  const context: Context = {
    root,
    version: authority.version,
    cli: asString(identity.cli),
    identity,
    original,
    record,
    payloadRoots,
    payload: packagePayload(modules, payloadRoots),
  };
  const versionBefore = {
    project: inventory(root),
    modules: inventory(join(root, "node_modules")),
  };
  record.json("version-before.filesystem.json", versionBefore);
  const custody = observed(root, identity, record, "public-version");
  const version = record.run(
    "public-version",
    [process.execPath, "--require", hook, context.cli, "--version"],
    root,
    custody.env,
  );
  const versionAfter = { project: inventory(root), modules: inventory(join(root, "node_modules")) };
  record.json("version-after.filesystem.json", versionAfter);
  assertVersionUnchanged(versionBefore, versionAfter);
  success(version);
  assert.equal(version.stdout, "vize " + authority.version + "\n");
  verifyJournal(custody.journal, identity, null);
  return context;
}

if (import.meta.main) {
  assert.equal(process.argv[2], "--probe");
  const [root, output] = process.argv.slice(3);
  assert(root && output);
  probe(root, output);
}
