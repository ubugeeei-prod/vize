import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import { beforeDiscriminatedJsxUnionIn } from "../../tools/support/compat/content-mapper/discriminated-jsx-union-transition.ts";
import {
  beforeCanonLeftovers,
  beforeStrictExtraKeysIn,
} from "../../tools/support/compat/content-mapper/strict-extra-keys-transition.ts";
import { beforeSharedListenerRelationSource } from "./support/listener-relation-source-transition.ts";

const root = new URL("../../", import.meta.url);
const read = (path: string) => fs.readFileSync(new URL(path, root), "utf8");
const base = "crates/vize_canon/src/virtual_ts/helpers/";
const corpus =
  "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/";
const rust = read(
  "crates/vize_canon/tests/empty_vue_star_exports/membership/constructor_without_args/payload_update/current_loose_emit.rs",
);
const recordPath = corpus + "current-loose-emitter-update-11863.json";
const genericRecordPath = corpus + "generic-event-specialization-update-11815.json";
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
function loadGeneric(raw = read(genericRecordPath)): Record {
  assert.equal(hash(raw), "d9eee3c8eb982eba5e2144cca457ecf0b15e1edbb5948b019dd404e6c10ab845");
  const record = JSON.parse(raw) as Record;
  assert.equal(record.nativeAcceptanceCredit, false);
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
  const generic = loadGeneric();
  for (const entry of record.files) {
    const current = read(entry.path);
    const successor = generic.files.find((candidate) => candidate.path === entry.path);
    assert.ok(successor);
    const owned = inverse(current, successor);
    const previous = inverse(owned, entry);
    assert.equal(previous.replace(entry.edits[0]!.before, entry.edits[0]!.after), owned);
    assert.equal(owned.replace(successor.edits[0]!.before, successor.edits[0]!.after), current);
    mutations(owned, entry);
    mutations(current, successor);
  }
  assert.throws(() => load(read(recordPath) + "\n"));
});
void test("the complete source preamble reconstructs the unchanged runtime and historical laws", () => {
  const record = load(),
    generic = loadGeneric(),
    decl = base + "declarations/";
  assert.deepEqual(
    identity(
      beforeSharedListenerRelationSource(
        record.preambleProducer.path,
        read(record.preambleProducer.path),
      ),
    ),
    {
      bytes: record.preambleProducer.bytes,
      sha256: record.preambleProducer.sha256,
    },
  );
  // This finite expansion is admitted only for the exact macro producer above.
  // Its compiled constant is checked independently by the native tests.
  const predecessorRead = (path: string) => beforeCanonLeftovers(path, read(path));
  const current = [
    predecessorRead(decl + "shared-header.d.ts.txt"),
    predecessorRead(decl + "core-types.d.ts.txt"),
    predecessorRead(decl + "vue3-types.d.ts.txt"),
    predecessorRead(decl + "prop-keys.d.ts.txt"),
    "\n",
    predecessorRead("crates/vize_canon/src/virtual_ts/component_prop_helpers.txt"),
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
    ].map((name) => predecessorRead(decl + name + ".d.ts.txt")),
    predecessorRead(base + "pattern_matching.d.ts"),
  ].join("");
  assert.deepEqual(identity(current), {
    bytes: 46426,
    sha256: "8340b8b945e5e3ca8fcb61b7e8f0ca467ae1a603292c07f1e78f3e1226ca6522",
  });
  const relaxedCurrent = beforeStrictExtraKeysIn(current);
  assert.deepEqual(identity(relaxedCurrent), {
    bytes: 46557,
    sha256: "635acca0b25877f2132902fe777d894287100a7c993066066a67812deab2f255",
  });
  const integratedCurrent = beforeDiscriminatedJsxUnionIn(relaxedCurrent);
  assert.deepEqual(identity(integratedCurrent), {
    bytes: 46534,
    sha256: "7be160e1798f4368642597b67db232b1a72309dc18eb637a4ec7d868f3457bcb",
  });
  const integratedTransition = read(
    "tests/_fixtures/differential/typechecker/tsx-single-required-exact-keys/integrated-source-transition-11964.json",
  );
  assert.equal(
    hash(integratedTransition),
    "9f4babc2f1d99fc129ceab018266503a78b60d8174374122fcce679694159518",
  );
  const repair = (
    JSON.parse(integratedTransition) as {
      file: { edits: Array<{ before: string; after: string }> };
    }
  ).file.edits.at(-1)!;
  assert.equal(integratedCurrent.split(repair.after).length, 2);
  const repairedCurrent = integratedCurrent.replace(repair.after, repair.before);
  assert.deepEqual(identity(repairedCurrent), {
    bytes: 46506,
    sha256: "f3f628ff406ce57acce118f1213e000930f8a70d20afdb07fbfedc6e274b32e9",
  });
  const integratedSourcePath =
    "crates/vize_canon/tests/empty_vue_star_exports/membership/constructor_without_args/payload_update/integrated_source.rs";
  const integratedSource = beforeSharedListenerRelationSource(
    integratedSourcePath,
    read(integratedSourcePath),
  );
  assert.equal(
    hash(integratedSource),
    "40ec1d06ab3d9e34442d45d2f6ab1270734537e33e19e6be04308e25277cfc24",
  );
  assert.ok(integratedSource.includes(identity(repairedCurrent).sha256));
  const genericEdit = generic.preamble.edits[0]!;
  assert.equal(repairedCurrent.split(genericEdit.after).length, 2);
  const owned = repairedCurrent.replace(genericEdit.after, genericEdit.before);
  assert.deepEqual(identity(owned), {
    bytes: 46322,
    sha256: "aa205818790c5f3c384a3dc57d63b851f970700076c60340499f9f2bdd40f9da",
  });
  const edit = record.preamble.edits[0]!;
  assert.equal(owned.split(edit.after).length, 2);
  const previous = owned.replace(edit.after, edit.before);
  assert.deepEqual(identity(previous), {
    bytes: 45686,
    sha256: "1291d74aaa2ff4e079ab8eee8feb22bd3b2ce23ed75ed57a25dcf1b0c9e4d36d",
  });
  const runtimeAfter =
    "type __RuntimePropValue<T> = T extends (...args: any[]) => infer V ? V : T extends abstract new (...args: any[]) => infer V ? V : never;";
  const runtimeBefore =
    "type __RuntimePropValue<T> = T extends abstract new (...args: any[]) => infer V ? V : T extends (...args: any[]) => infer V ? V : never;";
  assert.equal(previous.split(runtimeAfter).length, 2);
  assert.equal(
    hash(previous.replace(runtimeAfter, runtimeBefore)),
    "31f44873c84434d22f94670e6cb558d5d0aa1408723bf6f6214308e0ad4239cd",
  );
  assert.equal(previous.replace(edit.before, edit.after), owned);
  assert.equal(owned.replace(genericEdit.before, genericEdit.after), repairedCurrent);
});
