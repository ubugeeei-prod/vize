/** Restore the pre-#13693 generator before the older source custody layers. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { lstatSync, readFileSync } from "node:fs";

type Identity = { bytes: number; sha256: string };
type FileTransition = {
  path: string;
  before: Identity;
  after: Identity;
  hunks: { before: string; after: string }[];
};
type SourceRecord = {
  schema: string;
  pullRequest: number;
  credit: { compiled: boolean; native: boolean; performance: boolean };
  files: FileTransition[];
  supportingSources: (Identity & { path: string; role: string })[];
};

const bytes = readFileSync(
  new URL(
    "../../../../tests/_fixtures/tooling/default-export-projection-13693.json",
    import.meta.url,
  ),
);
assert.equal(
  createHash("sha256").update(bytes).digest("hex"),
  "7a5b4a543771cadb66b9e41b6797fb365d97eaccd40d74ad3f3e2b094528cf6b",
);
const record = JSON.parse(bytes.toString("utf8")) as SourceRecord;
assert.equal(record.schema, "vize.defaultExportProjectionSource.v1");
assert.equal(record.pullRequest, 13693);
assert.deepEqual(record.credit, { compiled: false, native: false, performance: false });

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

/** Foreign paths, older images, and changed successor images pass through untouched. */
export function beforeDefaultExportProjection(
  path: string,
  current: string,
  sourceRoot = new URL("../../../../", import.meta.url),
): string {
  const file = record.files.find((candidate) => candidate.path === path);
  if (!file || identity(current).sha256 !== file.after.sha256) return current;
  assert.deepEqual(identity(current), file.after);
  for (const source of record.supportingSources) {
    const path = new URL(source.path, sourceRoot);
    assert.equal(lstatSync(path).isFile(), true);
    const bytes = readFileSync(path);
    assert.equal(bytes.length, source.bytes);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), source.sha256);
  }
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length - 1, 1);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before);
  return current;
}
