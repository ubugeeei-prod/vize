import assert from "node:assert/strict";
import { test } from "node:test";

import { parsePeakRss, runMeasured } from "../../tools/fixtures/typecheck-process-run.mjs";

test("measured typecheck processes record duration and peak RSS", () => {
  const result = runMeasured(process.execPath, ["-e", "process.stdout.write('ok')"], {
    encoding: "utf8",
  });
  assert.equal(result.status, 0);
  assert.equal(result.stdout, "ok");
  assert.ok(Number.isSafeInteger(result.durationMs));
  assert.ok(result.durationMs >= 0);
  assert.ok(Number.isSafeInteger(result.peakRssBytes));
  assert.ok(result.peakRssBytes > 0);
});

test("peak RSS parser preserves the platform unit contract", () => {
  assert.equal(parsePeakRss("  123 maximum resident set size\n", "darwin"), 123);
  assert.equal(parsePeakRss("Maximum resident set size (kbytes): 123\n", "linux"), 123 * 1024);
});

test("peak RSS parser fails closed on unsupported or malformed evidence", () => {
  assert.throws(() => parsePeakRss("123", "win32"), /unsupported/);
  assert.throws(() => parsePeakRss("not metrics", "linux"), /output is invalid/);
  assert.throws(
    () => parsePeakRss("Maximum resident set size (kbytes): 0\n", "linux"),
    /positive safe integer/,
  );
});
