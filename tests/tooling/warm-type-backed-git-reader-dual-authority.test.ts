import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import {
  admittedReview,
  qualifyDualCurrentFrame,
} from "../performance/support/warm-type-backed-git-reader-dual-frame.ts";

const root = path.resolve(import.meta.dirname, "../..");

test("a genuine current112 receipt cannot mutate its complete admitted JSON authority", () => {
  const packet = qualifyDualCurrentFrame(root);
  const review = admittedReview(root, packet);
  const whole = JSON.stringify(review);
  assert.equal(packet.completeCombinedPairs, 112);
  assert.equal(Reflect.set(review, "beforeSource", "f".repeat(40)), false);
  assert.equal(Reflect.set(review, "driver", "foreign.ts"), false);
  for (const role of ["sdk", "component"] as const) {
    const original = review.original[role];
    assert.equal(Reflect.set(original, "source", "f".repeat(40)), false);
    assert.equal(Reflect.set(original, "module", "foreign.ts"), false);
    const firstFile = original.files.find(() => true);
    const firstRow = original.rows.find(() => true);
    const firstRetained = original.retained.find(() => true);
    assert(firstFile && firstRow && firstRetained);
    for (const cell of [firstFile, firstRow.currentAfter, firstRetained]) {
      assert.equal(Reflect.set(cell, "sha256", "f".repeat(64)), false);
      assert.equal(Reflect.defineProperty(cell, "mode", { value: "100755" }), false);
    }
  }
  const first = review.rows.find(() => true);
  const before = review.beforeRows.find((row) => row.cell)?.cell;
  assert(first && before);
  assert.equal(Reflect.set(first, "file", "foreign.rs"), false);
  for (const cell of [first.final, first.sdk, first.component, before, review.beforeCommit]) {
    assert(cell);
    assert.equal(Reflect.set(cell, "bytes", cell.bytes + 1), false);
    assert.equal(Reflect.set(cell, "blob", "f".repeat(40)), false);
  }
  for (const list of [
    review.rows,
    review.beforeRows,
    review.original.sdk.files,
    review.original.sdk.rows,
    review.original.sdk.retained,
    review.original.component.files,
    review.original.component.rows,
    review.original.component.retained,
    review.changes.sdk,
    review.changes.component,
  ]) {
    assert.equal(Reflect.deleteProperty(list, "0"), false);
    assert.equal(Reflect.set(list, "length", list.length - 1), false);
    assert.throws(() => Array.prototype.pop.call(list), TypeError);
  }
  assert.equal(JSON.stringify(review), whole);
  const fresh = qualifyDualCurrentFrame(root);
  assert.equal(fresh.head, packet.head);
  assert.equal(fresh.completeCombinedPairs, 112);
  assert.equal(JSON.stringify(admittedReview(root, packet)), whole);
});
