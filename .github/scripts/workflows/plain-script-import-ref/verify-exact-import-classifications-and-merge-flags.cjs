const assert = require("node:assert/strict");
const fs = require("node:fs");
assert(
  fs
    .readFileSync(process.env.VIZE10903_PROOF + "/classification-test.log", "utf8")
    .includes("1 passed; 0 failed"),
);
