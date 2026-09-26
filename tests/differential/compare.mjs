import assert from "node:assert/strict";

// Offsets are byte offsets; display never slices UTF-8 strings at arbitrary bytes.
export function compareBytes(expected, actual) {
  assert(Buffer.isBuffer(expected) && Buffer.isBuffer(actual), "comparison requires raw buffers");
  let offset = 0;
  const shared = Math.min(expected.length, actual.length);
  while (offset < shared && expected[offset] === actual[offset]) offset += 1;
  return offset === shared && expected.length === actual.length
    ? { state: "equal" }
    : {
        state: "different",
        byteOffset: offset,
        expectedLength: expected.length,
        actualLength: actual.length,
        expectedHex: expected.subarray(offset, offset + 8).toString("hex"),
        actualHex: actual.subarray(offset, offset + 8).toString("hex"),
      };
}
