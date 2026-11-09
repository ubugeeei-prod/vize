import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

import { instanceInputKeysHelperEdit } from "../../../tools/support/compat/content-mapper/instance-input-keys-transition.ts";
import { repoRoot } from "../_helpers/tool-commands.ts";

type CanonLeftoversReceipt = {
  compiledPreamble: { edits: Array<{ before: string; after: string }> };
  files: SourceTransition[];
};
type Identity = { bytes: number; sha256: string };
type SourceTransition = {
  path: string;
  before: Identity;
  after: Identity;
  edits: Array<{ before: string; after: string }>;
};

const receiptBytes = fs.readFileSync(
  path.join(
    repoRoot,
    "crates/vize_canon/tests/empty_vue_star_exports/membership/constructor_without_args/payload_update/canon_leftovers_11499.json",
  ),
);
assert.equal(
  createHash("sha256").update(receiptBytes).digest("hex"),
  "250e01138d163f77a5d98b20bc97f562a663ebb924c17c810b9d9cf02280128e",
);
const receipt = JSON.parse(receiptBytes.toString("utf8")) as CanonLeftoversReceipt;
export const canonLeftovers = receipt.compiledPreamble.edits;

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

export function beforeCanonLeftoversSource(file: string, current: string): string {
  const transition = receipt.files.find((candidate) => candidate.path === file);
  if (!transition) return current;
  assert.deepEqual(identity(current), transition.after);
  let previous = current;
  for (const edit of transition.edits) {
    assert.equal(previous.split(edit.after).length - 1, 1);
    previous = previous.replace(edit.after, () => edit.before);
  }
  assert.deepEqual(identity(previous), transition.before);
  return previous;
}

const previousConstructor =
  "type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, {}, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<$ComponentInstance, Props, {}, $AuthoredProps, $JsxProps>;\n";
const currentConstructor =
  "type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, {}, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<$ComponentInstance, {\n    $props: Props;\n    readonly __vizeRawProps?: Props;\n    $emit: $PublicEmit<Emits>;\n    $slots: $PublicSlots<$Slots>;\n}, $ComponentPublicBase, Props, {}, $AuthoredProps, $JsxProps>;\n";
const previousBuildConstructor =
  'type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, import("./__vize_helpers.js").__EmitProps<Emits>, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<$ComponentInstance, Props, import("./__vize_helpers.js").__EmitProps<Emits>, $AuthoredProps, $JsxProps>;\n';
const currentBuildConstructor =
  'type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, import("./__vize_helpers.js").__EmitProps<Emits>, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<$ComponentInstance, {\n    $props: Props & import("./__vize_helpers.js").__EmitProps<Emits>;\n    readonly __vizeRawProps?: Props;\n    $emit: $PublicEmit<Emits>;\n    $slots: $PublicSlots<$Slots>;\n}, $ComponentPublicBase, Props, import("./__vize_helpers.js").__EmitProps<Emits>, $AuthoredProps, $JsxProps>;\n';
const exportEdits = [
  ["$LiteralKeys, $Replace", "$LiteralKeys, $PatternSafeOmit, $Replace"],
  [
    "$IsStringLiteral, $ParametersToFns",
    "$IsStringLiteral, $LiteralParameterKeys, $ParametersForKey, $LiteralParametersToFns, $ParametersToFns",
  ],
  ["__EmitOptions, $Camelize", "__EmitOptions, $LiteralEmitOptions, $Camelize"],
  [
    "$HandlerKey, __EmitProps",
    "$HandlerKey, $EmitPropsFromOptions, $ResolvedEmitProps, __EmitProps",
  ],
] as const;
const encoded = (text: string): string => JSON.stringify(text).slice(1, -1);

function outputTransition(
  current: Buffer,
  direction: "forward" | "inverse",
  constructorBefore: string,
  constructorAfter: string,
): Buffer {
  let text = current.toString("utf8");
  // #11654 rewrote one helper line after these layers; undo it first, redo it last.
  const keys = instanceInputKeysHelperEdit();
  if (direction === "inverse") {
    assert.equal(text.split(encoded(keys.after)).length - 1, 1);
    text = text.replace(encoded(keys.after), () => encoded(keys.before));
  }
  if (direction === "inverse") {
    const from = encoded(constructorAfter),
      to = encoded(constructorBefore);
    assert.equal(text.split(from).length - 1, 1);
    text = text.replace(from, to);
  }
  const edits = direction === "inverse" ? canonLeftovers : [...canonLeftovers].reverse();
  for (const edit of edits) {
    const from = encoded(direction === "inverse" ? edit.after : edit.before),
      to = encoded(direction === "inverse" ? edit.before : edit.after);
    if (direction === "inverse") {
      assert.equal(text.split(from).length - 1, 1);
      text = text.replace(from, () => to);
    } else if (text.includes(from)) {
      text = text.replaceAll(from, () => to);
    }
  }
  const memberships = direction === "inverse" ? exportEdits : [...exportEdits].reverse();
  for (const [before, after] of memberships) {
    const from = direction === "inverse" ? after : before,
      to = direction === "inverse" ? before : after;
    assert.equal(text.split(from).length - 1, 1);
    text = text.replace(from, () => to);
  }
  if (direction === "forward") {
    const from = encoded(constructorBefore),
      to = encoded(constructorAfter);
    assert.ok(text.split(from).length - 1 >= 1);
    text = text.replaceAll(from, () => to);
    assert.equal(text.split(encoded(keys.before)).length - 1, 1);
    text = text.replace(encoded(keys.before), () => encoded(keys.after));
  }
  return Buffer.from(text);
}

export function canonLeftoversOutputTransition(
  current: Buffer,
  direction: "forward" | "inverse",
): Buffer {
  return outputTransition(current, direction, previousConstructor, currentConstructor);
}

/** Replay the same helper successor around the declaration-build constructor shape. */
export function canonLeftoversBuildOutputTransition(
  current: Buffer,
  direction: "forward" | "inverse",
): Buffer {
  return outputTransition(current, direction, previousBuildConstructor, currentBuildConstructor);
}
