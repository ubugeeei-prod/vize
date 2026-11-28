import assert from "node:assert/strict";
import test from "node:test";
import { measureNative } from "./imported-props-worker.mjs";

test("measurements compile every input for all three targets inside one closed window", () => {
  const files = Array.from({ length: 1000 }, (_, index) => ({
    path: `${index}.vue`,
    source: String(index),
  }));
  for (const chunk of [32, 128, 1000]) {
    let open = false;
    let opened = 0;
    let closed = 0;
    const seen = [];
    const native = {
      beginTypeResolutionBatch() {
        assert.equal(open, false);
        open = true;
        opened++;
        return {
          close() {
            open = false;
            closed++;
          },
        };
      },
      compileSfcBatchWithResults(batch, options) {
        assert.equal(open, true);
        assert.equal(options.threads, undefined, "use the process-global pool");
        seen.push(...batch.map((file) => [file.path, options.ssr]));
        return { failedCount: 0, successCount: batch.length, results: batch };
      },
    };
    const measured = measureNative(native, files, chunk);
    assert.equal(opened, 1);
    assert.equal(closed, 1);
    assert.equal(measured.calls, 3 * Math.ceil(files.length / chunk));
    assert.deepEqual(
      seen,
      [false, true, false].flatMap((ssr) => files.map((file) => [file.path, ssr])),
    );
    assert.deepEqual(measured.packets, [files, files, files]);
    assert(measured.wallMs >= 0);
    assert(measured.cpuMs >= 0);
  }
});

test("failed compilation closes the stability window", () => {
  let closed = false;
  const native = {
    beginTypeResolutionBatch() {
      return {
        close() {
          closed = true;
        },
      };
    },
    compileSfcBatchWithResults() {
      throw new Error("compiler failure");
    },
  };
  assert.throws(
    () => measureNative(native, [{ path: "App.vue", source: "" }], 32),
    /compiler failure/u,
  );
  assert.equal(closed, true);
});
