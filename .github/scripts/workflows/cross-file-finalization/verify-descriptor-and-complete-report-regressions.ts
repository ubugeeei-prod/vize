import assert from "node:assert/strict";
import fs from "node:fs";
const raw = fs.readFileSync("target/vize-tests/cross-file-sfc-effects/rust.stdout", "utf8");
assert.match(raw, /test result: ok\. 5 passed; 0 failed; 0 ignored;/u);
