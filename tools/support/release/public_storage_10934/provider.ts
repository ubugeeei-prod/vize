import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import type { Authority } from "./authority.ts";
import { asRecord, asString, inside, parse, read, run, save, sha256 } from "./io.ts";

export const NATIVE_PACKAGE = "@vizejs/native-linux-x64-gnu";
const GO_PACKAGE = "@typescript/typescript-linux-x64";
const GO_VERSION = "7.1.0-beta";
const PLATFORMS = [
  "darwin-arm64", "darwin-x64", "linux-arm64-gnu", "linux-arm64-musl",
  "linux-x64-gnu", "linux-x64-musl", "win32-arm64-msvc", "win32-x64-msvc",
];
interface RegistryRecord {
  name: string;
  version: string;
  integrity: string;
  tarball: string;
  metadataUrl: string;
  metadataSha256: string;
}
async function registry(name: string, version: string, evidence: string): Promise<RegistryRecord> {
  const metadataUrl = "https://registry.npmjs.org/" + encodeURIComponent(name) + "/" + version;
  const response = await fetch(metadataUrl);
  assert.equal(response.status, 200, metadataUrl);
  const bytes = new Uint8Array(await response.arrayBuffer());
  const metadata = asRecord(parse(bytes));
  assert.equal(metadata.name, name);
  assert.equal(metadata.version, version);
  const dist = asRecord(metadata.dist);
  const integrity = asString(dist.integrity);
  assert.match(integrity, /^sha512-[A-Za-z0-9+/]+=*$/);
  const tarball = asString(dist.tarball);
  assert.equal(new URL(tarball).origin, "https://registry.npmjs.org");
  fs.writeFileSync(path.join(evidence, "registry-" + encodeURIComponent(name) + ".raw.json"), bytes);
  return { name, version, integrity, tarball, metadataUrl, metadataSha256: sha256(bytes) };
}
export async function bootstrap(root: string, evidence: string, authority: Authority) {
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  assert.ok(process.report);
  const report: unknown = process.report.getReport();
  assert.equal(typeof asRecord(asRecord(report).header).glibcVersionRuntime, "string");
  const names = ["vize", "@vizejs/native", ...PLATFORMS.map((item) => "@vizejs/native-" + item)];
  const registryRecords = await Promise.all(names.map((name) => registry(name, authority.version, evidence)));
  for (const record of registryRecords) {
    const expected = asRecord(authority.npmSRI[record.name]);
    assert.equal(record.integrity, expected.integrity, record.name);
    assert.equal(record.tarball, expected.tarball, record.name);
  }
  const extra = await Promise.all([
    registry("typescript", GO_VERSION, evidence),
    registry(GO_PACKAGE, GO_VERSION, evidence),
    registry("vue", "3.6.0-rc.13", evidence),
  ]);
  const provider = path.join(root, "provider");
  fs.mkdirSync(provider);
  assert.equal(fs.existsSync(path.join(provider, "node_modules")), false);
  save(path.join(provider, "package.json"), {
    name: "vize-public-storage-10934", private: true, type: "module",
    dependencies: {
      vize: authority.version, "@vizejs/native": authority.version,
      [NATIVE_PACKAGE]: authority.version, vue: "3.6.0-rc.13",
      "typescript-native": "npm:typescript@" + GO_VERSION,
    },
  });
  const config = path.join(provider, ".npmrc");
  fs.writeFileSync(config, "registry=https://registry.npmjs.org/\n");
  const args = [
    "install", "--ignore-scripts", "--legacy-peer-deps", "--no-audit", "--no-fund",
    "--prefer-online", "--userconfig", config, "--cache", path.join(root, "npm-cache"),
  ];
  run("npm", args, provider, path.join(evidence, "install"));
  const lockFile = path.join(provider, "package-lock.json");
  const packages = asRecord(asRecord(read(lockFile)).packages);
  for (const record of [...registryRecords, ...extra]) {
    const installedName = record.name === "typescript" ? "typescript-native" : record.name;
    const entry = asRecord(packages["node_modules/" + installedName]);
    assert.equal(entry.version, record.version, record.name);
    assert.equal(entry.integrity, record.integrity, record.name);
    assert.equal(entry.resolved, record.tarball, record.name);
  }
  for (const name of ["vize", "@vizejs/native", NATIVE_PACKAGE]) {
    const directory = inside(provider, path.join(provider, "node_modules", name));
    assert.equal(asRecord(read(path.join(directory, "package.json"))).version, authority.version);
  }
  const backend = inside(provider, path.join(provider, "node_modules", GO_PACKAGE, "lib/tsc"));
  const backendBytes = fs.readFileSync(backend);
  assert.equal(backendBytes.subarray(0, 4).toString("hex"), "7f454c46", "native backend must be ELF");
  const backendVersion = run(backend, ["--version"], provider, path.join(evidence, "backend-version"));
  assert.equal(backendVersion.stdout.trim(), "Version " + GO_VERSION);
  const backendIdentity = {
    path: backend, package: GO_PACKAGE, version: GO_VERSION,
    sha256: sha256(backendBytes), bytes: backendBytes.length,
    registry: extra.find((item) => item.name === GO_PACKAGE),
  };
  fs.copyFileSync(lockFile, path.join(evidence, "package-lock.json"));
  fs.copyFileSync(path.join(provider, "package.json"), path.join(evidence, "provider-package.json"));
  save(path.join(evidence, "provider-custody.json"), {
    provider, node: process.version, platform: process.platform, arch: process.arch,
    registryRecords, extraRegistryRecords: extra, lockSha256: sha256(fs.readFileSync(lockFile)),
    backend: backendIdentity, installScripts: false, sourceOverride: false,
  });
  return { provider, backend, backendIdentity };
}
