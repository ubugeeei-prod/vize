import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { assertPreservedCallbackPacket } from "../../tools/support/compat/content-mapper/callback-owner-packet-parity.ts";
import { guardedInferenceEdits } from "../../tools/support/compat/content-mapper/callback-owner-current-source.ts";
import { beforeEmitMemberInfer } from "./support/emit-member-infer-source.ts";
import { beforeGenericEventSpecialization } from "./support/generic-event-specialization-source.ts";
import {
  beforeStaticListenerSource,
  staticListenerSource,
} from "./support/static-listener-prop-source.ts";
const root = new URL("../../", import.meta.url);
const read = (path: string) => fs.readFileSync(new URL(path, root), "utf8");
const emitPath = "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-inference.d.ts.txt";
function template(source: string) {
  const match = source.match(/"(\{indent\}type \{inferred_args\} = (?:[^"\\]|\\.)*;\\n)"/u);
  assert.ok(match);
  const values: Record<string, string> = {
    indent: "  ",
    inferred_args: "__Comp_1_save_inferred_emit_args",
    library: "__Comp_1__library",
    emit_props: "__vize_emit_props_0",
    prop_key: "onSave",
    loose_args: "unknown[]",
  };
  return (JSON.parse(`"${match[1]}"`) as string).replace(
    /\{\{|\}\}|\{([a-z_]+)\}/gu,
    (token, name?: string) => (token === "{{" ? "{" : token === "}}" ? "}" : values[name!]!),
  );
}
function fixture(newline = "\n") {
  const currentEmit = read(emitPath),
    memberInferPredecessor = beforeEmitMemberInfer(read, emitPath, currentEmit),
    oldEmit = beforeGenericEventSpecialization(read, emitPath, currentEmit);
  const currentGenerator = read(staticListenerSource),
    oldGenerator = beforeStaticListenerSource(read, staticListenerSource, currentGenerator);
  const first = `const café = '雪🧭';${newline}`,
    last = `void café;${newline}`,
    source = first + last;
  const packet = (scaffold: string, helper: string) => {
    const text = `// 日本 🧭${newline}${first}${scaffold}${last}export type { $Slots as Slots };\n${helper}`;
    const firstAt = Buffer.byteLength(text.slice(0, text.indexOf("café"))),
      secondAt = Buffer.byteLength(text.slice(0, text.lastIndexOf("café"))),
      width = Buffer.byteLength("café");
    return {
      text,
      extension: ".tsx",
      diagnostics: [],
      futureMetadata: { retained: true },
      mappings: [
        [firstAt, width, 6, width, 0, 1048575],
        [secondAt, width, Buffer.byteLength(first + "void "), width, 0, 1048575],
      ],
      semanticLinks: [
        {
          sourceStart: firstAt,
          sourceLength: width,
          targetStart: secondAt,
          targetLength: width,
          kind: "exact-owned",
        },
      ],
      diagnosticDirectives: {
        directives: [
          [0, 0, 0, firstAt, 0, 0],
          [6, width, firstAt, firstAt + width, 1, 0],
          [0, 0, firstAt + width, secondAt, 0, 0],
          [0, 0, secondAt + width, Buffer.byteLength(text), 0, 0],
        ],
        unusedExpectDirectiveDiagnostics: [],
      },
    };
  };
  return {
    before: packet(template(oldGenerator), oldEmit),
    memberInferPredecessor: packet(template(currentGenerator), memberInferPredecessor),
    after: packet(template(currentGenerator), currentEmit),
    source,
  };
}
void test("current callback packets retain every field under exact d9 and static source edits", () => {
  for (const newline of ["\n", "\r\n"]) {
    const { before, after, source } = fixture(newline);
    assertPreservedCallbackPacket(before, after, source);
  }
});
void test("callback history accepts the member-infer predecessor as one whole helper edit", () => {
  for (const newline of ["\n", "\r\n"]) {
    const { memberInferPredecessor, after, source } = fixture(newline);
    assertPreservedCallbackPacket(memberInferPredecessor, after, source);
  }
});
void test("guarded listener inference is certified as one exact source edit", () => {
  for (const newline of ["\n", "\r\n"]) {
    const prefix = `// 日本 🧭${newline}`;
    const direct =
      `  const __vize_emit_props_0 = ` +
      `(undefined as unknown as __Comp_0_emit_resolver)(Comp)({${newline}` +
      `    // @ts-ignore Inference-only prop; the canonical component-props call owns diagnostics.${newline}` +
      `    "value": state.value,${newline}` +
      `  });${newline}`;
    const guarded =
      `  const __vize_emit_props_0 = (() => {${newline}` +
      `    // @ts-ignore Inference-only guard; authored binding checks own diagnostics.${newline}` +
      `    if ((state.value)) return ` +
      `(undefined as unknown as __Comp_0_emit_resolver)(Comp)({${newline}` +
      `      // @ts-ignore Inference-only prop; the canonical component-props call owns diagnostics.${newline}` +
      `      "value": state.value,${newline}` +
      `    });${newline}` +
      `    return undefined as never;${newline}` +
      `  })();${newline}`;
    const before = prefix + direct + `void state;${newline}`;
    const after = prefix + guarded + `void state;${newline}`;
    const edits = guardedInferenceEdits(before, after);
    assert.equal(edits.length, 1);
    let actual = Buffer.from(before);
    for (const edit of [...edits].sort((left, right) => right.start - left.start)) {
      actual = Buffer.concat([
        actual.subarray(0, edit.start),
        edit.replacement,
        actual.subarray(edit.end),
      ]);
    }
    assert.equal(actual.toString(), after);
  }
});
void test("the real callback entry refuses unknown producers, code, tuple fields and suppression changes", () => {
  const { before, after, source } = fixture();
  const check = (packet: typeof after) =>
    assert.throws(() => assertPreservedCallbackPacket(before, packet, source));
  for (const mutation of [
    { ...after, text: after.text + "type Unknown = any;\n" },
    {
      ...after,
      text: after.text.replace(
        "? unknown[] : typeof __vize_emit_props_0",
        "? __L : typeof __vize_emit_props_0",
      ),
    },
    { ...after, text: after.text.replace("type $DeferredEmitProps<", "type $ForeignEmitProps<") },
    {
      ...after,
      text: after.text.replace("export type { $Slots as Slots };", "export type Slots = any;"),
    },
    { ...after, futureMetadata: { retained: false } },
    { ...after, diagnostics: [{ code: 2322 }] as never[] },
  ])
    check(mutation);
  for (let row = 0; row < after.mappings.length; row++)
    for (let field = 0; field < 6; field++) {
      const changed = structuredClone(after);
      changed.mappings[row]![field]!++;
      check(changed);
    }
  for (let row = 0; row < after.diagnosticDirectives.directives.length; row++)
    for (let field = 0; field < 6; field++) {
      const changed = structuredClone(after);
      changed.diagnosticDirectives.directives[row]![field]!++;
      check(changed);
    }
  for (const field of ["sourceStart", "sourceLength", "targetStart", "targetLength"] as const) {
    const changed = structuredClone(after);
    changed.semanticLinks[0]![field]++;
    check(changed);
  }
  const missing = structuredClone(after);
  missing.diagnosticDirectives.directives.splice(0, 1);
  check(missing);
  const missingMapping = structuredClone(after);
  missingMapping.mappings.splice(0, 1);
  check(missingMapping);
  const duplicateMapping = structuredClone(after);
  duplicateMapping.mappings.push(duplicateMapping.mappings[0]!);
  check(duplicateMapping);
  const reordered = structuredClone(after);
  reordered.mappings.reverse();
  check(reordered);
  const changedLink = structuredClone(after);
  changedLink.semanticLinks[0]!.kind = "foreign";
  check(changedLink);
  assert.throws(() =>
    assertPreservedCallbackPacket(before, { ...after, unknownField: true }, source),
  );
  assert.throws(() =>
    assertPreservedCallbackPacket(before, after, source + '<component :is="target" />'),
  );
  for (const path of [
    emitPath,
    staticListenerSource,
    "tests/_fixtures/tooling/static-listener-prop-update-11815.json",
    "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/generic-event-specialization-update-11815.json",
  ])
    assert.throws(() =>
      assertPreservedCallbackPacket(
        before,
        after,
        source,
        undefined,
        (selected) => read(selected) + (selected === path ? "\n" : ""),
      ),
    );
});
