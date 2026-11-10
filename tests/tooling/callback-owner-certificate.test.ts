import {
  callbackInertBindings,
  callbackOwnerProjects,
} from "../../tools/support/compat/content-mapper/callback-owner-projects.ts";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { assertDynamicEventArgumentReference } from "../../tools/support/compat/content-mapper/callback-owner-reference.ts";
import {
  assertCallbackNativeAttributeSource,
  assertPreservedCallbackPacket,
} from "../../tools/support/compat/content-mapper/callback-owner-packet-parity.ts";
import {
  currentNativeAttributeAliases,
  previousNativeAttributeAliases,
  currentSlotsExport,
  previousSlotsExport,
  nativeAttributesPath,
} from "../../tools/support/compat/content-mapper/constructor-projection-parity.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("native callback aliases retain the complete source asset and exact UTF-8 geometry", () => {
  const current = fs.readFileSync(path.join(root, nativeAttributesPath), "utf8");
  const previous = current.replace(currentNativeAttributeAliases, previousNativeAttributeAliases);
  assertCallbackNativeAttributeSource(Buffer.from(current));
  for (const changed of [
    current + "\n",
    current.replace("$ComponentAttrCamel", "$Wrong"),
    previous,
  ])
    assert.throws(() => assertCallbackNativeAttributeSource(Buffer.from(changed)));
  for (const newline of ["\n", "\r\n"]) {
    const source = `const café = '雪🧭';${newline}`,
      prefix = `// 日本 🧭${newline}`;
    const packet = (asset: string, slots: string) => {
      const start = Buffer.byteLength(prefix + asset),
        length = Buffer.byteLength(source);
      const name = start + 6,
        width = Buffer.byteLength("café");
      return {
        extension: ".tsx",
        diagnostics: [],
        futureMetadata: { preserved: true },
        text: `${prefix}${asset}${source}// @vize-map: ${start}:${start + length} -> 0:${length}${newline}${slots}\n`,
        mappings: [[name, width, 6, width, 0, 1048575]],
        semanticLinks: [
          { sourceStart: name, sourceLength: width, targetStart: name, targetLength: width },
        ],
        diagnosticDirectives: { directives: [[6, width, name, name + width, 0, 0]] },
      };
    };
    for (const [beforeAsset, afterAsset] of [
      ["", ""],
      [previous, previous],
      [current, current],
      [previous, current],
    ])
      assertPreservedCallbackPacket(
        packet(beforeAsset, previousSlotsExport),
        packet(afterAsset, currentSlotsExport),
        source,
      );
    const before = packet(previous, previousSlotsExport),
      after = packet(current, currentSlotsExport);
    assert.throws(() => assertPreservedCallbackPacket(before, after));
    assert.throws(() => assertPreservedCallbackPacket(before, after, source.slice(0, 2)));
    for (const [a, b] of [
      [previous + previous, current],
      [previous + current, current],
      [previous, current + current],
      [current, previous],
      ["", current],
      [previous, ""],
      [previous, current.replace("Extract<", "Awaited<")],
    ])
      assert.throws(() =>
        assertPreservedCallbackPacket(
          packet(a, previousSlotsExport),
          packet(b, currentSlotsExport),
          source,
        ),
      );
    const comment = after.text.match(/\/\/ @vize-map: [^\n]+/)![0];
    for (const mutation of [
      { ...after, text: after.text.replace(comment, comment.replace(/\d+/, "1")) },
      { ...after, text: after.text.replace(comment, comment.replace("-> 0:", "-> 1:")) },
      { ...after, text: after.text.replace(comment, "") },
      { ...after, text: after.text.replace(comment, `${comment}\n${comment}`) },
      { ...after, text: after.text + "type Unknown = any;\n" },
      { ...after, mappings: [[after.mappings[0][0] + 1, 5, 6, 5, 0, 1048575]] },
      { ...after, diagnostics: [{ start: 0, length: 1, code: 2322 }] },
      { ...after, futureMetadata: { preserved: false } },
    ])
      assert.throws(() => assertPreservedCallbackPacket(before, mutation, source));
    const alias = Buffer.byteLength(
      before.text.slice(0, before.text.indexOf(previousNativeAttributeAliases)),
    );
    for (const [start, length] of [
      [alias - 1, 2],
      [alias, 0],
    ])
      assert.throws(() =>
        assertPreservedCallbackPacket(
          { ...before, mappings: [[start, length, 0, length, 0, 0]] },
          { ...after, mappings: [[start, length, 0, length, 0, 0]] },
          source,
        ),
      );
    assert.throws(() =>
      assertPreservedCallbackPacket(
        { ...before, text: before.text.replace("$ComponentAttrCamel", "$Wrong") },
        { ...after, text: after.text.replace("$ComponentAttrCamel", "$Wrong") },
        source,
      ),
    );
    assert.throws(() =>
      assertPreservedCallbackPacket(
        { ...before, mappings: [[before.mappings[0][0] + 4, 1, 10, 1, 0, 0]] },
        { ...after, mappings: [[after.mappings[0][0] + 4, 1, 10, 1, 0, 0]] },
        source,
      ),
    );
    const invalidDirective = (packet: typeof before) => ({
      ...packet,
      diagnosticDirectives: {
        directives: [
          [
            Buffer.byteLength(source) + 1,
            0,
            packet.mappings[0][0],
            packet.mappings[0][0] + 5,
            0,
            0,
          ],
        ],
      },
    });
    assert.throws(() =>
      assertPreservedCallbackPacket(invalidDirective(before), invalidDirective(after), source),
    );
    assert.throws(() =>
      assertPreservedCallbackPacket(
        { ...before, mappings: [[before.mappings[0][0] + 4, 1, 6, 5, 0, 0]] },
        { ...after, mappings: [[after.mappings[0][0] + 4, 1, 6, 5, 0, 0]] },
        source,
      ),
    );
  }
});

