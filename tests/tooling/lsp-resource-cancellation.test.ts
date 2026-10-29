import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {
  auditCancellationPublications,
  cancellationBatch,
  cancellationCycles,
  diagnosticRecoveryLifecycle,
  recordCancellationPublications,
  sourceSha256,
  summarizeCancellationDurations,
} from "../../tools/support/compat/davinci/lib/lsp-resource-cancellation.ts";
import {
  gateCleanupStatus,
  nativeFrameEvidence,
  resourceCorsaGateScript,
} from "../../tools/support/compat/davinci/lib/lsp-resource-corsa-gate.ts";
import {
  createCancellationPublicationProgress,
  finalizeCancellationProfileLink,
  rssDeltaMib,
  updateCancellationPublicationProgress,
  type AllocationProfileLink,
} from "../../tools/support/compat/davinci/lib/lsp-resource-cancellation-run.ts";

test("resource cancellation frames retain exact write order, version and source identity", () => {
  const source = "<template><p>日本語</p></template>\n";
  const { messages, receipt } = cancellationBatch(7, "file:///workspace/Component.vue", 12, source);

  assert.deepEqual(messages, [
    { jsonrpc: "2.0", method: "$/cancelRequest", params: { id: 7 } },
    {
      jsonrpc: "2.0",
      method: "textDocument/didChange",
      params: {
        textDocument: { uri: "file:///workspace/Component.vue", version: 12 },
        contentChanges: [{ text: source }],
      },
    },
  ]);
  assert.deepEqual(receipt, {
    raw_request_id: 7,
    wire_order: ["$/cancelRequest", "textDocument/didChange"],
    version: 12,
    source_bytes: Buffer.byteLength(source),
    source_sha256: sourceSha256(source),
    source_utf8_base64: Buffer.from(source).toString("base64"),
  });
});

test("native gate evidence preserves the complete response frame", () => {
  const message = {
    jsonrpc: "2.0" as const,
    id: 19,
    result: { contents: { kind: "markdown", value: "日本語 hover" } },
  };
  const body = JSON.stringify(message);
  const frame = Buffer.from(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
  assert.deepEqual(nativeFrameEvidence(frame), {
    bytes: frame.byteLength,
    sha256: createHash("sha256").update(frame).digest("hex"),
    base64: frame.toString("base64"),
    message,
  });
  assert.throws(() => nativeFrameEvidence(frame.subarray(0, -1)), /body must be complete/);
});

test("generated repeatable Corsa gate is valid CommonJS", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-resource-gate-test-"));
  try {
    const proxy = path.join(directory, "corsa-gate.cjs");
    fs.writeFileSync(proxy, resourceCorsaGateScript());
    execFileSync(process.execPath, ["--check", proxy], { stdio: "pipe" });
  } finally {
    fs.rmSync(directory, { force: true, recursive: true });
  }
});

test("resource cancellation statistics use every nearest-rank sample", () => {
  const values = Array.from({ length: cancellationCycles }, (_, index) => index + 1);
  assert.deepEqual(summarizeCancellationDurations(values), {
    samples: 40,
    min_ms: 1,
    p50_ms: 20,
    p95_ms: 38,
    p99_ms: 40,
    max_ms: 40,
    sample_policy: {
      population: "all 40 admitted real stdio textDocument/hover cancellation responses",
      warmupSamplesExcluded: 0,
      outlierSamplesExcluded: 0,
      percentileMethod: "nearest-rank order statistic at sorted[ceil(q * n) - 1]",
    },
  });
  assert.throws(() => summarizeCancellationDurations(values.slice(1)), /complete cancellation/);
  assert.throws(
    () => summarizeCancellationDurations([...values.slice(0, -1), Number.NaN]),
    /finite non-negative/,
  );
});

