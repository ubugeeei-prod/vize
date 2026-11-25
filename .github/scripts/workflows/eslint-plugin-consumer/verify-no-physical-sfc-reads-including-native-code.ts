import { requiredEnv } from "../../../../tools/ci/lib/scripts/files.ts";
import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
const lines = readFileSync(
  `${process.env.VIZE_ESLINT_EVIDENCE}/source-opens.raw.log`,
  "utf8",
).split("\n");
const opens = lines.filter((line) => line.includes(requiredEnv("VIZE_ESLINT_WATCHED_SOURCE")));
const reads = opens.filter((line) => /O_RDONLY|O_RDWR/.test(line));
assert.ok(
  opens.some((line) => /O_WRONLY/.test(line)),
  "physical sentinel source write must be traced",
);
assert.deepEqual(reads, [], "unsaved lintText must not read the physical SFC");
writeFileSync(
  `${process.env.VIZE_ESLINT_EVIDENCE}/source-reads.json`,
  JSON.stringify(
    { source: process.env.VIZE_ESLINT_WATCHED_SOURCE, opens, sourceReadCount: reads.length },
    null,
    2,
  ) + "\n",
);
