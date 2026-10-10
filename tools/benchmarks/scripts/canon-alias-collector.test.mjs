import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { cases } from "./canon-alias-corpus.mjs";
import {
  expectedFiles,
  sourceContract,
  successor,
  successorSha256,
  successorUrl,
} from "./canon-alias-collector.mjs";
import { captureAliasCorpus, main } from "./canon-alias-native.mjs";

const resolver = {
  historical: "5a43aa38a8beb0f4005afb4e7ef94e2fcf96dfef53c9b19166471d10e7add833",
  selected: "3d97f55712486403c0accc0db1121391ab70140177deb8630ce591e4df4781e8",
};
const collector = {
  historical: "5a612ebb5f2e263a404c8692fc5847862929842e07d837c0e6eb1d22bf4ffd5b",
  selected: "f2f15711265c21874d604f5b4b32d9f0c3d138ec9b353a35f5b21176e8c0a63e",
};

void test("the successor preserves the original corpus and pins both independent source axes", () => {
  assert.equal(successorSha256, "3e8eb8c815da36acd27f2252461e39ec0786f88557f01eb5849371d173bce295");
  assert.deepEqual(successor.resolverSources, resolver);
  assert.deepEqual(successor.collectorSources, collector);
  for (const resolverMode of ["historical", "selected"]) {
    for (const collectorMode of ["historical", "selected"]) {
      assert.deepEqual(sourceContract(resolver[resolverMode], collector[collectorMode], "base"), {
        side: "base",
        resolverSha256: resolver[resolverMode],
        collectorSha256: collector[collectorMode],
        resolverMode,
        collectorMode,
      });
    }
  }
  assert.throws(
    () => sourceContract("0".repeat(64), collector.selected, "base"),
    /Unknown Canon resolver/,
  );
  assert.throws(
    () => sourceContract(resolver.selected, "0".repeat(64), "base"),
    /Unknown CLI collector/,
  );
  assert.throws(
    () => sourceContract(resolver.historical, collector.selected, "head"),
    /Head must satisfy selected resolver/,
  );
  assert.throws(
    () => sourceContract(resolver.selected, collector.historical, "head"),
    /Head must satisfy selected CLI membership/,
  );
});

void test("only the two missing-selected memberships change; all original case fields stay exact", () => {
  const before = structuredClone(cases);
  const old = sourceContract(resolver.selected, collector.historical, "base");
  const current = sourceContract(resolver.selected, collector.selected, "head");
  assert.deepEqual(
    cases.map((item) => [item.id, expectedFiles(item, old)]),
    [
      ["exact-clean", ["App.tsx", "exact.tsx"]],
      ["exact-broken", ["App.tsx", "exact.tsx"]],
      ["exact-equal-rank", ["App.tsx", "exact.tsx"]],
      ["exact-target-array-fallback", ["App.tsx", "exact.tsx"]],
      ["selected-exact-missing", ["App.tsx", "wildcard.tsx"]],
      ["selected-longer-wildcard-missing", ["App.tsx", "genericx.tsx"]],
      ["unaffected-exact-clean", ["App.tsx", "exact.tsx"]],
      ["unaffected-exact-broken", ["App.tsx", "exact.tsx"]],
    ],
  );
  assert.deepEqual(
    cases.map((item) => [item.id, expectedFiles(item, current)]),
    [
      ["exact-clean", ["App.tsx", "exact.tsx"]],
      ["exact-broken", ["App.tsx", "exact.tsx"]],
      ["exact-equal-rank", ["App.tsx", "exact.tsx"]],
      ["exact-target-array-fallback", ["App.tsx", "exact.tsx"]],
      ["selected-exact-missing", ["App.tsx"]],
      ["selected-longer-wildcard-missing", ["App.tsx"]],
      ["unaffected-exact-clean", ["App.tsx", "exact.tsx"]],
      ["unaffected-exact-broken", ["App.tsx", "exact.tsx"]],
    ],
  );
  assert.deepEqual(cases, before);
  assert.throws(() => expectedFiles({ ...cases[0], after: 2307 }, current), /entire original case/);
  assert.throws(
    () => expectedFiles(cases[0], { ...current, collectorMode: "historical" }),
    /complete source-bound contract/,
  );
});

void test("unknown actual source rejects before preparation can execute a trap CLI", () => {
  const root = mkdtempSync(join(os.tmpdir(), "canon-collector-source-control-"));
  const marker = join(root, "CLI-MUST-NOT-RUN");
  const resolverPath = "crates/vize_canon/src/batch/virtual_project/dependency_scan/resolution.rs";
  const collectorPath = "crates/vize/src/commands/check/imports_aliases.rs";
  const oldCollector = execFileSync("tar", [
    "-xOzf",
    fileURLToPath(new URL("historical-eight-case.tar.gz", successorUrl)),
    collectorPath,
  ]);
  assert.equal(createHash("sha256").update(oldCollector).digest("hex"), collector.historical);
  const selectedResolver = readFileSync(new URL(`../../../${resolverPath}`, import.meta.url));
  const put = (side, path, bytes) => {
    const target = join(root, side, path);
    mkdirSync(dirname(target), { recursive: true });
    writeFileSync(target, bytes);
  };
  try {
    for (const side of ["base", "head"]) {
      put(side, resolverPath, selectedResolver);
      put(side, collectorPath, oldCollector);
      put(side, "target/ci-opt/vize", `#!/bin/sh\n: > '${marker}'\nexit 0\n`);
      chmodSync(join(root, side, "target/ci-opt/vize"), 0o755);
    }
    for (const [id, source, pattern] of [
      ["unknown-resolver", resolverPath, /Unknown Canon resolver/],
      ["unknown-collector", collectorPath, /Unknown CLI collector/],
      ["known-wrong-head", null, /Head must satisfy selected CLI membership/],
    ]) {
      put("head", resolverPath, selectedResolver);
      put("head", collectorPath, oldCollector);
      if (source) put("head", source, "// unqualified source\n");
      assert.throws(
        () =>
          main([
            join(root, "base/target/ci-opt/vize"),
            join(root, "head/target/ci-opt/vize"),
            join(root, `${id}.json`),
          ]),
        pattern,
      );
      const failure = JSON.parse(readFileSync(join(root, `${id}.json.samples/failure.json`)));
      assert.match(failure.message, pattern);
      assert.equal(existsSync(marker), false);
    }
    assert.throws(() => captureAliasCorpus({ commands: { base: [], head: [] } }), TypeError);
    assert.equal(existsSync(marker), false);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
