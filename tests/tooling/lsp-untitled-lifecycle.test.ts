import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

test("vize lsp keeps an untitled Vue document live through open, change, and close", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-untitled-lifecycle");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();
  const uri = "untitled:Untitled-1";

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    });

    const malformedSource = "<template><div></div>";
    session.notify("textDocument/didOpen", {
      textDocument: {
        uri,
        languageId: "vue",
        version: 1,
        text: malformedSource,
      },
    });

    const openPublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri),
    )) as PublishDiagnosticsParams;
    assert.equal(openPublish.version, 1);
    assert.deepEqual(openPublish.diagnostics, [
      {
        message: "Malformed <template> block: the closing tag is missing.",
        range: {
          start: { line: 0, character: 0 },
          end: { line: 0, character: malformedSource.length },
        },
        severity: 1,
        source: "vize/sfc",
      },
    ]);

    session.notify("textDocument/didChange", {
      textDocument: { uri, version: 2 },
      contentChanges: [
        {
          range: {
            start: { line: 0, character: malformedSource.length },
            end: { line: 0, character: malformedSource.length },
          },
          text: "</template>",
        },
      ],
    });

    const changePublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri) && params.version === 2,
    )) as PublishDiagnosticsParams;
    assert.equal(changePublish.version, 2);
    assert.deepEqual(changePublish.diagnostics, []);

    session.notify("textDocument/didClose", {
      textDocument: { uri },
    });

    const closePublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri) && params.version === undefined,
    )) as PublishDiagnosticsParams;
    assert.equal(closePublish.version, undefined);
    assert.deepEqual(closePublish.diagnostics, []);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
