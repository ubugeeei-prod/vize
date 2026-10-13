import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { pathToFileURL } from "node:url";
import { ownedCorsaSource } from "./owned-corsa-source-custody.mjs";
import { authenticateProcessReceiptVendorSource } from "./owned-corsa-vendor-source.mjs";

export function authenticateOwnedCorsaPackageSource(root, sourceSha) {
  assert.match(sourceSha, /^[a-f0-9]{40}$/u);
  const value = authenticateProcessReceiptVendorSource(root);
  assert.equal(value.receipt.head, sourceSha);
  const actualBodies = ownedCorsaSource.rows.map(({ file }) => {
    const actual = value.actualReader("after", file);
    return {
      file,
      mode: actual.mode,
      blob: actual.blob,
      bytes: actual.body.length,
      sha256: createHash("sha256").update(actual.body).digest("hex"),
    };
  });
  return { ...value.receipt, sourceContext: value.sourceContext, actualBodies };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  assert.equal(process.argv.length, 4);
  process.stdout.write(
    JSON.stringify(authenticateOwnedCorsaPackageSource(process.argv[2], process.argv[3])) + "\n",
  );
}
