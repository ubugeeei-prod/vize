import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import {
  installedHtmlPreflight,
  sourceRoot,
  wholeProcessError,
} from "./oxlint-installed-html-authority.ts";
import { exactPath, sha256 } from "./n8n-installed-authority.ts";

// Adapt the unchanged reviewed collector libraries; never extract or execute a public archive.
export const stockModules = [
  "tools/support/release/public_install/identity.ts",
  "tools/support/release/public_install/registry.ts",
  "tools/support/release/public_install/archive.ts",
  "tools/support/archive/tar.ts",
];
export const stockArchiveProgram = String.raw`
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
const digest = (data) => createHash("sha256").update(data).digest("hex");
const plans = JSON.parse(fs.readFileSync(0, "utf8"));
const names = Array.isArray(plans) ? plans.map((item) => item?.name) : [];
const versions = new Set(Array.isArray(plans) ? plans.map((item) => item?.version) : []);
if (!Array.isArray(plans) || plans.length < 2 || plans.length > 10
    || versions.size !== 1 || !(versions.has("1.78.0") || versions.has("1.86.0"))
    || !names.includes("oxlint") || !names.includes("@oxlint/binding-darwin-arm64")
    || names.some((name) => name !== "oxlint" && !String(name ?? "").startsWith("@oxlint/binding-")))
  throw new Error("finite pinned stock wrapper and native public archive plans required");
const root = process.argv[1];
const modules = [];
for (const name of ${JSON.stringify(stockModules)}) {
  const filename = path.join(root, name);
  if (fs.realpathSync(filename) !== filename) throw new Error("reviewed collector module path redirected");
  const data = fs.readFileSync(filename);
  modules.push({ path: filename, sha256: digest(data), bytes: data.length });
}
const load = (name) => import(pathToFileURL(path.join(root, "tools/support/release/public_install", name)).href);
const identity = await load("identity.ts");
const registry = await load("registry.ts");
const archive = await load("archive.ts");
identity.rejectOverrides(process.env);
const proofs = [];
for (const item of plans) {
  const metadata = [];
  const observedRead = async (url) => {
    const value = await registry.read(url);
    metadata.push({ url, metadata: value });
    return value;
  };
  await registry.packageMetadata(item.name, item.version, item, undefined, observedRead);
  const payload = await registry.readBytes(item.resolved);
  if (payload.length > 64 * 1024 * 1024) throw new Error("pinned stock archive exceeds its finite 64 MiB bound");
  const comparison = archive.archiveComparison(item.directory, payload, item.integrity);
  proofs.push({ input: item, metadataResponses: metadata, archiveByteLength: payload.length,
    archiveSHA256: digest(payload), comparison,
    scope: "third-party stock public archive equality; no Vize provenance or semantic oracle claim" });
}
const runtime = fs.realpathSync(process.execPath);
process.stdout.write(JSON.stringify({ schema: "vize.oxlint.stock-public-archives-v1", modules,
  node: { path: runtime, version: process.version, sha256: digest(fs.readFileSync(runtime)) },
  packages: proofs }) + "\n");
`;

export interface StockArchiveInput {
  name: string;
  version: string;
  resolved: string;
  integrity: string;
  directory: string;
  files: Array<{ path: string; bytes: number; sha256: string }>;
  [field: string]: unknown;
}

export function verifyStockArchives(packages: StockArchiveInput[], receiptPath: string) {
  installedHtmlPreflight();
  const input = JSON.stringify(packages);
  const args = ["--input-type=module", "-e", stockArchiveProgram, sourceRoot];
  const result = spawnSync(process.execPath, args, {
    cwd: sourceRoot,
    input,
    timeout: 60_000,
    maxBuffer: 64 * 1024 * 1024,
  });
  const packet = {
    scope: "stock public archive custody only; no installed Vize execution or semantic credit",
    command: process.execPath,
    args,
    input,
    adapterSha256: sha256(stockArchiveProgram),
    pid: result.pid,
    status: result.status,
    signal: result.signal,
    error: result.error ? wholeProcessError(result.error) : null,
    stdoutBytes: Array.from(result.stdout ?? []),
    stderrBytes: Array.from(result.stderr ?? []),
  };
  fs.writeFileSync(receiptPath, JSON.stringify(packet, null, 2) + "\n", { flag: "wx" });
  assert.equal(result.error, undefined, JSON.stringify(packet));
  assert.equal(result.signal, null, JSON.stringify(packet));
  assert.equal(result.status, 0, result.stdout.toString("utf8") + result.stderr.toString("utf8"));
  assert.equal(result.stderr.length, 0);
  const proof = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(result.stdout));
  assert.equal(proof.schema, "vize.oxlint.stock-public-archives-v1");
  assert.equal(exactPath(proof.node.path), proof.node.path);
  assert.equal(sha256(fs.readFileSync(proof.node.path)), proof.node.sha256);
  assert.equal(typeof proof.node.version, "string");
  assert.equal(proof.packages.length, packages.length);
  assert.deepEqual(
    proof.modules,
    stockModules.map((name) => {
      const filename = path.join(sourceRoot, name);
      const bytes = fs.readFileSync(filename);
      return { path: filename, sha256: sha256(bytes), bytes: bytes.length };
    }),
  );
  for (let index = 0; index < packages.length; index++) {
    const actual = proof.packages[index];
    assert.deepEqual(actual.input, packages[index]);
    assert.equal(actual.comparison.allInstalledFileBytesEqualPublicArchive, true);
    assert.equal(actual.comparison.fileCount, packages[index].files.length);
    assert.deepEqual(actual.comparison.files, packages[index].files);
    assert.equal(
      actual.comparison.tarballSHA512,
      Buffer.from(packages[index].integrity.slice(7), "base64").toString("hex"),
    );
  }
  return { receiptPath, receiptSha256: sha256(fs.readFileSync(receiptPath)), packet, proof };
}
