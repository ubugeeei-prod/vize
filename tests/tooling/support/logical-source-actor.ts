import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { gunzipSync } from "node:zlib";

type Authority = {
  afterRevision: string;
  files: Array<{ path: string; afterSha256: string; afterBytes: number }>;
};
const object = (value: unknown, keys: string[]): Record<string, unknown> => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  assert.deepEqual(Object.keys(value).sort(), keys.sort());
  return value as Record<string, unknown>;
};
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

/** Decode only the eight production images owned by the unchanged authority. */
export function decodeLogicalSourceActor(value: unknown, authority: Authority) {
  const actor = object(value, ["schema", "revision", "files"]);
  assert.equal(actor.schema, "vize.sealed-logical-source-actor.v1");
  assert.equal(actor.revision, authority.afterRevision);
  assert.equal(authority.files.length, 8);
  assert.ok(Array.isArray(actor.files));
  assert.deepEqual(
    actor.files.map((entry) => object(entry, ["path", "base64"]).path),
    authority.files.map((entry) => entry.path),
  );
  const result = new Map<string, Buffer>();
  for (const [index, entry] of actor.files.entries()) {
    const file = object(entry, ["path", "base64"]);
    const expected = authority.files[index]!;
    assert.equal(typeof file.base64, "string");
    const bytes = Buffer.from(String(file.base64), "base64");
    assert.equal(bytes.toString("base64"), file.base64, "canonical source encoding required");
    assert.equal(bytes.length, expected.afterBytes, expected.path);
    assert.equal(hash(bytes), expected.afterSha256, expected.path);
    assert.ok(!result.has(expected.path), "one image per production identity required");
    result.set(expected.path, bytes);
  }
  return result;
}

export function readLogicalSourceActor(root: string, authority: Authority) {
  const bytes = fs.readFileSync(
    path.join(root, "tests/_fixtures/tooling/logical-source-actor-11647/after.json.gz"),
  );
  assert.equal(hash(bytes), "d3b4b96a46ca33364d0e32cea69ddeb96394c3f0492917374d324349b16e2eb5");
  return decodeLogicalSourceActor(JSON.parse(gunzipSync(bytes).toString("utf8")), authority);
}
