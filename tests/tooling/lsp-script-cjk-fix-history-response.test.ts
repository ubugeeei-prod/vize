import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { materializeWorkspace } from "../differential/lsp-manifest.ts";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import type { LspInitializationOptions, LspPosition } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

const fixtureDir = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../tests/_fixtures/differential/lsp/script-cjk-navigation",
);

type FixtureCase = {
  provenance: { fixCommit: string };
  input: { source: string; runtimePath: string; sha256: string };
  expected: { source: string };
  documentVersion: number;
  languageId: string;
  initializationOptions: LspInitializationOptions;
  method: string;
  request: { position: LspPosition };
};

void test("Unicode script definition history preserves the complete UTF-16 location", async () => {
  const fixture = JSON.parse(
    fs.readFileSync(path.join(fixtureDir, "case.json"), "utf8"),
  ) as FixtureCase;
  assert.equal(fixture.provenance.fixCommit, "370caeafee07cec57e78a7b0c75569b299ccd160");
  assert.equal(fixture.method, "textDocument/definition");

  const source = fs.readFileSync(path.join(fixtureDir, fixture.input.source), "utf8");
  assert.equal(createHash("sha256").update(source).digest("hex"), fixture.input.sha256);
  const expected = JSON.parse(
    fs.readFileSync(path.join(fixtureDir, fixture.expected.source), "utf8"),
  ) as unknown;
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-script-cjk-history-"));
  const entry = path.join(workspace, fixture.input.runtimePath);
  const uri = pathToFileURL(entry).href;
  const session = new LspSession();

  try {
    fs.writeFileSync(entry, source, "utf8");
    await session.initialize(workspace, fixture.initializationOptions);
    session.notify("textDocument/didOpen", {
      textDocument: {
        uri,
        languageId: fixture.languageId,
        version: fixture.documentVersion,
        text: source,
      },
    });
    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, uri),
    );

    const response = await session.request(fixture.method, {
      textDocument: { uri },
      position: fixture.request.position,
    });
    assert.deepEqual(response, materializeWorkspace(expected, pathToFileURL(workspace).href));
  } finally {
    await session.shutdown();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
