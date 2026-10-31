/** Exact observed embedded callback scaffolding; shared references require separate evidence. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import {
  currentNativeAttributeAliases,
  currentSlotsExport,
  currentSlotsMarker,
  nativeAttributesPath,
  previousNativeAttributeAliases,
  previousSlotsExport,
  previousSlotsMarker,
} from "./constructor-projection-parity.ts";

type Edit = { start: number; end: number; replacement: Buffer };
const nativeRoot =
  "{ readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: infer __F }";
const argumentSuffix =
  ") extends infer __A extends unknown[] ? (__A extends [] ? unknown[] : __A) : unknown[] : unknown[]\n";

/** The complete producer asset, including every unchanged declaration, is finite. */
export function assertCallbackNativeAttributeSource(asset: Buffer) {
  const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
  assert.equal(digest(asset), "d6d9a045d67eb75182f5009c6f587b2a0b4b36c3ab050927c1fd9b0d863d4b55");
  const text = asset.toString();
  assert.equal(text.split(currentNativeAttributeAliases).length, 2);
  assert.equal(text.split(previousNativeAttributeAliases).length, 1);
  assert.equal(
    digest(
      Buffer.from(text.replace(currentNativeAttributeAliases, previousNativeAttributeAliases)),
    ),
    "3f1176407725275b766b28d28045ed9cf0344e7203ccf60059b70a1c31934e28",
  );
}
const nativeAttributeSource = readFileSync(
  new URL(`../../../../${nativeAttributesPath}`, import.meta.url),
);
assertCallbackNativeAttributeSource(nativeAttributeSource);
const currentNativeAttributeSource = nativeAttributeSource.toString();
const previousNativeAttributeSource = currentNativeAttributeSource.replace(
  currentNativeAttributeAliases,
  previousNativeAttributeAliases,
);

function nativeAttributeTransition(before: string, after: string): boolean {
  const counts = [before, after].map((text) => [
    text.split(previousNativeAttributeAliases).length - 1,
    text.split(currentNativeAttributeAliases).length - 1,
  ]);
  if (
    [
      [0, 0],
      [1, 0],
      [0, 1],
    ].some((same) => counts.every((count) => count[0] === same[0] && count[1] === same[1]))
  )
    return false;
  assert.deepEqual(
    counts,
    [
      [1, 0],
      [0, 1],
    ],
    "only the exact native alias migration may change",
  );
  assert.equal(
    before.split(previousNativeAttributeSource).length,
    2,
    "the previous aliases belong to the complete certified producer asset",
  );
  assert.equal(
    after.split(currentNativeAttributeSource).length,
    2,
    "the current aliases belong to the complete certified producer asset",
  );
  return true;
}
const point = (offset: number, edits: Edit[]) =>
  offset +
  edits
    .filter((edit) => edit.end <= offset)
    .reduce((delta, edit) => delta + edit.replacement.length - (edit.end - edit.start), 0);

export function callbackNativeEventGate(component: string): string {
  return `"__vizeEmitProps" extends keyof { [__K in keyof typeof ${component} as string extends __K ? never : number extends __K ? never : symbol extends __K ? never : __K]: unknown } ? `;
}

