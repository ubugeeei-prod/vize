import assert from "node:assert/strict";
import fs from "node:fs";
assert.match(
  fs.readFileSync("target/vize-tests/cross-file-source-updates/rust.stdout", "utf8"),
  /test result: ok\. 5 passed; 0 failed; 0 ignored;/u,
);
