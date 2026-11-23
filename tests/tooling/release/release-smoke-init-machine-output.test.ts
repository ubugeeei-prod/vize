import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  checkReport,
  projectLocalVizeBin,
} from "../../../tools/support/compat/npm/smoke-release-init-project.ts";

function temporaryProject(name: string): string {
  return fs.mkdtempSync(path.join(os.tmpdir(), `vize-fresh-machine-${name}-`));
}

function installProjectLocalVize(projectRoot: string, program: URL): void {
  const bin = projectLocalVizeBin(projectRoot);
  fs.mkdirSync(path.dirname(bin), { recursive: true });
  fs.copyFileSync(program, bin);
  fs.copyFileSync(
    new URL("../programs/process/exit-with-status.js", import.meta.url),
    path.join(projectRoot, "fallback.cjs"),
  );
  fs.writeFileSync(
    path.join(projectRoot, "package.json"),
    `${JSON.stringify(
      {
        scripts: {
          "vize:check": "node fallback.cjs 7",
        },
      },
      null,
      2,
    )}\n`,
  );
}

test("machine checks read JSON from the project-local vize binary", () => {
  const root = temporaryProject("json");
  installProjectLocalVize(
    root,
    new URL("../programs/release/record-machine-check.cjs", import.meta.url),
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
  installProjectLocalVize(
    root,
    new URL("../programs/release/empty-machine-check.cjs", import.meta.url),
  );

  assert.throws(
    () => checkReport(root),
    /project-local vize check did not produce JSON\ncommand: .*status: 0\nsignal: <none>\nstdout\/stderr: <empty>/su,
  );
});
