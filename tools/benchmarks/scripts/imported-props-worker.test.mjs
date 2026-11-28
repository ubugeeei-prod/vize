import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {
  arrayIsolation,
  freshness,
  measureNative,
  prototypeIsolation,
} from "./imported-props-worker.mjs";
import { separateDependencies } from "./imported-props-fixture.mjs";
import { measuredStacks } from "./imported-props-profile.mjs";

await test("sample attribution keeps only cold and warm compile intervals", () => {
  const block = (time, frame) => `node 10/11 ${time}: cycles:\n 123 ${frame} (head.node)\n`;
  const script = [
    block("0.1", "startup"),
    block("1.25", "cold"),
    block("2.01", "validation"),
    block("3.000000001", "warm"),
    block("4.1", "freshness"),
  ].join("\n");
  const selected = measuredStacks(script, {
    cold: { startNs: "1000000000", endNs: "2000000000" },
    warm: { startNs: "3000000000", endNs: "4000000000" },
  });
  assert.equal(selected.length, 2);
  assert(selected[0].includes("cold"));
  assert(selected[1].includes("warm"));
});

await test("array controls reject shared mutable dependency arrays", () => {
  const shared = ["/type.ts"];
  assert.throws(() => arrayIsolation([{ dependencies: shared }, { dependencies: shared }]));
  const left = ["/type.ts"];
  const right = ["/type.ts"];
  arrayIsolation([{ dependencies: left }, { dependencies: right }]);
  assert.deepEqual(left, ["/type.ts"]);
  assert.deepEqual(right, ["/type.ts"]);
});

await test("prototype controls reject inherited setters and restore the prototype", () => {
  const unsafeBoundary = {
    compileSfcBatchWithResults() {
      const holder = {};
      holder.packet = {};
    },
    compileSfcBatchForTargets() {},
  };
  assert.throws(() => prototypeIsolation(unsafeBoundary, []), /inherited packet setter/u);
  assert.equal(Object.getOwnPropertyDescriptor(Object.prototype, "packet"), undefined);
});

await test("the dependency miss control retains distinct complete module lists", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-imported-props-paths-"));
  const components = path.join(root, "src/components");
  fs.mkdirSync(components, { recursive: true });
  for (const index of [0, 1]) {
    fs.writeFileSync(
      path.join(components, `Item${index}.vue`),
      '<script setup lang="ts">import type { ItemProps } from "../types"; defineProps<ItemProps>()</script>',
    );
  }
  try {
    const expected = separateDependencies(root, 20);
    const entries = Object.entries(expected);
    assert.equal(entries.length, 2);
    for (const [file, dependencies] of entries) {
      assert.equal(dependencies.length, 26);
      assert.equal(new Set(dependencies).size, 26);
      assert.deepEqual(
        dependencies,
        [...dependencies].sort((left, right) => (left === right ? 0 : left < right ? -1 : 1)),
      );
      assert(fs.readFileSync(file, "utf8").includes('"./types"'));
      for (const dependency of dependencies.filter(
        (dependency) => !dependency.endsWith("tsconfig.json"),
      ))
        assert(fs.statSync(dependency).isFile());
    }
    assert.equal(entries[0][1].filter((name) => entries[1][1].includes(name)).length, 3);
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

await test("measurements compile every input for all three targets inside one closed window", () => {
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
        return { failedCount: 0, successCount: batch.length, results: batch, timeMs: 2 };
      },
    };
    const measured = measureNative(native, files, chunk);
    assert.equal(opened, 1);
    assert.equal(closed, 1);
    assert.equal(measured.calls, 3 * Math.ceil(files.length / chunk));
    assert.equal(measured.nativeCoreMs, 2 * measured.calls);
    assert.deepEqual(
      seen,
      [false, true, false].flatMap((ssr) => files.map((file) => [file.path, ssr])),
    );
    assert.deepEqual(measured.packets, [files, files, files]);
    assert(measured.wallMs >= 0);
    assert(measured.cpuMs >= 0);
  }
});

await test("failed compilation closes the stability window", () => {
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

await test("freshness controls retain exact timestamps and restore remounted sources", () => {
  freshnessFixture((root, file, original) => {
    const receipt = freshness(fakeNative(file, false), root);
    assert.equal(receipt.restoredMtimeNs, receipt.editedMtimeNs);
    assert.equal(fs.readFileSync(file, "utf8"), original);
    assert.equal(fs.existsSync(path.join(root, "src/retired-models")), false);
  });
});

await test("freshness controls reject a cache using only length and timestamp", () => {
  freshnessFixture((root, file) => {
    assert.throws(() => freshness(fakeNative(file, true), root), /Number/u);
  });
});
