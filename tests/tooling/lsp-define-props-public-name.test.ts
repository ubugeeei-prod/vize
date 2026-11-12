import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { LspInitializationOptions, PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

const fixtureDir = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../tests/_fixtures/differential/lsp/define-props-public-name",
);

type FixtureCase = {
  files: Array<{ source: string; runtimePath: string; sha256: string }>;
  entry: string;
  expected: { source: string; sha256: string };
  documentVersion: number;
  initializationOptions: LspInitializationOptions;
  initialDiagnostics: unknown[];
  method: string;
};

/**
 * Editor diagnostics name a `defineProps<T>()` result the way Vue declares it
 * (`DefineProps<LooseRequired<T>, K>`), never Vize's private `__DefineProps`
 * copy, exactly as the pinned initial packet records.
 */
test("editor diagnostics name defineProps results by Vue's public type names", async () => {
  const fixture = JSON.parse(
    fs.readFileSync(path.join(fixtureDir, "case.json"), "utf8"),
  ) as FixtureCase;
  fs.mkdirSync(testOutputRoot, { recursive: true });
  const workspace = fs.mkdtempSync(path.join(testOutputRoot, "define-props-public-name-"));
  const session = new LspSession();
  try {
    for (const file of fixture.files) {
      const bytes = fs.readFileSync(path.join(fixtureDir, file.source));
      assert.equal(createHash("sha256").update(bytes).digest("hex"), file.sha256);
      fs.writeFileSync(path.join(workspace, file.runtimePath), bytes);
    }
    const entry = path.join(workspace, fixture.entry);
    const uri = pathToFileURL(entry).href;
    await session.initialize(workspace, fixture.initializationOptions);
    session.notify("textDocument/didOpen", {
      textDocument: {
        uri,
        languageId: "vue",
        version: fixture.documentVersion,
        text: fs.readFileSync(entry, "utf8"),
      },
    });
    const published = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri),
      60_000,
    )) as PublishDiagnosticsParams;
    assert.deepEqual(published.diagnostics, fixture.initialDiagnostics);

    const colors = await session.request(fixture.method, { textDocument: { uri } });
    const expectedBytes = fs.readFileSync(path.join(fixtureDir, fixture.expected.source));
    assert.equal(createHash("sha256").update(expectedBytes).digest("hex"), fixture.expected.sha256);
    assert.deepEqual(colors, JSON.parse(expectedBytes.toString("utf8")));
  } finally {
    await session.shutdown().catch(() => undefined);
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
