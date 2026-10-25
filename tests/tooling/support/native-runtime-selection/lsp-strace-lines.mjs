import assert from "node:assert/strict";
import fs from "node:fs";

function text(parts) {
  const bytes = parts.length === 1 ? parts[0] : Buffer.concat(parts);
  const value = bytes.toString("utf8");
  assert.deepEqual(Buffer.from(value), bytes, "lossless UTF8 in each complete raw trace line");
  return value;
}

/** Preserve every raw line without creating one V8 string for the whole file. */
export function* readStraceLines(file) {
  const descriptor = fs.openSync(file, "r");
  try {
    const chunk = Buffer.allocUnsafe(64 * 1024);
    let parts = [];
    for (;;) {
      const count = fs.readSync(descriptor, chunk, 0, chunk.length, null);
      if (count === 0) break;
      // Unfinished lines own their bytes across subsequent reads of chunk.
      const bytes = Buffer.from(chunk.subarray(0, count));
      let start = 0;
      for (let end = bytes.indexOf(10); end >= 0; end = bytes.indexOf(10, start)) {
        parts.push(bytes.subarray(start, end));
        yield text(parts);
        parts = [];
        start = end + 1;
      }
      if (start < count) parts.push(bytes.subarray(start));
    }
    if (parts.length !== 0) yield text(parts);
  } finally {
    fs.closeSync(descriptor);
  }
}
