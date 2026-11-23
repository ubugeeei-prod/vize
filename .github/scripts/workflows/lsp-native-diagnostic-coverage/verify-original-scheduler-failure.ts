import assert from "node:assert/strict";
import { textFile } from "../shared/files.ts";
const text = textFile(process.argv[2]!);
for (const marker of [
  /running 1 test/u,
  /test result: FAILED[.] 0 passed; 1 failed; 0 ignored/u,
  /accepted distinct URIs cannot be evicted/u,
  /left:\s*64/u,
  /right:\s*65/u,
])
  assert.match(text, marker);
console.log("Original source conservation: exact named failure 64 != 65 verified");
