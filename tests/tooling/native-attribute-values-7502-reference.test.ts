// Pure contract laws. Actual native acceptance is mandatory in the hosted judge.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import {
  rawProcess7502,
  validateProcess7502,
} from "./support/native-attribute-values-7502-build.ts";
import {
  envelopes7502,
  validateEnvelopes7502,
} from "./support/native-attribute-values-7502-envelopes.ts";
import {
  exactKeys7502,
  fixtureSha7502,
  fixtureUrl7502,
  hash7502,
  loadInputs7502,
  outputUrl7502,
  requireReviewed7502,
} from "./support/native-attribute-values-7502-inputs.ts";
import { mandatory7502Paths } from "./support/native-attribute-values-7502-judge.ts";
import { coordinate7502 } from "./support/native-attribute-values-7502-maps.ts";
import { validateCapture7502 } from "./support/native-attribute-values-7502-oracle.ts";

test("verbatim original fourteen pins retain original HTML/options and precise two lower controls", () => {
  const bytes = readFileSync(fixtureUrl7502),
    pack = loadInputs7502(bytes);
  assert.equal(hash7502(bytes), fixtureSha7502);
  assert.equal(pack.fixtures.length, 14);
  assert.deepEqual(
    pack.fixtures
      .filter((row: any) => row.disposition === "lower-refusal")
      .map((row: any) => [row.id, row.source]),
    [
      ["original-regression-11", '<div class="a&amp;amp;b">x</div>'],
      ["original-regression-12", '<div title="a &amp;lt; b">&amp;lt;</div>'],
    ],
  );
  assert.equal(pack.fixtures[0].source, '<div title="a &amp;lt; b">x</div>');
  assert.equal(pack.fixtures[5].source, '<div title="a&b">x</div>');
  assert.equal(pack.fixtures[5].existingExpectedTemplateHtml, '<div title="a&b">x</div>');
  assert.equal(pack.fixtures[8].source, "<div title=a&b>x</div>");
  assert.equal(pack.fixtures[8].existingExpectedTemplateHtml, '<div title="a&amp;b">x</div>');
  for (const mutate of [
    (value: any) => (value.fixtures[0].source += "\n"),
    (value: any) => (value.fixtures[0].existingOptions = "selected custom options"),
    (value: any) => (value.fixtures[11].disposition = "positive"),
  ]) {
    const changed = structuredClone(pack);
    mutate(changed);
    assert.throws(
      () => loadInputs7502(Buffer.from(JSON.stringify(changed))),
      /original input pack drift/,
    );
  }
});

test("unfrozen/null, missing fields and incomplete rows cannot grant whole native output acceptance", () => {
  const stored = JSON.parse(readFileSync(outputUrl7502, "utf8"));
  exactKeys7502(stored, ["schema", "version", "state", "capture"]);
  assert.equal(stored.schema, "vize.native-attribute-values-7502.reviewed-output");
  assert.equal(stored.version, 1);
  assert(["unfrozen", "reviewed"].includes(stored.state));
  if (stored.state === "unfrozen") assert.equal(stored.capture, null);
  else validateCapture7502(stored.capture); // Schema only; never hosted runtime credit.
  const unfrozen = {
    schema: "vize.native-attribute-values-7502.reviewed-output",
    version: 1,
    state: "unfrozen",
    capture: null,
  };
  const synthetic = { syntheticContractOnly: true };
  assert.throws(() => requireReviewed7502(synthetic, unfrozen), /remain unfrozen/);
  assert.throws(
    () => requireReviewed7502(synthetic, { ...unfrozen, state: "reviewed" }),
    /complete reviewed capture/,
  );
  assert.throws(() =>
    requireReviewed7502(synthetic, {
      ...unfrozen,
      state: "reviewed",
      capture: {},
      unexpected: true,
    }),
  );
  assert.throws(() =>
    requireReviewed7502(synthetic, { ...unfrozen, state: "reviewed", capture: { other: true } }),
  );
  assert.throws(() => validateCapture7502({ rows: [] }));
  assert.throws(() => validateEnvelopes7502({ rows: [] }));
  assert.throws(() => exactKeys7502({ present: null }, ["present", "missing"]));
  assert.throws(() => exactKeys7502({ present: null, extra: null }, ["present"]));
});

test("mandatory hosted paths reject absent capture instead of using fixture native modules", () => {
  assert.throws(() => mandatory7502Paths({}), /actual hosted evidence path required/);
  assert.throws(() =>
    mandatory7502Paths({ VIZE_NATIVE_ATTRIBUTE_VALUES_7502_CAPTURE: "fixture-code.json" }),
  );
  assert.deepEqual(
    mandatory7502Paths({
      VIZE_NATIVE_ATTRIBUTE_VALUES_7502_EVIDENCE_DIR: "/evidence",
      VIZE_NATIVE_ATTRIBUTE_VALUES_7502_CAPTURE: "/evidence/first.capture.json",
      VIZE_NATIVE_ATTRIBUTE_VALUES_7502_BUILD_RECEIPT: "/evidence/build-receipt.json",
    }),
    {
      directory: "/evidence",
      capture: "/evidence/first.capture.json",
      build: "/evidence/build-receipt.json",
    },
  );
});

test("raw failure streams retain exact bytes even on throw and reject tampering", () => {
  // Synthetic schema controls confer no source-built or native runtime credit.
  const raw = rawProcess7502({
    status: null,
    signal: "SIGTERM",
    error: new Error("synthetic failure"),
    stdout: Buffer.from([0, 255, 10]),
    stderr: Buffer.from("synthetic error\n"),
  });
  validateProcess7502(raw);
  assert.equal(raw.exitStatus, null);
  assert.equal(raw.signal, "SIGTERM");
  assert.equal(raw.processError, "synthetic failure");
  assert(Buffer.from(raw.stdoutBase64, "base64").equals(Buffer.from([0, 255, 10])));
  for (const changed of [
    { ...raw, stdoutSha256: "0".repeat(64) },
    { ...raw, stderrBase64: "!" },
    { ...raw, hidden: true },
    { ...raw, processError: 3 },
  ])
    assert.throws(() => validateProcess7502(changed));
});

test("byte-map endpoints preserve UTF-8 and independent envelope coverage remains complete", () => {
  assert.deepEqual(coordinate7502("雪\n🌸", 3), [0, 1]);
  assert.deepEqual(coordinate7502("雪\n🌸", 4), [1, 0]);
  assert.deepEqual(coordinate7502("雪\n🌸", 8), [1, 2]);
  for (const offset of [-1, 1, 2, 5, 9]) assert.throws(() => coordinate7502("雪\n🌸", offset));
  assert.deepEqual(
    envelopes7502.map((entry) => entry.issue),
    ["Script(Ordinary)", "Script(Setup)", "Style", "Style"],
  );
  assert.equal(loadInputs7502().fixtures.length * envelopes7502.length * 3 * 2, 336);
});