test("computed mutation reference admission requires every independent diagnostic and state", () => {
  for (const line of [8, 9]) {
    const span = {
      file: "reference.ts",
      start: 0,
      end: 5,
      line,
      column: 7,
      endLine: line,
      endColumn: 12,
      text: "owner",
    };
    const stdout = `reference.ts(${line},7): error TS2322: Type '(field: undefined) => false' is not assignable to type '(x: number) => boolean'.\n  Types of parameters 'field' and 'x' are incompatible.\n    Type 'number' is not assignable to type 'undefined'.\n`;
    const reference = {
      argv: [],
      cwd: ".",
      status: 2,
      signal: null,
      error: null,
      stderr: "",
      stdout,
    };
    const native = { ...reference, status: 1 };
    assertDynamicEventArgumentReference(reference, native, span);
    for (const changed of [
      { ...reference, status: 0 },
      { ...reference, stderr: "warning" },
      { ...reference, signal: "SIGTERM" },
      { ...reference, stdout: stdout + stdout },
      { ...reference, stdout: stdout.replace("TS2322", "TS2339") },
      { ...reference, stdout: stdout.replace(",7)", ",8)") },
      { ...reference, stdout: stdout.replace("undefined", "unknown") },
      { ...reference, stdout: stdout.split("\n").slice(0, 1).join("\n") + "\n" },
    ])
      assert.throws(() => assertDynamicEventArgumentReference(changed, native, span));
    assert.throws(() =>
      assertDynamicEventArgumentReference(reference, { ...native, status: 0 }, span),
    );
    assert.throws(() =>
      assertDynamicEventArgumentReference(
        reference,
        { ...native, stdout: stdout.replace("number", "string") },
        span,
      ),
    );
  }
});

