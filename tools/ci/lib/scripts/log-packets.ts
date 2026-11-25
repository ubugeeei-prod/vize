import assert from "node:assert/strict";
import { textFile, writeJson } from "./files.ts";

const [log, marker, expected, output] = process.argv.slice(2);
assert.ok(
  log && marker && expected && output,
  "expected a log, marker, row count, and output file",
);
const rows = textFile(log)
  .split(/\r?\n/u)
  .flatMap((line) => {
    const start = line.indexOf(marker);
    return start === -1 ? [] : [JSON.parse(line.slice(start + marker.length)) as unknown];
  });
assert.equal(rows.length, Number(expected), marker);
writeJson(output, rows.length === 1 ? rows[0] : rows);
