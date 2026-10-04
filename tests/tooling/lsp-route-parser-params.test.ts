import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

const fixtureRoot = path.join(root, "tests/_fixtures/differential/lsp/router-param-parsers");
const reference = JSON.parse(fs.readFileSync(path.join(fixtureRoot, "references.json"), "utf8"));

test("source-built LSP recognizes file-route parser names and clears genuine unknown params", async (t) => {
  const reported = fs.readFileSync(path.join(fixtureRoot, "Reported.vue.txt"));
  assert.equal(createHash("sha256").update(reported).digest("hex"), reference.originalSourceSha256);
  assert.equal(reference.schema, "vize.lsp.route-param-parser.regression");
  assert.equal(reference.version, 1);
  assert.equal(reference.cases.length, 7);
  assert.equal(reference.cases[0].source, reported.toString("utf8"));

  // Unlike the historical merge-only sessions, this bounded regression also
  // runs on PRs and requires their existing receipted source-built executable.
  const binary = path.join(root, "target/ci", process.platform === "win32" ? "vize.exe" : "vize");
  resolveVizeLaunchCommand(undefined, binary, { required: true });
  const previousBinary = process.env.VIZE_LSP_BIN;
  const previousRequired = process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD;
  fs.mkdirSync(testOutputRoot, { recursive: true });
  const workspace = fs.mkdtempSync(path.join(testOutputRoot, "lsp-route-parser-params-"));
  let session: LspSession | undefined;
  try {
    process.env.VIZE_LSP_BIN = binary;
    process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD = "1";
    session = new LspSession();
    await session.initialize(workspace, reference.initializationOptions);
    for (const row of reference.cases) {
      const file = path.join(workspace, row.file);
      const uri = pathToFileURL(file).href;
      fs.mkdirSync(path.dirname(file), { recursive: true });
      fs.writeFileSync(file, row.source);
      session.notify("textDocument/didOpen", {
        textDocument: { uri, languageId: "vue", version: 1, text: row.source },
      });
      await assertDiagnostics(uri, 1, []);
      const completion = await session.request("textDocument/completion", {
        textDocument: { uri },
        position: row.completionPosition,
      });
      assert.deepEqual(completion, row.completion, row.id);

      session.notify("textDocument/didChange", {
        textDocument: { uri, version: 2 },
        contentChanges: [{ text: row.invalidSource }],
      });
      await assertDiagnostics(uri, 2, row.invalidDiagnostics);
      session.notify("textDocument/didChange", {
        textDocument: { uri, version: 3 },
        contentChanges: [{ text: row.source }],
      });
      await assertDiagnostics(uri, 3, []);
      t.diagnostic(
        `route parser corpus ${row.id}: complete open/change/recovery and completion passed`,
      );
    }

    // The report distinguishes the existing Patina CLI from editor ecosystem
    // diagnostics. Preserve its complete empty diagnostic result and input.
    const lint = spawnSync(
      binary,
      [
        "lint",
        ...reference.cases.map((row: { file: string }) => row.file),
        "--no-config",
        "--format",
        "json",
      ],
      {
        cwd: workspace,
        encoding: "utf8",
        timeout: 30_000,
      },
    );
    assert.ifError(lint.error);
    assert.equal(lint.status, 0, lint.stderr);
    assert.equal(lint.signal, null);
    const files = JSON.parse(lint.stdout);
    const expectedFiles = reference.cases
      .map((row: { file: string }) => ({
        file: row.file,
        messages: [],
        errorCount: 0,
        warningCount: 0,
      }))
      .sort((left: { file: string }, right: { file: string }) =>
        left.file < right.file ? -1 : left.file > right.file ? 1 : 0,
      );
    assert.deepEqual(files, expectedFiles);
    for (const row of reference.cases) {
      assert.equal(fs.readFileSync(path.join(workspace, row.file), "utf8"), row.source);
    }
    t.diagnostic(
      "source-built CLI lint: all seven authored page inputs have complete empty diagnostics",
    );
  } finally {
    try {
      await session?.shutdown();
    } finally {
      if (previousBinary === undefined) delete process.env.VIZE_LSP_BIN;
      else process.env.VIZE_LSP_BIN = previousBinary;
      if (previousRequired === undefined) delete process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD;
      else process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD = previousRequired;
      fs.rmSync(workspace, { recursive: true, force: true });
    }
  }

  async function assertDiagnostics(uri: string, version: number, diagnostics: unknown[]) {
    assert.ok(session);
    const publication = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri) && params.version === version,
    )) as PublishDiagnosticsParams;
    assert.deepEqual(publication, { uri, version, diagnostics });
  }
});
