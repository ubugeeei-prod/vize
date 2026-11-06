import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export type Identity = { sha256: string; bytes: number };
type Direction = "forward" | "inverse";

const identity = (bytes: Buffer): Identity => ({
  sha256: createHash("sha256").update(bytes).digest("hex"),
  bytes: bytes.length,
});
const encoded = (text: string): string => JSON.stringify(text).slice(1, -1);
const originalConstructorSource: Identity = {
  bytes: 10645,
  sha256: "94e915b4968a2d7d4b0d1b4292b74aa24cbc7434e4b7d8f191213b59cfdbb853",
};
const constructorSourceEdit = {
  before:
    [
      "        append!(",
      "            *ts,",
      '            "type $ComponentConstructor = new <$AuthoredProps = unknown>(props?: $AuthoredProps & $ComponentInput<{props_ref}, {emit_props_ref}, $AuthoredProps>, ...args: any[]) => $UsePublicInstance<$AuthoredProps> extends true ? $ComponentInstance : $ReplaceComponentProps<{own}, $AuthoredKeyWitness<$AuthoredProps> & $ComponentInputProps<{props_ref}, {emit_props_ref}> & $ComponentInputGuard<{props_ref}, {emit_props_ref}, $AuthoredProps>> & "',
      "        );",
      "        if aliases.has_authored_default {",
      "            append!(",
      "                *ts,",
      "                \"Omit<{authored_instance}, '$props' | '$emit' | '$slots'> & \"",
      "            );",
      "        }",
      "        instance_base(",
      "            ts,",
      "            legacy_vue2,",
      "            dialect,",
      "            aliases.has_exposed_type,",
      "            (aliases.has_root_el, aliases.has_refs),",
      "        );",
      '        ts.push_str(";\\n");',
    ].join("\n") + "\n",
  after:
    [
      "        // Constructor arguments (templates, `new`) bind `$AuthoredProps` and",
      "        // leave undeclared attributes to the consumer's own excess-property",
      "        // check; TSX binds `$JsxProps` and stays exact (#11629).",
      "        append!(",
      "            *ts,",
      '            "type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<{props_ref}, {emit_props_ref}, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<$ComponentInstance, {props_ref}, {emit_props_ref}, $AuthoredProps, $JsxProps>;\\n"',
      "        );",
    ].join("\n") + "\n",
};

export function constructorSourceTransition(
  current: Buffer,
  direction: Direction,
  historical: Identity,
): Buffer {
  const [fromIdentity, toIdentity, from, to] =
    direction === "inverse"
      ? ([
          originalConstructorSource,
          historical,
          constructorSourceEdit.after,
          constructorSourceEdit.before,
        ] as const)
      : ([
          historical,
          originalConstructorSource,
          constructorSourceEdit.before,
          constructorSourceEdit.after,
        ] as const);
  assert.deepEqual(identity(current), {
    bytes: fromIdentity.bytes,
    sha256: fromIdentity.sha256,
  });
  const text = current.toString("utf8");
  assert.equal(text.split(from).length - 1, 1);
  assert.equal(text.split(to).length - 1, 0);
  const changed = Buffer.from(text.replace(from, to));
  assert.deepEqual(identity(changed), {
    bytes: toIdentity.bytes,
    sha256: toIdentity.sha256,
  });
  return changed;
}
const ownSplitConstructorSource: Identity = {
  bytes: 11141,
  sha256: "48518a57f01d82dc1a07eac20951951191638d92dd94dcd978843741cf5a7dd3",
};
const elidedConstructorSource: Identity = {
  bytes: 11079,
  sha256: "c8a043b7eca7b7401ccd2f2bd3eb0b63149740f9982da9e68247fcb65ff3119e",
};
const ownSplitSourceEdits = [
  {
    before: ["        } else {", '            String::from("{}")', "        };"].join("\n") + "\n",
    after:
      [
        "        } else {",
        '            String::from("{}")',
        "        };",
        "        let mut component_instance_base = String::default();",
        "        if aliases.has_authored_default {",
        "            append!(",
        "                component_instance_base,",
        "                \"Omit<{authored_instance}, '$props' | '$emit' | '$slots'> & \"",
        "            );",
        "        }",
        "        instance_base(",
        "            &mut component_instance_base,",
        "            legacy_vue2,",
        "            dialect,",
        "            aliases.has_exposed_type,",
        "            (aliases.has_root_el, aliases.has_refs),",
        "        );",
      ].join("\n") + "\n",
  },
  {
    before:
      '            "type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<{props_ref}, {emit_props_ref}, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<$ComponentInstance, {props_ref}, {emit_props_ref}, $AuthoredProps, $JsxProps>;\\n"',
    after:
      '            "type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<{props_ref}, {emit_props_ref}, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<{own}, {props_ref}, {emit_props_ref}, $AuthoredProps, $JsxProps> & {component_instance_base};\\n"',
  },
] as const;
const elidedSourceEdits = [
  {
    before:
      [
        "    if aliases.has_authored_default && legacy_component {",
        '        append!(*ts, "type $ComponentInstance = {authored_instance} & ");',
        "    } else if aliases.has_authored_default {",
        "        append!(",
        "            *ts,",
        "            \"type $ComponentInstance = Omit<{authored_instance}, '$props' | '$emit' | '$slots'> & \"",
        "        );",
        "    } else {",
        '        ts.push_str("type $ComponentInstance = ");',
        "    }",
        "    // Keep the public instance's anonymous object unchanged for diagnostic text.",
        "    // Reuse its four owned fields inline without a capturable utility lookup.",
        "    let mut own = String::default();",
        "    let fields = if legacy_component { &mut *ts } else { &mut own };",
      ].join("\n") + "\n",
    after:
      [
        "    // Vue 2 still exposes the named instance through its overload. Vue 3 maps",
        "    // only the four owned fields inline and intersects the unmapped public",
        "    // base, so emitting the old whole-instance alias would be dead work and a",
        "    // `noUnusedLocals` diagnostic in editor projects.",
        "    let mut own = String::default();",
        "    let fields = if legacy_component {",
        "        if aliases.has_authored_default {",
        '            append!(*ts, "type $ComponentInstance = {authored_instance} & ");',
        "        } else {",
        '            ts.push_str("type $ComponentInstance = ");',
        "        }",
        "        &mut *ts",
        "    } else {",
        "        &mut own",
        "    };",
      ].join("\n") + "\n",
  },
  {
    before:
      [
        "    fields.push('}');",
        "    if !legacy_component {",
        "        ts.push_str(&own);",
        "    }",
        '    ts.push_str(" & ");',
        "    instance_base(",
        "        ts,",
        "        legacy_vue2,",
        "        dialect,",
        "        aliases.has_exposed_type,",
        "        (aliases.has_root_el, aliases.has_refs),",
        "    );",
        '    ts.push_str(";\\n");',
        "    if legacy_component {",
      ].join("\n") + "\n",
    after:
      [
        "    fields.push('}');",
        "    if legacy_component {",
        '        ts.push_str(" & ");',
        "        instance_base(",
        "            ts,",
        "            legacy_vue2,",
        "            dialect,",
        "            aliases.has_exposed_type,",
        "            (aliases.has_root_el, aliases.has_refs),",
        "        );",
        '        ts.push_str(";\\n");',
      ].join("\n") + "\n",
  },
] as const;

