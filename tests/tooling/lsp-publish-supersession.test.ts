import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

// Publish supersession invariants (#3315).
//
// `publish_changed_diagnostics` may abandon a pass whose version has already
// been superseded, and `publish_diagnostics` drops a result whose version went
// stale while it was being collected. Both are latency wins and both are
// allowed to skip intermediate versions — but neither may drop the *final*
// publish or leave a stale payload behind it, because that is a document whose
// squiggles no longer match its text and no further edit is coming to fix it.
//
// Every delivered record must be an exact authored payload in strictly
// increasing version order, with the initial and final versions required.
// Intermediate versions may be superseded, but duplicates, unknown versions,
// incorrect payloads and anything trailing the final version are forbidden.
//
// Type checking is off: the assertions are about which versions reach the
// client, so the payloads come from the deterministic parse/lint path and the
// test needs no Corsa backend. Configuration admission now yields even with
// typecheck:false, so the rapid stream may skip intermediate publications.
// A separate sequential stream awaits every publication and retains all five
// exact payload goldens, including both malformed intermediate versions.

// An unterminated `<template>` block: a `vize/sfc` parse diagnostic, so the
// broken payload is produced by the parser and needs no lint or type backend.
const BROKEN = `<template>
  <p>{{ message }}</p>
`;

const BROKEN_CODES = ["vize/sfc:Malformed <template> block: the closing tag is missing."];

const CLEAN = `<script setup lang="ts">
const message = 'hi'
</script>

<template>
  <p>{{ message }}</p>
</template>
`;

type PublishRecord = { version: number | null; codes: string[] };

function codesOf(params: PublishDiagnosticsParams): string[] {
  return params.diagnostics
    .map((diagnostic) => `${diagnostic.source ?? "?"}:${diagnostic.message}`)
    .sort();
}

const AUTHORED: PublishRecord[] = [
  { version: 1, codes: [] },
  { version: 2, codes: BROKEN_CODES },
  { version: 3, codes: [] },
  { version: 4, codes: BROKEN_CODES },
  { version: 5, codes: [] },
];
const CLOSED: PublishRecord = { version: null, codes: [] };

function assertPublishHistory(history: PublishRecord[], closed = false): void {
  const publications = closed ? history.slice(0, -1) : history;
  if (closed) assert.deepEqual(history.at(-1), CLOSED);
  assert.deepEqual(publications[0], AUTHORED[0]);
  let previousVersion = 0;
  for (const record of publications) {
    assert.ok(record.version !== null, "close must follow the final publication");
    const authored = AUTHORED.find(({ version }) => version === record.version);
    assert.ok(authored, `unknown published version ${record.version}`);
    assert.deepEqual(record, authored, `incorrect payload for version ${record.version}`);
    assert.ok(record.version > previousVersion, "published versions must strictly increase");
    previousVersion = record.version;
  }
  assert.equal(previousVersion, 5, "the final authored version must publish");
}

test("publish history oracle accepts every complete authored subsequence", () => {
  // All eight subsets of the three optional intermediate versions, including
  // the original exhaustive stream and the maximally superseded [1, 5].
  for (let mask = 0; mask < 8; mask++) {
    const history = [
      AUTHORED[0],
      ...AUTHORED.slice(1, -1).filter((_, index) => (mask & (1 << index)) !== 0),
      AUTHORED[4],
    ];
    assertPublishHistory(history);
    assertPublishHistory([...history, CLOSED], true);
  }
});