test("inert navigation copies retain the whole packet and every mapped callback owner", () => {
  for (const newline of ["\n", "\r\n"]) {
    for (const prefix of ["", "<!-- 日本 🧭 -->"]) {
      const value = "(field: typeof state.value) => !!field";
      const source = `${prefix}<template>${newline}<Comp :pick="${value}" /></template>`;
      const nameAt = source.indexOf(":pick") + 1;
      const valueAt = source.indexOf(value);
      const binding = {
        callbackMarker: true,
        name: { start: nameAt, end: nameAt + 4, text: "pick" },
        value: { start: valueAt, end: valueAt + value.length, text: value },
      };
      const text = `// 日本 🧭\nconst checked: (x: number) => boolean = ${value};\n  // @ts-ignore Inference-only call; the authored prop checker owns diagnostics.\n  const __vize_events_resolved_0 = (undefined as unknown as __vize_events_resolver_0)({\n    // @ts-ignore Inference-only callback prop; mapped prop owner checks diagnostics.\n    "pick": ${value},\n  });\n`;
      const key = Buffer.byteLength(text.slice(0, text.indexOf('"pick"')));
      const generatedValue = key + 8;
      const originalValue = Buffer.byteLength(source.slice(0, valueAt));
      const rows = [
        [Buffer.byteLength(source.slice(0, nameAt)), 4, key, key + 6, 0, 0],
        [originalValue, value.length, generatedValue, generatedValue + value.length, 0, 0],
      ];
      const mapped = Buffer.byteLength(text.slice(0, text.indexOf(value)));
      const before = {
        text,
        extension: ".tsx",
        diagnostics: [],
        futureMetadata: { unchanged: true },
        mappings: [[mapped, value.length, originalValue, value.length, 0, 1048575]],
      };
      const after = {
        ...before,
        diagnosticDirectives: { directives: rows, unusedExpectDirectiveDiagnostics: [] },
      };
      assertPreservedCallbackPacket(before, after, source, [binding]);
      assertPreservedCallbackPacket(after, after, source, [binding]);
      assert.throws(() => assertPreservedCallbackPacket(before, after, source));
      assert.throws(() => assertPreservedCallbackPacket(before, after, source, []));
      for (const changed of [
        { ...after, text: text + "void missing;\n" },
        { ...after, diagnostics: [{ code: 2322 }] },
        { ...after, futureMetadata: { unchanged: false } },
        {
          ...after,
          diagnosticDirectives: { ...after.diagnosticDirectives, directives: rows.slice(0, 1) },
        },
        {
          ...after,
          diagnosticDirectives: { ...after.diagnosticDirectives, directives: [...rows, rows[0]] },
        },
        {
          ...after,
          diagnosticDirectives: {
            ...after.diagnosticDirectives,
            directives: rows.map((row) => [...row.slice(0, 4), 1, 0]),
          },
        },
        {
          ...after,
          diagnosticDirectives: {
            ...after.diagnosticDirectives,
            directives: rows.map((row) => [
              ...row.slice(0, 2),
              mapped,
              mapped + value.length,
              0,
              0,
            ]),
          },
        },
        {
          ...after,
          diagnosticDirectives: {
            ...after.diagnosticDirectives,
            unusedExpectDirectiveDiagnostics: [{ code: 4 }],
          },
        },
      ])
        assert.throws(() => assertPreservedCallbackPacket(before, changed, source, [binding]));
      const overlapping = {
        ...before,
        mappings: [
          ...before.mappings,
          [generatedValue, value.length, originalValue, value.length, 0, 0],
        ],
      };
      assert.throws(() =>
        assertPreservedCallbackPacket(
          overlapping,
          { ...after, mappings: overlapping.mappings },
          source,
          [binding],
        ),
      );
      assert.throws(() =>
        assertPreservedCallbackPacket(before, after, source, [
          { ...binding, value: { ...binding.value, start: valueAt + 1 } },
        ]),
      );
      const old = {
        directives: [[1, 1, 0, 1, 1, 3]],
        unusedExpectDirectiveDiagnostics: [{ preserved: true }],
      };
      const prior = { ...before, diagnosticDirectives: old };
      const expected = {
        ...after,
        diagnosticDirectives: { ...old, directives: [...old.directives, ...rows] },
      };
      assertPreservedCallbackPacket(prior, expected, source, [binding]);
      assert.throws(() => assertPreservedCallbackPacket(prior, after, source, [binding]));
    }
  }
});

