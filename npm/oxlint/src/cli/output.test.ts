import assert from "node:assert/strict";
import test from "node:test";

import { rewriteReportedPaths } from "./output.ts";

void test("rewriteReportedPaths rewrites both absolute and relative temporary filenames", () => {
  const replacements = new Map<string, string>([
    ["/repo/node_modules/.vize/oxlint-plugin-vize/100-abcd/0-Example.vue", "/repo/src/Example.vue"],
    ["node_modules/.vize/oxlint-plugin-vize/100-abcd/0-Example.vue", "/repo/src/Example.vue"],
  ]);

  assert.equal(
    rewriteReportedPaths(
      [
        "node_modules/.vize/oxlint-plugin-vize/100-abcd/0-Example.vue",
        "/repo/node_modules/.vize/oxlint-plugin-vize/100-abcd/0-Example.vue",
      ].join("\n"),
      replacements,
    ),
    ["/repo/src/Example.vue", "/repo/src/Example.vue"].join("\n"),
  );
});

void test("rewriteReportedPaths rewrites Windows paths embedded in JSON string values", () => {
  const tempFilename = String.raw`C:\repo\node_modules\.vize\oxlint-plugin-vize\100-abcd\0-Example.vue`;
  const originalFilename = String.raw`C:\repo\src\Example.vue`;
  const replacements = new Map<string, string>([[tempFilename, originalFilename]]);
  const output = JSON.stringify({
    filename: tempFilename,
    message: `reported from ${tempFilename}`,
  });

  const rewritten = rewriteReportedPaths(output, replacements);

  assert.deepEqual(JSON.parse(rewritten), {
    filename: originalFilename,
    message: `reported from ${originalFilename}`,
  });
  assert.doesNotMatch(rewritten, /node_modules/u);
  assert.doesNotMatch(rewritten, /\.vize/u);
});

void test("rewriteReportedPaths rewrites quoted POSIX paths embedded in JSON string values", () => {
  const tempFilename = '/repo/node_modules/.vize/oxlint-plugin-vize/100-abcd/0-"Example".vue';
  const originalFilename = '/repo/src/"Example".vue';
  const replacements = new Map<string, string>([[tempFilename, originalFilename]]);
  const output = JSON.stringify({
    filename: tempFilename,
  });

  const rewritten = rewriteReportedPaths(output, replacements);

  assert.deepEqual(JSON.parse(rewritten), {
    filename: originalFilename,
  });
  assert.doesNotMatch(rewritten, /node_modules/u);
  assert.doesNotMatch(rewritten, /\.vize/u);
});
