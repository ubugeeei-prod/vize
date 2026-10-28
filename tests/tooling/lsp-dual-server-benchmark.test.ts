import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "../_helpers/realworld-patch.ts";
import {
  assertOwnedWorkspaceCachesAbsent,
  offsetToPosition,
  rangeForNeedle,
  resetOwnedWorkspaceCaches,
  wordRanges,
} from "../performance/support/dual-lsp-fixture.ts";
import {
  assertAuthoredHovers,
  authoredLocations,
  assertExactProductDiagnostics,
  diagnosticBarrierCount,
  diagnosticDeadlineMs,
  diagnosticQuietMs,
  expectedDiagnostic,
  linuxProcessIdentity,
  waitForExpectedDiagnostic,
} from "../performance/support/dual-lsp-oracles.ts";
import {
  BackgroundTaskCustody,
  commonInitializeParams,
  dualLspToolReceipt,
  primaryOperationReceipt,
  productOrder,
  stableOperationTranscript,
} from "../performance/support/dual-lsp-product.ts";
import {
  assertRevisionReceiptReachability,
  cleanupFailuresFor,
  collectCleanupFailures,
  distribution,
  initializeWithCleanupCustody,
  selectAuthoritativeFailure,
  summarizeScheduleReceipt,
  summarizeSamples,
} from "../performance/support/dual-lsp-report.ts";
import {
  BoundedNotificationHistory,
  supportsLspMethod,
  type ClientOperation,
} from "../performance/support/dual-lsp-wire.ts";
import { processTreeRss } from "../performance/support/process-metrics.ts";

const sourceRoot = path.join(repoRoot, "tests/_fixtures/performance/lsp-dual-server/sources");
const sourceV1 = fs.readFileSync(path.join(sourceRoot, "App.v1.vue.txt"), "utf8");
const sourceV2 = fs.readFileSync(path.join(sourceRoot, "App.v2.vue.txt"), "utf8");

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

test("fixture anchors use UTF-16 positions and exact authored reference sets", () => {
  const ranges = wordRanges(sourceV2, "message");
  assert.equal(ranges.length, 3);
  const emojiUsage = sourceV2.indexOf("message", sourceV2.indexOf("🚀"));
  assert.deepEqual(offsetToPosition(sourceV2, emojiUsage), ranges[1].start);
  assert.equal(ranges[1].start.character, 13, "the astral symbol occupies two UTF-16 units");
  assert.deepEqual(rangeForNeedle(sourceV2, "broken"), {
    start: { line: 5, character: 6 },
    end: { line: 5, character: 12 },
  });
});

test("definition and reference locations require an authored URI", () => {
  const range = rangeForNeedle(sourceV2, "message");
  assert.deepEqual(authoredLocations({ uri: "file:///workspace/src/App.vue", range }), [
    { uri: "file:///workspace/src/App.vue", range },
  ]);
  assert.throws(() => authoredLocations({ range }), /location lacks an authored URI/u);
  assert.throws(
    () => authoredLocations({ targetUri: 42, targetSelectionRange: range }),
    /location lacks an authored URI/u,
  );
});

test("every admitted hover carries the exact authored range", () => {
  const target = rangeForNeedle(sourceV2, "message");
  assert.doesNotThrow(() =>
    assertAuthoredHovers([{ contents: "message: string", range: target }], target),
  );
  assert.throws(
    () => assertAuthoredHovers([{ contents: "message: string" }], target),
    /hover omitted its authored range/u,
  );
  assert.throws(
    () =>
      assertAuthoredHovers(
        [
          {
            contents: "message: string",
            range: {
              start: target.start,
              end: { ...target.end, character: target.end.character + 1 },
            },
          },
        ],
        target,
      ),
    /hover escaped the authored token/u,
  );
});

test("diagnostic revisions have mutually exclusive complete fingerprints", () => {
  const uri = "file:///workspace/src/App.vue";
  const v1 = expectedDiagnostic(uri, sourceV1, 1);
  const v2 = expectedDiagnostic(uri, sourceV2, 2);
  assert.notEqual(v1, v2);
  assert.match(v1, /Type 'string' is not assignable to type 'number'/);
  assert.match(v2, /Type 'number' is not assignable to type 'string'/);
  assert.ok(diagnosticQuietMs >= 250, "late diagnostics need a material quiescence window");
  assert.equal(diagnosticBarrierCount, 2);
});

