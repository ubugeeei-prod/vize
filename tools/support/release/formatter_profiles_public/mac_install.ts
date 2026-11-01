import assert from "node:assert/strict";
import { createRequire } from "node:module";
import {
  closeSync,
  constants,
  fstatSync,
  ftruncateSync,
  lstatSync,
  mkdirSync,
  openSync,
  realpathSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { asArray, asRecord, asString, readJsonRecord } from "../../json-value.ts";
import { npmArgv, Recorder, success } from "../../config/windows_consumer/common.ts";
import { digest, inside, inventory } from "../editor_vite_stub/filesystem.ts";
import {
  assertLockedPackage,
  validateNativeReceipt,
} from "../constructor_original27_public/authority.ts";
import { assertMacHost } from "./mac_contract.ts";
import type { MacAuthority } from "./mac_authority.ts";

const platformPackage = "@vizejs/native-darwin-arm64";
export const macInstallArgs = (version: string): string[] => ["install", "vize@" + version];

export function macRootSeal(root: string) {
  const stat = lstatSync(root);
  assert(stat.isDirectory() && !stat.isSymbolicLink());
  assert.equal(realpathSync(root), root);
  return { root, dev: stat.dev, ino: stat.ino };
}
export type MacRootSeal = ReturnType<typeof macRootSeal>;
export function assertMacInputs(seal: MacRootSeal): void {
  assert.deepEqual(macRootSeal(seal.root), seal, "the owned consumer directory identity changed");
  for (const name of ["a.vue", "Unselected.vue", "vite.config.mjs"]) {
    const file = join(seal.root, name),
      stat = lstatSync(file);
    assert(
      stat.isFile() && !stat.isSymbolicLink() && stat.nlink === 1,
      "command input must be one owned regular file: " + name,
    );
    assert.equal(realpathSync(file), file);
  }
}
export function resetMacInput(seal: MacRootSeal, source: string): void {
  assertMacInputs(seal);
  const fd = openSync(join(seal.root, "a.vue"), constants.O_WRONLY | constants.O_NOFOLLOW);
  try {
    const stat = fstatSync(fd);
    assert(stat.isFile() && stat.nlink === 1);
    // Truncate only after validating the opened non-symlink descriptor.
    ftruncateSync(fd, 0);
    writeFileSync(fd, source);
  } finally {
    closeSync(fd);
  }
}

/** A separate process loads only the freshly installed arm64 addon, then exposes its real identity. */
export function probeMac(rootArg: string, output: string): void {
  assertMacHost(process.platform, process.arch, process.version);
  const root = realpathSync(rootArg),
    app = createRequire(join(root, "package.json"));
  const entry = realpathSync(app.resolve("vize")),
    cliRoot = resolve(dirname(entry), "..");
  const manifest = readJsonRecord(join(cliRoot, "package.json"));
  assert.equal(manifest.name, "vize");
  assert.equal(realpathSync(resolve(cliRoot, asString(manifest.main))), entry);
  const cli = realpathSync(resolve(cliRoot, asString(asRecord(manifest.bin).vize)));
  const fromCli = createRequire(entry),
    nativeEntry = realpathSync(fromCli.resolve("@vizejs/native"));
  const fromNative = createRequire(nativeEntry),
    binary = realpathSync(fromNative.resolve(platformPackage));
  const native = asRecord(fromCli("@vizejs/native"));
  const helper = asRecord(fromNative("./native-binding-identity.js"));
  const token = native.getNativeBuildToken,
    identity = helper.getBindingIdentity;
  assert.equal(typeof token, "function");
  assert.equal(typeof identity, "function");
  const binding = asRecord(
    Reflect.apply(identity as (...args: unknown[]) => unknown, helper, [native]),
  );
  assert.equal(binding.kind, "native");
  assert.equal(binding.token, Reflect.apply(token as (...args: unknown[]) => unknown, native, []));
  assert.equal(fromNative.cache[binary]?.loaded, true);
  const roots = [cliRoot, dirname(nativeEntry), dirname(binary)].map((path) => realpathSync(path));
  for (const file of [entry, cli, nativeEntry, binary])
    assert(inside(root, file) && statSync(file).isFile());
  for (const path of roots) assert(inside(root, path));
  writeFileSync(
    output,
    JSON.stringify(
      {
        node: { path: process.execPath, version: process.version, ...digest(process.execPath) },
        platform: process.platform,
        arch: process.arch,
        targetPlatform: "darwin-arm64",
        cli,
        cliEntry: entry,
        cliWrapper: digest(cli),
        cliModule: digest(entry),
        nativeEntry,
        nativeWrapper: digest(nativeEntry),
        binary: { path: binary, ...digest(binary) },
        identity: binding,
        packageRoots: roots,
        versions: roots.map((path) => ({
          path,
          ...readJsonRecord(join(path, "package.json")),
          manifestDigest: digest(join(path, "package.json")),
        })),
      },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
}

export function installMac(root: string, authority: MacAuthority, record: Recorder) {
  assertMacHost(process.platform, process.arch, process.version);
  mkdirSync(root, { recursive: false });
  assert.deepEqual(
    inventory(root).map((row) => row.path),
    [""],
    "literal install starts in an empty directory",
  );
  record.json("install-policy.json", {
    literalCommand: "npm install vize@" + authority.version,
    argv: macInstallArgs(authority.version),
    lifecycleScripts: "npm default, not suppressed",
    registry: "https://registry.npmjs.org/ via existing Recorder cleanEnv",
    adaptation:
      "npm bundled with exact Node25.8.1; fresh noninteractive Actions host, not the reporter's exact macOS image",
  });
  success(record.run("npm-version", npmArgv("--version"), root));
  const lifecycle = record.run(
    "npm-default-lifecycle-policy",
    npmArgv("config", "get", "ignore-scripts"),
    root,
  );
  success(lifecycle);
  assert.equal(
    lifecycle.stdout,
    "false\n",
    "literal original install requires npm's enabled default lifecycle policy",
  );
  success(
    record.run(
      "public-install",
      npmArgv(...macInstallArgs(authority.version)),
      root,
      undefined,
      600_000,
    ),
  );
  const before = inventory(root);
  record.json("installed-before-identity.json", before);
  const output = join(record.root, "installed-identity.json");
  success(
    record.run(
      "installed-identity",
      [process.execPath, fileURLToPath(import.meta.url), "--probe", root, output],
      root,
    ),
  );
  const after = inventory(root);
  record.json("installed-after-identity.json", after);
  assert.deepEqual(
    after,
    before,
    "identity probe must preserve all installed and project bytes/links",
  );
  const identity = readJsonRecord(output);
  assert.equal(identity.platform, "darwin");
  assert.equal(identity.arch, "arm64");
  const versions = asArray(identity.versions).map((value) => asRecord(value));
  assert.deepEqual(
    versions.map((row) => row.name),
    ["vize", "@vizejs/native", platformPackage],
  );
  for (const row of versions) assert.equal(row.version, authority.version);
  const binary = asRecord(identity.binary),
    binding = asRecord(identity.identity);
  validateNativeReceipt(
    authority.capsule,
    {
      target: "darwin-arm64",
      bytes: Number(binary.bytes),
      sha256: asString(binary.sha256),
      token: asString(binding.token),
    },
    authority.version,
    authority.candidate,
    authority.run,
  );
  const lock = readJsonRecord(join(root, "package-lock.json"));
  record.json("installed-lock.json", lock);
  record.json("installed-manifest.json", readJsonRecord(join(root, "package.json")));
  for (const name of ["vize", "@vizejs/native", platformPackage]) {
    assertLockedPackage(lock, authority.capsule, name, authority.version);
    const packet = record.run(
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
    success(packet);
    const dist = asRecord(JSON.parse(packet.stdout) as unknown),
      expected = asRecord(asRecord(authority.capsule.npmSRI)[name]);
    assert.equal(dist.integrity, expected.integrity);
    assert.equal(dist.tarball, expected.tarball);
  }
  const final = inventory(root);
  record.json("installed-complete.json", final);
  assert.deepEqual(final, before, "all registry reads must preserve the whole installation");
  const rootSeal = macRootSeal(root);
  record.json("owned-root-seal.json", rootSeal);
  return {
    root,
    rootSeal,
    identity,
    cli: asString(identity.cli),
    record,
    version: authority.version,
  };
}
export type MacContext = ReturnType<typeof installMac>;

if (import.meta.main) {
  assert.equal(process.argv[2], "--probe");
  const [root, output] = process.argv.slice(3);
  assert(root && output);
  probeMac(root, output);
}