test("diagnostic audit checks known nondecreasing revisions per opened URI", () => {
  const childUri = "file:///workspace/Component00000.vue";
  const parentUri = "file:///workspace/Component00001.vue";
  const clean = sourceSha256("clean");
  const broken = sourceSha256("broken");
  const parent = sourceSha256("parent");
  const publications = [
    {
      received_order: 1,
      received_ns: "10",
      uri: parentUri,
      version: 1,
      diagnostics: [{ code: 2322 }],
      diagnostics_sha256: sourceSha256('[{"code":2322}]'),
    },
    {
      received_order: 2,
      received_ns: "20",
      uri: childUri,
      version: 2,
      diagnostics: [{ code: 2322 }],
      diagnostics_sha256: sourceSha256('[{"code":2322}]'),
    },
    {
      received_order: 3,
      received_ns: "30",
      uri: parentUri,
      version: 1,
      diagnostics: [],
      diagnostics_sha256: sourceSha256("[]"),
    },
    {
      received_order: 4,
      received_ns: "40",
      uri: childUri,
      version: 3,
      diagnostics: [],
      diagnostics_sha256: sourceSha256("[]"),
    },
  ];
  const knownDocuments = new Map([
    [
      childUri,
      new Map([
        [2, broken],
        [3, clean],
      ]),
    ],
    [parentUri, new Map([[1, parent]])],
  ]);
  const audit = auditCancellationPublications(
    publications,
    knownDocuments,
    { uri: childUri, version: 3 },
    publications.length,
  );
  assert.deepEqual(audit.tracked_uris, [childUri, parentUri].sort());
  assert.deepEqual(audit.versions_by_uri[childUri], [2, 3]);
  assert.deepEqual(audit.versions_by_uri[parentUri], [1, 1]);
  assert.equal(audit.publications_after_final_fence, 0);
  assert.equal(audit.rows[3].source_sha256, clean);
  assert.throws(
    () =>
      auditCancellationPublications(
        publications,
        new Map([
          [childUri, new Map([[3, clean]])],
          [parentUri, new Map([[1, parent]])],
        ]),
        { uri: childUri, version: 3 },
        publications.length,
      ),
    /must name a sent source clone/,
  );
  assert.throws(
    () =>
      auditCancellationPublications(publications, knownDocuments, { uri: childUri, version: 3 }, 1),
    /after the final clean recovery fence/,
  );
  assert.throws(
    () =>
      auditCancellationPublications(
        [publications[3], publications[0], publications[1], publications[2]],
        knownDocuments,
        { uri: childUri, version: 2 },
        publications.length,
      ),
    /nondecreasing/,
  );
  assert.throws(
    () =>
      auditCancellationPublications(
        [publications[0], publications[1], publications[3]],
        knownDocuments,
        { uri: childUri, version: 3 },
        3,
      ),
    /final diagnostics.*must be clean/,
  );
});

test("diagnostic audit rejects a missing opened parent publication", () => {
  const childUri = "file:///workspace/Component00000.vue";
  const parentUri = "file:///workspace/Component00001.vue";
  const childHash = sourceSha256("child");
  const publications = [
    {
      received_order: 1,
      received_ns: "10",
      uri: childUri,
      version: 3,
      diagnostics: [],
      diagnostics_sha256: sourceSha256("[]"),
    },
  ];
  assert.throws(
    () =>
      auditCancellationPublications(
        publications,
        new Map([
          [childUri, new Map([[3, childHash]])],
          [parentUri, new Map([[1, sourceSha256("parent")]])],
        ]),
        { uri: childUri, version: 3 },
        publications.length,
      ),
    /every known opened document must publish diagnostics/,
  );
});