test("a delayed exact diagnostic is awaited before qualification", async () => {
  const uri = "file:///workspace/src/App.vue";
  const expected = expectedDiagnostic(uri, sourceV2, 2);
  const exact = JSON.parse(expected) as {
    range: { start: unknown; end: unknown };
    code: string;
    message: string;
  };
  const publication = {
    uri,
    diagnostics: [
      {
        range: { end: exact.range.end, start: exact.range.start },
        code: exact.code,
        message: exact.message,
        severity: 1,
      },
    ],
  };
  let release: (() => void) | undefined;
  let predicate: ((params: unknown) => boolean) | undefined;
  const waiter = {
    waitForNotification(
      method: string,
      afterSequence: number,
      candidate: (params: unknown) => boolean,
      timeoutMs?: number,
    ): Promise<unknown> {
      assert.equal(method, "textDocument/publishDiagnostics");
      assert.equal(afterSequence, 7);
      assert.equal(timeoutMs, diagnosticDeadlineMs);
      predicate = candidate;
      return new Promise((resolve) => {
        release = () => resolve({});
      });
    },
  };
  let settled = false;
  const pending = waitForExpectedDiagnostic(waiter, 7, expected).then(() => {
    settled = true;
  });
  await new Promise<void>((resolve) => setImmediate(resolve));
  assert.equal(settled, false);
  assert.ok(predicate);
  assert.equal(predicate(publication), true);
  assert.ok(release);
  release();
  await pending;
  assert.equal(settled, true);
});

test("diagnostic admission covers both halves of the Vue product", () => {
  const uri = "file:///workspace/src/App.vue";
  const expected = expectedDiagnostic(uri, sourceV2, 2);
  const exact = JSON.parse(expected) as {
    uri: string;
    range: { start: unknown; end: unknown };
    code: string;
    message: string;
  };
  const publication = {
    uri,
    diagnostics: [
      {
        range: { end: exact.range.end, start: exact.range.start },
        code: exact.code,
        message: exact.message,
        severity: 1,
      },
    ],
  };
  const emptyPrimary = {
    provider: "vue-language-server",
    receivedAtMs: 1,
    params: { uri, diagnostics: [] },
  };
  const semanticExact = {
    provider: "typescript-language-server",
    receivedAtMs: 2,
    params: publication,
  };
  assert.equal(
    assertExactProductDiagnostics("vue-language-tools", [emptyPrimary, semanticExact], expected, 2)
      .reportedVersion,
    null,
  );
  assert.throws(
    () =>
      assertExactProductDiagnostics(
        "vue-language-tools",
        [
          emptyPrimary,
          semanticExact,
          {
            provider: "typescript-language-server",
            receivedAtMs: 3,
            params: { uri, diagnostics: [] },
          },
        ],
        expected,
        2,
      ),
    /terminal product diagnostics were cleared/u,
  );
  assert.throws(() =>
    assertExactProductDiagnostics(
      "vue-language-tools",
      [
        {
          provider: "vue-language-server",
          receivedAtMs: 1,
          params: {
            uri,
            diagnostics: [
              {
                ...JSON.parse(expectedDiagnostic(uri, sourceV1, 1)),
                severity: 1,
              },
            ],
          },
        },
        semanticExact,
      ],
      expected,
      2,
    ),
  );
  assert.throws(() =>
    assertExactProductDiagnostics(
      "vue-language-tools",
      [semanticExact, { ...semanticExact, provider: "vue-language-server" }],
      expected,
      2,
    ),
  );
  assert.throws(
    () =>
      assertExactProductDiagnostics(
        "vue-language-tools",
        [
          {
            ...semanticExact,
            params: {
              uri,
              diagnostics: [
                {
                  range: { end: exact.range.end, start: exact.range.start },
                  code: exact.code,
                  message: { text: exact.message },
                  severity: 1,
                },
              ],
            },
          },
        ],
        expected,
        2,
      ),
    /diagnostic multiset is not exact/u,
  );
});

test("startup, diagnostics, and query packets remain separate distributions", () => {
  const summary = summarizeSamples([
    {
      startupMs: 10,
      revisionReceiptSha256: "a".repeat(64),
      diagnosticV1QualificationMs: 20,
      diagnosticV2QualificationMs: 30,
      totalMs: 40,
      timingsMs: { hover: 5, rename: 35 },
      rss: { totalKiB: 50, processes: 2, roots: [] },
    },
  ]);
  assert.equal(summary.startupMs.p50, 10);
  assert.equal(summary.diagnosticV1QualificationMs.p50, 20);
  assert.equal(summary.diagnosticV2QualificationMs.p50, 30);
  assert.equal(summary.totalMs.p50, 40);
});

test("all warm packets reach their shared unchanged-revision receipt", () => {
  const receipt = "a".repeat(64);
  const samples = Array.from({ length: 100 }, () => ({
    revisionReceiptSha256: receipt,
    totalMs: 1,
    timingsMs: { hover: 1 },
    rss: { totalKiB: 1, processes: 1, roots: [] },
  }));
  assert.deepEqual(assertRevisionReceiptReachability(samples, [receipt]), {
    samples: 100,
    availableReceipts: 1,
    referencedReceipts: 1,
  });
  assert.throws(() =>
    assertRevisionReceiptReachability(
      [{ ...samples[0], revisionReceiptSha256: "b".repeat(64) }],
      [receipt],
    ),
  );
});

