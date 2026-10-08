// Closed current references preserve complete original manifests and outputs.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";

const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const DIRECTORY = "tests/_fixtures/differential/formatter-regressions/sole-child-width-7876";
const AUTHORITY = `${DIRECTORY}/current-vue-references.json`;
const HASH = "bf8c553b9fd93908061acbeb8a239a0ee945c7f59209ce25d406d53ef28a3d36";
const CLI_MANIFEST = "tests/_fixtures/differential/formatter/manifest.json";
const CLI_HASH = "6ab3a71cbb9ec75a4073a315c1f9610b12673c60ff2178517aed94cbb60621e0";
const API_MANIFEST = "tests/_fixtures/differential/formatter-history/vue-version-manifest.json";
const API_HASH = "e18b9c2595b7a5dbbe655930d83692e2ecdb04e1a60e0e7a2ec490577e0c88fc";
const OWNER = "crates/vize_glyph/tests/vue2_filters.rs";
const ORIGINAL_OWNER = "12e3264f660e8d19aee1c949e0789a8137eb6c0f31ec6ef8ee1e30fa2815cb95";
const CURRENT_OWNER = "9223c36becb9c341533eef9029d9a60607806ae05977119ae5ec61376dd92d11";
const IDS = ["sfc-vue2-filter-chain-crlf", "sfc-vue3-bitwise-or"];
const API_IDS = ["vue-version/sfc/filter-chain-crlf-v2", "vue-version/sfc/bitwise-or-v3"];

function bytes(root, relative) {
  const file = fs.realpathSync(path.join(root, relative));
  const contained = path.relative(fs.realpathSync(root), file);
  assert(contained && !contained.startsWith("..") && !path.isAbsolute(contained));
  return fs.readFileSync(file);
}

export function soleChildVueAuthority(root) {
  const raw = bytes(root, AUTHORITY);
  assert.equal(hash(raw), HASH, "closed Vue current authority changed");
  const result = JSON.parse(raw.toString());
  assert.equal(result.schema, "vize.independent-vue-history-current-reference");
  assert.equal(result.version, 1);
  assert.deepEqual(
    result.records.map((row) => row.id),
    [
      IDS[0],
      "sfc-vue2-7-filter-chain",
      IDS[1],
      "native-hardcoded-2",
      "native-hardcoded-2.7",
      "native-hardcoded-3",
    ],
  );
  assert.equal(hash(bytes(root, CLI_MANIFEST)), CLI_HASH, "whole shared16 manifest changed");
  assert.equal(hash(bytes(root, API_MANIFEST)), API_HASH, "whole Vue14 manifest changed");
  assert.equal(result.nativeOwner.path, "tests/tooling/glyph-vue2-native.test.ts");
  assert.equal(hash(Buffer.from(result.nativeOwner.content)), result.nativeOwner.sha256);
  assert.equal(Buffer.byteLength(result.nativeOwner.content), result.nativeOwner.bytes);
  assert.deepEqual(
    bytes(root, `${DIRECTORY}/glyph-vue2-native.original.ts.txt`),
    Buffer.from(result.nativeOwner.content),
    "complete original native law owner changed",
  );
  for (const row of result.records) {
    for (const asset of [row.input, row.historical, row.config].filter(Boolean)) {
      assert.equal(hash(Buffer.from(asset.content)), asset.sha256);
      if (Object.hasOwn(asset, "bytes"))
        assert.equal(Buffer.byteLength(asset.content), asset.bytes);
      // Native records name the owner of a literal, rather than an SFC asset.
      // Its complete original owner is held separately above.
      if (row.id.startsWith("native-hardcoded-")) {
        assert.equal(asset.path, result.nativeOwner.path);
      } else if (asset.path) {
        assert.deepEqual(bytes(root, asset.path), Buffer.from(asset.content));
      }
    }
    assert.equal(hash(Buffer.from(row.expected)), row.expectedSha256);
  }
  return result;
}

