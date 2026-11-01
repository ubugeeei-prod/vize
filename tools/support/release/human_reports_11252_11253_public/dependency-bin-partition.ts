import assert from "node:assert/strict";
import { posix as path } from "node:path";
import { deriveHumanBins, publicFiles, regular } from "./dependency-bin-inputs.ts";
import type { HumanBinFacts } from "./dependency-bin-types.ts";
import { absoluteUnix, renderPnpm12161Unix, within } from "./pnpm-shim-render.ts";
import { sha256 } from "./value.ts";

/** HUMAN-only closed union. This is not the shared archive comparator and
 * never reports extra manager bytes as published archive/SDK/native payload. */
export function humanDependencyBinPartition(facts: HumanBinFacts) {
  const bins = deriveHumanBins(facts),
    payload = publicFiles(facts.parent);
  const physical = new Map<string, HumanBinFacts["inventory"][number]>();
  for (const item of facts.inventory) {
    absoluteUnix(item.path);
    assert(within(facts.parent.directory, item.path));
    assert.equal(item.realPath, item.path, "redirected inventory entry");
    assert(!physical.has(item.path), "duplicate actual inventory entry");
    physical.set(item.path, item);
  }
  const expectedDirectories = new Set<string>();
  const expectedFiles = new Map<
    string,
    { body: Buffer; owner: "PUBLIC_PAYLOAD" | "SOURCE_CONFORMANT_BIN" }
  >();
  for (const [relative, body] of payload)
    expectedFiles.set(path.join(facts.parent.directory, relative), {
      body,
      owner: "PUBLIC_PAYLOAD",
    });
  for (const bin of bins) {
    assert(!expectedFiles.has(bin.shimPath));
    expectedFiles.set(bin.shimPath, {
      body: renderPnpm12161Unix(bin),
      owner: "SOURCE_CONFORMANT_BIN",
    });
  }
  for (const file of expectedFiles.keys()) {
    let directory = path.dirname(file);
    while (directory !== facts.parent.directory) {
      assert(within(facts.parent.directory, directory));
      expectedDirectories.add(directory);
      directory = path.dirname(directory);
    }
  }
  assert.equal(
    physical.size,
    expectedFiles.size + expectedDirectories.size,
    "full actual inventory differs from closed published+manager union",
  );
  for (const directory of expectedDirectories) {
    const item = physical.get(directory);
    assert(item && item.kind === "directory" && item.body === null);
  }
  const files = [...expectedFiles].map(([filename, expected]) => {
    const item = physical.get(filename);
    assert(item, "missing whole expected file");
    const raw = regular(item, filename);
    assert(raw.equals(expected.body), "whole payload/shim body differs");
    return {
      path: filename,
      bytes: raw.length,
      sha256: sha256(raw),
      bucket: expected.owner,
      rawBase64: expected.owner === "SOURCE_CONFORMANT_BIN" ? raw.toString("base64") : null,
      sdkCredit: false,
      nativeCredit: false,
      publicSriCredit: expected.owner === "PUBLIC_PAYLOAD",
    };
  });
  if (facts.parent.name === "vue") assert.equal(payload.size, 37);
  assert.equal(files.filter((file) => file.bucket === "PUBLIC_PAYLOAD").length, payload.size);
  return {
    schema: "vize.humanPnpm12161ClosedPartition.v1",
    role: facts.role,
    publicPayloadCount: payload.size,
    sourceConformantBinCount: bins.length,
    actualInventoryEntries: physical.size,
    everyActualEntryAccountedFor: true,
    publicPayloadBytesEqualArchive: true,
    allInstalledBytesEqualArchive: false,
    sourceInputPolicyGitBlob: facts.expectedInputPolicyGitBlob,
    files,
    accepted: false,
    productQualified: false,
    afterQualified: false,
    issueQualified: false,
    generatorSourceConformanceOnly: true,
    actualWriterRuntimeAttribution: false,
  };
}
