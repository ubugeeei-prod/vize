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
// The whole publish stream is asserted with `deepEqual`, not probed, so a run
// that skips a version, reorders two, or emits one extra republish after the
// final version fails here instead of decaying quietly.
//
// Type checking is off: the assertions are about which versions reach the
// client, so the payloads come from the deterministic parse/lint path and the
// test needs no Corsa backend. In that configuration nothing on the didChange
// path yields, so every version is expected to publish and the sequence below
// is exhaustive — a future supersession that starts dropping versions here has
// to change this list deliberately rather than silently.

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

test("vize lsp converges on the final version under rapid successive didChange", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-publish-supersession");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: false });

    const filePath = path.join(workspaceDir, "Churn.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, CLEAN, "utf8");

    const publishes: PublishRecord[] = [];
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

    // Four rapid edits with no wait between them, alternating broken/clean, so
    // an intermediate publish is distinguishable from the final one by payload
    // as well as by version.
    const sources = [BROKEN, CLEAN, BROKEN, CLEAN];
    sources.forEach((text, index) => {
      session.notify("textDocument/didChange", {
        textDocument: { uri, version: index + 2 },
        contentChanges: [{ text }],
      });
    });
    const finalVersion = sources.length + 1;
    await session.waitForNotification("textDocument/publishDiagnostics", (params) => {
      const publish = params as PublishDiagnosticsParams;
      return isDiagnosticsForUri(publish, uri) && publish.version === finalVersion;
    });

    assert.deepEqual(publishes, [
      { version: 1, codes: [] },
      { version: 2, codes: BROKEN_CODES },
      { version: 3, codes: [] },
      { version: 4, codes: BROKEN_CODES },
      { version: 5, codes: [] },
    ]);

    // Nothing may trail the final version: a superseded pass that published
    // late would leave squiggles describing text the document no longer has.
    session.notify("textDocument/didClose", { textDocument: { uri } });
    await session.waitForNotification("textDocument/publishDiagnostics", (params) => {
      const publish = params as PublishDiagnosticsParams;
      return isDiagnosticsForUri(publish, uri) && publish.version == null;
    });
    assert.deepEqual(publishes, [
      { version: 1, codes: [] },
      { version: 2, codes: BROKEN_CODES },
      { version: 3, codes: [] },
      { version: 4, codes: BROKEN_CODES },
      { version: 5, codes: [] },
      { version: null, codes: [] },
    ]);
  } finally {
    await session.shutdown();
  }
});
