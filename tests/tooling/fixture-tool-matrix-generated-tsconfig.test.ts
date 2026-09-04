import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const toolPath = path.join(root, "tools", "commands", "fixtures", "tool-matrix-report.rs");

test("fixture tool matrix pins no-tsconfig typecheck projects to a fixture-local config", () => {
  const outputDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-fixture-tool-matrix-tsconfig-"));
  try {
    const result = spawnSync(
      "rust-script",
      [
        toolPath,
        "--dry-run",
        "--project",
        "splitpanes",
        "--tool",
        "typechecker",
        "--output-dir",
        outputDir,
      ],
      { cwd: root, encoding: "utf8" },
    );
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    const report = JSON.parse(fs.readFileSync(path.join(outputDir, "summary.json"), "utf8"));
    const command = report.projects[0].runs[0].command as string;
    assert.match(command, /check \*\*\/\*\.vue --format json --no-config/);
    assert.match(command, /--tsconfig \.vize-fixture-typecheck-splitpanes\.tsconfig\.json/);
  } finally {
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});