function scaffoldEdits(text: string, markerTransition: boolean, nativeTransition: boolean): Edit[] {
  const edits: Edit[] = [];
  const add = (start: number, end: number, replacement: string) => {
    edits.push({
      start: Buffer.byteLength(text.slice(0, start)),
      end: Buffer.byteLength(text.slice(0, end)),
      replacement: Buffer.from(replacement),
    });
  };
  if (nativeTransition) {
    const start = text.indexOf(previousNativeAttributeAliases);
    add(start, start + previousNativeAttributeAliases.length, currentNativeAttributeAliases);
  }
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
      /^[ \t]*\? \$IsAny<__F> extends true \? unknown\[\] : NonNullable<__F> extends infer __P \? \(/u,
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
      /^\$IsAny<__F> extends true \? any : NonNullable<__F> extends \{ .+?\?: infer __H \} \? Extract<NonNullable<__H>, \(\.\.\.args: any\[\]\) => any> extends \(\.\.\.args: any\[\]\) => infer __R \? __R : any : any : any;\n$/u,
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
    if (before === previousSlotsMarker && !markerTransition) continue;
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

/** Reuse the constructor certificate's self-described map-comment law. */
function mapCommentEdits(
  before: string,
  after: string,
  beforeMappings: number[][],
  afterMappings: number[][],
  sourceBytes: number,
  edits: Edit[],
) {
  const comments = (text: string, mappings: number[][]) =>
    [...text.matchAll(/^([ \t]*)\/\/ @vize-map: (\d+):(\d+) -> (\d+):(\d+)\r?$/gmu)]
      .map((match) => ({
        text: match[0],
        indent: match[1],
        values: match.slice(2, 6).map(Number),
        start: Buffer.byteLength(text.slice(0, match.index)),
      }))
      .filter(
        (comment) =>
          !mappings.some(
            ([start, length]) =>
              start < comment.start + Buffer.byteLength(comment.text) &&
              comment.start < start + length,
          ),
      );
  const original = comments(before, beforeMappings),
    actual = comments(after, afterMappings);
  assert.equal(actual.length, original.length, "map comments cannot be added or removed");
  for (const [index, comment] of original.entries()) {
    const [g0, g1, s0, s1] = comment.values;
    assert.ok(comment.values.every(Number.isSafeInteger));
    assert.ok(0 <= g0 && g0 <= g1 && g1 <= comment.start);
    assert.ok(0 <= s0 && s0 <= s1 && s1 <= sourceBytes);
    for (const edit of edits)
      assert.ok(
        !(g0 < edit.end && edit.start < g1),
        "a map comment cannot claim changed scaffolding",
      );
    const values = [point(g0, edits), point(g1, edits), s0, s1];
    assert.deepEqual(actual[index].values, values, "only derived map coordinates may change");
    assert.equal(actual[index].start, point(comment.start, edits));
    const replacement = `${comment.indent}// @vize-map: ${values[0]}:${values[1]} -> ${s0}:${s1}${comment.text.endsWith("\r") ? "\r" : ""}`;
    assert.equal(actual[index].text, replacement);
    edits.push({
      start: comment.start,
      end: comment.start + Buffer.byteLength(comment.text),
      replacement: Buffer.from(replacement),
    });
  }
  edits.sort((left, right) => left.start - right.start);
  for (let index = 1; index < edits.length; index++) {
    assert.ok(edits[index - 1].end <= edits[index].start);
    assert.notEqual(edits[index - 1].start, edits[index].start);
  }
}

/** UTF-8 byte ranges may move around scaffolding, but may never claim it. */
export function assertPreservedCallbackPacket(
  before: Record<string, unknown>,
  after: Record<string, unknown>,
  authoredSource?: string,
) {
  assert.equal(typeof before.text, "string");
  assert.equal(typeof after.text, "string");
  if (before.text === after.text) {
    assert.deepEqual(after, before, "all unchanged callback fields stay exact");
    return;
  }
  const original = before.text as string;
  const actualText = after.text as string;
  assert.equal(original.split(previousSlotsExport).length, 2);
  assert.equal(original.split(currentSlotsExport).length, 1);
  assert.equal(actualText.split(previousSlotsExport).length, 1);
  assert.equal(actualText.split(currentSlotsExport).length, 2);
  const markerTransition =
    original.includes(previousSlotsMarker) && !actualText.includes(previousSlotsMarker);
  if (markerTransition) {
    assert.equal(original.split(previousSlotsMarker).length, 2);
    assert.equal(original.split(currentSlotsMarker).length, 1);
    assert.equal(actualText.split(previousSlotsMarker).length, 1);
    assert.equal(actualText.split(currentSlotsMarker).length, 2);
    assert.ok(original.indexOf(previousSlotsMarker) > original.indexOf(previousSlotsExport));
  }
  const source = Buffer.from(original);
  const actual = Buffer.from(actualText);
  assert.ok(Array.isArray(before.mappings) && Array.isArray(after.mappings));
  const nativeTransition = nativeAttributeTransition(original, actualText);
  const edits = scaffoldEdits(original, markerTransition, nativeTransition);
  if (nativeTransition) {
    assert.equal(typeof authoredSource, "string", "native relocation requires its authored source");
    mapCommentEdits(
      original,
      actualText,
      before.mappings as number[][],
      after.mappings as number[][],
      Buffer.byteLength(authoredSource!),
      edits,
    );
  }
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
  const boundary = Buffer.byteLength(original.slice(0, exportAt));
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
    const shifted = point(start, edits);
    assert.equal(point(end, edits) - shifted, length);
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
    if (authoredSource !== undefined) {
      const authored = Buffer.from(authoredSource);
      assert.ok(tuple[2] + tuple[3] <= authored.length);
      const decoder = new TextDecoder("utf-8", { fatal: true });
      decoder.decode(authored.subarray(0, tuple[2]));
      decoder.decode(authored.subarray(tuple[2], tuple[2] + tuple[3]));
    }
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
