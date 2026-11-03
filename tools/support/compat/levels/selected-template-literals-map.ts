// Check the actual recorded ranges and serialized segment starts.
import assert from "node:assert/strict";

export interface NativeLink {
  readonly authored: readonly number[];
  readonly generated: readonly number[];
  readonly name: unknown;
  readonly segment: boolean;
}
export interface NativeOutput {
  readonly code: string;
  readonly source: string;
  readonly map: unknown;
  readonly links: unknown;
}
const isRecord = (value: unknown): value is Readonly<Record<string, unknown>> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

function span(bytes: Buffer, value: unknown): asserts value is readonly [number, number] {
  assert(Array.isArray(value) && value.length === 2, "a complete byte range is required");
  for (const at of value as unknown[]) {
    assert(
      typeof at === "number" &&
        Number.isInteger(at) &&
        at >= 0 &&
        at <= 0xffffffff &&
        at <= bytes.length,
    );
    assert(at === bytes.length || ((bytes[at] ?? 0) & 0xc0) !== 0x80, "UTF-8 boundary");
  }
  const [start, end]: readonly unknown[] = value;
  assert(typeof start === "number" && typeof end === "number" && start <= end, "ordered range");
}
function position(bytes: Buffer, offset: number): [number, number] {
  // Match the existing native map's JS line-terminator convention, not Vue normalization.
  const text = bytes.toString("utf8");
  const target = bytes.subarray(0, offset).toString("utf8").length;
  let line = 0;
  let start = 0;
  for (let at = 0; at < target;) {
    const width =
      text[at] === "\r" && text[at + 1] === "\n"
        ? 2
        : /[\r\n\u2028\u2029]/u.test(text[at] ?? "")
          ? 1
          : 0;
    if (width && at + width <= target) {
      at += width;
      line++;
      start = at;
    } else {
      at++;
    }
  }
  return [line, target - start];
}
function mappings(text: string): [number, number, number, number][] {
  const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  let source = 0;
  let line = 0;
  let column = 0;
  const points: [number, number, number, number][] = [];
  for (const [generatedLine, row] of text.split(";").entries()) {
    let generatedColumn = 0;
    if (!row) continue;
    for (const encoded of row.split(",")) {
      assert(encoded.length, "no empty mapping segment");
      const fields: number[] = [];
      let value = 0;
      let factor = 1;
      for (const digit of encoded) {
        const number = alphabet.indexOf(digit);
        assert(number >= 0, "VLQ digit");
        value += (number & 31) * factor;
        assert(Number.isSafeInteger(value));
        if (number & 32) {
          factor *= 32;
          assert(Number.isSafeInteger(factor));
        } else {
          fields.push(value % 2 ? -Math.floor(value / 2) : value / 2);
          value = 0;
          factor = 1;
        }
      }
      assert.equal(factor, 1, "complete VLQ");
      assert.equal(fields.length, 4, "anonymous single-source segment");
      const [generated = 0, sourceDelta = 0, lineDelta = 0, columnDelta = 0] = fields;
      generatedColumn += generated;
      source += sourceDelta;
      line += lineDelta;
      column += columnDelta;
      assert(source === 0 && generatedColumn >= 0 && line >= 0 && column >= 0);
      points.push([generatedLine, generatedColumn, line, column]);
    }
  }
  return points;
}
export function checkNativeMap(
  native: NativeOutput,
  fixture: { readonly id: string },
  filename: string,
): void {
  const map = native.map;
  assert(isRecord(map));
  assert.deepEqual(Object.keys(map).sort(), [
    "file",
    "mappings",
    "names",
    "sources",
    "sourcesContent",
    "version",
  ]);
  assert.equal(map.version, 3);
  assert.equal(map.file, filename);
  assert.deepEqual(map.sources, [filename]);
  assert.deepEqual(map.sourcesContent, [native.source]);
  assert.deepEqual(map.names, []);
  assert(typeof map.mappings === "string");
  assert(map.mappings.length > 0);
  assert(Array.isArray(native.links) && native.links.length > 0, "actual recorded links required");
  const code = Buffer.from(native.code);
  const source = Buffer.from(native.source);
  for (const link of native.links as readonly unknown[]) {
    assert(isRecord(link));
    assert.deepEqual(Object.keys(link).sort(), ["authored", "generated", "name", "segment"]);
    span(code, link.generated);
    span(source, link.authored);
    assert.equal(link.name, null);
    assert.equal(typeof link.segment, "boolean");
  }
  const links = native.links as readonly NativeLink[];
  const starts = links
    .filter((link) => link.segment)
    .sort((left, right) => (left.generated[0] ?? 0) - (right.generated[0] ?? 0))
    .map((link) => [
      ...position(code, link.generated[0] ?? 0),
      ...position(source, link.authored[0] ?? 0),
    ]);
  assert.deepEqual(mappings(map.mappings), starts, fixture.id + ": actual segment starts");
}
