import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";
import { sha256 } from "./lint-default-correctness-reference.mjs";
import { assertReservedApiOrderSource } from "./support/runtime-props-api-source-custody.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "tests/_fixtures/differential/lint/reserved-prefix-8090");
const read = (name) => fs.readFileSync(path.join(fixture, name));
const json = (name) => JSON.parse(read(name));
const frozen = {
  "api-order-main-source-custody.json":
    "67cd13b5da7a18095622e1f8b68d15c4d7dcb87116498c7b4d0f33eabfa68945",
  "api-order-successors.json": "f4b162011d5aa250c55b1cc0bcb33ce1ce2e87286e8cf0c8eeed7f84f353c6e6",
  "cases.json": "016061653fcd7781d87c2d542ee4d982a013e7059e1858ca9405cd589cae5d77",
  "bare-parent-successors.json": "24f0a6e69dab5ae91929d0d2b4022de46d099048ba3782f8fd8c48480485ec84",
  "default-route-companions.json":
    "07267827ba3f3fc6cf789e6939a2b58f61758198be2527fa59ece5cae2470800",
  "bare-parent-route-successors.json":
    "54610b0976da6e8571d4e170cb4593bfdca4b2d8650025fa882b795f50b3aed8",
};

await test("no-template API successors retain complete rule order and initial authority bytes", () => {
  for (const [name, digest] of Object.entries(frozen))
    assert.equal(sha256(read(name)), digest, name);
  const successor = json("api-order-successors.json");
  assert.equal(successor.schema, "vize.lint.reserved-prefix-api-order-successors");
  assert.equal(successor.version, 1);
  assert.equal(successor.sourceCommit, "344346c80b9e0e51aa1319ce9fb3e3cc37deea4e");
  assert.deepEqual(successor.scope.apiOrder, [
    "script/no-reserved-keys",
    "script/no-reserved-props",
  ]);
  assert.equal(successor.scope.productionChanged, false);
  assert.deepEqual(successor.summary, {
    coreDiagnosticCount: 211,
    bareDiagnosticCount: 11,
    diagnosticMetadataChanges: 0,
  });
  for (const authority of Object.values(successor.authorities)) {
    assert.equal(authority.sha256, frozen[authority.path]);
    assert.equal(read(authority.path).length, authority.bytes);
  }
  assert.deepEqual(
    successor.sources.map(({ path }) => path),
    [
      "crates/vize_patina/src/linter/engine/sfc.rs",
      "crates/vize_patina/src/linter/engine.rs",
      "crates/vize_patina/src/linter/script_rules.rs",
      "crates/vize_patina/src/linter/script_rules/registry/rules.rs",
    ],
  );
  const current = json("api-order-main-source-custody.json");
  assert.equal(current.schema, "vize.lint.reserved-prefix-api-main-source-custody");
  assert.equal(current.version, 1);
  assert.equal(current.originalPrefixHead, "83fb46be270ac2141d7a05d0cf1b152d306efd00");
  assert.equal(current.freshMain, "67e3d0e6775cc3a1f29bfb9309d991252f9e75ff");
  assert.equal(current.includedSetupMerge, "2284cb1d47009c5d19a4c626e7029d5c22d6bdda");
  assert.deepEqual(current.previousSources, successor.sources);
  assert.equal(current.originalAuthority.path, "api-order-successors.json");
  assert.equal(current.originalAuthority.sha256, frozen[current.originalAuthority.path]);
  assert.deepEqual(
    current.currentSources.map(({ path }) => path),
    [
      ...successor.sources.map(({ path }) => path),
      "crates/vize_patina/src/linter/script_rules/sfc.rs",
    ],
  );
  for (const source of current.currentSources) {
    assertReservedApiOrderSource(root, source);
  }
  const script = fs.readFileSync(path.join(root, current.retainedRuleIterator.path), "utf8");
  const iterator = Buffer.from(
    script
      .slice(
        script.indexOf("fn active_builtin_script_rule_entries("),
        script.indexOf("fn builtin_script_rule_entry("),
      )
      .trimEnd() + "\n",
  );
  assert.equal(sha256(iterator), current.retainedRuleIterator.sha256);
  assert.equal(iterator.length, current.retainedRuleIterator.bytes);
  const failure = read(current.refusal.path);
  assert.equal(sha256(failure), current.refusal.sha256);
  const rawFailure = gunzipSync(failure);
  assert.equal(sha256(rawFailure), current.refusal.rawSha256);
  assert.equal(rawFailure.length, current.refusal.rawBytes);
  assert.match(
    rawFailure.toString(),
    /AssertionError.*crates\/vize_patina\/src\/linter\/script_rules\.rs/,
  );
  const core = json("cases.json"),
    initialCore = json("default-route-companions.json");
  const bare = json("bare-parent-successors.json"),
    initialBare = json("bare-parent-route-successors.json");
  const bareCorpus = json("../bare-reserved-8090/cases.json");
  assert.equal(sha256(read("../bare-reserved-8090/cases.json")), bare.parentCorpus.sha256);
  const reordered = { core: [], bare: [] };
  for (const [kind, expected, target, initial, carriers, count] of [
    ["core", successor.coreCases, core.cases, initialCore.cases, core.cases, 20],
    ["bare", successor.bareCases, bare.cases, initialBare.cases, bareCorpus.cases, 16],
  ]) {
    assert.equal(expected.length, count);
    assert.equal(target.length, count);
    assert.equal(initial.length, count);
    let diagnostics = 0;
    expected.forEach((entry, index) => {
      const original = target[index],
        companion = initial[index],
        carrier = carriers[index];
      for (const field of ["id", "filename", "sourceSha256"]) {
        assert.equal(entry[field], original[field]);
        assert.equal(entry[field], carrier[field]);
      }
      assert.equal(companion.id, entry.id);
      assert.equal(companion.sourceSha256, entry.sourceSha256);
      assert.equal(sha256(Buffer.from(carrier.source)), entry.sourceSha256);
      const prefix = kind === "core" ? "" : "../bare-reserved-8090/";
      const source = read(prefix + carrier.filename + ".fixture");
      assert.equal(source.toString(), carrier.source);
      assert.equal(sha256(source), entry.sourceSha256);
      assert.equal(/<(?:template|style)(?:\s|>)/.test(carrier.source), false);
      // The full literal successor preserves every diagnostic field and order.
      assert.deepEqual(
        entry.diagnostics,
        [...original.diagnostics, ...companion.diagnostics],
        entry.id,
      );
      diagnostics += entry.diagnostics.length;
      if (!sameOrder(entry.diagnostics, companion.combinedApiDiagnostics))
        reordered[kind].push(entry.id);
    });
    assert.equal(diagnostics, kind === "core" ? 211 : 11);
  }
  assert.deepEqual(reordered, {
    core: ["exact24-props", "exact24-props-array", "quoted-escaped-props"],
    bare: [],
  });
});

function sameOrder(current, initial) {
  // Initial interleaved vectors stay readable and unchanged as historical proof.
  return JSON.stringify(current) === JSON.stringify(initial);
}
