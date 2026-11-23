import assert from "node:assert/strict";
import fs from "node:fs";
assert.equal(
  JSON.parse(fs.readFileSync("target/ci/vize.differential-build.json", "utf8")).sourceRevision,
  process.env.SOURCE_SHA,
);
