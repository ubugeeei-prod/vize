import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const authoritySha256 = "f36cfb0cacc0058c44ae6ad5529bc25390f265dcb8796d601e1652603aee8b8d";
const historicSha256 = "dad987b8bbfd1d89640a4dc600da57545b038ab7006bede88453b91648439c26";
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const oldSuggestion = "Wrap in onMounted() or use import.meta.client check";
const newSuggestion =
  "Wrap in onMounted() or guard with !import.meta.env.SSR, import.meta.client, or typeof window !== 'undefined'";

export function browserOriginCurrentReference(root, source, supplied) {
  const directory = path.join(root, "crates/vize/tests/fixtures/issue-7907");
  const bytes = supplied ?? fs.readFileSync(path.join(directory, "current-reference-7908.json"));
  assert.equal(hash(bytes), authoritySha256, "current authority must be byte-exact");
  assert.equal(hash(fs.readFileSync(path.join(directory, "source.json"))), historicSha256);
  const authority = JSON.parse(bytes);
  assert.equal(authority.schema, "vize.browser-origin.current-reference");
  assert.equal(authority.version, 1);
  assert.equal(authority.historicCorpusSha256, historicSha256);
  assert.equal(authority.cases.length, 1);
  const row = authority.cases[0];
  assert.equal(row.path, "PageTitle.vue");
  const entry = source.cases.find((entry) => entry.path === row.path);
  assert.ok(entry);
  assert.equal(entry.bytes, row.bytes);
  assert.equal(entry.sha256, row.sha256);
  const input = fs.readFileSync(path.join(directory, entry.file));
  assert.equal(input.length, 227);
  assert.equal(hash(input), row.sha256);
  assert.deepEqual(row.historicalExpected, entry.expected);
  assert.deepEqual(row.historicalDoctorLocations, entry.doctorLocations);
  assert.deepEqual(row.currentExpected, [
    { file: "PageTitle.vue", messages: [], errorCount: 0, warningCount: 0 },
  ]);
  assert.deepEqual(row.currentDoctorLocations, []);
  assert.notDeepEqual(row.currentExpected, row.historicalExpected);
  assert.notDeepEqual(row.currentDoctorLocations, row.historicalDoctorLocations);
  const historicalLaw = fs.readFileSync(path.join(directory, authority.historicProducerLaw.file));
  assert.equal(hash(historicalLaw), authority.historicProducerLaw.sha256);
  assert.equal(historicalLaw.toString().split(oldSuggestion).length, 2);
  assert.equal(
    fs.readFileSync(
      path.join(root, "crates/vize_croquis_cf/tests/browser_diagnostic_origin.rs"),
      "utf8",
    ),
    historicalLaw
      .toString()
      .replace(`Some("${oldSuggestion}")`, `Some(\n            "${newSuggestion}"\n        )`),
    "both complete current producer laws preserve every assertion except the authored help transition",
  );
  return { authoritySha256, row };
}
