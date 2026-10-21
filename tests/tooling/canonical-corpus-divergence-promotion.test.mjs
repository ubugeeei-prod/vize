import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import {
  createPromotionPlan,
  publishPromotion,
} from "../../tools/support/compat/github/canonical-corpus-divergence-promotion.mjs";

const sourceCommit = "a".repeat(40);
const context = { sourceCommit };

function reductionDirectory(status = "reduced") {
  const directory = mkdtempSync(join(tmpdir(), "vize-promotion-"));
  const witness = "<template><div/></template>";
  if (status === "reduced") {
    writeFileSync(join(directory, "canonical-dom-divergence.reduced.vue"), witness);
  }
  writeFileSync(
    join(directory, "reduction.json"),
    JSON.stringify({
      schema: "vize.realProjectDivergenceReductionReceipt",
      version: 1,
      status,
      divergenceCount: status === "reduced" ? 1 : 0,
      fingerprint: status === "reduced" ? "abc123abc123" : undefined,
      sourcePath: "tests/_fixtures/_git/example/src/App.vue",
      sourceBytes: 200,
      reducedFile: status === "reduced" ? "canonical-dom-divergence.reduced.vue" : null,
      reducedBytes: status === "reduced" ? Buffer.byteLength(witness) : null,
      reducedSha256:
        status === "reduced" ? createHash("sha256").update(witness).digest("hex") : null,
      sourceCommit,
      runUrl: "https://github.com/ubugeeei-prod/vize/actions/runs/1234",
    }),
  );
  return directory;
}

await test("clean reduction evidence creates no fixture promotion", () => {
  const directory = reductionDirectory("clean");
  try {
    assert.equal(createPromotionPlan(directory), null);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

await test("a reduced witness becomes a stable minimal fixture plan", () => {
  const directory = reductionDirectory();
  try {
    const plan = createPromotionPlan(directory);
    assert.equal(plan.fingerprint, "abc123abc123");
    assert.equal(plan.branch, "automation/davinci-dom-abc123abc123");
    assert.equal(plan.files.length, 2);
    assert.equal(
      plan.files[0].path,
      "tests/_fixtures/differential/compiler/real-project-dom-reductions/abc123abc123/witness.vue.txt",
    );
    assert.equal(plan.files[0].content, "<template><div/></template>");
    const provenance = JSON.parse(plan.files[1].content);
    assert.equal(provenance.fingerprint, plan.fingerprint);
    assert.equal(provenance.sourceCommit, sourceCommit);
    assert.doesNotMatch(plan.body, /tests\/_fixtures\/_git\/example.*<template>/s);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

await test("promotion deduplicates exact fixtures and exact draft branches", () => {
  const directory = reductionDirectory();
  try {
    const plan = createPromotionPlan(directory);
    const existingFixture = publishPromotion(plan, context, {
      fixtureAtBase: (path) => plan.files.find((file) => file.path === path).content,
      findOpenPull: () => assert.fail("fixture should deduplicate before PR lookup"),
      createPull: () => assert.fail("fixture should not create a PR"),
    });
    assert.equal(existingFixture.status, "existing-fixture");

    let creates = 0;
    const existingPull = publishPromotion(plan, context, {
      fixtureAtBase: () => null,
      findOpenPull: () => ({ number: 91, html_url: "https://example.test/pull/91" }),
      createPull: () => {
        creates += 1;
      },
    });
    assert.equal(existingPull.status, "existing-pr");
    assert.equal(existingPull.pullRequestNumber, 91);
    assert.equal(creates, 0);

    const created = publishPromotion(plan, context, {
      fixtureAtBase: () => null,
      findOpenPull: () => null,
      createPull: (received) => {
        creates += 1;
        assert.equal(received, plan);
        return { number: 92, html_url: "https://example.test/pull/92" };
      },
    });
    assert.equal(created.status, "created");
    assert.equal(created.pullRequestNumber, 92);
    assert.equal(creates, 1);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
