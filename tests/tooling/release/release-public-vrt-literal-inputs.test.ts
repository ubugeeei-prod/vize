import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { mkdtemp, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import {
  literalVariants,
  literalVrtSources,
  materializeLiteralVrtInputs,
  readLiteralVrtInputs,
} from "../../../tools/support/release/public_acceptance/vrt_literal.ts";

const fixtures = fileURLToPath(new URL("../fixtures/musea/literal-variant-keys/", import.meta.url));
const vector = fileURLToPath(
  new URL("../../_fixtures/differential/musea/literal-variant-keys.json", import.meta.url),
);
function read(source: string) {
  assert.ok(literalVrtSources.includes(source as (typeof literalVrtSources)[number]));
  return readFileSync(
    source.endsWith(".json") ? vector : path.join(fixtures, path.basename(source)),
  );
}

// Authored input/filesystem controls only; never call the installed build or any surface factory.
void test("all exact Controls2/Keys3 inputs materialize once with full byte custody", async (t) => {
  const consumer = await mkdtemp(path.join(os.tmpdir(), "vize-literal-input-"));
  t.after(() => rm(consumer, { recursive: true }));
  const records = materializeLiteralVrtInputs(consumer, read);
  assert.equal(records.length, 3);
  assert.deepEqual(
    records.map((record) => record.sourceFile),
    [...literalVrtSources],
  );
  for (const record of records)
    assert.deepEqual(readFileSync(path.join(consumer, record.file)), read(record.sourceFile));
  const sources = readLiteralVrtInputs(consumer);
  assert.deepEqual(
    sources.map((source) => source.path),
    literalVrtSources.slice(0, 2),
  );
  for (const source of sources) assert.deepEqual(source.bytes, read(source.path));
  assert.deepEqual(
    literalVariants.Controls.map(([name]) => name),
    ["Default", "Custom theme"],
  );
  assert.deepEqual(
    literalVariants.Keys.map(([name]) => name),
    ["__proto__", "constructor", "hasOwnProperty"],
  );
  assert.throws(() => materializeLiteralVrtInputs(consumer, read), { code: "EEXIST" });
});

void test("either altered authored Art is refused by the full frozen source hash", async (t) => {
  for (const title of ["Controls", "Keys"]) {
    const consumer = await mkdtemp(path.join(os.tmpdir(), "vize-literal-art-refusal-"));
    t.after(() => rm(consumer, { recursive: true }));
    materializeLiteralVrtInputs(consumer, read);
    writeFileSync(path.join(consumer, "trusted-vrt-inputs", `${title}.art.vue`), "changed\n");
    assert.throws(() => readLiteralVrtInputs(consumer), /Frozen literal authored bytes changed/);
  }
});

void test("changed vector cannot authorize a foreign source path or fewer reserved names", async (t) => {
  const consumer = await mkdtemp(path.join(os.tmpdir(), "vize-literal-vector-refusal-"));
  t.after(() => rm(consumer, { recursive: true }));
  materializeLiteralVrtInputs(consumer, read);
  const changed = JSON.parse(readFileSync(vector, "utf8"));
  changed.sources[0].path = "../../foreign.art.vue";
  changed.variants.Keys = ["__proto__"];
  writeFileSync(
    path.join(consumer, "trusted-vrt-inputs/literal-variant-keys.json"),
    JSON.stringify(changed),
  );
  assert.throws(() => readLiteralVrtInputs(consumer), /Frozen literal vector changed/);
});
