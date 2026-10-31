/** The observed RefFactory-only postimage; native inputs/raw output stay intact. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { hash } from "./canon-instance-directives-io.ts";

export type FactoryWhole = {
  beforeWholeUtf8: string;
  beforeBytes: number;
  beforeSha256: string;
  afterWholeUtf8: string;
  afterBytes: number;
  afterSha256: string;
  insertionWholeUtf8: string;
  insertionBytes: number;
  insertionSha256: string;
};
export type FactoryAuthority = {
  version: number;
  kind: string;
  nativeAcceptanceCredit: boolean;
  projectionAcceptanceCredit: boolean;
  frozenC3: { path: string; sha256: string };
  source: FactoryWhole & { path: string };
  declaration: { path: string; wholeUtf8: string; bytes: number; sha256: string };
  preamble: FactoryWhole;
};
export const refFactoryRecord =
  "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/ref-factory-update-20261031.json";
export const refFactoryRecordSha256 =
  "43717139ecb09bdfe6f9d44613807c69644e2efb41536cb3651663e6ac548643";

export function refFactoryPrevious(current: string, whole: FactoryWhole): string {
  assert.equal(current, whole.afterWholeUtf8);
  assert.equal(Buffer.byteLength(current), whole.afterBytes);
  assert.equal(hash(current), whole.afterSha256);
  const insertion = whole.insertionWholeUtf8;
  assert(insertion.length > 0);
  assert.equal(Buffer.byteLength(insertion), whole.insertionBytes);
  assert.equal(hash(insertion), whole.insertionSha256);
  assert.equal(current.split(insertion).length, 2);
  const before = current.replace(insertion, "");
  assert.equal(before, whole.beforeWholeUtf8);
  assert.equal(Buffer.byteLength(before), whole.beforeBytes);
  assert.equal(hash(before), whole.beforeSha256);
  return before;
}

export function readRefFactoryAuthority(root = path.resolve(".")): FactoryAuthority {
  const bytes = fs.readFileSync(path.join(root, refFactoryRecord));
  assert.equal(hash(bytes), refFactoryRecordSha256);
  const record = JSON.parse(bytes.toString("utf8")) as FactoryAuthority;
  assert.equal(record.version, 1);
  assert.equal(record.kind, "exact-ref-factory-source-preamble-inverse");
  assert.equal(record.nativeAcceptanceCredit, false);
  assert.equal(record.projectionAcceptanceCredit, false);
  assert.equal(record.source.path, "crates/vize_canon/src/virtual_ts/helpers/preamble.rs");
  refFactoryPrevious(fs.readFileSync(path.join(root, record.source.path), "utf8"), record.source);
  assert.equal(
    record.declaration.path,
    "crates/vize_canon/src/virtual_ts/helpers/declarations/ref-instance-factory.d.ts.txt",
  );
  const declaration = fs.readFileSync(path.join(root, record.declaration.path));
  assert.equal(declaration.toString("utf8"), record.declaration.wholeUtf8);
  assert.equal(declaration.length, record.declaration.bytes);
  assert.equal(hash(declaration), record.declaration.sha256);
  const frozenBytes = fs.readFileSync(path.join(root, record.frozenC3.path));
  assert.equal(hash(frozenBytes), record.frozenC3.sha256);
  const frozen = JSON.parse(frozenBytes.toString("utf8")) as {
    logicalPreamble: { afterWholeUtf8: string };
  };
  assert.equal(record.preamble.beforeWholeUtf8, frozen.logicalPreamble.afterWholeUtf8);
  refFactoryPrevious(record.preamble.afterWholeUtf8, record.preamble);
  return record;
}

/** Only the helper certificate comparison admits this one finite insertion. */
export function refFactoryCurrentPreamble(expectedPrevious: string): string {
  const record = readRefFactoryAuthority();
  assert.equal(
    refFactoryPrevious(record.preamble.afterWholeUtf8, record.preamble),
    expectedPrevious,
  );
  return record.preamble.afterWholeUtf8;
}
