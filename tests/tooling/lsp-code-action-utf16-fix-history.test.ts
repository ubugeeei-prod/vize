import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { materializeWorkspace } from "../differential/lsp-manifest.ts";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import type { LspDiagnostic, PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixtureDir = path.join(root, "tests/_fixtures/differential/lsp/code-action-utf16-original");

type FixtureCase = {
  provenance: { fixCommit: string };
  input: { runtimePath: string };
  initializationOptions: {
    editor: boolean;
    lint: boolean;
    typecheck: boolean;
    codeActions: boolean;
  };
  documentVersion: number;
  method: string;
  request: {
    params: {
      range: LspDiagnostic["range"];
    };
  };
};

void test("UTF-16 code-action history reuses the published diagnostic in the complete response", async () => {
  const fixture = JSON.parse(
    fs.readFileSync(path.join(fixtureDir, "case.json"), "utf8"),
  ) as FixtureCase;
  assert.equal(fixture.provenance.fixCommit, "4671a8c0f9c4d0127aba18aa2a108f2e6152de6a");
  assert.equal(fixture.method, "textDocument/codeAction");

  const source = fs.readFileSync(path.join(fixtureDir, "CodeActionUtf16.vue.txt"), "utf8");
  const expected = JSON.parse(
    fs.readFileSync(
      path.join(fixtureDir, "response.with-published-diagnostic.expected.json"),
      "utf8",
    ),
  ) as unknown;
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-code-action-utf16-"));
  const entry = path.join(workspace, fixture.input.runtimePath);
  const uri = pathToFileURL(entry).href;
  const session = new LspSession();

  try {
    fs.writeFileSync(entry, source, "utf8");
    await session.initialize(workspace, fixture.initializationOptions);
    session.notify("textDocument/didOpen", {
      textDocument: {
        uri,
        languageId: "vue",
        version: fixture.documentVersion,
        text: source,
      },
    });

    const published = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, uri) &&
        params.version === fixture.documentVersion &&
        params.diagnostics.some(
          (diagnostic) =>
            diagnostic.source === "vize/lint" && diagnostic.code === "vue/no-multi-spaces",
        ),
    )) as PublishDiagnosticsParams;
    const diagnostic = published.diagnostics.find(
      (candidate): candidate is LspDiagnostic =>
        candidate.source === "vize/lint" && candidate.code === "vue/no-multi-spaces",
    );
    assert(diagnostic, JSON.stringify(published));
    assert.deepEqual(diagnostic.range, fixture.request.params.range);

    const response = await session.request(fixture.method, {
      textDocument: { uri },
      range: fixture.request.params.range,
      context: { diagnostics: [diagnostic] },
    });
    assert.deepEqual(response, materializeWorkspace(expected, pathToFileURL(workspace).href));
  } finally {
    await session.shutdown();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
