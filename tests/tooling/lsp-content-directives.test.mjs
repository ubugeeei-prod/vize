import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";
import { root } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

await test("genuine source-built LSP applies exact configured content directives and preserves whole findings", async () => {
  const fixture = path.join(root, "crates/vize_patina/tests/fixtures/issue-8018");
  const manifest = JSON.parse(fs.readFileSync(path.join(fixture, "source.json"), "utf8"));
  for (const row of [...manifest.inputs, ...manifest.authored]) {
    const bytes = fs.readFileSync(path.join(fixture, row.file));
    assert.equal(bytes.length, row.bytes);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), row.sha256);
  }
  const reference = JSON.parse(fs.readFileSync(path.join(fixture, "references.json"), "utf8"));
  assert.equal(reference.scenarios.length, 11);
  const binary = path.join(root, "target/ci", process.platform === "win32" ? "vize.exe" : "vize");
  resolveVizeLaunchCommand(undefined, binary, { required: true });
  const artifact = path.join(root, "target/differential/lsp-content-directives.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = {
    schema: "vize.lsp.content-directives.execution",
    manifest,
    initializationOptions: reference.initializationOptions,
    sessions: [],
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const previousBinary = process.env.VIZE_LSP_BIN;
  const previousRequired = process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD;
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lsp-content-directives-"));
  let session;
  try {
    process.env.VIZE_LSP_BIN = binary;
    process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD = "1";
    for (const row of manifest.inputs)
      fs.copyFileSync(path.join(fixture, row.file), path.join(workspace, row.path));
    fs.copyFileSync(
      path.join(fixture, "Controls.vue.fixture"),
      path.join(workspace, "Controls.vue"),
    );
    for (const scenario of reference.scenarios) {
      fs.writeFileSync(path.join(workspace, "vize.config.json"), JSON.stringify(scenario.config));
      session = new LspSession();
      const observation = { id: scenario.id, config: scenario.config, publications: [] };
      evidence.sessions.push(observation);
      persist();
      try {
        await session.initialize(workspace, reference.initializationOptions);
        const text = fs.readFileSync(path.join(workspace, scenario.file), "utf8");
        const uri = pathToFileURL(path.join(workspace, scenario.file)).href;
        for (let version = 1; version <= 3; version++) {
          // Same-length unsaved edits preserve all authored ranges; restore originals.
          const content =
            version === 2 ? text.replace("Saved", "Ready").replace('"notice"', '"ready!"') : text;
          if (version === 1)
            session.notify("textDocument/didOpen", {
              textDocument: { uri, languageId: "vue", version, text: content },
            });
          else
            session.notify("textDocument/didChange", {
              textDocument: { uri, version },
              contentChanges: [{ text: content }],
            });
          const publication = await session.waitForNotification(
            "textDocument/publishDiagnostics",
            (params) => isDiagnosticsForUri(params, uri) && params.version === version,
          );
          observation.publications.push({
            version,
            content,
            expected: { uri, version, diagnostics: scenario.expectedLsp },
            actual: publication,
          });
          persist();
          assert.deepEqual(
            publication,
            { uri, version, diagnostics: scenario.expectedLsp },
            scenario.id,
          );
        }
      } finally {
        try {
          await session.shutdown();
        } finally {
          session = undefined;
        }
      }
    }
    for (const row of manifest.inputs) {
      if (row.path !== "vize.config.json")
        assert.equal(
          createHash("sha256")
            .update(fs.readFileSync(path.join(workspace, row.path)))
            .digest("hex"),
          row.sha256,
        );
    }
  } finally {
    try {
      await session?.shutdown();
    } finally {
      try {
        persist();
      } finally {
        if (previousBinary === undefined) delete process.env.VIZE_LSP_BIN;
        else process.env.VIZE_LSP_BIN = previousBinary;
        if (previousRequired === undefined) delete process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD;
        else process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD = previousRequired;
        fs.rmSync(workspace, { recursive: true, force: true });
      }
    }
  }
});
