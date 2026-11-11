/**
 * Invert the fallthrough source gate, the key-set `$AttrValue` and the shared
 * emit gate (producers, helper assets, SDK entries and the compiled preamble)
 * before older emitter custody.
 */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export const sharedUsageGatesRecord = "tests/_fixtures/tooling/shared-usage-gates.json";
const recordSha256 = "a4ec4599ec136412e1a3995a314eeb4341614d75f3b15803db9209184acf6568";

export const previousAttrValueKeySet = `type $AttrValue<R, K, C, V, P = R & (C extends { __vizeEmitProps?: infer E } ? 0 extends (1 & E) ? {} : NonNullable<E> : {}) & import("vue").VNodeProps & import("vue").AllowedComponentProps & import("vue").ComponentCustomProps, __V = P extends unknown ? (K extends keyof P ? P[K] : never) : never> = [__V] extends [never] ? (C extends { readonly __vizeComponentMarker: true; readonly __vizeHasFallthroughProps: true; readonly __vizeFallthroughProps?: infer A } ? 0 extends (1 & A) ? {} : NonNullable<A> : {}) extends infer F ? K extends keyof F ? {} extends Pick<F, K> ? V : unknown : V : V : unknown;\n`;
export const currentAttrValueKeySet = `type $AttrValue<R, K, C, V, P = never> = K extends ([P] extends [never] ? (R extends unknown ? keyof R : never) | keyof (C extends { __vizeEmitProps?: infer E } ? 0 extends (1 & E) ? {} : NonNullable<E> : {}) | keyof import("vue").VNodeProps | keyof import("vue").AllowedComponentProps | keyof import("vue").ComponentCustomProps : keyof P) ? unknown : (C extends { readonly __vizeComponentMarker: true; readonly __vizeHasFallthroughProps: true; readonly __vizeFallthroughProps?: infer A } ? 0 extends (1 & A) ? {} : NonNullable<A> : {}) extends infer F ? K extends keyof F ? {} extends Pick<F, K> ? V : unknown : V : V;\n`;

type Identity = { bytes: number; sha256: string };
type Entry = {
  path: string;
  before: Identity;
  after: Identity;
  hunks: Array<{ before: string; after: string }>;
};
interface Transition {
  schema: string;
  credit: { compiled: boolean; native: boolean; performance: boolean };
  files: Entry[];
  compiledPreamble: Entry;
  compiledStrictPreamble: Entry;
}

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

let record: Transition | undefined;
function load(): Transition {
  if (record) return record;
  const raw = readFileSync(
    new URL(`../../../../${sharedUsageGatesRecord}`, import.meta.url),
    "utf8",
  );
  assert.equal(identity(raw).sha256, recordSha256, "immutable shared usage gates transition");
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.sharedUsageGatesSource.v1");
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/** The pre-gate bytes of a recorded producer's current image; older images pass through. */
export function beforeSharedUsageGates(path: string, current: string): string {
  const file = load().files.find((entry) => entry.path === path);
  if (!file) return current;
  const now = identity(current);
  if (now.bytes !== file.after.bytes || now.sha256 !== file.after.sha256) return current;
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length, 2, `shared usage gates hunk: ${path}`);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before, `shared usage gates predecessor: ${path}`);
  return current;
}

/** Advance a composed shared preamble across the fallthrough source gate, the key-set attr value and the shared emit gate. */
export function applySharedUsageGatesIn(previous: string): string {
  let current = previous;
  for (const hunk of load().compiledPreamble.hunks) {
    assert.equal(current.split(hunk.before).length, 2, "shared usage gates predecessor line");
    current = current.replace(hunk.before, () => hunk.after);
  }
  return current;
}

/** Inverse of {@link applySharedUsageGatesIn}. */
export function beforeSharedUsageGatesIn(current: string): string {
  let previous = current;
  for (const hunk of load().compiledPreamble.hunks) {
    assert.equal(previous.split(hunk.after).length, 2, "shared usage gates current lines");
    previous = previous.replace(hunk.after, () => hunk.before);
  }
  return previous;
}

/**
 * Certify the generated component-contract field introduced by the raw
 * fallthrough-source gate. Every prior marker gains exactly one bounded field;
 * packets that already contain the field, or do not use the marker, are inert.
 */
export function certifyFallthroughSourceFields(
  before: Buffer,
  after: Buffer,
): Array<{ start: number; end: number; replacement: Buffer; at: number; label: string }> {
  const marker = " readonly __vizeHasFallthroughProps: true;";
  const field =
    / readonly __vizeFallthroughSource\?: [^;\r\n]+;(?= readonly __vizeHasFallthroughProps: true;)/gu;
  const beforeText = before.toString();
  const afterText = after.toString();
  const beforeFields = [...beforeText.matchAll(field)];
  const afterFields = [...afterText.matchAll(field)];
  if (beforeFields.length === afterFields.length) return [];
  assert.equal(beforeFields.length, 0, "fallthrough-source predecessor fields");
  const markerOffsets = [...beforeText.matchAll(new RegExp(marker, "gu"))]
    .filter((match) => {
      assert.ok(match.index != null);
      const lineStart = beforeText.lastIndexOf("\n", match.index) + 1;
      const prefix = beforeText.slice(lineStart, match.index);
      return (
        prefix.includes(" readonly __vizeComponentMarker: true;") &&
        prefix.includes(" readonly __vizeRawProps?: ")
      );
    })
    .map((match) => Buffer.byteLength(beforeText.slice(0, match.index)));
  assert.equal(
    afterFields.length,
    markerOffsets.length,
    "each fallthrough marker must gain one raw source field",
  );
  return afterFields.map((match, index) => {
    assert.ok(match.index != null && match[0].length > 0, "bounded fallthrough-source field");
    const replacement = Buffer.from(match[0]);
    const start = markerOffsets[index];
    return {
      start,
      end: start,
      replacement,
      at: Buffer.byteLength(afterText.slice(0, match.index)),
      label: "raw fallthrough source field",
    };
  });
}
