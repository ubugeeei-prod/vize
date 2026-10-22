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
  "../../tests/_fixtures/differential/lsp/pug-attribute-navigation-8908",
);

type FixtureFile = {
  source: string;
  runtimePath: string;
  sha256: string;
};

type FixtureCase = {
  provenance: { fixCommit: string };
  files: FixtureFile[];
  entry: string;
  expected: { source: string; sha256: string };
  documentVersion: number;
  languageId: string;
  initializationOptions: LspInitializationOptions;
  method: string;
  request: { position: LspPosition };
};

void test("Pug attribute definition history preserves the complete authored location", async () => {
  const fixture = JSON.parse(
    fs.readFileSync(path.join(fixtureDir, "definition.case.json"), "utf8"),
  ) as FixtureCase;
  assert.equal(fixture.provenance.fixCommit, "2d421a2b428015d7de1f1c6396fe0e0154a7263b");
  assert.equal(fixture.method, "textDocument/definition");

  const expectedText = fs.readFileSync(path.join(fixtureDir, fixture.expected.source), "utf8");
  assert.equal(createHash("sha256").update(expectedText).digest("hex"), fixture.expected.sha256);
  const expected = JSON.parse(expectedText) as unknown;
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-pug-attribute-history-"));
  const entry = path.join(workspace, fixture.entry);
  const uri = pathToFileURL(entry).href;
  const session = new LspSession();

  try {
    for (const file of fixture.files) {
      const source = fs.readFileSync(path.join(fixtureDir, file.source), "utf8");
      assert.equal(createHash("sha256").update(source).digest("hex"), file.sha256);
      const runtimePath = path.join(workspace, file.runtimePath);
      fs.mkdirSync(path.dirname(runtimePath), { recursive: true });
      fs.writeFileSync(runtimePath, source, "utf8");
    }
    const source = fs.readFileSync(entry, "utf8");
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
