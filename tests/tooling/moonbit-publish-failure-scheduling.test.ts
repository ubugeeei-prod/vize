import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";

import { runMoonScript } from "./_helpers/moonbit.ts";
import { maxActivePublishers } from "./support/moonbit-publish-concurrency-fixture.ts";
import { failedPublishFixture } from "./support/moonbit-publish-failure-fixture.ts";

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
      const result = runMoonScript(
        "publish_npm_package_dirs",
        [fixture.baseDir, ...args, "--receipt", fixture.receiptPath, "--provenance"],
        { env: fixture.env },
      );
      assert.equal(result.status, 7, `${result.stderr}\n${result.stdout}`);
      assert.equal(fixture.failureObserved(), true, "the actual parent must observe the failure");
      const events = fixture.events();
      const launched = Array.from({ length: workers }, (_, index) => `pkg-${index}`);
      const skipped = Array.from({ length: 8 - workers }, (_, index) => `pkg-${index + workers}`);
      assert.deepEqual(
        events
          .filter((event) => event.phase === "start")
          .map((event) => event.package)
          .toSorted((a, b) => a.localeCompare(b)),
        launched,
      );
      assert.deepEqual(
        events
          .filter((event) => event.phase === "end")
          .map((event) => event.package)
          .toSorted((a, b) => a.localeCompare(b)),
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
