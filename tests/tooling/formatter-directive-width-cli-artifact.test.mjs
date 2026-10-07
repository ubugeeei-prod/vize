import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { preservedDirectiveWidthCliManifest } from "../differential/formatter-directive-width-cli-artifact.ts";
import { sha256 } from "../differential/manifest.mjs";
import {
  loadFormatterHistoryAudit,
  validateFormatterHistoryAudit,
} from "../differential/formatter-history-audit.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const manifest = "tests/_fixtures/differential/formatter/manifest.json";
const asset =
  "tests/_fixtures/differential/formatter-regressions/directive-print-width-7876/shared-manifest.eed471b.original.json.txt";
const pin = {
  path: manifest,
  sha256: "40acde7c1eba953724a1a05b60f4744948bb5f48687ed43272981e30bf5c65a4",
  cases: 13,
};

void test("one closed current CLI extension preserves every original manifest byte", (t) => {
  const current = fs.readFileSync(path.join(root, manifest));
  const original = fs.readFileSync(path.join(root, asset));
  assert.deepEqual(preservedDirectiveWidthCliManifest(root, pin, current), original);
  assert.equal(sha256(original), pin.sha256);
  assert.equal(preservedDirectiveWidthCliManifest(root, pin, original), null);
  for (const change of [
    { ...pin, sha256: "0".repeat(64) },
    { ...pin, cases: 14 },
  ])
    assert.throws(() => preservedDirectiveWidthCliManifest(root, change, current));
  assert.equal(
    preservedDirectiveWidthCliManifest(root, { ...pin, path: "another-manifest.json" }, current),
    null,
  );
  for (const mutate of [
    (copy) => copy.cases.pop(),
    (copy) => copy.cases.push(copy.cases.at(-1)),
    (copy) => copy.cases.reverse(),
    (copy) => (copy.cases[0].inputs.files[0].sha256 = "0".repeat(64)),
    (copy) => (copy.cases[0].expectations.legacy.artifacts[0].sha256 = "0".repeat(64)),
    (copy) => (copy.adapterOptions.passes = 1),
    (copy) => (copy.cases.at(-1).adapters.native = "forged-native"),
  ]) {
    const copy = JSON.parse(current.toString());
    mutate(copy);
    assert.throws(() =>
      preservedDirectiveWidthCliManifest(root, pin, Buffer.from(JSON.stringify(copy))),
    );
  }
  assert.throws(() =>
    preservedDirectiveWidthCliManifest(root, pin, Buffer.concat([current, Buffer.from("\n")])),
  );
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "directive-width-cli-authority-"));
  t.after(() => fs.rmSync(scratch, { recursive: true, force: true }));
  const target = path.join(scratch, asset);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, original);
  assert.deepEqual(preservedDirectiveWidthCliManifest(scratch, pin, current), original);
  fs.appendFileSync(target, "\n");
  assert.throws(() => preservedDirectiveWidthCliManifest(scratch, pin, current));
  fs.unlinkSync(target);
  fs.symlinkSync(path.join(root, asset), target);
  assert.throws(() => preservedDirectiveWidthCliManifest(scratch, pin, current));
});

void test("historical audit rejects metadata rebasing and a redirected frozen manifest", () => {
  const { audit } = loadFormatterHistoryAudit(root);
  const current = fs.readFileSync(path.join(root, manifest));
  for (const changed of [
    { ...audit.cliManifestPin, path: asset },
    { ...audit.cliManifestPin, sha256: sha256(current), cases: 14 },
  ]) {
    const copy = structuredClone(audit);
    copy.cliManifestPin = changed;
    assert.throws(() => validateFormatterHistoryAudit(copy, root));
  }
});
