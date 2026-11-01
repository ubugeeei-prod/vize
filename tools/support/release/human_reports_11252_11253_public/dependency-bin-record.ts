import assert from "node:assert/strict";
import { posix as path } from "node:path";
import type { Image, InstallReceipt, PackageReceipt, Request } from "./model.ts";
import { deriveRecordedHumanBins } from "./dependency-bin-inputs.ts";
import { binInventory, contextFacts } from "./dependency-bin-context.ts";
import { assertRecordedBinPolicy } from "./dependency-bin-policy.ts";
import { renderPnpm12161Unix } from "./pnpm-shim-render.ts";
import { bytes, gitBlob, sha256 } from "./value.ts";

type BinPackage = Extract<PackageReceipt, { allInstalledBytesEqualArchive: false }>;
/** Validate archived records without opening their historical paths. The
 * actual authenticated controller performed the tarball comparison; omitted
 * public payloads are not claimed independently rehashed by this reader. */
export function validateBinRecord(
  pkg: BinPackage,
  install: InstallReceipt,
  reviewed?: Request["reviewedController"],
): void {
  assert.equal(pkg.name, "vue");
  assert.equal(pkg.version, "3.5.44");
  const context = pkg.binContext,
    proof = pkg.sourceConformance;
  assert.equal(context.root, install.root);
  assert.deepEqual(context.host, { platform: install.platform, arch: install.arch });
  assert.deepEqual(context.policy, install.binPolicy);
  if (reviewed) assertRecordedBinPolicy(context.policy, reviewed);
  const rawPolicy = bytes(context.policy.rawBase64);
  assert.equal(gitBlob(rawPolicy), context.policy.gitBlob);
  assert.equal(sha256(rawPolicy), context.policy.sha256);
  assert.equal(rawPolicy.length, context.policy.bytes);
  assert.equal(context.lockRawBase64, install.lockRawBase64);
  assert.deepEqual(context.manager.originalImage, install.pnpm);
  assert.deepEqual(context.manager.currentImage, install.pnpm);
  assert.equal(context.manager.reportedVersion, install.pnpmVersion);
  const facts = contextFacts(context, null, []);
  const bins = deriveRecordedHumanBins(facts);
  assert.equal(context.parent.directory, pkg.directory);
  assert.equal(context.parent.name, pkg.name);
  assert.equal(context.parent.version, pkg.version);
  assert.equal(context.parent.resolved, pkg.resolved);
  assert.equal(context.parent.integrity, pkg.integrity);
  for (const owner of [context.parent, ...context.dependencyOwners]) {
    const records = install.packages.filter((row) => row.directory === owner.directory);
    assert.equal(records.length, 1);
    const actual = records[0]!;
    assert.equal(actual.name, owner.name);
    assert.equal(actual.version, owner.version);
    assert.equal(actual.resolved, owner.resolved);
    assert.equal(actual.integrity, owner.integrity);
    const manifest = bytes(owner.manifestRawBase64);
    const images = actual.files.filter((file) => file.path === owner.directory + "/package.json");
    assert.equal(images.length, 1);
    assert.equal(images[0]!.bytes, manifest.length);
    assert.equal(images[0]!.sha256, sha256(manifest));
  }
  const owner = context.dependencyOwners.find((row) => row.name === "typescript");
  assert(owner);
  const ownerRecord = install.packages.find((row) => row.directory === owner.directory);
  assert(ownerRecord);
  for (const target of context.targets) {
    const images: Image[] = ownerRecord.files.filter((row) => row.path === target.path);
    assert.equal(images.length, 1);
    assert.equal(images[0]!.bytes, target.bytes);
    assert.equal(images[0]!.sha256, target.sha256);
  }
  assert.equal(proof.schema, "vize.humanPnpm12161ClosedPartition.v1");
  assert.equal(proof.role, context.role);
  assert.equal(proof.sourceInputPolicyGitBlob, context.policy.gitBlob);
  for (const key of [
    "accepted",
    "productQualified",
    "afterQualified",
    "issueQualified",
    "allInstalledBytesEqualArchive",
    "actualWriterRuntimeAttribution",
  ] as const)
    assert.equal(proof[key], false);
  assert.equal(proof.generatorSourceConformanceOnly, true);
  assert.equal(proof.publicPayloadBytesEqualArchive, true);
  assert.equal(proof.everyActualEntryAccountedFor, true);
  assert.equal(proof.publicPayloadCount, 37);
  assert.equal(proof.sourceConformantBinCount, bins.length);
  const published = proof.files.filter((row) => row.bucket === "PUBLIC_PAYLOAD");
  const generated = proof.files.filter((row) => row.bucket === "SOURCE_CONFORMANT_BIN");
  assert.equal(published.length, 37);
  assert.equal(generated.length, bins.length);
  assert.equal(published.length + generated.length, proof.files.length);
  assert.equal(new Set(proof.files.map((row) => row.path)).size, proof.files.length);
  const directorySet = new Set<string>();
  for (const row of proof.files) {
    assert(row.path.startsWith(pkg.directory + "/") && path.normalize(row.path) === row.path);
    assert.equal(row.nativeCredit, false);
    assert.equal(row.sdkCredit, false);
    assert.equal(row.publicSriCredit, row.bucket === "PUBLIC_PAYLOAD");
    if (row.bucket === "PUBLIC_PAYLOAD") assert.equal(row.rawBase64, null);
    let directory = path.dirname(row.path);
    while (directory !== pkg.directory) {
      assert(directory.startsWith(pkg.directory + "/"));
      directorySet.add(directory);
      directory = path.dirname(directory);
    }
  }
  for (const bin of bins) {
    const rows = generated.filter((row) => row.path === bin.shimPath);
    assert.equal(rows.length, 1);
    const expected = renderPnpm12161Unix(bin),
      row = rows[0]!;
    assert.equal(row.bytes, expected.length);
    assert.equal(row.sha256, sha256(expected));
    assert(bytes(row.rawBase64!).equals(expected), "whole source-conformant bin bytes required");
  }
  assert.equal(proof.actualInventoryEntries, proof.files.length + directorySet.size);
  assert.deepEqual(
    pkg.files,
    proof.files.map(({ path, bytes, sha256 }) => ({ path, bytes, sha256 })),
  );
}

/** Recheck the COMPLETE live union, including structural directories. Unknown
 * empty directories, links, extra files and any byte change all refuse. */
export function recheckBinPackage(pkg: BinPackage, install: InstallReceipt): void {
  validateBinRecord(pkg, install);
  const expected = new Map(pkg.files.map((row) => [row.path, row]));
  const directories = new Set<string>();
  for (const file of expected.keys()) {
    let directory = path.dirname(file);
    while (directory !== pkg.directory) {
      directories.add(directory);
      directory = path.dirname(directory);
    }
  }
  const actual = binInventory(pkg.directory);
  assert.equal(actual.length, expected.size + directories.size);
  for (const row of actual) {
    assert.equal(row.path, row.realPath);
    if (directories.has(row.path)) {
      assert.equal(row.kind, "directory");
      assert.equal(row.body, null);
    } else {
      const image = expected.get(row.path);
      assert(image);
      assert.equal(row.kind, "file");
      assert(row.body);
      assert.equal(row.body.length, image.bytes);
      assert.equal(sha256(row.body), image.sha256);
    }
  }
}
