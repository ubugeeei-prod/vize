import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { createHash } from "node:crypto";
import { assertTypedContextWrapperMappings } from "../../tools/support/github/typed-context-wrapper-mappings.ts";

const family = "tests/_fixtures/differential/typechecker/template-context-undefined-10667";
const capture = JSON.parse(readFileSync(`${family}/wrapper-successor/capture.json.txt`, "utf8"));
const producer = readFileSync("crates/vize_canon/src/batch/virtual_project/content_mapper.rs");
const integratedCapture = (): typeof capture => {
  const data = structuredClone(capture);
  const generated = Buffer.from(data.current.text);
  const declaration = generated.indexOf("declare const Route: typeof __vize_component__;");
  assert(declaration >= 0, "captured generated component declaration");
  data.current.mappings.push([
    declaration + Buffer.byteLength("declare const "),
    Buffer.byteLength("Route"),
    0,
    0,
    1,
    248,
  ]);
  return data;
};
const verify = (data = integratedCapture(), bytes = producer) =>
  assertTypedContextWrapperMappings(
    data.source,
    data.before,
    data.current,
    data.projectedHistoricalMappings,
    bytes,
  );

void test("actual captured current mapping retains the old packet and adds both authored wrapper atoms", () => {
  verify();
});

void test("captured whole packets and the existing source fixture retain their literal custody", () => {
  const bytes = readFileSync(`${family}/wrapper-successor/capture.json.txt`);
  const provenance = JSON.parse(
    readFileSync(`${family}/wrapper-successor/provenance.json.txt`, "utf8"),
  );
  assert.equal(createHash("sha256").update(bytes).digest("hex"), provenance.captureSha256);
  assert.equal(
    provenance.captureSha256,
    "a8efaafcd018cbb2b315c5a7e26a06e646ba6fd1c2885489b495cdeabd044e16",
  );
  assert.equal(readFileSync(`${family}/sources/prop-used.vue.txt`, "utf8"), capture.source);
  assert.equal(provenance.before, "7a4ab266003bfea2d13d1f5901b55d11b031a702");
  assert.equal(provenance.source, "bb46093387c1c9ef00fe4659eb01dea3b18c5f58");
});

void test("a separately certified helper prefix relocation moves both atoms without a fixed prefix delta", () => {
  const data = integratedCapture();
  const prefix = "// certified helper bytes\n";
  data.current.text = prefix + data.current.text;
  for (const row of data.projectedHistoricalMappings) row[0] += Buffer.byteLength(prefix);
  for (const row of data.current.mappings) row[0] += Buffer.byteLength(prefix);
  verify(data);
});

void test("unknown producer, authored source and opaque packet fields are refused", () => {
  assert.throws(
    () => verify(capture, Buffer.concat([producer, Buffer.from("\n")])),
    /unqualified content-mapper producer/u,
  );
  const changed = structuredClone(capture);
  changed.source += "\n";
  assert.throws(() => verify(changed), /unqualified typed-context source/u);
  for (const side of ["before", "current"]) {
    const data = integratedCapture();
    data[side].futureMetadata = { enabled: true };
    assert.throws(() => verify(data), /deep-equal/u);
  }
});

void test("every historical tuple field and its ordering remain mandatory", () => {
  for (let row = 0; row < capture.projectedHistoricalMappings.length; row++) {
    for (let field = 0; field < 6; field++) {
      const data = integratedCapture();
      data.projectedHistoricalMappings[row][field]++;
      assert.throws(() => verify(data), /all projected historical tuples must remain exact/u);
    }
  }
  const data = integratedCapture();
  data.projectedHistoricalMappings.reverse();
  assert.throws(() => verify(data), /all projected historical tuples must remain exact/u);
});

void test("both atoms keep every field, including diagnostic-only features, and ordered whole equality", () => {
  for (const row of [9, 11]) {
    for (let field = 0; field < 6; field++) {
      const changed = integratedCapture();
      changed.current.mappings[row][field]++;
      assert.throws(() => verify(changed), /whole ordered current mapping successor/u);
    }
  }
  const reversed = integratedCapture();
  reversed.current.mappings.reverse();
  assert.throws(() => verify(reversed), /whole ordered current mapping successor/u);
  const missing = integratedCapture();
  missing.current.mappings.pop();
  assert.throws(() => verify(missing), /whole ordered current mapping successor/u);
});

void test("a different or duplicated generated wrapper cannot reuse the successor", () => {
  const changed = integratedCapture();
  changed.current.text = changed.current.text.replace(
    "void (undefined); // Interpolation",
    "void [undefined]; // Interpolation",
  );
  assert.throws(() => verify(changed), /unique interpolation/u);
  changed.current.text = capture.current.text + "void (undefined); // Interpolation";
  assert.throws(() => verify(changed), /unique interpolation/u);
});
