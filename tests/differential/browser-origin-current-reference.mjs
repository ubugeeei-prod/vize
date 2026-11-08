import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const authoritySha256 = "f36cfb0cacc0058c44ae6ad5529bc25390f265dcb8796d601e1652603aee8b8d";
const unicodeColumnAuthoritySha256 =
  "5b575ad51651ea3830ebc424d4bdd20f8c1648d16a49e11aff988822406a06a1";
const historicSha256 = "dad987b8bbfd1d89640a4dc600da57545b038ab7006bede88453b91648439c26";
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const oldSuggestion = "Wrap in onMounted() or use import.meta.client check";
const newSuggestion =
  "Wrap in onMounted() or guard with !import.meta.env.SSR, import.meta.client, or typeof window !== 'undefined'";

const currentAuthorities = [
  { file: "current-reference-7908.json", sha256: authoritySha256 },
  { file: "current-reference-12069.json", sha256: unicodeColumnAuthoritySha256 },
];

function validateAuthority(directory, source, descriptor, supplied) {
  const bytes = supplied ?? fs.readFileSync(path.join(directory, descriptor.file));
  assert.equal(hash(bytes), descriptor.sha256, `${descriptor.file} must be byte-exact`);
  const authority = JSON.parse(bytes);
  assert.equal(authority.schema, "vize.browser-origin.current-reference");
  assert.equal(authority.version, 1);
  assert.equal(authority.historicCorpusSha256, historicSha256);
  assert.equal(authority.cases.length, 1);
  const row = authority.cases[0];
  const entry = source.cases.find((entry) => entry.path === row.path);
  assert.ok(entry);
  assert.equal(entry.bytes, row.bytes);
  assert.equal(entry.sha256, row.sha256);
  const input = fs.readFileSync(path.join(directory, entry.file));
  assert.equal(input.length, row.bytes);
  assert.equal(hash(input), row.sha256);
  assert.deepEqual(row.historicalExpected, entry.expected);
  assert.deepEqual(row.historicalDoctorLocations, entry.doctorLocations);
  assert(
    JSON.stringify(row.currentExpected) !== JSON.stringify(row.historicalExpected) ||
      JSON.stringify(row.currentDoctorLocations) !== JSON.stringify(row.historicalDoctorLocations),
    `${row.path} must record a current behavior change`,
  );

  if (authority.historicProducerLaw) {
    const historicalLaw = fs.readFileSync(path.join(directory, authority.historicProducerLaw.file));
    assert.equal(hash(historicalLaw), authority.historicProducerLaw.sha256);
    assert.equal(historicalLaw.toString().split(oldSuggestion).length, 2);
  }

  return row;
}

export function browserOriginCurrentReference(root, source, supplied) {
  const directory = path.join(root, "crates/vize/tests/fixtures/issue-7907");
  assert.equal(hash(fs.readFileSync(path.join(directory, "source.json"))), historicSha256);
  const rows = currentAuthorities.map((descriptor, index) =>
    validateAuthority(directory, source, descriptor, index === 0 ? supplied : undefined),
  );
  assert.deepEqual(
    rows.map((row) => row.path),
    ["PageTitle.vue", "UnicodeCrlf.vue"],
  );
  assert.deepEqual(rows[0].currentExpected, [
    { file: "PageTitle.vue", messages: [], errorCount: 0, warningCount: 0 },
  ]);
  assert.deepEqual(rows[0].currentDoctorLocations, []);
  assert.deepEqual(
    rows[1].currentExpected[0].messages.map(({ column, endColumn }) => [column, endColumn]),
    [
      [38, 39],
      [31, 32],
    ],
  );

  // The live Rust test is edited by other work; require the authored help
  // transition in it without pinning the rest of the file.
  const currentLaw = fs.readFileSync(
    path.join(root, "crates/vize_croquis_cf/tests/browser_diagnostic_origin.rs"),
    "utf8",
  );
  assert(currentLaw.includes(newSuggestion), "current producer law keeps the authored help");
  assert(
    !currentLaw.includes(`Some("${oldSuggestion}")`),
    "current producer law drops the old help",
  );

  return {
    authorities: currentAuthorities,
    rows,
    byPath: Object.fromEntries(rows.map((row) => [row.path, row])),
  };
}
