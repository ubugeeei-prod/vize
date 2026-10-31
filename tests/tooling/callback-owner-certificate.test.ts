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
