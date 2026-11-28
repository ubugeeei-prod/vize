import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  leaked,
  parseArguments,
  parseTimes,
  percentile,
  type RunReport,
} from "../../tools/ci/lib/scripts/temp-leak-gate.ts";

const gate = fileURLToPath(
  new URL("../../tools/ci/lib/scripts/temp-leak-gate.ts", import.meta.url),
);

function lane(script: string, ...options: string[]) {
  const runnerTemp = fs.mkdtempSync(path.join(os.tmpdir(), "vize-leak-gate-test-"));
  try {
    const result = spawnSync(
      process.execPath,
      [gate, "--label", "lane", ...options, "--", process.execPath, "-e", script],
      {
        encoding: "utf8",
        env: { ...process.env, RUNNER_TEMP: runnerTemp, GITHUB_STEP_SUMMARY: "" },
      },
    );
    const reports = result.stdout
      .split("\n")
      .filter((line) => line.startsWith("vize-temp-leak "))
      .map((line) => JSON.parse(line.slice("vize-temp-leak ".length)) as RunReport);
    // The private directory is gone after every run.
    assert.deepEqual(fs.readdirSync(runnerTemp), []);
    return { status: result.status, stderr: result.stderr, reports };
  } finally {
    fs.rmSync(runnerTemp, { recursive: true, force: true });
  }
}

const write = (entries: Record<string, string>) =>
  `const fs = require("node:fs"), os = require("node:os"), path = require("node:path");
for (const [name, body] of Object.entries(${JSON.stringify(entries)})) {
  const file = path.join(os.tmpdir(), name);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  if (body !== "/") fs.writeFileSync(file, body);
}`;

test("a lane that leaves Vize storage in its temp directory fails with the sizes", () => {
  const result = lane(write({ "vize-node-abc/preprocess.mjs": "12345", "other/x": "1" }));
  assert.equal(result.status, 1);
  assert.equal(
    result.stderr,
    "lane: left vize-node-abc in the temp directory (1 files, 5 bytes)\n",
  );
  const [report] = result.reports;
  assert.deepEqual(
    { files: report.files, bytes: report.bytes, entries: report.entries, leaked: report.leaked },
    {
      files: 2,
      bytes: 6,
      entries: [
        { name: "other", files: 1, bytes: 1 },
        { name: "vize-node-abc", files: 1, bytes: 5 },
      ],
      leaked: ["vize-node-abc"],
    },
  );
  assert.equal(typeof report.wallMs, "number");
  if (process.platform !== "win32") assert.equal(typeof report.sysMs, "number");
});

test("empty retained roots and foreign entries pass; repeats report percentiles", () => {
  const result = lane(
    write({ "vize-canon/editor/sessions/": "/", "npm-cache/x": "1" }),
    "--repeat",
    "3",
  );
  assert.equal(result.status, 0);
  assert.equal(result.stderr, "");
  assert.deepEqual(
    result.reports.map((report) => [report.run, report.leaked]),
    [
      [1, []],
      [2, []],
      [3, []],
    ],
  );
});

test("the lane's own failure status wins over a leak", () => {
  const result = lane(`${write({ ".vize-vp-1.json": "{}" })}\nprocess.exit(7);`);
  assert.equal(result.status, 7);
  assert.deepEqual(result.reports[0].leaked, [".vize-vp-1.json"]);
});

test("ownership, CPU times and percentiles are parsed exactly", () => {
  const entry = (name: string, files: number) => ({ name, files, bytes: files });
  assert.deepEqual(
    leaked([
      entry("vize-canon", 0),
      entry("vize-canon-501", 2),
      entry("vize-oxlint-501", 0),
      entry("oxlint-vize-ab12", 0),
      entry(".vize-write-1.tmp", 1),
      entry("node-compile-cache", 3),
    ]),
    ["vize-canon-501", "oxlint-vize-ab12", ".vize-write-1.tmp"],
  );
  assert.deepEqual(parseTimes("0m0.010s 0m0.002s\n1m2.500s 0m0.250s\n"), {
    userMs: 62500,
    sysMs: 250,
  });
  assert.equal(parseTimes(""), null);
  assert.equal(percentile([5, 1, 4, 2, 3], 0.5), 3);
  assert.equal(percentile([5, 1, 4, 2, 3], 0.95), 5);
  assert.deepEqual(parseArguments(["--label", "x", "--", "a", "--b"]), {
    label: "x",
    repeat: 1,
    command: ["a", "--b"],
  });
  assert.throws(() => parseArguments(["--label", "x"]), /usage/u);
});
