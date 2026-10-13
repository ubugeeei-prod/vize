import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  ownedCorsaCurrentBaseline,
  ownedCorsaNativeGatesMainBaseline,
  ownedCorsaMergedMainBaseline,
  ownedCorsaLatestMergedBaseline,
  ownedCorsaDocumentationMainBaseline,
  ownedCorsaRecoveryMainBaseline,
  ownedCorsaMuseaDocumentationMainBaseline,
  ownedCorsaAccessibilityDocumentationMainBaseline,
  ownedCorsaNoMutatingPropsMainBaseline,
  ownedCorsaJapaneseComponentDocumentationMainBaseline,
  retainedOwnedCorsaBaseline,
} from "../../tools/support/compat/github/owned-corsa-current-baseline.mjs";
import {
  ownedCorsaSource,
  qualifyOwnedCorsaSource,
} from "../../tools/support/compat/github/owned-corsa-source-custody.mjs";
import { ownedCorsaReader } from "../../tools/support/compat/github/owned-corsa-source-reader.mjs";
import { ensureHostedLiteralSource } from "../performance/support/warm-type-backed-git-reader-literal-source.ts";
import { nativeLexicalCompositionBodyPairs } from "../performance/support/warm-type-backed-native-lexical-composition-host.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const git = (cwd, args, options = {}) =>
  execFileSync("git", ["--no-replace-objects", ...args], { cwd, ...options });
const head = git(root, ["rev-parse", "HEAD"]).toString().trim();
const baseline = ownedCorsaCurrentBaseline.revision;
for (const revision of [
  baseline,
  ownedCorsaSource.baseline,
  ownedCorsaMergedMainBaseline.revision,
  ownedCorsaLatestMergedBaseline.revision,
  ownedCorsaDocumentationMainBaseline.revision,
  ownedCorsaRecoveryMainBaseline.revision,
  ownedCorsaMuseaDocumentationMainBaseline.revision,
  ownedCorsaAccessibilityDocumentationMainBaseline.revision,
  ownedCorsaNoMutatingPropsMainBaseline.revision,
  ownedCorsaJapaneseComponentDocumentationMainBaseline.revision,
  ownedCorsaNativeGatesMainBaseline.revision,
])
  ensureHostedLiteralSource(root, revision);
const reader = ownedCorsaReader(root, baseline, head);
const input = {
  baseline: retainedOwnedCorsaBaseline(root, baseline),
  production: ownedCorsaSource.rows.map(({ file }) => file),
  alreadyQualified: new Set(),
  reader,
  historicalPairs: nativeLexicalCompositionBodyPairs,
};

void test("literal current main and all474 raw/physical pair cells retain the original237 authority", () => {
  assert.equal(input.baseline, ownedCorsaSource.baseline);
  const result = qualifyOwnedCorsaSource(input);
  assert(result);
  assert.equal(result.completeProductionBodies, 237);
  assert.equal(result.runtimeCredit, false);
  assert.equal(result.performanceCredit, false);
  assert.equal(
    retainedOwnedCorsaBaseline(root, ownedCorsaSource.baseline),
    ownedCorsaSource.baseline,
  );
  for (const context of [
    ownedCorsaNativeGatesMainBaseline,
    ownedCorsaMergedMainBaseline,
    ownedCorsaLatestMergedBaseline,
    ownedCorsaDocumentationMainBaseline,
    ownedCorsaRecoveryMainBaseline,
    ownedCorsaMuseaDocumentationMainBaseline,
    ownedCorsaAccessibilityDocumentationMainBaseline,
    ownedCorsaNoMutatingPropsMainBaseline,
    ownedCorsaJapaneseComponentDocumentationMainBaseline,
  ]) {
    const baseline = context.revision;
    assert.equal(retainedOwnedCorsaBaseline(root, baseline), ownedCorsaSource.baseline);
    assert(qualifyOwnedCorsaSource({ ...input, reader: ownedCorsaReader(root, baseline, head) }));
  }
});

void test("unknown identities and partial or foreign before cells cannot borrow the current context", () => {
  for (const unknown of ["f".repeat(40), head]) {
    const retained = retainedOwnedCorsaBaseline(root, unknown);
    assert.equal(retained, unknown);
    assert.equal(qualifyOwnedCorsaSource({ ...input, baseline: retained }), null);
  }
  const file = "Cargo.lock";
  const before = reader("before", file);
  assert(before);
  for (const changed of [null, { ...before, body: Buffer.from("foreign complete lock") }])
    assert.equal(
      qualifyOwnedCorsaSource({
        ...input,
        reader: (side, member) =>
          side === "before" && member === file ? changed : reader(side, member),
      }),
      null,
    );
  assert.equal(qualifyOwnedCorsaSource({ ...input, production: input.production.slice(1) }), null);
});

void test("missing raw commit and genuine private replacement refs refuse before projection", (t) => {
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "owned-corsa-current-base-"));
  t.after(() => fs.rmSync(temp, { recursive: true, force: true }));
  const privateRoot = path.join(temp, "repo.git");
  git(temp, ["init", "--bare", privateRoot], { stdio: "pipe" });
  assert.throws(() => retainedOwnedCorsaBaseline(privateRoot, baseline));
  const raw = git(root, ["cat-file", "commit", baseline]);
  const actual = git(privateRoot, ["hash-object", "-w", "-t", "commit", "--stdin"], {
    input: raw,
  })
    .toString()
    .trim();
  assert.equal(actual, baseline);
  assert.equal(retainedOwnedCorsaBaseline(privateRoot, baseline), ownedCorsaSource.baseline);
  const changed = Buffer.concat([raw, Buffer.from("\nforeign source message\n")]);
  const foreign = git(privateRoot, ["hash-object", "-w", "-t", "commit", "--stdin"], {
    input: changed,
  })
    .toString()
    .trim();
  git(privateRoot, ["update-ref", "refs/replace/" + baseline, foreign]);
  assert.throws(() => retainedOwnedCorsaBaseline(privateRoot, baseline));
  git(privateRoot, ["update-ref", "-d", "refs/replace/" + baseline]);
  assert.equal(retainedOwnedCorsaBaseline(privateRoot, baseline), ownedCorsaSource.baseline);
});