function currentReference(root, id, input, historical) {
  const row = soleChildVueAuthority(root).records.find((record) => record.id === id);
  assert(IDS.includes(id), "unregistered shared current reference");
  assert.equal(hash(input), row.input.sha256, "original whole input changed");
  assert.equal(hash(historical), row.historical.sha256, "whole historical output changed");
  const file = path.posix.join(
    path.posix.dirname(row.historical.path),
    "sole-child-width.current.expected.txt",
  );
  const currentExpected = bytes(root, file);
  assert.deepEqual(
    currentExpected,
    Buffer.from(row.expected),
    "authored whole current bytes changed",
  );
  assert.equal(hash(currentExpected), row.expectedSha256);
  assert(!currentExpected.equals(historical), "historical mismatch remains observable");
  return {
    currentExpected,
    currentReference: {
      issue: 7876,
      authority: { path: AUTHORITY, sha256: HASH },
      inputSha256: row.input.sha256,
      historicalExpectedSha256: row.historical.sha256,
      currentExpectedSha256: row.expectedSha256,
      justification: row.authority,
    },
  };
}

export function soleChildWidthCliReference(root, fixture) {
  const id = fixture.id.replace(/^formatter\/sfc\/vue/, "sfc-vue");
  if (!IDS.includes(id)) return {};
  assert.equal(fixture.id, `formatter/sfc/${id.slice("sfc-".length)}`);
  const row = soleChildVueAuthority(root).records.find((record) => record.id === id);
  const original = JSON.parse(bytes(root, CLI_MANIFEST).toString()).cases.find(
    (record) => record.id === fixture.id,
  );
  for (const [field, value] of Object.entries(original))
    assert.deepEqual(fixture[field], value, `original CLI ${field} changed`);
  assert.deepEqual(fixture.argv, ["fmt", "--config", "vize.config.json", "--write", "App.vue"]);
  assert.deepEqual(
    fixture.config.map(({ path: filename, sha256 }) => ({ path: filename, sha256 })),
    [{ path: "vize.config.json", sha256: row.config.sha256 }],
  );
  assert.deepEqual(
    fixture.config[0].bytes,
    Buffer.from(row.config.content),
    "complete executed CLI config bytes changed",
  );
  assert.equal(hash(fixture.config[0].bytes), row.config.sha256);
  return currentReference(root, id, fixture.input, fixture.expected);
}

export function soleChildWidthApiReference(root, fixture, input, historical) {
  if (!API_IDS.includes(fixture.id)) return {};
  const original = JSON.parse(bytes(root, API_MANIFEST).toString()).cases.find(
    (row) => row.id === fixture.id,
  );
  for (const [field, value] of Object.entries(original))
    assert.deepEqual(fixture[field], value, `original API ${field} changed`);
  assert.equal(fixture.outcome, undefined);
  return currentReference(root, IDS[API_IDS.indexOf(fixture.id)], input, historical);
}

function preservedOwner(root, originalHash) {
  assert.equal(originalHash, ORIGINAL_OWNER, "original Vue law owner pin changed");
  const original = bytes(root, `${DIRECTORY}/vue2_filters.original.rs.txt`);
  assert.equal(hash(original), ORIGINAL_OWNER, "complete original Vue law asset changed");
  const current = bytes(root, OWNER);
  assert.equal(hash(current), CURRENT_OWNER, "complete current Vue law owner changed");
  let restored = current.toString();
  for (const id of IDS) {
    const prefix = `tests/_fixtures/differential/formatter/${id}/`;
    const after = `${prefix}sole-child-width.current.expected.txt`;
    assert.equal(restored.split(after).length - 1, 1, "expected path transition is not unique");
    restored = restored.replace(after, `${prefix}reference.expected.txt`);
  }
  assert.deepEqual(Buffer.from(restored), original, "only two expected-path literals may change");
  soleChildVueAuthority(root);
  return original;
}

export function resolveSoleChildVueSource(root, artifact) {
  return artifact.path === OWNER ? preservedOwner(root, artifact.sha256) : null;
}

export function validateSoleChildVueWitness(root, entry, name) {
  if (entry.path !== OWNER) return false;
  assert.deepEqual(
    entry.revisions,
    ["c681035734fa84c223ee77395ea26bdd07297fb2", "cc87bb5960ea9e49e82672205df919de58bb4b24"],
    "original Vue law revisions changed",
  );
  assert(
    [
      "configured_sfc_filter_corpus_preserves_every_byte_and_reaches_a_fixed_point",
      "explicit_filter_version_preserves_hyphenated_names_and_default_is_vue3",
      "pipes_in_javascript_payloads_and_events_keep_their_javascript_meaning",
    ].includes(name),
    "unregistered current Vue law",
  );
  const original = preservedOwner(root, entry.sha256);
  assert(original.toString().includes(`fn ${name}(`));
  return true;
}
