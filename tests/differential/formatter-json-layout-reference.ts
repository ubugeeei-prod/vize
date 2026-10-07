// Only six complete historical JSON outputs change for #7928. Historical
// bytes and source witnesses remain immutable and separately compared.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "./manifest.mjs";

const IDS = [
  "literal/empty-collections-stay-compact/0",
  "literal/honors-custom-indent-width/0",
  "literal/jsonc-drops-trailing-comma/0",
  "literal/preserves-key-order-from-source/0",
  "literal/preserves-valid-number-tokens/0",
  "literal/pretty-prints-minified-object/0",
];
const AUTHORITY = "tests/_fixtures/differential/formatter-history/current-references-7928.json";
const AUTHORITY_SHA256 = "07c65e17b175d4fc037ec876955369b69a5ee48c2f3574ffa78442638ecfc554";

export function jsonLayoutCurrentReference(
  root: string,
  fixture: any,
  input: Buffer,
  historical: Buffer,
) {
  if (!IDS.includes(fixture.id)) return {};
  const bytes = fs.readFileSync(path.join(root, AUTHORITY));
  assert.equal(sha256(bytes), AUTHORITY_SHA256, "reviewed JSON layout authority changed");
  const authority = JSON.parse(bytes.toString());
  assert.equal(authority.schema, "vize.formatter-current-reference-refinements");
  assert.equal(authority.version, 1);
  assert.equal(authority.issue, 7928);
  assert.deepEqual(
    authority.cases.map((row: any) => row.id),
    IDS,
  );
  const row = authority.cases[IDS.indexOf(fixture.id)];
  assert.equal(fixture.api, row.api);
  assert.equal(fixture.kind, row.kind);
  assert.equal(fixture.profile, "default");
  assert.equal(fixture.outcome, undefined);
  assert.deepEqual(fixture.options, row.options, "original JSON layout options changed");
  assert.deepEqual(fixture.witness, row.witness, "original JSON layout witness changed");
  assert.equal(sha256(input), row.inputSha256, "original JSON layout input changed");
  assert.equal(sha256(historical), row.historicalExpectedSha256, "historical JSON output changed");
  const current = Buffer.from(row.currentExpected);
  assert.equal(sha256(current), row.currentExpectedSha256, "current JSON output changed");
  assert(!current.equals(historical), "historical JSON mismatch must remain observable");
  if (fixture.api === "format_json") {
    assert.deepEqual(JSON.parse(current.toString()), JSON.parse(input.toString()));
  }
  return {
    currentExpected: current,
    currentReference: {
      issue: 7928,
      witness: row.witness,
      authority: { path: AUTHORITY, sha256: AUTHORITY_SHA256 },
      inputSha256: row.inputSha256,
      historicalExpectedSha256: row.historicalExpectedSha256,
      currentExpectedSha256: row.currentExpectedSha256,
      provenance: row.provenance,
      justification: row.justification,
    },
  };
}
