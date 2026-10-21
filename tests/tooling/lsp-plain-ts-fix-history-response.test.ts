import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { materializeWorkspace } from "../differential/lsp-manifest.ts";
import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { root } from "./support/lsp/paths.ts";
import type { LspInitializationOptions, PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

const fixtureDir = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../tests/_fixtures/differential/lsp/plain-ts-typecheck",
);

type FixtureCase = {
  provenance: { fixCommit: string };
  input: { source: string; runtimePath: string; sha256: string };
  config: { source: string; runtimePath: string };
  expected: { source: string };
  documentVersion: number;
  languageId: string;
  initializationOptions: LspInitializationOptions;
  method: string;
};

void test("plain TypeScript fix history publishes the complete TS2322 response", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "TypeScript 7/Corsa runtime for plain TypeScript response history",
    "TypeScript 7/Corsa runtime not found; skipping plain TypeScript response history",
  );
  if (corsaPath == null) return;

  const fixture = JSON.parse(
    fs.readFileSync(path.join(fixtureDir, "case.json"), "utf8"),
  ) as FixtureCase;
  assert.equal(fixture.provenance.fixCommit, "7100d7a790146d4252103088f69dbf15756e36a5");
  assert.equal(fixture.method, "textDocument/publishDiagnostics");

  const source = fs.readFileSync(path.join(fixtureDir, fixture.input.source), "utf8");
  assert.equal(createHash("sha256").update(source).digest("hex"), fixture.input.sha256);
  const config = fs.readFileSync(path.join(fixtureDir, fixture.config.source), "utf8");
  const expected = JSON.parse(
    fs.readFileSync(path.join(fixtureDir, fixture.expected.source), "utf8"),
  ) as unknown;
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-plain-ts-history-"));
  const entry = path.join(workspace, fixture.input.runtimePath);
  const uri = pathToFileURL(entry).href;
  const session = new LspSession();

  try {
    fs.writeFileSync(entry, source, "utf8");
    fs.writeFileSync(path.join(workspace, fixture.config.runtimePath), config, "utf8");
    writeVueConfigFixture(workspace, {
      lsp: { lint: false, typecheck: true },
      typeChecker: { corsaPath },
    });
    await session.initialize(workspace, fixture.initializationOptions);
    session.notify("textDocument/didOpen", {
      textDocument: {
        uri,
        languageId: fixture.languageId,
        version: fixture.documentVersion,
        text: source,
      },
    });

    const published = (await session.waitForNotification(
      fixture.method,
      (params) =>
        isDiagnosticsForUri(params, uri) &&
        params.version === fixture.documentVersion &&
        params.diagnostics.some((diagnostic) => diagnostic.code === 2322),
    )) as PublishDiagnosticsParams;
    assert.deepEqual(published, materializeWorkspace(expected, pathToFileURL(workspace).href));
  } finally {
    await session.shutdown();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
