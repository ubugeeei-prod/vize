import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function snapshotFiles(...segments: string[]): string[] {
  const directory = path.join(root, ...segments);
  return fs
    .readdirSync(directory)
    .filter((file) => file.endsWith(".snap"))
    .sort()
    .map((file) => path.join(directory, file));
}

function readJsonSnapshot(file: string): unknown {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

test("check snapshots are complete JSON baselines", () => {
  for (const snapshot of snapshotFiles("tests", "snapshots", "check", "__snapshots__")) {
    const data = readJsonSnapshot(snapshot);

    assert.ok(data && typeof data === "object" && !Array.isArray(data), snapshot);
    const baseline = data as {
      files?: unknown[];
      fileCount?: unknown;
      errorCount?: unknown;
      warningCount?: unknown;
    };

    assert.ok(Array.isArray(baseline.files), snapshot);
    assert.equal(baseline.fileCount, baseline.files.length, snapshot);
    assert.equal(typeof baseline.errorCount, "number", snapshot);
    assert.equal(typeof baseline.warningCount, "number", snapshot);

    for (const file of baseline.files) {
      assert.ok(file && typeof file === "object", snapshot);
      const entry = file as { file?: unknown; virtualTs?: unknown; diagnostics?: unknown };
      assert.equal(typeof entry.file, "string", snapshot);
      assert.equal(typeof entry.virtualTs, "string", snapshot);
      assert.ok(Array.isArray(entry.diagnostics), snapshot);
    }
  }
});

test("lint snapshots include rule documentation and consistent message counts", () => {
  for (const snapshot of snapshotFiles("tests", "snapshots", "lint", "__snapshots__")) {
    const data = readJsonSnapshot(snapshot);

    assert.ok(Array.isArray(data), snapshot);
    assert.ok(data.length > 0, snapshot);

    for (const entry of data as Array<{
      file?: unknown;
      messages?: unknown[];
      errorCount?: unknown;
      warningCount?: unknown;
    }>) {
      assert.equal(typeof entry.file, "string", snapshot);
      assert.ok(Array.isArray(entry.messages), snapshot);
      assert.equal(typeof entry.errorCount, "number", snapshot);
      assert.equal(typeof entry.warningCount, "number", snapshot);

      let errors = 0;
      let warnings = 0;
      for (const message of entry.messages as Array<{
        ruleId?: unknown;
        ruleDocsPath?: unknown;
        message?: unknown;
        severity?: unknown;
      }>) {
        const ruleDocsPath = message.ruleDocsPath;
        assert.equal(typeof message.ruleId, "string", snapshot);
        if (typeof ruleDocsPath !== "string") {
          assert.fail(`${snapshot}: missing ruleDocsPath`);
        }
        assert.equal(typeof message.message, "string", snapshot);
        assert.ok(
          fs.existsSync(path.join(root, ruleDocsPath)),
          `${snapshot}: missing ${ruleDocsPath}`,
        );

        if (message.severity === 2) {
          errors++;
        } else if (message.severity === 1) {
          warnings++;
        } else {
          assert.fail(`${snapshot}: unexpected severity ${String(message.severity)}`);
        }
      }

      assert.equal(entry.errorCount, errors, snapshot);
      assert.equal(entry.warningCount, warnings, snapshot);
    }
  }
});