test(
  "Linux process-tree receipts bind birth, executable, command, and sampled RSS",
  { skip: process.platform !== "linux" },
  () => {
    const tree = processTreeRss(process.pid);
    assert.ok(tree);
    const member = tree.members.find((entry) => entry.pid === process.pid);
    assert.ok(member);
    const identity = linuxProcessIdentity(member, process.pid);
    assert.equal(identity.role, "root");
    assert.equal(identity.pid, process.pid);
    assert.match(identity.birthTicks, /^\d+$/u);
    assert.ok(path.isAbsolute(identity.executable));
    assert.ok(identity.command.length > 0);
    assert.match(identity.commandSha256, /^[0-9a-f]{64}$/u);
    assert.equal(identity.sampledRssKiB, member.rssKiB);
  },
);

test("the isolated provider pins every process in the Vue stack", () => {
  const provider = path.join(sourceRoot, "../provider");
  const manifest = JSON.parse(fs.readFileSync(path.join(provider, "package.json"), "utf8"));
  assert.deepEqual(manifest.dependencies, {
    "@vue/language-server": "3.3.13",
    "@vue/typescript-plugin": "3.3.13",
    typescript: "6.0.3",
    "typescript-language-server": "6.0.2",
    vue: "3.6.0-rc.11",
  });
  const lock = fs.readFileSync(path.join(provider, "pnpm-lock.yaml"), "utf8");
  for (const receipt of [
    "'@vue/language-server@3.3.13'",
    "'@vue/typescript-plugin@3.3.13'",
    "typescript-language-server@6.0.2",
  ]) {
    assert.ok(lock.includes(receipt), `isolated lock omitted ${receipt}`);
  }
  assert.match(createHash("sha256").update(lock).digest("hex"), /^[0-9a-f]{64}$/u);
  const tools = dualLspToolReceipt();
  assert.equal((tools.vueLanguageTools as { version: string }).version, "3.3.13");
  assert.equal((tools.typescriptLanguageServer as { version: string }).version, "6.0.2");
  assert.equal((tools.vueTypescriptPlugin as { version: string }).version, "3.3.13");
  assert.equal((tools.typescript as { version: string }).version, "6.0.3");
});

test("prepareRename requires the explicit prepare capability", () => {
  assert.equal(supportsLspMethod({ renameProvider: true }, "textDocument/rename"), true);
  assert.equal(supportsLspMethod({ renameProvider: true }, "textDocument/prepareRename"), false);
  assert.equal(
    supportsLspMethod({ renameProvider: { prepareProvider: true } }, "textDocument/prepareRename"),
    true,
  );
});

test("late background bridge rejection is settled before admission", async () => {
  const custody = new BackgroundTaskCustody();
  const failure = new Error("late bridge rejection");
  const delayed = new Promise<void>((_resolve, reject) => {
    setImmediate(() => reject(failure));
  });
  void custody.track(delayed).catch(() => undefined);
  await assert.rejects(
    () => custody.settle(),
    (error: unknown) => error instanceof AggregateError && error.errors.includes(failure),
  );
});

test("primary operation receipts enforce human-readable exact method counts", () => {
  const operation = (kind: ClientOperation["kind"], method: string): ClientOperation => ({
    kind,
    method,
    params: null,
  });
  const operations = [
    operation("request", "initialize"),
    operation("notification", "initialized"),
    operation("notification", "textDocument/didOpen"),
    operation("request", "textDocument/documentSymbol"),
    operation("request", "textDocument/documentSymbol"),
    operation("notification", "textDocument/didChange"),
    operation("request", "textDocument/documentSymbol"),
    operation("request", "textDocument/documentSymbol"),
    operation("request", "textDocument/completion"),
    operation("request", "textDocument/hover"),
    operation("request", "textDocument/definition"),
    operation("request", "textDocument/references"),
    operation("request", "textDocument/prepareRename"),
    operation("request", "textDocument/rename"),
  ];
  const receipt = primaryOperationReceipt(operations, 1, 2);
  assert.equal(receipt.totalOperations, 14);
  assert.equal(receipt.methodCounts["request textDocument/documentSymbol"], 4);
  assert.equal(receipt.methodCounts["request textDocument/rename"], 1);
  assert.throws(
    () => primaryOperationReceipt(operations.slice(0, -1), 1, 2),
    /method order drifted/u,
  );
});

test("the primary initialize and operation transcript is byte stable", () => {
  const params = commonInitializeParams("/workspace", "/typescript/lib");
  const operations = [
    { kind: "request" as const, method: "initialize", params },
    { kind: "notification" as const, method: "initialized", params: {} },
  ];
  assert.equal(
    stableOperationTranscript(operations),
    stableOperationTranscript(structuredClone(operations)),
  );
});