const invalidHistories: Array<{ name: string; history: PublishRecord[]; closed?: boolean }> = [
  { name: "missing initial", history: AUTHORED.slice(1) },
  { name: "missing final", history: AUTHORED.slice(0, -1) },
  { name: "wrong payload", history: [AUTHORED[0], { version: 2, codes: [] }, AUTHORED[4]] },
  { name: "wrong final payload", history: [AUTHORED[0], { version: 5, codes: BROKEN_CODES }] },
  { name: "duplicate", history: [AUTHORED[0], AUTHORED[1], AUTHORED[1], AUTHORED[4]] },
  { name: "out of order", history: [AUTHORED[0], AUTHORED[2], AUTHORED[1], AUTHORED[4]] },
  { name: "stale after final", history: [...AUTHORED, AUTHORED[1], CLOSED], closed: true },
  { name: "duplicate final", history: [...AUTHORED, AUTHORED[4], CLOSED], closed: true },
  { name: "unknown version", history: [AUTHORED[0], { version: 2.5, codes: [] }, AUTHORED[4]] },
  { name: "missing close", history: AUTHORED, closed: true },
  { name: "early close", history: [AUTHORED[0], CLOSED, AUTHORED[4], CLOSED], closed: true },
  { name: "payload after close", history: [...AUTHORED, CLOSED, AUTHORED[4]], closed: true },
];
for (const { name, history, closed } of invalidHistories) {
  test(`publish history oracle rejects ${name}`, () => {
    assert.throws(() => assertPublishHistory(history, closed));
  });
}

for (const sequential of [false, true]) {
  test(
    sequential
      ? "vize lsp publishes every authored payload under sequential didChange"
      : "vize lsp converges on the final version under rapid successive didChange",
    async () => {
      const testRootDir = path.join(
        testOutputRoot,
        `lsp-publish-supersession-${sequential ? "sequential" : "rapid"}`,
      );
      fs.mkdirSync(testRootDir, { recursive: true });
      const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
      const session = new LspSession();
      const publishes: PublishRecord[] = [];

      try {
        await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: false });

        const filePath = path.join(workspaceDir, "Churn.vue");
        const uri = pathToFileURL(filePath).href;
        fs.writeFileSync(filePath, CLEAN, "utf8");

        session.notificationObservers.push((method, params) => {
          if (method !== "textDocument/publishDiagnostics") return;
          const publish = params as PublishDiagnosticsParams;
          if (!isDiagnosticsForUri(publish, uri)) return;
          publishes.push({ version: publish.version ?? null, codes: codesOf(publish) });
        });

        session.notify("textDocument/didOpen", {
          textDocument: { uri, languageId: "vue", version: 1, text: CLEAN },
        });
        await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
          isDiagnosticsForUri(params as PublishDiagnosticsParams, uri),
        );
        assert.deepEqual(publishes, [{ version: 1, codes: [] }]);

        // The rapid case sends the original four edits without any intervening
        // wait. The sequential control awaits each exact authored publication.
        // Broken/clean alternation distinguishes their payloads as well as versions.
        const sources = [BROKEN, CLEAN, BROKEN, CLEAN];
        for (const [index, text] of sources.entries()) {
          session.notify("textDocument/didChange", {
            textDocument: { uri, version: index + 2 },
            contentChanges: [{ text }],
          });
          if (sequential) {
            await session.waitForNotification("textDocument/publishDiagnostics", (params) => {
              const publish = params as PublishDiagnosticsParams;
              return isDiagnosticsForUri(publish, uri) && publish.version === index + 2;
            });
            assert.deepEqual(publishes, AUTHORED.slice(0, index + 2));
          }
        }
        const finalVersion = sources.length + 1;
        if (!sequential)
          await session.waitForNotification("textDocument/publishDiagnostics", (params) => {
            const publish = params as PublishDiagnosticsParams;
            return isDiagnosticsForUri(publish, uri) && publish.version === finalVersion;
          });

        assertPublishHistory(publishes);

        // Nothing may trail the final version: a superseded pass that published
        // late would leave squiggles describing text the document no longer has.
        session.notify("textDocument/didClose", { textDocument: { uri } });
        await session.waitForNotification("textDocument/publishDiagnostics", (params) => {
          const publish = params as PublishDiagnosticsParams;
          return isDiagnosticsForUri(publish, uri) && publish.version == null;
        });
        assertPublishHistory(publishes, true);
      } finally {
        await session.shutdown();
      }
      // Retain the observer through the real shutdown response/exit as well, so
      // a delayed stale publication cannot escape the complete history check.
      assertPublishHistory(publishes, true);
    },
  );
}
