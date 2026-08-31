import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  checkReport,
  projectLocalVizeBin,
} from "../../legacy-tools/npm/smoke-release-init-project.mjs";

function temporaryProject(name: string): string {
  return fs.mkdtempSync(path.join(os.tmpdir(), `vize-fresh-machine-${name}-`));
}

function writeProjectLocalVize(projectRoot: string, source: string): void {
  const bin = projectLocalVizeBin(projectRoot);
  fs.mkdirSync(path.dirname(bin), { recursive: true });
  fs.writeFileSync(bin, source);
  fs.writeFileSync(
    path.join(projectRoot, "package.json"),
    `${JSON.stringify(
      {
        scripts: {
          "vize:check": 'node -e "process.exit(7)"',
        },
      },
      null,
      2,
    )}\n`,
  );
}

test("machine checks read JSON from the project-local vize binary", () => {
  const root = temporaryProject("json");
  writeProjectLocalVize(
    root,
    [
      'const fs = require("node:fs");',
      "fs.writeFileSync(",
      '  "machine-invocation.json",',
      "  JSON.stringify({ argv: process.argv.slice(2), corsa: process.env.CORSA_PATH ?? null }),",
      ");",
      "process.stdout.write(",
      "  JSON.stringify({ files: [], errorCount: 0, warningCount: 0, fileCount: 0 }),",
      ");",
      "",
    ].join("\n"),
  );

  const previous = process.env.CORSA_PATH;
  process.env.CORSA_PATH = "/host/corsa";
  try {
    const result = checkReport(root);

    assert.equal(result.status, 0);
    assert.deepEqual(result.report, {
      files: [],
      errorCount: 0,
      warningCount: 0,
      fileCount: 0,
    });
    assert.deepEqual(
      JSON.parse(fs.readFileSync(path.join(root, "machine-invocation.json"), "utf8")),
      {
        argv: ["check", "--format", "json", "--quiet"],
        corsa: null,
      },
    );
  } finally {
    if (previous === undefined) delete process.env.CORSA_PATH;
    else process.env.CORSA_PATH = previous;
  }
});

test("machine checks fail closed when project-local vize writes no JSON", () => {
  const root = temporaryProject("empty");
  writeProjectLocalVize(root, "process.exit(0);\n");

  assert.throws(
    () => checkReport(root),
    /project-local vize check did not produce JSON\ncommand: .*status: 0\nsignal: <none>\nstdout\/stderr: <empty>/su,
  );
});
