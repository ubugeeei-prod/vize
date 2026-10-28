/** Exact inverse for the single unknown-host token and its generated geometry. */
import assert from "node:assert/strict";
import { isRecord } from "./canon-instance-directives-model.ts";
type ObjectValue = Record<string, unknown>;
function object(value: unknown): ObjectValue {
  assert.ok(isRecord(value));
  return value;
}
function array(value: unknown): unknown[] {
  assert.ok(Array.isArray(value));
  return value;
}
function numeric(value: unknown): number {
  assert.ok(typeof value === "number" && Number.isSafeInteger(value));
  return value;
}
function text(value: unknown): string {
  assert.ok(typeof value === "string");
  return value;
}
export function invertUnknownSlotText(generated: string): { original: string; editStart: number } {
  const needle = "? __S : any : unknown;",
    at = generated.indexOf(needle);
  assert.ok(at >= 0 && generated.indexOf(needle, at + 1) < 0);
  const editStart = Buffer.byteLength(generated.slice(0, at) + "? __S : any : ");
  const original = generated
    .replace(needle, "? __S : any : any;")
    .replace(
      /\/\/ @vize-map: (\d+):(\d+) -> (\d+):(\d+)/gu,
      (_whole: string, a: string, b: string, c: string, d: string) => {
        assert.ok(+a > editStart && +b > editStart);
        return "// @vize-map: " + (+a - 4) + ":" + (+b - 4) + " -> " + c + ":" + d;
      },
    );
  return { original, editStart };
}
export function assertUnknownSlotPacket(beforeValue: unknown, afterValue: unknown) {
  const before = object(beforeValue),
    after = object(afterValue);
  const inverse = invertUnknownSlotText(text(after.text));
  assert.equal(inverse.original, text(before.text));
  const position = (value: unknown) => {
    const at = numeric(value);
    assert.ok(
      at <= inverse.editStart || at >= inverse.editStart + 7,
      "authored geometry cannot end inside the synthetic changed type",
    );
    return at <= inverse.editStart ? at : at - 4;
  };
  const range = (startValue: unknown, lengthValue: unknown) => {
    const start = numeric(startValue),
      end = start + numeric(lengthValue);
    return [position(start), position(end) - position(start)];
  };
  const mappings = array(after.mappings).map((value) => {
    const tuple = array(value);
    assert.equal(tuple.length, 6);
    const [start, length] = range(tuple[0], tuple[1]);
    return [start, length, ...tuple.slice(2)];
  });
  const semanticLinks = array(after.semanticLinks).map((value) => {
    const link = object(value);
    const [sourceStart, sourceLength] = range(link.sourceStart, link.sourceLength);
    const [targetStart, targetLength] = range(link.targetStart, link.targetLength);
    return { ...link, sourceStart, sourceLength, targetStart, targetLength };
  });
  assert.deepEqual({ ...after, text: inverse.original, mappings, semanticLinks }, before);
  return { editStart: inverse.editStart, bytesAdded: 4, fullPacketInverseEqual: true };
}
