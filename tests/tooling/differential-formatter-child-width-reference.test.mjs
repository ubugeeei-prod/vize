import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { loadFormatterManifest } from "../differential/manifest.mjs";
import {
  soleChildVueAuthority,
  soleChildWidthApiReference,
  soleChildWidthCliReference,
  resolveSoleChildVueSource,
  validateSoleChildVueWitness,
} from "../differential/formatter-sole-child-width-reference.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const directory = "tests/_fixtures/differential/formatter-regressions/sole-child-width-7876";
const apiPath = "tests/_fixtures/differential/formatter-history/vue-version-manifest.json";
const cliPath = "tests/_fixtures/differential/formatter/manifest.json";
const owner = "crates/vize_glyph/tests/vue2_filters.rs";
const originalHash = "12e3264f660e8d19aee1c949e0789a8137eb6c0f31ec6ef8ee1e30fa2815cb95";

void test("sole-child Vue references retain old whole bytes and reject altered invocation custody", (t) => {
  const authority = soleChildVueAuthority(root);
  const cli = loadFormatterManifest(path.join(root, cliPath));
  const api = JSON.parse(fs.readFileSync(path.join(root, apiPath))).cases;
  for (const [cliId, apiId] of [
    ["formatter/sfc/vue2-filter-chain-crlf", "vue-version/sfc/filter-chain-crlf-v2"],
    ["formatter/sfc/vue3-bitwise-or", "vue-version/sfc/bitwise-or-v3"],
  ]) {
    const fixture = cli.cases.find((row) => row.id === cliId);
    const result = soleChildWidthCliReference(root, fixture);
    assert.equal(result.currentReference.issue, 7876);
    assert(!result.currentExpected.equals(fixture.expected));
    assert.equal(
      result.currentReference.historicalExpectedSha256,
      fixture.expectations.legacy.artifacts[0].sha256,
    );
    const original = api.find((row) => row.id === apiId);
    assert.deepEqual(
      soleChildWidthApiReference(root, original, fixture.input, fixture.expected),
      result,
    );
    for (const mutate of [
      (row) => {
        row.argv = ["fmt", "--no-config", "--write", "App.vue"];
      },
      (row) => {
        row.config = [];
      },
      (row) => {
        row.config[0].sha256 = "0".repeat(64);
      },
      (row) => {
        row.config[0].bytes = Buffer.from(
          '{"vue":{"version":"2"},"formatter":{"printWidth":90}}\n',
        );
      },
      (row) => {
        row.input = Buffer.concat([row.input, Buffer.from("\n")]);
      },
      (row) => {
        row.expected = Buffer.concat([row.expected, Buffer.from("\n")]);
      },
      (row) => {
        row.witnesses = [];
      },
      (row) => {
        row.provenance = {};
      },
      (row) => {
        row.adapters = {};
      },
      (row) => {
        row.inputs = {};
      },
    ]) {
      const altered = {
        ...fixture,
        argv: [...fixture.argv],
        config: fixture.config.map((row) => ({ ...row })),
      };
      mutate(altered);
      assert.throws(() => soleChildWidthCliReference(root, altered));
    }
    for (const mutate of [
      (row) => {
        row.vueVersion = "2.7";
      },
      (row) => {
        row.options.userOverrides.printWidth = 101;
      },
      (row) => {
        row.witness.function = "invented";
      },
      (row) => {
        row.input.sha256 = "0".repeat(64);
      },
      (row) => {
        row.expected.sha256 = "0".repeat(64);
      },
      (row) => {
        row.originalInputWitnesses = [];
      },
      (row) => {
        row.outcome = { kind: "typed-error" };
      },
    ]) {
      const altered = structuredClone(original);
      mutate(altered);
      assert.throws(() =>
        soleChildWidthApiReference(root, altered, fixture.input, fixture.expected),
      );
    }
  }
  assert.deepEqual(
    soleChildWidthCliReference(
      root,
      cli.cases.find((row) => row.id === "formatter/sfc/vue2-7-filter-chain"),
    ),
    {},
  );
  assert.deepEqual(
    soleChildWidthApiReference(root, { id: "unknown" }, Buffer.alloc(0), Buffer.alloc(0)),
    {},
  );
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "vize-child-width-reference-"));
  t.after(() => fs.rmSync(scratch, { recursive: true, force: true }));
  const files = [
    cliPath,
    apiPath,
    owner,
    `${directory}/current-vue-references.json`,
    `${directory}/vue2_filters.original.rs.txt`,
    `${directory}/glyph-vue2-native.original.ts.txt`,
  ];
  for (const row of authority.records.slice(0, 3)) {
    files.push(row.input.path, row.historical.path, row.config.path);
    if (row.referenceChanged)
      files.push(
        path.posix.join(
          path.posix.dirname(row.historical.path),
          "sole-child-width.current.expected.txt",
        ),
      );
  }
  for (const relative of files) {
    const target = path.join(scratch, relative);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.copyFileSync(path.join(root, relative), target);
  }
  const first = cli.cases.find((row) => row.id === "formatter/sfc/vue2-filter-chain-crlf");
  const artifact = { path: owner, sha256: originalHash };
  assert.deepEqual(
    resolveSoleChildVueSource(scratch, artifact),
    fs.readFileSync(path.join(root, `${directory}/vue2_filters.original.rs.txt`)),
  );
  const entry = {
    ...artifact,
    revisions: [
      "c681035734fa84c223ee77395ea26bdd07297fb2",
      "cc87bb5960ea9e49e82672205df919de58bb4b24",
    ],
  };
  assert(
    validateSoleChildVueWitness(
      scratch,
      entry,
      "configured_sfc_filter_corpus_preserves_every_byte_and_reaches_a_fixed_point",
    ),
  );
  assert.throws(() => validateSoleChildVueWitness(scratch, entry, "invented"));
  assert.throws(() => resolveSoleChildVueSource(scratch, { ...artifact, sha256: "0".repeat(64) }));
  const controls = [
    cliPath,
    apiPath,
    `${directory}/current-vue-references.json`,
    `${directory}/glyph-vue2-native.original.ts.txt`,
    authority.records[0].input.path,
    authority.records[0].historical.path,
    authority.records[0].config.path,
    path.posix.join(
      path.posix.dirname(authority.records[0].historical.path),
      "sole-child-width.current.expected.txt",
    ),
  ];
  for (const relative of controls) {
    const target = path.join(scratch, relative),
      original = fs.readFileSync(target);
    try {
      fs.appendFileSync(target, "\n");
      assert.throws(() => soleChildWidthCliReference(scratch, first), relative);
    } finally {
      fs.writeFileSync(target, original);
    }
  }
  for (const relative of [owner, `${directory}/vue2_filters.original.rs.txt`]) {
    const target = path.join(scratch, relative),
      original = fs.readFileSync(target);
    try {
      fs.appendFileSync(target, "\n");
      assert.throws(() => resolveSoleChildVueSource(scratch, artifact), relative);
    } finally {
      fs.writeFileSync(target, original);
    }
  }
});
