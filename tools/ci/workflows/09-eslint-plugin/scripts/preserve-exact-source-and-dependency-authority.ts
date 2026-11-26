import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
const sha = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
assert.equal(sha, process.env.SOURCE_SHA);
const files = [
  "Cargo.lock",
  "pnpm-lock.yaml",
  "npm/linter/eslint/package.json",
  "npm/linter/eslint/src/index.ts",
  "npm/linter/eslint/test/consumer.test.mjs",
  "npm/linter/eslint/test/packed-consumer.mjs",
  "npm/linter/eslint/test/legacy-consumer.ts",
  ".github/workflows/09-eslint-plugin.yml",
  "npm/linter/oxlint/src/eslint.ts",
  "npm/linter/oxlint/src/eslint-source.ts",
  "npm/linter/oxlint/src/native.ts",
];
const inputs = Object.fromEntries(
  files.map((file) => {
    const bytes = readFileSync(file);
    return [
      file,
      { sha256: createHash("sha256").update(bytes).digest("hex"), bytes: bytes.length },
    ];
  }),
);
writeFileSync(
  `${process.env.VIZE_ESLINT_EVIDENCE}/source.json`,
  JSON.stringify({ sha, inputs }, null, 2) + "\n",
);
