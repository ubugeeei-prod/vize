import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import {
  assertSavedNativeProjection,
  nativeVueCheckName,
  nativeVueSources,
} from "../../tools/benchmarks/scripts/typechecker-native-phase-membership.mjs";

const file = new URL("../_fixtures/ci/native-phase-membership-original.json", import.meta.url);
const stat = fs.lstatSync(file);
assert(stat.isFile() && (stat.mode & 0o777) === 0o644);
const bytes = fs.readFileSync(file);
const sha = (body) => createHash("sha256").update(body).digest("hex");
assert.equal(sha(bytes), "c7c12caf0399493ab456974022ee3399626c0b3601a09a4afc340f54b6a1add9");
const fixture = JSON.parse(bytes);
assert.equal(fixture.schema, 1);
assert.equal(fixture.sourceSha, "6be9ffbbca772b49c76dfae80282349d5f662716");

void test("actual selected TypeScript and JavaScript native owners match complete saved byte witnesses", () => {
  for (const entry of fixture.cases) {
    const shards = [entry.selectedNativeShard];
    assert.equal(nativeVueCheckName(entry.authored, shards), entry.expectedNativeName);
    assertSavedNativeProjection(entry.saved, shards);
    assert.deepEqual([...nativeVueSources(shards)], [entry.authored]);
  }
});

void test("same exact native owner in multiple programs and shared helpers retain all byte checks", () => {
  const entry = fixture.cases[1];
  const shard = entry.selectedNativeShard;
  const shards = [shard, structuredClone(shard)];
  assert.equal(nativeVueCheckName(entry.authored, shards), entry.expectedNativeName);
  assertSavedNativeProjection(entry.saved, shards);
  assertSavedNativeProjection(fixture.shared.saved, fixture.shared.selectedShards);
});

void test("actual packet mutations refuse missing, dual, unknown and unsupported checker names", () => {
  for (const entry of fixture.cases) {
    const shard = entry.selectedNativeShard;
    assert.throws(() => nativeVueCheckName(entry.authored, [{ ...shard, members: [] }]));
    const other = entry.expectedNativeName.endsWith(".__vize_javascript__.ts")
      ? `${entry.authored}.ts`
      : `${entry.authored}.__vize_javascript__.ts`;
    const alternate = {
      ...shard.members[0],
      path: join(shard.cwd, other),
    };
    assert.throws(() =>
      nativeVueCheckName(entry.authored, [{ ...shard, members: [...shard.members, alternate] }]),
    );
    for (const suffix of [".__vize_foreign__.ts", ".tsx", ".__vize_javascript__.tsx"]) {
      const foreign = { ...shard.members[0], path: join(shard.cwd, entry.authored + suffix) };
      assert.throws(() => nativeVueCheckName(entry.authored, [{ ...shard, members: [foreign] }]));
      assert.throws(() =>
        nativeVueCheckName(entry.authored, [{ ...shard, members: [...shard.members, foreign] }]),
      );
    }
    assert.throws(() =>
      assertSavedNativeProjection({ ...entry.saved, file: "foreign.virtual.ts" }, [shard]),
    );
  }
});

void test("one changed hash or byte count in any captured native match still refuses", () => {
  for (const entry of fixture.cases) {
    for (const changes of [{ bytes: entry.saved.bytes + 1 }, { sha256: "0".repeat(64) }]) {
      const changed = {
        ...entry.selectedNativeShard,
        members: [{ ...entry.selectedNativeShard.members[0], ...changes }],
      };
      assert.throws(() =>
        assertSavedNativeProjection(entry.saved, [entry.selectedNativeShard, changed]),
      );
    }
  }
  assert.throws(() => assertSavedNativeProjection(fixture.shared.saved, []));
});

void test("complete original runner and phase provenance source invert with every other gate intact", () => {
  for (const entry of fixture.originalDrivers) {
    const original = Buffer.from(entry.rawBase64, "base64");
    assert.equal(original.toString("base64"), entry.rawBase64);
    assert.equal(original.length, entry.bytes);
    assert.equal(sha(original), entry.sha256);
    assert.equal(entry.mode, "100644");
    let current = fs.readFileSync(new URL(`../../${entry.file}`, import.meta.url), "utf8");
    for (const [before, after] of [...entry.forwardClauses].reverse()) {
      assert.equal(current.split(after).length, 2, entry.file);
      current = current.replace(after, before);
    }
    assert.deepEqual(Buffer.from(current), original, entry.file);
  }
});
