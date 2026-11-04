import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";

const root = new URL("../../", import.meta.url);
const read = (path: string) => fs.readFileSync(new URL(path, root), "utf8");
const base = "crates/vize_canon/src/virtual_ts/helpers/";
const corpus =
  "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/";
const rust = read(
  "crates/vize_canon/tests/empty_vue_star_exports/membership/constructor_without_args/payload_update/current_loose_emit.rs",
);
const recordPath = corpus + "current-loose-emitter-update-11863.json";
const recordSha = rust.match(/const RECORD_SHA: &str = "([a-f0-9]{64})";/u)?.[1];
assert.ok(recordSha);
type Identity = { bytes: number; sha256: string };
type Transition = { before: Identity; after: Identity; edits: { before: string; after: string }[] };
interface Record {
  version: number;
  kind: string;
  nativeAcceptanceCredit: boolean;
  historicalReconstructionOnly: boolean;
  files: (Transition & { path: string })[];
  preamble: Transition;
  preambleProducer: Identity & { path: string };
}
const hash = (text: string) => createHash("sha256").update(text).digest("hex");
const identity = (text: string) => ({ bytes: Buffer.byteLength(text), sha256: hash(text) });
function load(raw = read(recordPath)): Record {
  assert.equal(hash(raw), recordSha);
  const record = JSON.parse(raw) as Record;
  assert.equal(record.nativeAcceptanceCredit, false);
  assert.equal(record.historicalReconstructionOnly, true);
  assert.equal(record.files.length, 2);
  return record;
}
function inverse(current: string, entry: Transition): string {
  assert.deepEqual(identity(current), entry.after);
  for (const edit of entry.edits) {
    assert.ok(edit.after);
    assert.equal(current.split(edit.after).length, 2);
    current = current.replace(edit.after, edit.before);
  }
  assert.deepEqual(identity(current), entry.before);
  return current;
}
function mutations(current: string, entry: Transition) {
  for (const edit of entry.edits)
    for (const changed of [
      current.replace(edit.after, ""),
      current + edit.after,
      current.replace(edit.after, edit.before),
      current + "\n",
      current.replace("type ", "interface "),
    ]) {
      assert.notEqual(changed, current);
      assert.throws(() => inverse(changed, entry));
    }
}
void test("the current-only emitter layer preserves both exact historical source assets", () => {
  const record = load();
  for (const entry of record.files) {
    const current = read(entry.path);
    const previous = inverse(current, entry);
    assert.equal(previous.replace(entry.edits[0]!.before, entry.edits[0]!.after), current);
    mutations(current, entry);
  }
  assert.throws(() => load(read(recordPath) + "\n"));
});
void test("the complete source preamble reconstructs the unchanged runtime and historical laws", () => {
  const record = load(),
    decl = base + "declarations/";
  assert.deepEqual(identity(read(record.preambleProducer.path)), {
    bytes: record.preambleProducer.bytes,
    sha256: record.preambleProducer.sha256,
  });
  // This finite expansion is admitted only for the exact macro producer above.
  // Its compiled constant is checked independently by the native tests.
  const current = [
    read(decl + "shared-header.d.ts.txt"),
    read(decl + "core-types.d.ts.txt"),
    read(decl + "vue3-types.d.ts.txt"),
    read(decl + "prop-keys.d.ts.txt"),
    "\n",
    read(base + "../component_prop_helpers.txt"),
    ...[
      "native-elements",
      "component-global-attributes",
      "for-list",
      "directives",
      "shared-template-refs",
      "shared-slot-resolver",
      "shared-slot-payload",
      "emit-overloads",
      "shared-emit-props",
      "shared-emit-inference",
      "component-checks",
      "shared-macros",
    ].map((name) => read(decl + name + ".d.ts.txt")),
    read(base + "pattern_matching.d.ts"),
  ].join("");
  assert.equal(Buffer.byteLength(current), 45384);
  const previous = inverse(current, record.preamble);
  assert.equal(Buffer.byteLength(previous), 44748);
  assert.equal(hash(previous), "378516f3c59cecb166ab9197761b77679824ea7145efd84d752bf5afd3060b07");
  const runtimeAfter =
    "type __RuntimePropValue<T> = T extends (...args: any[]) => infer V ? V : T extends abstract new (...args: any[]) => infer V ? V : never;";
  const runtimeBefore =
    "type __RuntimePropValue<T> = T extends abstract new (...args: any[]) => infer V ? V : T extends (...args: any[]) => infer V ? V : never;";
  assert.equal(previous.split(runtimeAfter).length, 2);
  assert.equal(
    hash(previous.replace(runtimeAfter, runtimeBefore)),
    "08ebeeb3d5303bd130213edd33583849ef8e078d33c0e366562d40efd3a9985d",
  );
  assert.equal(
    previous.replace(record.preamble.edits[0]!.before, record.preamble.edits[0]!.after),
    current,
  );
  mutations(current, record.preamble);
});
