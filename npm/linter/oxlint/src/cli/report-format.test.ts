import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  renderConvertedReport,
  requestedConvertedFormat,
  withJsonReport,
} from "./report-format.ts";

void test("the last output option selects conversion and only report values become JSON", () => {
  assert.equal(requestedConvertedFormat(["-f", "github", "src"]), "github");
  assert.equal(requestedConvertedFormat(["--format=gitlab", "src"]), "gitlab");
  assert.equal(requestedConvertedFormat(["-f", "github", "-f", "unix"]), undefined);
  assert.equal(requestedConvertedFormat(["--", "--format=github"]), undefined);
  assert.deepEqual(withJsonReport(["-f", "github", "--format=gitlab", "--", "-f"]), [
    "-f",
    "json",
    "--format=json",
    "--",
    "-f",
  ]);
});

void test("GitHub output uses authored ranges and the native summary contract", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-github-report-"));
  try {
    fs.mkdirSync(path.join(root, "src"));
    fs.writeFileSync(path.join(root, "src/input.ts"), "const value = 1;\n");
    const report = JSON.stringify({
      diagnostics: [
        {
          message: "Unexpected value, 100%.\nRemove it.",
          code: "eslint(example)",
          severity: "error",
          filename: "src/input.ts",
          labels: [{ span: { offset: 6, length: 5, line: 1, column: 7 } }],
        },
      ],
      number_of_files: 1,
      number_of_rules: 97,
      threads_count: 12,
      start_time: 0.0065,
    });
    assert.equal(
      renderConvertedReport(report, "github", root),
      "::error file=src/input.ts,line=1,endLine=1,col=7,endColumn=12,title=eslint(example)::src/input.ts:1:7: Unexpected value, 100%25.%0ARemove it.\n\n" +
        "Found 0 warnings and 1 error.\n" +
        "Finished in 6ms on 1 file with 97 rules using 12 threads.\n",
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("GitLab output emits stable code-quality entries and clean reports emit an array", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-gitlab-report-"));
  try {
    fs.writeFileSync(path.join(root, "input.ts"), "bad();\n");
    const base = {
      number_of_files: 1,
      number_of_rules: 1,
      threads_count: 1,
      start_time: 1.25,
    };
    const report = JSON.stringify({
      ...base,
      diagnostics: [
        {
          message: "Bad call.",
          code: "eslint(example)",
          severity: "warning",
          filename: "input.ts",
          labels: [{ span: { offset: 0, length: 5, line: 1, column: 1 } }],
        },
      ],
    });
    const first = JSON.parse(renderConvertedReport(report, "gitlab", root));
    const second = JSON.parse(renderConvertedReport(report, "gitlab", root));
    assert.deepEqual(first, second);
    assert.deepEqual(
      { ...first[0], fingerprint: undefined },
      {
        description: "Bad call.",
        check_name: "eslint(example)",
        fingerprint: undefined,
        severity: "major",
        location: { path: "input.ts", lines: { begin: 1, end: 1 } },
      },
    );
    assert.match(first[0].fingerprint, /^[0-9a-f]{16}$/u);
    assert.equal(
      renderConvertedReport(JSON.stringify({ ...base, diagnostics: [] }), "gitlab", root),
      "[]\n",
    );
    assert.equal(
      renderConvertedReport(JSON.stringify({ ...base, diagnostics: [] }), "github", root),
      "Found 0 warnings and 0 errors.\n" +
        "Finished in 1.3s on 1 file with 1 rules using 1 threads.\n",
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("incomplete JSON fails closed instead of manufacturing a report", () => {
  assert.throws(
    () => renderConvertedReport('{"diagnostics":[]}', "github", process.cwd()),
    /complete Oxlint JSON report/u,
  );
});
