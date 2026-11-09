import assert from "node:assert/strict";
import { createHash } from "node:crypto";

type Identity = { sha256: string; bytes: number };

const identity = (bytes: Buffer): Identity => ({
  sha256: createHash("sha256").update(bytes).digest("hex"),
  bytes: bytes.length,
});

const currentConstructorSource: Identity = {
  bytes: 10946,
  sha256: "70ea8375a7bc7b9617866eefadb0d83aa074240a1c6b603d06af46ed230a2489",
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
      '            "type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<{props_ref}, {emit_props_ref}, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<$ComponentInstance, {own}, {base}, {props_ref}, {emit_props_ref}, $AuthoredProps, $JsxProps>;\\n"',
      "        );",
    ].join("\n") + "\n",
};

const authoredDefaultBaseEdit = {
  before: constructorSourceEdit.after,
  after:
    [
      "        // Constructor arguments (templates, `new`) bind `$AuthoredProps` and",
      "        // leave undeclared attributes to the consumer's own excess-property",
      "        // check; TSX binds `$JsxProps` and stays exact (#11629).",
      "        let constructor_base = if aliases.has_authored_default {",
      "            cstr!(\"Omit<{authored_instance}, '$props' | '$emit' | '$slots'> & {base}\")",
      "        } else {",
      "            base.clone()",
      "        };",
      "        append!(",
      "            *ts,",
      '            "type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<{props_ref}, {emit_props_ref}, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<$ComponentInstance, {own}, {constructor_base}, {props_ref}, {emit_props_ref}, $AuthoredProps, $JsxProps>;\\n"',
      "        );",
    ].join("\n") + "\n",
};

const constructorBaseSplitEdit = {
  before:
    [
      '    ts.push_str(" & ");',
      "    instance_base(",
      "        ts,",
      "        legacy_vue2,",
      "        dialect,",
      "        aliases.has_exposed_type,",
      "        (aliases.has_root_el, aliases.has_refs),",
      "    );",
      '    ts.push_str(";\\n");',
    ].join("\n") + "\n",
  after:
    [
      "    let mut base = String::default();",
      "    instance_base(",
      "        &mut base,",
      "        legacy_vue2,",
      "        dialect,",
      "        aliases.has_exposed_type,",
      "        (aliases.has_root_el, aliases.has_refs),",
      "    );",
      '    ts.push_str(" & ");',
      "    ts.push_str(&base);",
      '    ts.push_str(";\\n");',
    ].join("\n") + "\n",
};

export function constructorSourceTransition(
  current: Buffer,
  direction: "forward" | "inverse",
  historical: Identity,
): Buffer {
  const [fromIdentity, toIdentity, edits] =
    direction === "inverse"
      ? ([
          currentConstructorSource,
          historical,
          [
            [authoredDefaultBaseEdit.after, authoredDefaultBaseEdit.before],
            [constructorSourceEdit.after, constructorSourceEdit.before],
            [constructorBaseSplitEdit.after, constructorBaseSplitEdit.before],
          ],
        ] as const)
      : ([
          historical,
          currentConstructorSource,
          [
            [constructorBaseSplitEdit.before, constructorBaseSplitEdit.after],
            [constructorSourceEdit.before, constructorSourceEdit.after],
            [authoredDefaultBaseEdit.before, authoredDefaultBaseEdit.after],
          ],
        ] as const);
  assert.deepEqual(identity(current), {
    bytes: fromIdentity.bytes,
    sha256: fromIdentity.sha256,
  });
  let text = current.toString("utf8");
  for (const [from, to] of edits) {
    assert.equal(text.split(from).length - 1, 1);
    assert.equal(text.split(to).length - 1, 0);
    text = text.replace(from, to);
  }
  const changed = Buffer.from(text);
  assert.deepEqual(identity(changed), {
    bytes: toIdentity.bytes,
    sha256: toIdentity.sha256,
  });
  return changed;
}
