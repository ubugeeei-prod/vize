import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  controlledPublishFixture,
  maxActivePublishers,
} from "./support/publish-npm-package-dirs-fixture.ts";
import { failedPublishFixture } from "./support/publish-npm-package-dirs-failure-fixture.ts";
import { runReleaseCommand } from "./support/release-command.ts";

// Expected launch/skip sets come from the authored barrier and original sorted
// eight packages, independently of the implementation's receipt or output.
for (const [label, workers, args] of [
  ["default serial", 1, []],
  ["explicit serial", 1, ["--concurrency", "1"]],
  ["four active children", 4, ["--concurrency", "4"]],
] as const) {
  void test(`failed platform publisher stops queued launches in ${label}`, () => {
    const fixture = failedPublishFixture(workers);
    try {
      const result = fixture.runParent([
        fixture.baseDir,
        ...args,
        "--receipt",
        fixture.receiptPath,
        "--provenance",
      ]);
      assert.equal(result.status, 7, `${result.stderr}\n${result.stdout}`);
      assert.equal(fixture.failureObserved(), true, "the actual parent must observe the failure");
      const events = fixture.events();
      const launched = Array.from({ length: workers }, (_, index) => `pkg-${index}`);
      const skipped = Array.from({ length: 8 - workers }, (_, index) => `pkg-${index + workers}`);
      assert.deepEqual(
        events
          .filter((event) => event.phase === "start")
          .map((event) => event.package)
          .sort((a, b) => a.localeCompare(b)),
        launched,
      );
      assert.deepEqual(
        events
          .filter((event) => event.phase === "end")
          .map((event) => event.package)
          .sort((a, b) => a.localeCompare(b)),
        launched,
        "every active child must be reaped before the failed aggregate exits",
      );
      assert.equal(maxActivePublishers(events), workers);
      for (const event of events) assert.deepEqual(event.args, ["--provenance"]);
      for (const name of launched) {
        assert.match(result.stdout, new RegExp(`publisher stdout ${name}`));
        assert.match(result.stderr, new RegExp(`publisher stderr ${name}`));
      }
      const receipt = fixture.receipt() as ReturnType<typeof fixture.receipt> & {
        skipped: string[];
        diagnostics: Array<{ package_dir: string; stdout: string; stderr: string }>;
      };
      assert.equal(receipt.schema_version, 1);
      assert.equal(receipt.concurrency, workers);
      assert.equal(receipt.package_count, 8);
      assert.equal(receipt.exit_code, 7);
      assert.deepEqual(
        receipt.results.map((item) => path.basename(item.package_dir)),
        launched,
      );
      assert.deepEqual(
        receipt.results.map((item) => item.exit_code),
        [7, ...Array(workers - 1).fill(0)],
      );
      assert.deepEqual(
        receipt.skipped.map((directory) => path.basename(directory)),
        skipped,
      );
      assert.ok(receipt.results.every((item) => item.walltime_ms >= 0));
      assert.deepEqual(
        receipt.diagnostics,
        launched.map((name) => ({
          package_dir: path.join(fixture.baseDir, name),
          stdout: `\r\npublisher stdout ${name}\n\n`,
          stderr: `  publisher stderr ${name}\r\n\r\n`,
        })),
        "complete decoded child texts retain CRLF and boundary whitespace",
      );
    } finally {
      fixture.cleanup();
    }
  });
}

void test("the publish receipt keeps the original compact field order and escapes", () => {
  const fixture = controlledPublishFixture(2);
  try {
    const result = runReleaseCommand(
      "publish_npm_package_dirs",
      [fixture.baseDir, "--receipt", fixture.receiptPath],
      { env: fixture.env },
    );
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`);
    const dirs = ["pkg-0", "pkg-1"].map((name) => JSON.stringify(path.join(fixture.baseDir, name)));
    const raw = fs
      .readFileSync(fixture.receiptPath, "utf8")
      .replace(/"walltime_ms":\d+/g, '"walltime_ms":0');
    assert.equal(
      raw,
      `{"schema_version":1,"concurrency":1,"package_count":2,"walltime_ms":0,"exit_code":0,` +
        `"results":[{"package_dir":${dirs[0]},"exit_code":0,"walltime_ms":0},{"package_dir":${dirs[1]},"exit_code":0,"walltime_ms":0}],` +
        `"skipped":[],"diagnostics":[` +
        `{"package_dir":${dirs[0]},"stdout":"publisher stdout pkg-0\\n","stderr":"publisher stderr pkg-0\\n"},` +
        `{"package_dir":${dirs[1]},"stdout":"publisher stdout pkg-1\\n","stderr":"publisher stderr pkg-1\\n"}]}\n`,
    );
  } finally {
    fixture.cleanup();
  }
});