test("diagnostic recovery requires a fresh child then its importer publication", () => {
  const child = {
    received_order: 8,
    uri: "file:///workspace/src/Component00000.vue",
    version: 33,
    diagnostics: [],
  };
  const parent = {
    received_order: 9,
    uri: "file:///workspace/src/Component00001.vue",
    version: 1,
    diagnostics: [],
  };
  assert.deepEqual(diagnosticRecoveryLifecycle(7, child, parent), {
    after_received_order: 7,
    child,
    parent,
    child_before_importer: true,
  });
  assert.throws(
    () => diagnosticRecoveryLifecycle(8, child, parent),
    /child diagnostics must publish after the recovery boundary/,
  );
  assert.throws(
    () => diagnosticRecoveryLifecycle(7, child, { ...parent, received_order: 8 }),
    /importer diagnostics must publish after the clean child diagnostics/,
  );
  assert.throws(
    () => diagnosticRecoveryLifecycle(7, { ...child, diagnostics: [{ code: 2322 }] }, parent),
    /recovery child diagnostics must be clean/,
  );

  const notificationObservers: Array<(method: string, value: unknown) => void> = [];
  const recorder = recordCancellationPublications(
    { notificationObservers } as unknown as Parameters<typeof recordCancellationPublications>[0],
    "/workspace",
  );
  const historical = { uri: child.uri, version: child.version, diagnostics: [] };
  assert.equal(recorder.orderOf(historical), 0);
  notificationObservers[0]("textDocument/publishDiagnostics", historical);
  const importer = { uri: parent.uri, version: parent.version, diagnostics: [] };
  notificationObservers[0]("textDocument/publishDiagnostics", importer);
  assert.equal(recorder.orderOf(historical), 1);
  assert.equal(recorder.orderOf(importer), 2);
  assert.deepEqual(
    recorder.since(0).map(({ received_order, uri, version }) => ({
      received_order,
      uri,
      version,
    })),
    [
      { received_order: 1, uri: child.uri, version: child.version },
      { received_order: 2, uri: parent.uri, version: parent.version },
    ],
  );
  recorder.stop();
  assert.deepEqual(notificationObservers, []);
});

test("diagnostic progress distinguishes partial evidence from a complete measurement", () => {
  const progress = createCancellationPublicationProgress();
  assert.deepEqual(progress, {
    phase: "baseline-recovery",
    complete: false,
    recovery_boundary_received_order: 0,
    clean_fence_received_order: null,
    observed_count: 0,
    rows: [],
  });
  const row = {
    received_order: 1,
    received_ns: "10",
    uri: "file:///workspace/src/Component00000.vue",
    version: 33,
    diagnostics: [],
    diagnostics_sha256: sourceSha256("[]"),
  };
  updateCancellationPublicationProgress(progress, "baseline-quiescence", 0, 1, [row]);
  assert.equal(progress.complete, false);
  assert.equal(progress.observed_count, 1);
  assert.deepEqual(progress.rows, [row]);
  updateCancellationPublicationProgress(progress, "complete", 0, 1, [row]);
  assert.equal(progress.complete, true);
});

test("cleanup receipts distinguish graceful exit from forced completion", () => {
  assert.deepEqual(gateCleanupStatus(0, 0, 0), {
    no_orphans: true,
    forced_cleanup_complete: true,
  });
  assert.deepEqual(gateCleanupStatus(1, 1, 0), {
    no_orphans: false,
    forced_cleanup_complete: true,
  });
  assert.deepEqual(gateCleanupStatus(1, 2, 1), {
    no_orphans: false,
    forced_cleanup_complete: false,
  });
});

test("retained RSS rounds the raw-byte delta rather than displayed endpoints", () => {
  const oneMiB = 1024 * 1024;
  const before = Math.round(1.04 * oneMiB);
  const after = Math.round(1.06 * oneMiB);
  assert.equal(rssDeltaMib(after, before), 0);
});

test("profile linkage preserves primary failures when evidence persistence fails", () => {
  const link: AllocationProfileLink = {
    same_session: true,
    expected_path: "/profile/expected.json",
    receipt_field: "profile_json",
    scope: "test",
  };
  finalizeCancellationProfileLink(link, null, true);
  assert.equal(link.actual_path, null);
  assert.equal(link.path_matches, false);

  const failedPersistence: AllocationProfileLink = { ...link };
  assert.doesNotThrow(() =>
    finalizeCancellationProfileLink(failedPersistence, null, true, () => {
      throw new Error("persist failed");
    }),
  );
  assert.equal(failedPersistence.persistence_error, "Error: persist failed");

  assert.throws(
    () =>
      finalizeCancellationProfileLink({ ...link }, null, false, () => {
        throw new Error("persist failed");
      }),
    /persist failed/,
  );

  let persisted = false;
  assert.throws(
    () =>
      finalizeCancellationProfileLink({ ...link }, null, false, () => {
        persisted = true;
      }),
    /must belong to this exact session/,
  );
  assert.equal(persisted, true);
});
