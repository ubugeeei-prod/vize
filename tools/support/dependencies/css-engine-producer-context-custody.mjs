import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const sha = (body) => createHash("sha256").update(body).digest("hex");
const blob = (body) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
function regular(file) {
  const stat = fs.lstatSync(file);
  assert(stat.isFile() && (stat.mode & 0o777) === 0o644, file);
  return fs.readFileSync(file);
}
const bytes = regular(new URL("./css-engine-producer-context-source.json", import.meta.url));
assert.equal(sha(bytes), "ba9cfb3a569930a949d98eaa069bcf0fe0b2b12d4a4c89732b6471045c150775");
const data = JSON.parse(bytes);
assert.equal(data.version, 1);
assert.deepEqual(
  data.context.map(({ file }) => file),
  [
    "tools/support/dependencies/css-engine-producer-manifests.ts",
    "tools/support/dependencies/owned-corsa-producer-metadata.ts",
    "tools/support/dependencies/owned-corsa-producer-metadata.json",
    "tests/tooling/owned-corsa-css-producer-context.test.mjs",
  ],
);
assert.deepEqual(
  data.kernel.map(({ file }) => file),
  [
    "tools/commands/release/pr.rs",
    "tools/support/release/pr_pin.rs",
    "tools/support/release/pr_pin_metadata.rs",
    "tools/support/release/pr_pin_producer_metadata.rs",
  ],
);
const original = Buffer.from(data.original.rawBase64, "base64");
assert.equal(original.toString("base64"), data.original.rawBase64);
assert.equal(original.length, data.original.bytes);
assert.equal(sha(original), data.original.sha256);
assert.equal(blob(original), data.original.blob);
assert.equal(data.original.mode, "100644");
assert.equal(
  data.original.sha256,
  "31a86011e3d8cda9b3817b5cceb210f2b8d2099a8af033202755e09c73db7f1b",
);

/** Only the complete known context can expose the whole original helper checksum. */
export function authenticateCssProducerContext(root) {
  const actual = regular(path.join(root, data.original.file));
  if (sha(actual) === data.original.sha256) {
    assert.deepEqual(actual, original);
    return {
      originalSha256: data.original.sha256,
      actualSha256: sha(actual),
      contextBodies: 0,
      runtimeCredit: false,
    };
  }
  for (const row of [...data.context, ...data.kernel]) {
    assert.equal(row.mode, "100644");
    const body = regular(path.join(root, row.file));
    assert.equal(body.length, row.bytes, row.file);
    assert.equal(sha(body), row.sha256, row.file);
    assert.equal(blob(body), row.blob, row.file);
  }
  assert.deepEqual(
    data.contextSuccessors.map(({ file }) => file),
    [
      "tools/support/dependencies/owned-corsa-producer-metadata.ts",
      "tests/tooling/owned-corsa-css-producer-context.test.mjs",
    ],
  );
  const beforeChecksums = {
    "tools/support/dependencies/owned-corsa-producer-metadata.ts":
      "323e7ba4faad68bc81629b9ec75618ea8018dec9662df755ddfe527cc9b33df4",
    "tests/tooling/owned-corsa-css-producer-context.test.mjs":
      "36d58c12c817cef7811a86bf73b3e80167b61f053e8ab25befb4702803bc8b74",
  };
  for (const successor of data.contextSuccessors) {
    const before = successor.retainedBefore;
    const bytes = Buffer.from(before.rawBase64, "base64");
    assert.equal(bytes.toString("base64"), before.rawBase64);
    assert.equal(before.mode, "100644");
    assert.equal(bytes.length, before.bytes);
    assert.equal(sha(bytes), before.sha256);
    assert.equal(blob(bytes), before.blob);
    assert.equal(before.sha256, beforeChecksums[successor.file]);
    let forward = bytes.toString();
    for (const [old, next] of successor.forwardClauses) {
      assert.equal(forward.split(old).length, 2);
      forward = forward.replace(old, next);
    }
    assert.deepEqual(
      Buffer.from(forward),
      regular(path.join(root, successor.file)),
      "whole CSS provider context inverse",
    );
  }
  let recovered = actual.toString("utf8");
  for (const [after, before] of data.inverseClauses) {
    assert.equal(recovered.split(after).length, 2, "one named complete helper inverse clause");
    recovered = recovered.replace(after, before);
  }
  assert.deepEqual(Buffer.from(recovered), original, "whole original CSS helper inverse");
  return {
    originalSha256: data.original.sha256,
    actualSha256: sha(actual),
    contextBodies: 4,
    kernelBodies: 4,
    runtimeCredit: false,
  };
}
