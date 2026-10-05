import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";
import { root } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

const fixtureRoot = path.join(root, "tests/_fixtures/differential/lsp/pinia-availability");
const reference = JSON.parse(fs.readFileSync(path.join(fixtureRoot, "references.json"), "utf8"));

test("source-built LSP applies the CLI's per-project Pinia availability policy", async (t) => {
  assert.equal(reference.schema, "vize.lsp.pinia-availability.regression");
  assert.equal(reference.version, 1);
  const originals = reference.originalFiles.map(
    (row: { path: string; fixture: string; sha256: string }) => {
      const bytes = fs.readFileSync(path.join(fixtureRoot, row.fixture));
      assert.equal(createHash("sha256").update(bytes).digest("hex"), row.sha256);
      return { file: row.path, source: bytes.toString("utf8") };
    },
  );
  const source = originals.find((row: { file: string }) => row.file === "src/App.vue")!.source;
  const binary = path.join(root, "target/ci", process.platform === "win32" ? "vize.exe" : "vize");
  resolveVizeLaunchCommand(undefined, binary, { required: true });
  const previousBinary = process.env.VIZE_LSP_BIN;
  const previousRequired = process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD;
  // Keep installed repository dependencies outside the fixture's ancestor chain.
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lsp-pinia-"));
  let session: LspSession | undefined;
  try {
    process.env.VIZE_LSP_BIN = binary;
    process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD = "1";
    fs.mkdirSync(path.join(workspace, "src"));
    fs.writeFileSync(path.join(workspace, "package.json"), '{"private":true,"type":"module"}');
    fs.writeFileSync(
      path.join(workspace, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          strict: true,
          module: "ESNext",
          moduleResolution: "Bundler",
          target: "ES2022",
          noEmit: true,
        },
        include: ["src/**/*.ts", "src/**/*.vue"],
      }),
    );
    for (const row of originals) fs.writeFileSync(path.join(workspace, row.file), row.source);
    const uri = pathToFileURL(path.join(workspace, "src/App.vue")).href;
    session = new LspSession();
    await session.initialize(workspace, reference.initializationOptions);
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });
    await assertDiagnostics(uri, 1, []);
    t.diagnostic("original non-Pinia composable: complete didOpen diagnostics are empty");

    for (const preset of [undefined, "ecosystem", "opinionated"]) {
      const lint = spawnSync(
        binary,
        [
          "lint",
          "src/App.vue",
          "src/store.ts",
          "--no-config",
          "--format",
          "json",
          ...(preset ? ["--preset", preset] : []),
        ],
        { cwd: workspace, encoding: "utf8", timeout: 30_000 },
      );
      assert.ifError(lint.error);
      assert.equal(lint.status, 0, lint.stderr);
      assert.equal(lint.signal, null);
      assert.deepEqual(
        JSON.parse(lint.stdout),
        originals.map((row: { file: string }) => ({
          file: row.file,
          messages: [],
          errorCount: 0,
          warningCount: 0,
        })),
      );
      for (const row of originals) {
        assert.equal(fs.readFileSync(path.join(workspace, row.file), "utf8"), row.source);
      }
      t.diagnostic(
        `original CLI ${preset ?? "default"}: complete empty results and inputs conserved`,
      );
    }

    const pinia = path.join(workspace, "node_modules/pinia");
    fs.mkdirSync(pinia, { recursive: true });
    fs.writeFileSync(path.join(pinia, "package.json"), '{"name":"pinia","version":"3.0.4"}');
    fs.writeFileSync(path.join(workspace, "src/store.ts"), reference.piniaStore);
    await change(2, source, reference.warning);
    await change(3, reference.fixedSource, []);
    await change(4, source, reference.warning);
    t.diagnostic(
      "available ancestor Pinia: complete warning, storeToRefs recovery and warning restoration",
    );
    fs.rmSync(path.join(pinia, "package.json"));
    fs.writeFileSync(path.join(workspace, "src/store.ts"), originals[1].source);
    await change(5, source, []);
    t.diagnostic("removed Pinia identity: complete diagnostics clear without restarting the LSP");
    await session.shutdown();
    session = undefined;

    // Existing explicit rule disabling must still win when Pinia is available.
    fs.writeFileSync(path.join(pinia, "package.json"), '{"name":"pinia","version":"3.0.4"}');
    fs.writeFileSync(path.join(workspace, "src/store.ts"), reference.piniaStore);
    fs.writeFileSync(
      path.join(workspace, "vize.config.json"),
      JSON.stringify({
        linter: { rules: { "ecosystem/pinia-prefer-store-to-refs": "off" } },
      }),
    );
    session = new LspSession();
    await session.initialize(workspace, reference.initializationOptions);
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });
    await assertDiagnostics(uri, 1, []);
    await change(2, source, []);
    t.diagnostic(
      "available Pinia with explicit rule off: complete open/change diagnostics stay empty",
    );

    async function change(version: number, text: string, diagnostics: unknown[]) {
      assert.ok(session);
      session.notify("textDocument/didChange", {
        textDocument: { uri, version },
        contentChanges: [{ text }],
      });
      await assertDiagnostics(uri, version, diagnostics);
    }
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
