// Whole reviewed inheritance and the official reversible metadata transform preserve authority.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { authenticateOwnedCorsaProducerMetadata } from "./owned-corsa-producer-metadata.ts";

const checksum = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const frozenMetadata =
  "tests/_fixtures/differential/css-engine-vendor-exit/release-inheritance/metadata";

function originalMetadata(root: string) {
  const result = spawnSync(
    "rust-script",
    [path.join(root, "tools/commands/release/pr.rs"), "verify-producer-metadata", frozenMetadata],
    { cwd: root, encoding: "utf8", timeout: 180_000 },
  );
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
}

export function authenticateProducerMetadata(root: string) {
  const sources = new Map<string, { baseline: Buffer; actualSha256: string }>();
  const hashes = {
    "Cargo.toml": "38babfd635c5af9df1d401b0b59923940310a405868cbb0772d9bfefbd7e6e68",
    "Cargo.lock": "194c3098330cbeb91e441a2f035fe3cfb37650195c6ff04baf88f09a08378684",
  };
  for (const [file, sha256] of Object.entries(hashes)) {
    const baseline = fs.readFileSync(path.join(root, frozenMetadata, file));
    assert.equal(checksum(baseline), sha256, file);
    sources.set(file, {
      baseline,
      actualSha256: checksum(fs.readFileSync(path.join(root, file))),
    });
  }
  const receipt = authenticateOwnedCorsaProducerMetadata(root, sources) ?? originalMetadata(root);
  assert.equal(receipt.schema, 1);
  assert.equal(receipt.baselineVersion, "0.443.0");
  assert.equal(receipt.wholeForwardAndInverse, true);
  assert.equal(receipt.readOnly, true);
  assert.equal(receipt.publicationQualification, false);
  assert.deepEqual(
    receipt.files,
    ["Cargo.toml", "Cargo.lock"].map((file) => ({
      path: file,
      changed: checksum(sources.get(file)!.baseline) !== sources.get(file)!.actualSha256,
    })),
  );
  return {
    sources,
    receipt: {
      ...receipt,
      sources: Array.from(sources, ([file, source]) => ({
        path: file,
        baselineSha256: checksum(source.baseline),
        actualSha256: source.actualSha256,
      })),
    },
  };
}
const continuations: Record<
  string,
  { original: string; current: string; fragment: string; inverse: string }
> = {
  "Cargo.toml": {
    original: "c8fe5d4b91f49524fb4f8cd06c7453affd936b2373c29b622e94a01dc7a0793f",
    current: "38babfd635c5af9df1d401b0b59923940310a405868cbb0772d9bfefbd7e6e68",
    fragment:
      'parcel_sourcemap = { package = "vize_parcel_sourcemap", version = "=0.443.0", path = "vendor/parcel_sourcemap" }\n',
    inverse: "",
  },
  "vendor/lightningcss/Cargo.toml": {
    original: "21486cbc79ea73caf098501beef644a8d1a64f55706a9dc24b9e2fc4df4044aa",
    current: "4aa6d8ae5cc0c5525f67ccc16fee2c91e7b9cbd81b2159ff7c82704dae0da501",
    fragment: "[dependencies.parcel_sourcemap]\nworkspace = true\n",
    inverse:
      '[dependencies.parcel_sourcemap]\npackage = "vize_parcel_sourcemap"\nversion = "=0.443.0"\npath = "../parcel_sourcemap"\n',
  },
};

export function authenticateProducerSource(
  sourcePath: string,
  bytes: Buffer,
  originalSha256: string,
  metadata?: ReturnType<typeof authenticateProducerMetadata>,
) {
  const actualSha256 = checksum(bytes);
  const metadataSource = metadata?.sources.get(sourcePath);
  if (metadataSource) {
    assert.equal(actualSha256, metadataSource.actualSha256, sourcePath);
    bytes = metadataSource.baseline;
  }
  const authoritySha256 = checksum(bytes);
  const continuation = Object.hasOwn(continuations, sourcePath)
    ? continuations[sourcePath]
    : undefined;
  if (continuation) assert.equal(originalSha256, continuation.original, sourcePath);
  if (authoritySha256 === originalSha256) {
    return {
      path: sourcePath,
      actualSha256,
      originalSha256,
      reviewedContinuation: actualSha256 !== originalSha256,
    };
  }
  assert(continuation, `unreviewed producer input: ${sourcePath}`);
  assert.equal(authoritySha256, continuation.current, sourcePath);
  const text = bytes.toString("utf8");
  assert.equal(text.split(continuation.fragment).length, 2, sourcePath);
  const inverse = Buffer.from(text.replace(continuation.fragment, continuation.inverse));
  assert.equal(checksum(inverse), originalSha256, sourcePath);
  return { path: sourcePath, actualSha256, originalSha256, reviewedContinuation: true };
}
