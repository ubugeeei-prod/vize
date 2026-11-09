import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  certifiedHelperImages,
  requiredFallthroughGate,
  withLandedEdits,
} from "../../tools/support/compat/content-mapper/callback-owner-landed-contracts.ts";
import { beforeInstanceInputKeys } from "../../tools/support/compat/content-mapper/instance-input-keys-transition.ts";

type Edit = { start: number; end: number; replacement: Buffer };
const VT = 'import("vize/virtual-ts").';
const line = (prefix: string, n: number, tail: string) =>
  `  type __Child_CheckProps_${n} = __Child_Props_${n} & ${prefix}$EmitListeners<__Child_Component_${n}>${tail}`;
const ungated = (prefix: string, n: number) =>
  line(prefix, n, ` & ${prefix}$RequiredFallthroughProps<__Child_Component_${n}>;\n`);
const gated = (prefix: string, n: number) =>
  line(prefix, n, requiredFallthroughGate(prefix, `__Child_Component_${n}`));

function apply(text: string, edits: Edit[]): string {
  const source = Buffer.from(text);
  const chunks: Buffer[] = [];
  let position = 0;
  for (const edit of [...edits].sort((a, b) => a.start - b.start)) {
    chunks.push(source.subarray(position, edit.start), edit.replacement);
    position = edit.end;
  }
  chunks.push(source.subarray(position));
  return Buffer.concat(chunks).toString();
}

test("the #12457 gate is the exact emitted contract tail", () => {
  assert.equal(
    requiredFallthroughGate(VT, "__Child_Component_0"),
    ` & (${VT}$FallthroughProps<__Child_Component_0> extends infer __F ? [__F extends unknown ? {} extends __F ? never : __F : never] extends [never] ? ${VT}$RequiredFallthroughProps<__Child_Component_0, __F, never> : ${VT}$RequiredFallthroughProps<__Child_Component_0, __F> : never);\n`,
  );
});

test("each ungated contract line migrates only to its own gated line", () => {
  const head = "// ünïcode prefix\n";
  const before = `${head}${ungated(VT, 0)}const a = 1;\n${ungated("", 1)}`;
  const after = `${head}${gated(VT, 0)}const a = 1;\n${gated("", 1)}`;
  const edits = withLandedEdits(before, after, []);
  assert.equal(edits.length, 2);
  assert.equal(apply(before, edits), after);
  // Identical packets and unrelated changes certify nothing.
  assert.deepEqual(withLandedEdits(before, before, []), []);
  assert.deepEqual(withLandedEdits(after, after, []), []);
  const swapped = `${head}${gated(VT, 1)}const a = 1;\n${gated("", 0)}`;
  assert.notEqual(apply(before, withLandedEdits(before, swapped, [])), swapped);
  const relaxed = after.replaceAll("{} extends __F ? never : __F", "__F");
  assert.deepEqual(withLandedEdits(before, relaxed, []), []);
});

test("a gated line never overlaps another certified edit", () => {
  const before = ungated(VT, 0);
  const after = gated(VT, 0);
  const start = Buffer.byteLength(before.slice(0, before.lastIndexOf(" & ")));
  const overlapping = [{ start, end: start + 3, replacement: Buffer.from("x") }];
  assert.throws(() => withLandedEdits(before, after, overlapping), /disjoint/u);
});

test("the #12446 helper predecessor is certified once, before the current image", () => {
  const path = "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt";
  const current = readFileSync(new URL(`../../${path}`, import.meta.url), "utf8");
  const predecessor = beforeInstanceInputKeys(path, current);
  assert.notEqual(predecessor, current);
  assert.deepEqual(certifiedHelperImages(path, current, ["older"]), [
    "older",
    predecessor,
    current,
  ]);
  assert.deepEqual(certifiedHelperImages(path, current, [predecessor]), [predecessor, current]);
});
