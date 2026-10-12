import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import vm from "node:vm";
import { assertPublishManifestIsSanitized } from "./support/fake-cargo-publish.ts";

test("fake publication refuses dotted development tables and keeps genuine normal/build tables", () => {
  const cases: [string, number | null][] = [
    ["[dev-dependencies]\nx = '1'\n", 2],
    ["[dev-dependencies.assert_cmd]\nversion = '2'\n", 2],
    ["[target.'cfg(unix)'.dev-dependencies]\nx = '1'\n", 2],
    ['[target."cfg(feature = \\"a.b\\")".dev-dependencies.tempfile]\nversion = \'3\'\n', 2],
    ["[dependencies]\nnormal = '1'\n[build-dependencies.cc]\nversion = '1'\n", null],
    ["[target.'cfg(feature = \"a.dev-dependencies.fake\")'.dependencies]\nx = '1'\n", null],
    ["[target.'cfg(unix)'.build-dependencies.cc]\nversion = '1'\n", null],
  ];
  for (const [manifest, expected] of cases) {
    const stop = Symbol("fake child exit");
    let actual: number | null = null;
    try {
      vm.runInNewContext(assertPublishManifestIsSanitized.join("\n"), {
        args: ["publish", "-p", "vize_lightningcss"],
        fs: { readFileSync: () => manifest, existsSync: () => false },
        path,
        process: {
          cwd: () => "/fixture",
          exit: (code: number) => {
            actual = code;
            throw stop;
          },
        },
        console: { error: () => {} },
      });
    } catch (error) {
      assert.equal(error, stop);
    }
    assert.equal(actual, expected, manifest);
  }
});
