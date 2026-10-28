import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "../_helpers/realworld-patch.ts";
import {
  offsetToPosition,
  rangeForNeedle,
  wordRanges,
} from "../performance/support/dual-lsp-fixture.ts";
import {
  assertExactProductDiagnostics,
  diagnosticBarrierCount,
  diagnosticQuietMs,
  expectedDiagnostic,
} from "../performance/support/dual-lsp-oracles.ts";
import {
  commonInitializeParams,
  productOrder,
  stableOperationTranscript,
} from "../performance/support/dual-lsp-product.ts";
import {
  distribution,
  summarizeSamples,
} from "../performance/support/dual-lsp-report.ts";
import { supportsLspMethod } from "../performance/support/dual-lsp-wire.ts";

const sourceRoot = path.join(
  repoRoot,
  "tests/_fixtures/performance/lsp-dual-server/sources",
);
const sourceV1 = fs.readFileSync(path.join(sourceRoot, "App.v1.vue.txt"), "utf8");
const sourceV2 = fs.readFileSync(path.join(sourceRoot, "App.v2.vue.txt"), "utf8");

test("ABBA scheduling reverses every adjacent product pair", () => {
  assert.deepEqual(productOrder(0), ["vize", "vue-language-tools"]);
  assert.deepEqual(productOrder(1), ["vue-language-tools", "vize"]);
  assert.deepEqual(productOrder(2), productOrder(0));
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
    assertExactProductDiagnostics(
      "vue-language-tools",
      [emptyPrimary, semanticExact],
      expected,
      2,
    ).reportedVersion,
    null,
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
});

test("startup, diagnostics, and query packets remain separate distributions", () => {
  const summary = summarizeSamples([
    {
      startupMs: 10,
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
});

test("prepareRename requires the explicit prepare capability", () => {
  assert.equal(supportsLspMethod({ renameProvider: true }, "textDocument/rename"), true);
  assert.equal(supportsLspMethod({ renameProvider: true }, "textDocument/prepareRename"), false);
  assert.equal(
    supportsLspMethod({ renameProvider: { prepareProvider: true } }, "textDocument/prepareRename"),
    true,
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
