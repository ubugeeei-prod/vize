/** Exact unmapped callback scaffolding transitions; all other packet fields stay intact. */
import assert from "node:assert/strict";
import {
  currentSlotsExport,
  currentSlotsMarker,
  previousSlotsExport,
  previousSlotsMarker,
} from "./constructor-projection-parity.ts";

type Edit = { start: number; end: number; replacement: Buffer };
const nativeRoot =
  "{ readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: infer __F }";
const argumentSuffix =
  ") extends infer __A extends unknown[] ? (__A extends [] ? unknown[] : __A) : unknown[] : unknown[]\n";

export function callbackNativeEventGate(component: string): string {
  return `"__vizeEmitProps" extends keyof { [__K in keyof typeof ${component} as string extends __K ? never : number extends __K ? never : symbol extends __K ? never : __K]: unknown } ? `;
}

function scaffoldEdits(text: string): Edit[] {
  const edits: Edit[] = [];
  const add = (start: number, end: number, replacement: string) => {
    edits.push({
      start: Buffer.byteLength(text.slice(0, start)),
      end: Buffer.byteLength(text.slice(0, end)),
      replacement: Buffer.from(replacement),
    });
  };
  const argumentHeader =
    /^([ \t]*)type ([^\s=]+_fallthrough_args) = typeof (.+?) extends \{ readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps\?: infer __F \}\n/gmu;
  for (const match of text.matchAll(argumentHeader)) {
    const [header, indent, , component] = match;
    const start = match.index + header.length;
    const end = text.indexOf("\n", start) + 1;
    assert.ok(end > start, "complete native argument body required");
    const body = text.slice(start, end);
    assert.match(
      body,
      /^[ \t]*\? (?:\$IsAny|import\("vize:virtual-ts"\)\.\$IsAny)<__F> extends true \? unknown\[\] : NonNullable<__F> extends infer __P \? \(/u,
    );
    assert.ok(
      body.endsWith(argumentSuffix),
      "only the original native argument branch is admitted",
    );
    assert.equal(
      text.slice(end, end + `${indent}  : unknown[];\n`.length),
      `${indent}  : unknown[];\n`,
    );
    const gate = start + `${indent}  ? `.length;
    assert.equal(text.slice(start, gate), `${indent}  ? `);
    add(gate, gate, callbackNativeEventGate(component));
    const tail = end + `${indent}  : unknown[]`.length;
    add(tail, tail, " : unknown[]");
  }
  const returnHeader =
    /^([ \t]*)type ([^\s=]+_native_fallthrough) = typeof (.+?) extends \{ readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps\?: infer __F \} \? .*;\n/gmu;
  for (const match of text.matchAll(returnHeader)) {
    const [line, indent, name, component] = match;
    const prefix = `${indent}type ${name} = typeof ${component} extends ${nativeRoot} ? `;
    assert.ok(line.startsWith(prefix));
    assert.match(
      line.slice(prefix.length),
      /^(?:\$IsAny|import\("vize:virtual-ts"\)\.\$IsAny)<__F> extends true \? any : NonNullable<__F> extends \{ .+?\?: infer __H \} \? Extract<NonNullable<__H>, \(\.\.\.args: any\[\]\) => any> extends \(\.\.\.args: any\[\]\) => infer __R \? __R : any : any : any;\n$/u,
    );
    const gate = match.index + prefix.length;
    add(gate, gate, callbackNativeEventGate(component));
    const tail = match.index + line.length - 2;
    add(tail, tail, " : any");
  }
  for (const [before, after] of [
    [previousSlotsExport, currentSlotsExport],
    [previousSlotsMarker, currentSlotsMarker],
  ]) {
    const start = text.indexOf(before);
    if (start < 0) continue;
    assert.equal(text.indexOf(before, start + 1), -1, "a certified Slots token occurs once");
    if (before === previousSlotsMarker) {
      const exportAt = text.indexOf(previousSlotsExport);
      assert.ok(exportAt >= 0 && start > exportAt, "the marker follows its export");
    }
    add(start, start + before.length, after);
  }
  edits.sort((left, right) => left.start - right.start);
  for (let index = 1; index < edits.length; index++) {
    assert.ok(edits[index - 1].end <= edits[index].start, "scaffold edits are disjoint");
    assert.notEqual(edits[index - 1].start, edits[index].start, "edit boundary is unambiguous");
  }
  return edits;
}

/** UTF-8 byte ranges may move around scaffolding, but may never claim it. */
export function assertPreservedCallbackPacket(
  before: Record<string, unknown>,
  after: Record<string, unknown>,
) {
  assert.equal(typeof before.text, "string");
  assert.equal(typeof after.text, "string");
  if (before.text === after.text) {
    assert.deepEqual(after, before, "all unchanged callback fields stay exact");
    return;
  }
  const original = before.text as string;
  const source = Buffer.from(original);
  const actual = Buffer.from(after.text as string);
  const edits = scaffoldEdits(original);
  assert.ok(edits.length > 0, "only observed callback scaffolding may change");
  const chunks: Buffer[] = [];
  let position = 0;
  for (const edit of edits) {
    chunks.push(source.subarray(position, edit.start), edit.replacement);
    position = edit.end;
  }
  chunks.push(source.subarray(position));
  assert.deepEqual(Buffer.concat(chunks), actual, "the complete text has only certified edits");
  const exportAt = original.indexOf(previousSlotsExport);
  const boundary = exportAt < 0 ? source.length : Buffer.byteLength(original.slice(0, exportAt));
  const point = (offset: number) =>
    offset +
    edits
      .filter((edit) => edit.end <= offset)
      .reduce((delta, edit) => delta + edit.replacement.length - (edit.end - edit.start), 0);
  const range = (start: number, length: number): [number, number] => {
    assert.ok(Number.isSafeInteger(start) && start >= 0);
    assert.ok(Number.isSafeInteger(length) && length >= 0);
    const end = start + length;
    assert.ok(end <= boundary, "authored projection precedes the public Slots export");
    for (const edit of edits) {
      assert.ok(
        edit.start === edit.end
          ? !(start < edit.start && edit.start < end)
          : !(start < edit.end && edit.start < end),
        "authored projection cannot overlap scaffold edits",
      );
      assert.ok(
        ![edit.start, edit.end].includes(start) && ![edit.start, edit.end].includes(end),
        "authored projection cannot sit on an ambiguous scaffold boundary",
      );
    }
    const shifted = point(start);
    assert.equal(point(end) - shifted, length);
    assert.deepEqual(actual.subarray(shifted, shifted + length), source.subarray(start, end));
    return [shifted, length];
  };
  const expected = structuredClone(before);
  expected.text = after.text;
  assert.ok(Array.isArray(before.mappings));
  expected.mappings = (before.mappings as number[][]).map((tuple) => {
    assert.equal(tuple.length, 6);
    assert.ok(tuple.every(Number.isSafeInteger));
    assert.ok(tuple[2] >= 0 && tuple[3] >= 0);
    const [start, length] = range(tuple[0], tuple[1]);
    return [start, length, ...tuple.slice(2)];
  });
  if (before.semanticLinks) {
    expected.semanticLinks = (before.semanticLinks as Record<string, number>[]).map((link) => {
      const [sourceStart, sourceLength] = range(link.sourceStart, link.sourceLength);
      const [targetStart, targetLength] = range(link.targetStart, link.targetLength);
      return { ...link, sourceStart, sourceLength, targetStart, targetLength };
    });
  }
  if (before.diagnosticDirectives) {
    const directives = before.diagnosticDirectives as { directives: number[][] };
    expected.diagnosticDirectives = {
      ...directives,
      directives: directives.directives.map((tuple) => {
        assert.equal(tuple.length, 6);
        assert.ok(tuple.every(Number.isSafeInteger));
        assert.ok(tuple[0] >= 0 && tuple[1] >= 0);
        const [start, length] = range(tuple[2], tuple[3] - tuple[2]);
        return [tuple[0], tuple[1], start, start + length, ...tuple.slice(4)];
      }),
    };
  }
  assert.deepEqual(
    after,
    expected,
    "all authored spans, roles, order and other metadata stay exact",
  );
}
