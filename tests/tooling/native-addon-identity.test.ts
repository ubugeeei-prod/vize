import assert from "node:assert/strict";
import test from "node:test";
import { matchIdentity } from "../../tools/commands/levels/native-addon-identity.ts";

void test("only the producer's recorded identity at the same unique offset can change", () => {
  const base = Buffer.from("code/base/end");
  const change = Buffer.from("code/head/end");
  const old = Buffer.from("base");
  const next = Buffer.from("head");
  assert.deepEqual(matchIdentity(base, change, old, next), { offset: 5, bytes: 4 });
  assert.ok(base.equals(change));
  const other = Buffer.from("code/head/END");
  matchIdentity(base, other, old, next);
  assert.equal(base.equals(other), false);
  for (const value of ["head/code/end", "code/else/end", "code/head/head"])
    assert.throws(() => matchIdentity(base, Buffer.from(value), old, next));
  assert.throws(() => matchIdentity(Buffer.from("base/base"), Buffer.from("head/head"), old, next));
});
