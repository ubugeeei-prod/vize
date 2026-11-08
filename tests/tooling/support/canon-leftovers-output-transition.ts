import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

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
  "35da3b22f2108e76be0b096a27bf2a54e9f40dd999d5e8911a7b1f4b32483306",
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

export function canonLeftoversOutputTransition(
  current: Buffer,
  direction: "forward" | "inverse",
): Buffer {
  let text = current.toString("utf8");
  if (direction === "inverse") {
    const from = encoded(currentConstructor),
      to = encoded(previousConstructor);
    assert.equal(text.split(from).length - 1, 1);
    text = text.replace(from, to);
  }
  const edits = direction === "inverse" ? canonLeftovers : [...canonLeftovers].reverse();
  for (const edit of edits) {
    const from = encoded(direction === "inverse" ? edit.after : edit.before),
      to = encoded(direction === "inverse" ? edit.before : edit.after);
    if (text.includes(from)) text = text.replaceAll(from, () => to);
  }
  const memberships = direction === "inverse" ? exportEdits : [...exportEdits].reverse();
  for (const [before, after] of memberships) {
    const from = direction === "inverse" ? after : before,
      to = direction === "inverse" ? before : after;
    assert.equal(text.split(from).length - 1, 1);
    text = text.replace(from, () => to);
  }
  if (direction === "forward") {
    const from = encoded(previousConstructor),
      to = encoded(currentConstructor);
    assert.ok(text.split(from).length - 1 >= 1);
    text = text.replaceAll(from, () => to);
  }
  return Buffer.from(text);
}
