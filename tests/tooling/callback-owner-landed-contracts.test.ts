import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  certifiedHelperImages,
  navigationBlock,
  lazyPropValue,
  requiredFallthroughGate,
  withLandedEdits,
} from "../../tools/support/compat/content-mapper/callback-owner-landed-contracts.ts";
import { beforeInstanceInputKeys } from "../../tools/support/compat/content-mapper/instance-input-keys-transition.ts";
import { beforeUnownedOwnedKeys } from "../../tools/support/compat/content-mapper/unowned-owned-keys-transition.ts";

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

test("each landed helper predecessor is certified once, before the current image", () => {
  const path = "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt";
  const current = readFileSync(new URL(`../../${path}`, import.meta.url), "utf8");
  const inputKeys = beforeInstanceInputKeys(path, current);
  const ownedKeys = beforeUnownedOwnedKeys(path, current);
  assert.equal(new Set([inputKeys, ownedKeys, current]).size, 3);
  assert.deepEqual(certifiedHelperImages(path, current, ["older"]), [
    "older",
    inputKeys,
    ownedKeys,
    current,
  ]);
  assert.deepEqual(certifiedHelperImages(path, current, [inputKeys]), [
    inputKeys,
    ownedKeys,
    current,
  ]);
});

test("navigation keys move onto one ignored line around their unchanged names", () => {
  const prefix = "  const __vize_props_nav_3 = undefined as unknown as __Child_Props_3 & ";
  const owner = "__vize_props_nav_3";
  const before = `x\n${prefix}Record<string, unknown>;\n  void ${owner}.title;\n  void ${owner}['data-x'];\n  void ${owner}.ünï;\ny\n`;
  const after = `x\n${navigationBlock(prefix, owner, [".title", "['data-x']", ".ünï"])}y\n`;
  assert.ok(after.includes("// @ts-ignore Navigation-only references; no diagnostic owner."));
  const edits = withLandedEdits(before, after, []);
  assert.equal(apply(before, edits), after);
  const names = ["title", "data-x", "ünï"];
  for (const name of names) {
    const at = Buffer.byteLength(before.slice(0, before.indexOf(name)));
    const end = at + Buffer.byteLength(name);
    // The audit rejects an edit that overlaps or touches an authored span.
    assert.ok(
      !edits.some(
        (edit) =>
          (edit.start < end && at < edit.end) ||
          [edit.start, edit.end].some((edge) => edge === at || edge === end),
      ),
      name,
    );
  }
  assert.deepEqual(withLandedEdits(before, before, []), []);
  assert.deepEqual(withLandedEdits(before, after.replace(".title", ".other"), []), []);
});

test("an eager per-prop fallback migrates to the lazy helper around its unchanged key", () => {
  const vt = 'import("vize:virtual-ts").';
  const eager = (key: string, fallthrough: string) =>
    `${vt}$PropValue<__Child_ValueProps_0, '${key}', ${vt}$FallthroughValue<__Child_Component_0, '${fallthrough}'>>`;
  const before = `  type __Child_0_prop_title = ${eager("title", "title")};\n  type __Child_0_prop_as_child = ${vt}$PropValue<__Child_ValueProps_0, 'as-child', ${eager("asChild", "as-child")}>;\n`;
  const after = `  type __Child_0_prop_title = ${lazyPropValue(`${vt}$PropValue`, "__Child_ValueProps_0", "title", "__Child_Component_0", "title")};\n  type __Child_0_prop_as_child = ${vt}$PropValue<__Child_ValueProps_0, 'as-child', ${lazyPropValue(`${vt}$PropValue`, "__Child_ValueProps_0", "asChild", "__Child_Component_0", "as-child")}>;\n`;
  const edits = withLandedEdits(before, after, []);
  assert.equal(apply(before, edits), after);
  for (const key of ["'title'", "'asChild'"]) {
    const at = Buffer.byteLength(before.slice(0, before.indexOf(key) + 1));
    assert.ok(!edits.some((edit) => edit.start < at + key.length - 2 && at < edit.end), key);
  }
  assert.deepEqual(withLandedEdits(before, before, []), []);
  assert.deepEqual(
    withLandedEdits(before, after.replace("__Child_Component_0>", "__Other_Component_0>"), [])
      .length,
    2,
  );
});

test("a non-hoisted helper block gains `$PropValueOr` right after `$PropValue`", () => {
  const value =
    "  type $PropValue<P, K extends PropertyKey, F = unknown, __V = P extends unknown ? (K extends keyof P ? P[K] : never) : never> = [__V] extends [never] ? F : __V;\n";
  const added =
    "  type $PropValueOr<P, K extends PropertyKey, C, FK extends PropertyKey = K, __V = P extends unknown ? (K extends keyof P ? P[K] : never) : never> = [__V] extends [never] ? $FallthroughValue<C, FK> : __V;\n";
  const eager =
    "  type __Comp_0_prop_pick = $CallableProp<$PropValue<__Comp_ValueProps_0, 'pick', $FallthroughValue<__Comp_Component_0, 'pick'>>>;\n";
  const lazy = `  type __Comp_0_prop_pick = $CallableProp<${lazyPropValue("$PropValue", "__Comp_ValueProps_0", "pick", "__Comp_Component_0", "pick")}>;\n`;
  const before = `x\n${value}  type $CallableProp<T> = T;\n${eager}y\n`;
  const after = `x\n${value}${added}  type $CallableProp<T> = T;\n${lazy}y\n`;
  const edits = withLandedEdits(before, after, []);
  assert.equal(apply(before, edits), after);
  // The insertion alone does not certify a different helper body.
  assert.notEqual(
    apply(before, withLandedEdits(before, after.replace("$FallthroughValue<C, FK>", "C"), [])),
    after.replace("$FallthroughValue<C, FK>", "C"),
  );
  // A packet that already declares it gains nothing.
  assert.deepEqual(withLandedEdits(after, after, []), []);
  // The mapper's shared-helper prefix migrates as the batch prefix does.
  const vt = 'import("vize/virtual-ts").';
  const prefixed = `  type __Comp_0_prop_pick = ${vt}$PropValue<__Comp_ValueProps_0, 'pick', ${vt}$FallthroughValue<__Comp_Component_0, 'pick'>>;\n`;
  const current = `  type __Comp_0_prop_pick = ${lazyPropValue(`${vt}$PropValue`, "__Comp_ValueProps_0", "pick", "__Comp_Component_0", "pick")};\n`;
  assert.equal(apply(prefixed, withLandedEdits(prefixed, current, [])), current);
});

test("verbatim setup lines (#12481) land as two-byte deletions inside the setup frame", () => {
  const head = "function __setup() {\n  // User setup code\n";
  const map = (end: number) => `  // @vize-map: 42:${end} -> 0:20\n}\n`;
  const before = `${head}  const a = 1;\n  \n  a;\n${map(62)}`;
  const after = `${head}const a = 1;\n\na;\n${map(56)}`;
  const edits = withLandedEdits(before, after, []);
  assert.deepEqual(
    edits.map(({ start, end, replacement }) => [start, end, replacement.toString()]),
    [
      [42, 44, ""],
      [57, 59, ""],
      [60, 62, ""],
    ],
  );
  // The map comment's own end moves with them; the packet audit relocates it.
  assert.equal(apply(before, edits), `${head}const a = 1;\n\na;\n${map(62)}`);
  assert.deepEqual(withLandedEdits(after, after, []), []);
});