test("component prop navigation remains the sole callback hover owner", () => {
  const text = 'void __vize_props_nav_0.value;\n({ "value": handler });\n';
  const navigation = Buffer.from(text).indexOf("value");
  const check = Buffer.from(text).lastIndexOf("value");
  const before = {
    text,
    extension: ".tsx",
    diagnostics: [],
    mappings: [
      [navigation, 5, 20, 5, 0, 1048575],
      [check, 5, 20, 5, 0, 1048575],
    ],
  };
  const after = structuredClone(before);
  after.mappings[1]![5]! &= ~1;
  assertPreservedCallbackPacket(before, after);

  for (const mutate of [
    (packet: typeof after) => (packet.mappings[0]![5]! &= ~1),
    (packet: typeof after) => (packet.mappings[1]![5]! &= ~2),
    (packet: typeof after) => (packet.mappings[1]![2]! += 1),
    (packet: typeof after) =>
      (packet.text = packet.text.replace("__vize_props_nav_", "__vize_other_nav_")),
  ]) {
    const changed = structuredClone(after);
    mutate(changed);
    assert.throws(() => assertPreservedCallbackPacket(before, changed));
  }
});

test("the existing getter sibling has only its exact unmapped navigation rows", () => {
  const project = callbackOwnerProjects().find(
    (p) => p.id === "callback-owner-unsupported-sibling-object-getter-literal",
  )!;
  const source = project.inlineInputs["src/Min.vue"];
  const bindings = callbackInertBindings(project);
  assert.deepEqual(
    bindings.map((b) => [b.name.text, b.callbackMarker]),
    [
      ["pick", true],
      ["other", false],
    ],
  );
  assert.equal(bindings[1].value.text, "{ get value() { delete state.value; return 1 } }");
  let text = bindings.map((b, i) => `const checked${i} = ${b.value.text};\n`).join("");
  const mappings = bindings.map((b) => [
    Buffer.byteLength(text.slice(0, text.indexOf(b.value.text))),
    Buffer.byteLength(b.value.text),
    Buffer.byteLength(source.slice(0, b.value.start)),
    Buffer.byteLength(b.value.text),
    0,
    1048575,
  ]);
  text +=
    "  // @ts-ignore Inference-only call; the authored prop checker owns diagnostics.\n  const __vize_events_resolved_0 = (undefined as unknown as __vize_events_resolver_0)({\n";
  const rows: number[][] = [];
  for (const binding of bindings) {
    if (binding.callbackMarker)
      text +=
        "    // @ts-ignore Inference-only callback prop; mapped prop owner checks diagnostics.\n";
    const key = Buffer.byteLength(text) + 4;
    const value = key + Buffer.byteLength(`"${binding.name.text}": `);
    text += `    "${binding.name.text}": ${binding.value.text},\n`;
    rows.push([
      Buffer.byteLength(source.slice(0, binding.name.start)),
      Buffer.byteLength(binding.name.text),
      key,
      value - 2,
      0,
      0,
    ]);
    rows.push([
      Buffer.byteLength(source.slice(0, binding.value.start)),
      Buffer.byteLength(binding.value.text),
      value,
      value + Buffer.byteLength(binding.value.text),
      0,
      0,
    ]);
  }
  text += "  });\n";
  const before = {
    text,
    mappings,
    diagnostics: [],
    extension: ".tsx",
    foreign: { preserved: true },
  };
  const after = {
    ...before,
    diagnosticDirectives: { directives: rows, unusedExpectDirectiveDiagnostics: [] },
  };
  assertPreservedCallbackPacket(before, after, source, bindings);
  assert.throws(() => assertPreservedCallbackPacket(before, after, source, bindings.slice(0, 1)));
  assert.throws(() =>
    assertPreservedCallbackPacket(
      before,
      after,
      source,
      bindings.map((b) => ({ ...b, callbackMarker: true })),
    ),
  );
  for (const directives of [
    rows.slice(0, 3),
    [...rows, [0, 1, 0, 1, 0, 0]],
    rows.map((row, i) => (i === 3 ? [...row.slice(0, 3), row[3] + 1, 0, 0] : row)),
  ])
    assert.throws(() =>
      assertPreservedCallbackPacket(
        before,
        { ...after, diagnosticDirectives: { ...after.diagnosticDirectives, directives } },
        source,
        bindings,
      ),
    );
});
