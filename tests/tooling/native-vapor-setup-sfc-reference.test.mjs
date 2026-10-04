import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { compileVaporSetupSfcReference } from "./support/native-vapor-setup-oracle.mjs";
import { runVaporSetupSfcRuntime } from "./support/native-vapor-setup-runtime.mjs";

const ui = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const vue = createRequire(ui.resolve("vue-vapor-runtime/package.json"));
const { SourceMapConsumer } = createRequire(vue.resolve("@vue/compiler-sfc"))("source-map-js");
const pack = JSON.parse(
  readFileSync(
    new URL(
      "../../crates/vize_atelier_sfc/tests/fixtures/native_vapor_setup_sfc_vue_3_6_rc9.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const path = process.env.VIZE_NATIVE_VAPOR_SETUP_SFC_CAPTURE;
if (process.env.VIZE_NATIVE_VAPOR_REQUIRE_CAPTURE === "1")
  assert.ok(path, "mandatory fresh genuine native setup capture path");
// Local expected-module probes earn no actual native capture credit. Actions
// must read the original Rust-emitted complete modules, never this fallback.
const captures = path ? JSON.parse(readFileSync(path, "utf8")) : pack.fixtures;
assert.equal(captures.length, 17);
const fixtures = [];
const position = (text, offset) => {
  const lines = Buffer.from(text)
    .subarray(0, offset)
    .toString("utf8")
    .split(/\r\n|[\r\n\u2028\u2029]/u);
  return { line: lines.length, column: lines.at(-1).length };
};

await test("complete original setup native modules and named maps equal independent expectations", async () => {
  for (const expected of pack.fixtures) {
    const actual = captures.find((row) => row.id === expected.id);
    assert.ok(actual);
    assert.equal(actual.source, expected.source);
    assert.equal(actual.code, expected.code);
    assert.deepEqual(actual.map, expected.map);
    const consumer = new SourceMapConsumer(actual.map);
    assert.equal(consumer.sourceContentFor(pack.filename), expected.source);
    for (const link of expected.links) {
      const original = consumer.originalPositionFor(position(actual.code, link.generated));
      assert.deepEqual(
        { line: original.line, column: original.column },
        position(actual.source, link.source),
      );
      assert.equal(original.name, link.name);
    }
    // Stock production graph equality is a separate oracle. Native retains its
    // own declaration placement and never claims upstream code/hoist equality.
    const reference = await compileVaporSetupSfcReference(actual.source, actual.id);
    assert.deepEqual(JSON.parse(JSON.stringify(reference)), expected.reference);
    fixtures.push({ ...actual, reference });
  }
});

await test("real dev and production Vapor default setup mounts, hydrations and cleanup agree", async () => {
  assert.equal(fixtures.length, 17);
  const result = await runVaporSetupSfcRuntime({ fixtures });
  assert.equal(result.runtimes.length, 2);
  for (const runtime of result.runtimes) {
    assert.deepEqual(runtime.counts, {
      fixtureGroups: 17,
      configurations: 19,
      mounts: 38,
      clones: 38,
      hydratedConfigurations: 17,
      hydrationMounts: 34,
      retainedSSRNodes: 52,
      mountOnlyConfigurations: 2,
    });
  }
  assert.equal(result.processFrames.length, 8);
  assert.equal(
    result.processFrames.filter((frame) => !frame.expectedFailure && frame.exit === 0).length,
    2,
  );
  assert.equal(
    result.processFrames.filter((frame) => frame.expectedFailure && frame.exit === 1).length,
    6,
  );
  assert.ok(result.processFrames.every((frame) => frame.signal === null && frame.error === null));

  const output = process.env.VIZE_NATIVE_VAPOR_SETUP_SFC_RUNTIME_CAPTURE;
  if (process.env.VIZE_NATIVE_VAPOR_REQUIRE_CAPTURE === "1")
    assert.ok(output, "mandatory real runtime capture path");
  if (output)
    writeFileSync(
      output,
      JSON.stringify(
        {
          ...result,
          nativeAuthority: path
            ? "fresh-original-rust-capture"
            : "independent-frozen-expectations-only",
        },
        null,
        2,
      ) + "\n",
    );
});
