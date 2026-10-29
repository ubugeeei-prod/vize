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
  sourceSha256,
  summarizeCancellationDurations,
} from "../../tools/support/compat/davinci/lib/lsp-resource-cancellation.ts";
import {
  nativeFrameEvidence,
  resourceCorsaGateScript,
} from "../../tools/support/compat/davinci/lib/lsp-resource-corsa-gate.ts";
import {
  finalizeCancellationProfileLink,
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

test("diagnostic audit admits only known nondecreasing revisions with no late tail", () => {
  const clean = sourceSha256("clean");
  const broken = sourceSha256("broken");
  const publications = [
    {
      received_order: 1,
      received_ns: "10",
      uri: "file:///workspace/Component.vue",
      version: 2,
      diagnostics: [{ code: 2322 }],
      diagnostics_sha256: sourceSha256('[{"code":2322}]'),
    },
    {
      received_order: 2,
      received_ns: "20",
      uri: "file:///workspace/Component.vue",
      version: 3,
      diagnostics: [],
      diagnostics_sha256: sourceSha256("[]"),
    },
  ];
  const audit = auditCancellationPublications(
    publications,
    new Map([
      [2, broken],
      [3, clean],
    ]),
    3,
    publications.length,
  );
  assert.deepEqual(audit.versions, [2, 3]);
  assert.equal(audit.publications_after_final_fence, 0);
  assert.equal(audit.rows[1].source_sha256, clean);
  assert.throws(
    () => auditCancellationPublications(publications, new Map([[3, clean]]), 3, 2),
    /must name a sent source clone/,
  );
  assert.throws(
    () =>
      auditCancellationPublications(
        publications,
        new Map([
          [2, broken],
          [3, clean],
        ]),
        3,
        1,
      ),
    /after the final clean recovery fence/,
  );
  assert.throws(
    () =>
      auditCancellationPublications(
        [publications[1], publications[0]],
        new Map([
          [2, broken],
          [3, clean],
        ]),
        2,
        2,
      ),
    /nondecreasing/,
  );
});

test("profile linkage records mismatch without replacing a primary failure", () => {
  const link: AllocationProfileLink = {
    same_session: true,
    expected_path: "/profile/expected.json",
    receipt_field: "profile_json",
    scope: "test",
  };
  finalizeCancellationProfileLink(link, null, true);
  assert.equal(link.actual_path, null);
  assert.equal(link.path_matches, false);
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