function applyEdits(
  current: Buffer,
  direction: Direction,
  fromIdentity: Identity,
  toIdentity: Identity,
  edits: ReadonlyArray<{ before: string; after: string }>,
): Buffer {
  assert.deepEqual(identity(current), fromIdentity);
  let text = current.toString("utf8");
  const ordered = direction === "inverse" ? [...edits].reverse() : edits;
  for (const edit of ordered) {
    const [from, to] =
      direction === "inverse" ? [edit.after, edit.before] : [edit.before, edit.after];
    assert.equal(text.split(from).length - 1, 1);
    text = text.replace(from, to);
  }
  const changed = Buffer.from(text);
  assert.deepEqual(identity(changed), toIdentity);
  return changed;
}

export function ownSplitSourceTransition(current: Buffer, direction: Direction): Buffer {
  return applyEdits(
    current,
    direction,
    direction === "inverse" ? ownSplitConstructorSource : originalConstructorSource,
    direction === "inverse" ? originalConstructorSource : ownSplitConstructorSource,
    ownSplitSourceEdits,
  );
}

export function elidedSourceTransition(current: Buffer, direction: Direction): Buffer {
  return applyEdits(
    current,
    direction,
    direction === "inverse" ? elidedConstructorSource : ownSplitConstructorSource,
    direction === "inverse" ? ownSplitConstructorSource : elidedConstructorSource,
    elidedSourceEdits,
  );
}

const aliasedCheckOutput: Identity = {
  bytes: 15761,
  sha256: "178e1e0a0c9c5e741dc721929cd76de9d2692e16c4e8b23acad68e05e726965d",
};
const elidedCheckOutput: Identity = {
  bytes: 15714,
  sha256: "8422869d388021988ea57e3067d8900e2eef596c0f1f20b43221bd68cd0ba3d3",
};
const aliasedCheckInstance =
  "type $ComponentInstance = {\n    $props: Props;\n    readonly __vizeRawProps?: Props;\n    $emit: $PublicEmit<Emits>;\n    $slots: $PublicSlots<$Slots>;\n} & $ComponentPublicBase;\ntype $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, {}, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<$ComponentInstance, Props, {}, $AuthoredProps, $JsxProps>;\n";
const elidedCheckInstance =
  "type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, {}, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<{\n    $props: Props;\n    readonly __vizeRawProps?: Props;\n    $emit: $PublicEmit<Emits>;\n    $slots: $PublicSlots<$Slots>;\n}, Props, {}, $AuthoredProps, $JsxProps> & $ComponentPublicBase;\n";

export function elidedCheckOutputTransition(current: Buffer, direction: Direction): Buffer {
  const [fromIdentity, toIdentity, from, to] =
    direction === "inverse"
      ? [elidedCheckOutput, aliasedCheckOutput, elidedCheckInstance, aliasedCheckInstance]
      : [aliasedCheckOutput, elidedCheckOutput, aliasedCheckInstance, elidedCheckInstance];
  assert.deepEqual(identity(current), fromIdentity);
  const text = current.toString("utf8");
  const encodedFrom = encoded(from);
  const encodedTo = encoded(to);
  assert.equal(text.split(encodedFrom).length - 1, 1);
  assert.equal(text.split(encodedTo).length - 1, 0);
  const changed = Buffer.from(text.replace(encodedFrom, encodedTo));
  assert.deepEqual(identity(changed), toIdentity);
  return changed;
}
