import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  assertOwnedWorkspaceCachesAbsent,
  resetOwnedWorkspaceCaches,
} from "../performance/support/dual-lsp-fixture.ts";
import { productOrder } from "../performance/support/dual-lsp-product.ts";
import {
  cleanupFailuresFor,
  collectCleanupFailures,
  distribution,
  initializeWithCleanupCustody,
  selectAuthoritativeFailure,
  summarizeScheduleReceipt,
} from "../performance/support/dual-lsp-report.ts";
import { BoundedNotificationHistory } from "../performance/support/dual-lsp-wire.ts";

test("ABBA scheduling reverses every adjacent product pair", () => {
  assert.deepEqual(productOrder(0), ["vize", "vue-language-tools"]);
  assert.deepEqual(productOrder(1), ["vue-language-tools", "vize"]);
  assert.deepEqual(productOrder(2), productOrder(0));
  const receipt = summarizeScheduleReceipt(
    Array.from({ length: 100 }, (_, sampleIndex) => ({
      sampleIndex,
      order: productOrder(sampleIndex),
    })),
  );
  assert.equal(receipt.samples, 100);
  assert.equal(receipt.vizeFirst, 50);
  assert.equal(receipt.vueFirst, 50);
  assert.match(String(receipt.sha256), /^[0-9a-f]{64}$/u);
  assert.throws(
    () => summarizeScheduleReceipt([{ sampleIndex: 0, order: ["vue-language-tools", "vize"] }]),
    /must preserve the ABBA product order/u,
  );
});

test("primary failures keep identity while clean paths aggregate cleanup failures", async () => {
  const primary = { sentinel: "primary" };
  const firstCleanup = new Error("first cleanup");
  const secondCleanup = new Error("second cleanup");
  const selected = selectAuthoritativeFailure(
    true,
    primary,
    [firstCleanup, secondCleanup],
    "cleanup failed",
  );
  assert.equal(selected.kind, "primary");
  if (selected.kind !== "primary") throw new Error("primary selection was lost");
  assert.equal(selected.error, primary);
  assert.deepEqual(selected.cleanupFailures, [firstCleanup, secondCleanup]);

  const cleanupOnly = selectAuthoritativeFailure(
    false,
    undefined,
    [firstCleanup, secondCleanup],
    "cleanup failed",
  );
  assert.equal(cleanupOnly.kind, "cleanup");
  if (cleanupOnly.kind !== "cleanup") throw new Error("cleanup selection was lost");
  assert.deepEqual(cleanupOnly.error.errors, [firstCleanup, secondCleanup]);

  const primitivePrimary = "primitive primary";
  let observed: unknown;
  try {
    await initializeWithCleanupCustody(
      async () => {
        throw primitivePrimary;
      },
      async () => {
        throw firstCleanup;
      },
      "initialization cleanup failed",
    );
  } catch (error) {
    observed = error;
  }
  assert.equal(observed, primitivePrimary);
  assert.deepEqual(cleanupFailuresFor(observed), [firstCleanup]);
  assert.deepEqual(cleanupFailuresFor(cleanupOnly.error), [firstCleanup, secondCleanup]);

  const nestedPrimary = new AggregateError([new Error("primary child")], "primary aggregate");
  try {
    await initializeWithCleanupCustody(
      async () => {
        throw nestedPrimary;
      },
      async () => {
        throw firstCleanup;
      },
      "initialization cleanup failed",
    );
  } catch (error) {
    assert.equal(error, nestedPrimary);
    assert.deepEqual(cleanupFailuresFor(error), [firstCleanup]);
  }

  const attempts: string[] = [];
  const childFailures = await collectCleanupFailures([
    async () => {
      attempts.push("primary");
      throw firstCleanup;
    },
    async () => {
      attempts.push("semantic");
      throw secondCleanup;
    },
  ]);
  assert.deepEqual(attempts, ["primary", "semantic"]);
  assert.deepEqual(childFailures, [firstCleanup, secondCleanup]);
});

test("fresh cache reset receipts bind exact owned pre/post paths", () => {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lsp-cache-control-"));
  try {
    for (const relative of [".vize", "node_modules/.vize"]) {
      const target = path.join(workspace, relative);
      fs.mkdirSync(target, { recursive: true });
      fs.writeFileSync(path.join(target, "witness"), "owned");
    }
    const receipt = resetOwnedWorkspaceCaches(workspace);
    assert.deepEqual(
      receipt.map(({ path: target, existedBeforeReset, existsAfterReset }) => ({
        relative: path.relative(workspace, target),
        existedBeforeReset,
        existsAfterReset,
      })),
      [
        { relative: ".vize", existedBeforeReset: true, existsAfterReset: false },
        {
          relative: "node_modules/.vize",
          existedBeforeReset: true,
          existsAfterReset: false,
        },
      ],
    );
    assertOwnedWorkspaceCachesAbsent(receipt);
    const firstPath = receipt[0]?.path;
    assert.ok(firstPath);
    fs.mkdirSync(firstPath, { recursive: true });
    assert.throws(() => assertOwnedWorkspaceCachesAbsent(receipt), /reappeared before reuse/u);
  } finally {
    fs.rmSync(workspace, { force: true, recursive: true });
  }
});

test("notification history rejects cursors that predate eviction", () => {
  const history = new BoundedNotificationHistory(200);
  for (let sequence = 1; sequence <= 201; sequence += 1) {
    history.push({
      sequence,
      receivedAtMs: sequence,
      method: "textDocument/publishDiagnostics",
      params: { sequence },
    });
  }
  assert.throws(
    () => history.since("textDocument/publishDiagnostics", 0),
    /predates evicted sequence 1/u,
  );
  assert.equal(history.since("textDocument/publishDiagnostics", 1).length, 200);
});

test("nearest-rank distributions retain p50, p95, and p99", () => {
  assert.deepEqual(distribution(Array.from({ length: 100 }, (_, index) => index + 1)), {
    samples: 100,
    min: 1,
    p50: 50,
    p95: 95,
    p99: 99,
    max: 100,
  });
});
