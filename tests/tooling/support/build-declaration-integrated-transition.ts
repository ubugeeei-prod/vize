import assert from "node:assert/strict";
import { createHash } from "node:crypto";

import {
  applyDiscriminatedJsxUnionIn,
  beforeDiscriminatedJsxUnionIn,
} from "../../../tools/support/compat/content-mapper/discriminated-jsx-union-transition.ts";

type Identity = { sha256: string; bytes: number };
type Edit = { before: string; after: string };
type Successor = { output: { after: Identity }; edit: { after: string } };
type Direction = "forward" | "inverse";

const integrated: Identity = {
  bytes: 15308,
  sha256: "eaa98dbe1bd335c1d447c86a419b2053beab3aef7e0d617433dee5077bae2263",
};
const previousImport =
  'import type { $ComponentInputProps, $ComponentInputGuard, $ComponentInput, $AuthoredKeyWitness, $UsePublicInstance, $ReplaceComponentProps, $PublicEmit, $PublicSlots, $ComponentPublicBase } from "./__vize_helpers.js";\n';
const integratedImport =
  'import type { $ComponentInput, $ComponentInstanceFor, $PublicEmit, $PublicSlots, $ComponentPublicBase } from "./__vize_helpers.js";\n';
const integratedConstructor =
  'type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, import("./__vize_helpers.js").__EmitProps<Emits>, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<$ComponentInstance, Props, import("./__vize_helpers.js").__EmitProps<Emits>, $AuthoredProps, $JsxProps>;\n';
const encoded = (text: string): string => JSON.stringify(text).slice(1, -1);
const identity = (bytes: Buffer): Identity => ({
  bytes: bytes.length,
  sha256: createHash("sha256").update(bytes).digest("hex"),
});

export function createBuildIntegratedOutputTransition(
  successor: Successor,
  integratedEdits: Edit[],
): (current: Buffer, direction: Direction) => Buffer {
  const changes = [
    { before: encoded(previousImport), after: encoded(integratedImport) },
    { before: encoded(successor.edit.after), after: encoded(integratedConstructor) },
    ...integratedEdits.map((edit) => ({
      before: encoded(edit.before),
      after: encoded(edit.after),
    })),
    {
      before: "$ComponentInput, $AuthoredKeyWitness, $IsUnknown",
      after: "$ComponentInput, $AuthoredKeyWitness, $ComponentInstanceFor, $IsUnknown",
    },
  ];
  return (current, direction) => {
    const [fromIdentity, toIdentity] =
      direction === "forward"
        ? [successor.output.after, integrated]
        : [integrated, successor.output.after];
    assert.deepEqual(identity(current), fromIdentity);
    let text = current.toString("utf8");
    if (direction === "inverse") text = beforeDiscriminatedJsxUnionIn(text);
    for (const edit of direction === "forward" ? changes : [...changes].reverse()) {
      const [from, to] =
        direction === "forward" ? [edit.before, edit.after] : [edit.after, edit.before];
      assert.equal(text.split(from).length - 1, 1);
      assert.equal(text.split(to).length - 1, 0);
      text = text.replace(from, to);
    }
    if (direction === "forward") text = applyDiscriminatedJsxUnionIn(text);
    const changed = Buffer.from(text);
    assert.deepEqual(identity(changed), toIdentity);
    return changed;
  };
}
