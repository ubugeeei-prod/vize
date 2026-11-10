/** Exact observed embedded callback scaffolding; shared references require separate evidence. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { isDeepStrictEqual } from "node:util";
import {
  currentNativeAttributeAliases,
  currentSlotsExport,
  currentSlotsMarker,
  nativeAttributesPath,
  previousNativeAttributeAliases,
  previousSlotsExport,
  previousSlotsMarker,
} from "./constructor-projection-parity.ts";

import { withoutProducerUnmappedIgnores } from "./producer-unmapped-ignores.ts";
import {
  currentCallbackSourceEdits,
  type CallbackSourceReader,
} from "./callback-owner-current-source.ts";
import { withInertCallbackRows, type InertCallbackBinding } from "./callback-owner-inert-packet.ts";
import { mergedListenerWitnessEdits } from "./callback-owner-listener-witness.ts";
import { withoutReplayAliasRows } from "./callback-owner-replay-aliases.ts";

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

function utf8Range(bytes: Buffer, start: number, length: number) {
  assert.ok(Number.isSafeInteger(start) && Number.isSafeInteger(length));
  assert.ok(start >= 0 && length >= 0 && start + length <= bytes.length);
  const decoder = new TextDecoder("utf-8", { fatal: true });
  decoder.decode(bytes.subarray(0, start));
  decoder.decode(bytes.subarray(start, start + length));
}

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

function propertyKeyValue(key: string): string {
  if (key.startsWith('"')) {
    const value: unknown = JSON.parse(key);
    assert.ok(typeof value === "string", "listener keys are string literals");
    return value;
  }
  return key;
}

function assertInlineListenerRelation(
  relation: string,
  listenerKey: string,
  rawListenerKey: string,
) {
  const marker = "?: (...args: infer __A) => any } ? __A : __P extends { ";
  assert.ok(relation.startsWith("__P extends { "), "the original listener relation is complete");
  const propEnd = relation.indexOf(marker, "__P extends { ".length);
  assert.ok(propEnd > 0, "the original listener relation has its callable branch");
  const propKey = relation.slice("__P extends { ".length, propEnd);
  assert.equal(propertyKeyValue(propKey), propertyKeyValue(listenerKey));

  const missing =
    rawListenerKey === "never"
      ? "unknown[]"
      : (() => {
          const rawKey = propertyKeyValue(rawListenerKey);
          const rawProperty = /^[A-Za-z_$][\w$]*$/u.test(rawKey) ? rawKey : JSON.stringify(rawKey);
          return `__P extends { ${rawProperty}?: (...args: infer __A) => any } ? __A : unknown[]`;
        })();
  const expected =
    `__P extends { ${propKey}?: (...args: infer __A) => any } ? __A : ` +
    `__P extends { ${propKey}?: infer __H } ? ` +
    `[Extract<NonNullable<__H>, (...args: any) => any>] extends [never] ? ${missing} : ` +
    `Extract<NonNullable<__H>, (...args: any) => any> extends (...args: infer __A) => any ? __A : ${missing} : ${missing}`;
  assert.equal(relation, expected, "only the complete inline listener relation may be replaced");
}

/** Certify the finite inline-to-local relation transition emitted by this change. */
function listenerRelationEdits(
  original: string,
  actual: string,
  certifiedCurrentEdits: Edit[] = [],
): Edit[] {
  const edits: Edit[] = [];
  const aliases = new Set<string>();
  const missingPattern =
    /^([ \t]*)type ([^\s=]+_listener_missing)<__P, __Raw extends string \| number \| symbol> = \[__Raw\] extends \[never\] \? unknown\[\] : __P extends \{ \[__K in __Raw\]\?: \(\.\.\.args: infer __A\) => any \} \? __A : unknown\[\];\r?$/gmu;
  for (const match of actual.matchAll(missingPattern)) {
    const [missingLine, indent, missing] = match;
    const prefix = missing.slice(0, -"_listener_missing".length);
    const args = `${prefix}_listener_args`;
    const argsLine =
      `${indent}type ${args}<__P, __K extends string | number | symbol, __Raw extends string | number | symbol = never> = ` +
      `__P extends { [__Q in __K]?: (...args: infer __A) => any } ? __A : ` +
      `__P extends { [__Q in __K]?: infer __H } ? ` +
      `[Extract<NonNullable<__H>, (...args: any) => any>] extends [never] ? ${missing}<__P, __Raw> : ` +
      `Extract<NonNullable<__H>, (...args: any) => any> extends (...args: infer __A) => any ? __A : ` +
      `${missing}<__P, __Raw> : ${missing}<__P, __Raw>;`;
    assert.equal(actual[match.index + missingLine.length], "\n");
    const newline = missingLine.endsWith("\r") ? "\r\n" : "\n";
    const argsAt = match.index + missingLine.length + 1;
    assert.equal(actual.slice(argsAt, argsAt + argsLine.length), argsLine);
    const blockEnd = argsAt + argsLine.length + newline.length;
    assert.equal(actual.slice(argsAt + argsLine.length, blockEnd), newline);

    const library = `${indent}type ${prefix}_library`;
    assert.ok(
      actual.startsWith(library, blockEnd),
      "listener relations immediately precede their owner",
    );
    const block = actual.slice(match.index, blockEnd);
    const preservedAt = original.indexOf(block);
    if (preservedAt >= 0) {
      assert.equal(
        original.indexOf(block, preservedAt + 1),
        -1,
        "the preserved listener relations are unambiguous",
      );
      continue;
    }
    if (
      certifiedCurrentEdits.some(
        (edit) => edit.start === edit.end && edit.replacement.includes(Buffer.from(block)),
      )
    )
      continue;
    const insertion = original.indexOf(library);
    assert.ok(insertion >= 0, "the original listener owner remains present");
    assert.equal(original.indexOf(library, insertion + 1), -1, "the listener owner is unambiguous");
    assert.equal(original.includes(`${indent}type ${missing}<`), false);
    edits.push({
      start: Buffer.byteLength(original.slice(0, insertion)),
      end: Buffer.byteLength(original.slice(0, insertion)),
      replacement: Buffer.from(block),
    });
    aliases.add(args);
  }

  for (const args of aliases) {
    const escapePattern = (value: string) => value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
    const stringLiteral = String.raw`"(?:[^"\\]|\\.)*"`;
    const callPattern = new RegExp(
      String.raw`^([ \t]*)type ([^\s=]+_(?:prop|fallthrough)_args) = .*${escapePattern(args)}<__P, (${stringLiteral}), (never|${stringLiteral})>.*;\r?$`,
      "gmu",
    );
    let calls = 0;
    for (const match of actual.matchAll(callPattern)) {
      const [afterLine, , name, listenerKey, rawListenerKey] = match;
      const oldPattern = new RegExp(
        String.raw`^([ \t]*)type ${escapePattern(name)} = .*;\r?$`,
        "mu",
      );
      const beforeMatch = oldPattern.exec(original);
      assert.ok(beforeMatch, `the original ${name} relation remains present`);
      assert.equal(
        original.slice(beforeMatch.index + beforeMatch[0].length).match(/^\r?\n/u)?.[0],
        "\n",
      );
      const call = `${args}<__P, ${listenerKey}, ${rawListenerKey}>`;
      const callAt = afterLine.indexOf(call);
      assert.ok(callAt >= 0);
      assert.equal(afterLine.indexOf(call, callAt + 1), -1, "one relation call per generated line");
      const prefix = afterLine.slice(0, callAt);
      const suffix = afterLine.slice(callAt + call.length);
      assert.ok(beforeMatch[0].startsWith(prefix));
      assert.ok(beforeMatch[0].endsWith(suffix));
      assertInlineListenerRelation(
        beforeMatch[0].slice(prefix.length, beforeMatch[0].length - suffix.length),
        listenerKey,
        rawListenerKey,
      );
      const start = Buffer.byteLength(original.slice(0, beforeMatch.index));
      const end = start + Buffer.byteLength(beforeMatch[0]) + 1;
      edits.push({ start, end, replacement: Buffer.from(`${afterLine}\n`) });
      calls++;
    }
    assert.ok(calls > 0, `${args} must replace an observed inline relation`);
  }
  edits.sort((left, right) => left.start - right.start);
  for (let index = 1; index < edits.length; index++) {
    assert.ok(edits[index - 1]!.end <= edits[index]!.start, "listener relation edits are disjoint");
    assert.notEqual(edits[index - 1]!.start, edits[index]!.start);
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
        "frame" in edit || !(g0 < edit.end && edit.start < g1),
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
  inertBindings?: InertCallbackBinding[],
  currentSourceRead?: CallbackSourceReader,
) {
  assert.equal(typeof before.text, "string");
  assert.equal(typeof after.text, "string");
  if (before.text === after.text) {
    if (isDeepStrictEqual(after, before)) return;
    // #12520: the only certified mapping-only change unmaps loop replay aliases.
    if (isDeepStrictEqual(after, withoutReplayAliasRows(before))) return;
    const expected =
      inertBindings && authoredSource !== undefined
        ? withInertCallbackRows(before, after, authoredSource, inertBindings)
        : before;
    assert.deepEqual(after, expected, "all unchanged callback fields stay exact");
    return;
  }
  const original = before.text as string;
  const actualText = after.text as string;
  const currentSource = currentCallbackSourceEdits(
    original,
    actualText,
    authoredSource,
    currentSourceRead,
  );
  const currentEdits = currentSource.edits;
  const relationEdits = listenerRelationEdits(original, actualText, currentEdits);
  const witnessEdits = mergedListenerWitnessEdits(original);
  const slotsTransition = original.includes(previousSlotsExport);
  if (slotsTransition) {
    assert.equal(original.split(previousSlotsExport).length, 2);
    assert.equal(original.split(currentSlotsExport).length, 1);
    assert.equal(actualText.split(previousSlotsExport).length, 1);
    assert.equal(actualText.split(currentSlotsExport).length, 2);
  } else {
    assert.ok(
      currentEdits.length > 0 || relationEdits.length > 0 || witnessEdits.length > 0,
      "same current Slots requires certified emitter edits",
    );
    for (const text of [original, actualText]) {
      assert.equal(text.split(previousSlotsExport).length, 1);
      assert.equal(text.split(currentSlotsExport).length, 2);
    }
  }
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
  const authored = authoredSource === undefined ? null : Buffer.from(authoredSource);
  assert.ok(Array.isArray(before.mappings) && Array.isArray(after.mappings));
  const nativeTransition = nativeAttributeTransition(original, actualText);
  const edits = [
    ...scaffoldEdits(original, markerTransition, nativeTransition),
    ...currentEdits,
    ...relationEdits,
    ...witnessEdits,
  ].sort((left, right) => left.start - right.start);
  for (let index = 1; index < edits.length; index++) {
    assert.ok(edits[index - 1]!.end <= edits[index]!.start, "current scaffold edits are disjoint");
    assert.notEqual(edits[index - 1]!.start, edits[index]!.start);
  }
  if (
    nativeTransition ||
    currentEdits.length > 0 ||
    relationEdits.length > 0 ||
    witnessEdits.length > 0
  ) {
    assert.equal(
      typeof authoredSource,
      "string",
      "scaffold relocation requires its authored source",
    );
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
  const exportAt = original.indexOf(slotsTransition ? previousSlotsExport : currentSlotsExport);
  assert.ok(exportAt >= 0, "the public Slots export remains present");
  const boundary = Buffer.byteLength(original.slice(0, exportAt));
  const exportLineEnd = original.indexOf("\n", exportAt);
  assert.ok(exportLineEnd >= 0, "the public Slots export retains its line boundary");
  const afterExport = Buffer.byteLength(original.slice(0, exportLineEnd + 1));
  const range = (start: number, length: number, unmappedProducerTail = false): [number, number] => {
    assert.ok(Number.isSafeInteger(start) && start >= 0);
    assert.ok(Number.isSafeInteger(length) && length >= 0);
    const end = start + length;
    assert.ok(
      end <= boundary ||
        (unmappedProducerTail && length > 0 && start >= boundary) ||
        (relationEdits.length > 0 && start >= afterExport),
      "authored projection cannot claim the public Slots export",
    );
    for (const edit of edits) {
      assert.ok(
        edit.start === edit.end
          ? !(start < edit.start && edit.start < end)
          : !(start < edit.end && edit.start < end),
        "authored projection cannot overlap scaffold edits",
      );
      assert.ok(
        "frame" in edit || ![edit.start, edit.end].some((at) => at === start || at === end),
        "authored projection cannot sit on an ambiguous scaffold boundary",
      );
    }
    const shifted = point(start, edits);
    assert.equal(point(end, edits) - shifted, length);
    utf8Range(source, start, length);
    utf8Range(actual, shifted, length);
    assert.deepEqual(actual.subarray(shifted, shifted + length), source.subarray(start, end));
    return [shifted, length];
  };
  if (currentEdits.length > 0) {
    for (const packet of [before, after])
      withoutProducerUnmappedIgnores(
        {
          text: packet.text as string,
          mappings: packet.mappings as number[][],
          diagnosticDirectives: packet.diagnosticDirectives,
        },
        { requireComplete: true },
      );
  }
  const gapRange = (start: number, end: number): [number, number] => {
    assert.ok(
      Number.isSafeInteger(start) &&
        Number.isSafeInteger(end) &&
        0 <= start &&
        start <= end &&
        end <= source.length,
    );
    for (const edit of edits)
      if (start < edit.end && edit.start < end)
        assert.ok(
          start <= edit.start && edit.end <= end,
          "a producer gap may contain only whole certified edits",
        );
    const shifted = point(start, edits),
      finish = point(end, edits);
    assert.ok(shifted <= finish && finish <= actual.length);
    return [shifted, finish - shifted];
  };
  const comparisonBefore =
    currentEdits.length > 0
      ? withoutProducerUnmappedIgnores(
          before as typeof before & {
            text: string;
            mappings: number[][];
            diagnosticDirectives?: unknown;
          },
          { requireComplete: true },
        )
      : before;
  const expected = structuredClone(comparisonBefore) as Record<string, unknown> & {
    text: string;
    mappings: number[][];
    semanticLinks?: Record<string, number>[];
    diagnosticDirectives?: { directives: number[][] };
  };
  expected.text = after.text as string;
  assert.ok(Array.isArray(comparisonBefore.mappings));
  expected.mappings = (comparisonBefore.mappings as number[][]).map((tuple) => {
    assert.equal(tuple.length, 6);
    assert.ok(tuple.every(Number.isSafeInteger));
    assert.ok(tuple[2] >= 0 && tuple[3] >= 0);
    if (authored) utf8Range(authored, tuple[2], tuple[3]);
    const unmappedProducerTail = tuple[2] === 0 && tuple[3] === 0;
    const [start, length] = range(tuple[0], tuple[1], unmappedProducerTail);
    return [start, length, ...tuple.slice(2)];
  });
  expected.mappings.push(...currentSource.addedMappings);
  if (comparisonBefore.semanticLinks) {
    expected.semanticLinks = (comparisonBefore.semanticLinks as Record<string, number>[]).map(
      (link) => {
        const [sourceStart, sourceLength] = range(link.sourceStart, link.sourceLength);
        const [targetStart, targetLength] = range(link.targetStart, link.targetLength);
        return { ...link, sourceStart, sourceLength, targetStart, targetLength };
      },
    );
  }
  if (comparisonBefore.diagnosticDirectives) {
    const directives = comparisonBefore.diagnosticDirectives as { directives: number[][] };
    expected.diagnosticDirectives = {
      ...directives,
      directives: directives.directives.map((tuple) => {
        assert.equal(tuple.length, 6);
        assert.ok(tuple.every(Number.isSafeInteger));
        assert.ok(tuple[0] >= 0 && tuple[1] >= 0);
        if (authored) utf8Range(authored, tuple[0], tuple[1]);
        const syntheticGap = currentEdits.length > 0 && tuple[0] === 0 && tuple[1] === 0;
        if (syntheticGap)
          assert.deepEqual(
            tuple.slice(4),
            [0, 0],
            "only producer-owned IGNORE gaps may contain scaffold edits",
          );
        const [start, length] = syntheticGap
          ? gapRange(tuple[2], tuple[3])
          : range(tuple[2], tuple[3] - tuple[2]);
        return [tuple[0], tuple[1], start, start + length, ...tuple.slice(4)];
      }),
    };
  }
  const comparisonAfter =
    currentEdits.length > 0
      ? withoutProducerUnmappedIgnores(
          after as typeof after & {
            text: string;
            mappings: number[][];
            diagnosticDirectives?: unknown;
          },
          { requireComplete: true },
        )
      : after;
  assert.deepEqual(
    comparisonAfter,
    expected,
    "all authored spans, roles, order and other metadata stay exact",
  );
}
