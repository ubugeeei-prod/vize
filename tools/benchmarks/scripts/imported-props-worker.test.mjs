import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { freshness, measureNative } from "./imported-props-worker.mjs";

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

function freshnessFixture(check) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-imported-props-control-"));
  fs.mkdirSync(path.join(root, "src/models"), { recursive: true });
  const file = path.join(root, "src/models/Model0.ts");
  const source = "export interface Model0 { field0: string }";
  fs.writeFileSync(file, source);
  try {
    check(root, file, source);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

function fakeNative(file, metadataOnly) {
  let previous;
  return {
    beginTypeResolutionBatch() {
      return { close() {} };
    },
    compileSfcBatchWithResults() {
      const stamp = fs.statSync(file, { bigint: true });
      const key = `${stamp.mtimeNs}:${stamp.size}`;
      if (metadataOnly && previous?.key === key) return previous.result;
      const type = fs.readFileSync(file, "utf8").match(/field0: (string|number|boolean)/u)[1];
      const constructor = { string: "String", number: "Number", boolean: "Boolean" }[type];
      const result = { results: [{ errors: [], code: `field0: { type: ${constructor} }` }] };
      previous = { key, result };
      return result;
    },
  };
}

test("freshness controls retain exact timestamps and restore remounted sources", () => {
  freshnessFixture((root, file, original) => {
    const receipt = freshness(fakeNative(file, false), root);
    assert.equal(receipt.restoredMtimeNs, receipt.editedMtimeNs);
    assert.equal(fs.readFileSync(file, "utf8"), original);
    assert.equal(fs.existsSync(path.join(root, "src/retired-models")), false);
  });
});

test("freshness controls reject a cache using only length and timestamp", () => {
  freshnessFixture((root, file) => {
    assert.throws(() => freshness(fakeNative(file, true), root), /Number/u);
  });
});
