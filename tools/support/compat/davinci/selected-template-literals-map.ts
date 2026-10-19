// Check the actual recorded ranges and serialized segment starts.
import assert from "node:assert/strict";

function span(bytes, value) {
  assert(Array.isArray(value) && value.length === 2, "a complete byte range is required");
  for (const at of value) {
    assert(Number.isInteger(at) && at >= 0 && at <= 0xffffffff && at <= bytes.length);
    assert(at === bytes.length || (bytes[at] & 0xc0) !== 0x80, "UTF-8 boundary");
  }
  assert(value[0] <= value[1], "ordered range");
}
function position(bytes, offset) {
  // Match the existing native map's JS line-terminator convention, not Vue normalization.
  const text = bytes.toString("utf8");
  const target = bytes.subarray(0, offset).toString("utf8").length;
  let line = 0;
  let start = 0;
  for (let at = 0; at < target;) {
    const width =
      text[at] === "\r" && text[at + 1] === "\n" ? 2 : /[\r\n\u2028\u2029]/u.test(text[at]) ? 1 : 0;
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
function mappings(text) {
  const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  let source = 0;
  let line = 0;
  let column = 0;
  const points = [];
  for (const [generatedLine, row] of text.split(";").entries()) {
    let generatedColumn = 0;
    if (!row) continue;
    for (const encoded of row.split(",")) {
      assert(encoded.length, "no empty mapping segment");
      const fields = [];
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
      generatedColumn += fields[0];
      source += fields[1];
      line += fields[2];
      column += fields[3];
      assert(source === 0 && generatedColumn >= 0 && line >= 0 && column >= 0);
      points.push([generatedLine, generatedColumn, line, column]);
    }
  }
  return points;
}
export function checkNativeMap(native, fixture, filename) {
  const map = native.map;
  assert(map && typeof map === "object" && !Array.isArray(map));
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
  assert.equal(typeof map.mappings, "string");
  assert(map.mappings.length > 0);
  assert(Array.isArray(native.links) && native.links.length > 0, "actual recorded links required");
  const code = Buffer.from(native.code);
  const source = Buffer.from(native.source);
  for (const link of native.links) {
    assert.deepEqual(Object.keys(link).sort(), ["authored", "generated", "name", "segment"]);
    span(code, link.generated);
    span(source, link.authored);
    assert.equal(link.name, null);
    assert.equal(typeof link.segment, "boolean");
  }
  const starts = native.links
    .filter((link) => link.segment)
    .sort((left, right) => left.generated[0] - right.generated[0])
    .map((link) => [...position(code, link.generated[0]), ...position(source, link.authored[0])]);
  assert.deepEqual(mappings(map.mappings), starts, fixture.id + ": actual segment starts");
}
