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
  "../../tests/_fixtures/differential/lsp/pug-multiline-attributes-8908",
);

type FixtureCase = {
  provenance: { fixCommit: string };
  input: { source: string; runtimePath: string; sha256: string };
  expected: { source: string; sha256: string };
  documentVersion: number;
  languageId: string;
  initializationOptions: LspInitializationOptions;
  method: string;
  request: { position: LspPosition };
};

void test("multiline Pug attribute history preserves the complete binding location", async () => {
  const fixture = JSON.parse(
    fs.readFileSync(path.join(fixtureDir, "definition.case.json"), "utf8"),
  ) as FixtureCase;
  assert.equal(fixture.provenance.fixCommit, "2c4b8275af5548cf9e29ec377cc4afc4211f56f9");
  assert.equal(fixture.method, "textDocument/definition");

  const source = fs.readFileSync(path.join(fixtureDir, fixture.input.source), "utf8");
  assert.equal(createHash("sha256").update(source).digest("hex"), fixture.input.sha256);
  const expectedText = fs.readFileSync(path.join(fixtureDir, fixture.expected.source), "utf8");
  assert.equal(createHash("sha256").update(expectedText).digest("hex"), fixture.expected.sha256);
  const expected = JSON.parse(expectedText) as unknown;
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-pug-multiline-history-"));
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
