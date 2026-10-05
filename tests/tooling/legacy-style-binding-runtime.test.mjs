import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../../", import.meta.url));
const source = readFileSync(
  new URL(
    "../../crates/vize_atelier_vapor/tests/fixtures/static-style-merged-binding.input.txt",
    import.meta.url,
  ),
  "utf8",
);

await test("source-built retained Vapor #7600 fixture keeps static style through real runtime updates", () => {
  const compiled = spawnSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--locked",
      "--profile",
      "ci",
      "-p",
      "vize_atelier_vapor",
      "--example",
      "style_binding_runtime",
    ],
    { cwd: root, encoding: "utf8", timeout: 600_000, maxBuffer: 4 * 1024 * 1024 },
  );
  assert.ifError(compiled.error);
  assert.equal(compiled.signal, null, compiled.stderr);
  assert.equal(compiled.status, 0, compiled.stderr);
  const cases = JSON.parse(compiled.stdout);
  assert.deepEqual(
    cases.map((entry) => entry.whitespace),
    ["condense", "preserve"],
  );
  for (const entry of cases) {
    assert.equal(entry.source, source, "complete existing reporter fixture bytes");
    assert.equal(entry.retainedLane, true);
    assert.equal(entry.prefixIdentifiers, true);
    assert.deepEqual(entry.errorMessages, []);
    assert.equal(typeof entry.code, "string");
    assert.ok(entry.code.length > 0);
    const runtime = spawnSync(
      process.execPath,
      [fileURLToPath(new URL("./support/legacy-style-binding-runtime.mjs", import.meta.url))],
      {
        input: JSON.stringify(entry),
        encoding: "utf8",
        timeout: 60_000,
        maxBuffer: 4 * 1024 * 1024,
      },
    );
    assert.ifError(runtime.error);
    assert.equal(runtime.signal, null, runtime.stderr);
    assert.equal(runtime.status, 0, runtime.stderr);
    assert.equal(runtime.stderr, "", "runtime warnings and errors must not be discarded");
    const receipt = JSON.parse(runtime.stdout);
    assert.deepEqual(receipt.colors, ["red", "green", "red", "red"]);
    assert.deepEqual(receipt.backgrounds, ["blue", "", "", ""]);
    assert.equal(receipt.sameNode, true);
    assert.equal(receipt.unmounted, true);
    console.log(JSON.stringify({ compiler: entry, runtime: receipt }));
  }
});
