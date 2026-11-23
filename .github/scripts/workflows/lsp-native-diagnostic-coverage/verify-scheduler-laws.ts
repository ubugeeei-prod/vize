import assert from "node:assert/strict";
import { textFile } from "../shared/files.ts";
const match = /test result: ok[.] ([0-9]+) passed; 0 failed; 0 ignored/u.exec(
  textFile(process.argv[2]!),
);
assert.ok(
  match && Number(match[1]) > 0,
  "nonzero scheduler laws with zero ignored tests are required",
);
