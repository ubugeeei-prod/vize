import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  assertWarmupWitness,
  currentWarmupWitness,
  HISTORICAL_WARMUP_SHA,
  validateWarmupWitness,
  warmupWitnessCustody,
  WARMUP_WITNESS_PATH,
} from "./support/lsp/native-warmup-witness.ts";
import { sha256 } from "../differential/harness.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const directory = path.join(root, "tests/_fixtures/tooling/native-diagnostic-warmup-witness-11815");
const archive = fs.readFileSync(path.join(directory, "historical-witness.ts.txt"));
const current = fs.readFileSync(path.join(root, WARMUP_WITNESS_PATH));
const custody = JSON.parse(fs.readFileSync(path.join(directory, "custody.json"), "utf8"));
const oldCase = JSON.parse(
  fs.readFileSync(
    path.join(root, "tests/_fixtures/differential/lsp/native-diagnostic-warmup-10604/case.json"),
    "utf8",
  ),
);
const witness = oldCase.provenance.warmupWitness;

test("historical source remains exact and the current executor has separately derived custody", () => {
  assert.equal(sha256(archive), HISTORICAL_WARMUP_SHA);
  assert.deepEqual(currentWarmupWitness(archive), current);
  assert.deepEqual(assertWarmupWitness(root, witness), custody);
  assert.equal(custody.historical.sha256, witness.sha256);
  assert.notEqual(custody.currentConsumer.sha256, witness.sha256);
  assert.match(custody.currentConsumer.meaning, /no backend or stdio qualification/);
});

test("a foreign historical path or directly repinned historical witness is refused", () => {
  for (const changed of [
    { ...witness, path: "tests/tooling/another-warmup.test.ts" },
    { ...witness, sha256: sha256(current) },
  ]) {
    assert.throws(() => validateWarmupWitness(changed, archive, current, custody));
  }
});

test("altered archived bytes cannot become the historical law", () => {
  assert.throws(() => currentWarmupWitness(Buffer.concat([archive, Buffer.from("\n")])));
});

test("repinning current metadata cannot admit a different diagnostic predicate", () => {
  const changed = Buffer.from(current.toString("utf8").replace("=== 2739", "=== 2322"));
  assert.notDeepEqual(changed, current);
  assert.throws(() =>
    validateWarmupWitness(witness, archive, changed, warmupWitnessCustody(changed)),
  );
});

test("both post-resume cursors remain mandatory even with a matching new hash", () => {
  for (const cursor of ["resumedAt", "admittedResumedAt"]) {
    const changed = Buffer.from(
      current
        .toString("utf8")
        .replace(`const ${cursor} = notifications.length;`, `const ${cursor} = 0;`),
    );
    assert.notDeepEqual(changed, current);
    assert.throws(() =>
      validateWarmupWitness(witness, archive, changed, warmupWitnessCustody(changed)),
    );
  }
});

test("unknown provenance and removal of the authentication call are refused", () => {
  const unknown = structuredClone(custody);
  unknown.version = 2;
  assert.throws(() => validateWarmupWitness(witness, archive, current, unknown));
  const changed = Buffer.from(
    current
      .toString("utf8")
      .replace(
        "const warmupWitnessCustody = assertWarmupWitness(root, fixture.provenance.warmupWitness);",
        "const warmupWitnessCustody = {};",
      ),
  );
  assert.notDeepEqual(changed, current);
  assert.throws(() =>
    validateWarmupWitness(witness, archive, changed, warmupWitnessCustody(changed)),
  );
});
