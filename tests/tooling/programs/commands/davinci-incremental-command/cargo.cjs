#!/usr/bin/env node
const seeded = process.argv.includes("seeded-stale-cache");
process.stdout.write(
  [
    "files=3",
    "steps_applied=5",
    "comparisons=7",
    "blocks_compared=11",
    `mismatches=${seeded ? 2 : 0}`,
    "",
  ].join("\n"),
);
process.exitCode = seeded ? 1 : 0;
