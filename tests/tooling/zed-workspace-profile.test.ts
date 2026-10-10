import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "tests/_fixtures/differential/lsp/zed-workspace-profile");
const manifest = JSON.parse(fs.readFileSync(path.join(fixture, "case.json"), "utf8"));

test("Zed config regression retains the complete original config and authored SFC bytes", () => {
  for (const [name, digest] of Object.entries({
    ...manifest.originalSha256,
    ...manifest.controlSha256,
  })) {
    assert.equal(
      createHash("sha256")
        .update(fs.readFileSync(path.join(fixture, name)))
        .digest("hex"),
      digest,
    );
  }
  assert.equal(manifest.issue, 8007);
  assert.equal(manifest.relatedIssue, 7196);
  assert.equal(manifest.cases.length, 6);
  assert.equal(new Set(manifest.cases.map((row: { id: string }) => row.id)).size, 6);
});

test("current Zed formatting defaults preserve the frozen baseline and narrow successor", () => {
  const policy = JSON.parse(
    fs.readFileSync(path.join(fixture, "current-default-policy.json"), "utf8"),
  );
  assert.equal(
    createHash("sha256")
      .update(fs.readFileSync(path.join(fixture, policy.historical.path)))
      .digest("hex"),
    policy.historical.sha256,
  );
  assert.deepEqual(policy.historical, {
    path: "case.json",
    sha256: "68e99ed53e60082b57b1eb2a65ae37be3157f45d465bb29c5ea4f494a44e3cf9",
    caseId: "no-config-default",
  });
  const original = manifest.cases.find(
    (row: { id: string }) => row.id === policy.historical.caseId,
  );
  const { formattedSource, ...current } = policy.currentDefault;
  assert.deepEqual(current, {
    ...original,
    id: "no-config-current-default",
    formattingProvider: true,
    formattingProviderPresent: true,
  });
  assert.equal(
    formattedSource,
    '<script setup lang="ts">\nconst total = "3";\n</script>\n\n<template>\n  {{ total }}\n</template>\n',
  );
  const flags = { ...original.initializationOptions, formatting: false };
  assert.deepEqual(policy.explicitFormattingFalse, {
    ...original,
    id: "no-config-explicit-formatting-false",
    explicit: flags,
    initializationOptions: flags,
    formattingResult: null,
  });
});
