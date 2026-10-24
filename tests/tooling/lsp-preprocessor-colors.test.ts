import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { loadLspManifest } from "../differential/lsp-manifest.ts";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { root } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

const manifest = path.join(root, "tests/_fixtures/differential/lsp/manifest.json");
const fixture = loadLspManifest(manifest).cases.find(
  (row) => row.id === "lsp/regression/stylus-colors-10442",
);
assert.ok(fixture, "pinned Stylus colour fixture is required");
const expected = fixture.requests[0].result;
assert.ok(Array.isArray(expected));
assert.equal(expected.length, 5, "all five authored colour values are required");

test("indented colour requests keep complete authored vectors across preprocessors", async () => {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-indented-colours-"));
  const session = new LspSession();
  try {
    await session.initialize(workspace, { editor: true, typecheck: false, lint: false });
    let version = 0;
    for (const file of fixture.files) {
      for (const lang of ["sass", "stylus", "styl"]) {
        for (const ending of ["\n", "\r\n"]) {
          version += 1;
          const source = file.bytes
            .toString("utf8")
            .replace('lang="stylus"', `lang="${lang}"`)
            .replaceAll("\n", ending);
          const target = path.join(workspace, `Case${version}.vue`);
          const uri = pathToFileURL(target).href;
          fs.writeFileSync(target, source);
          session.notify("textDocument/didOpen", {
            textDocument: { uri, languageId: "vue", version: 1, text: source },
          });
          await session.waitForNotification(
            "textDocument/publishDiagnostics",
            (params) => isDiagnosticsForUri(params, uri),
            120_000,
          );
          const result = await session.request("textDocument/documentColor", {
            textDocument: { uri },
          });
          assert.deepEqual(
            result,
            expected,
            `${file.runtimePath}/${lang}/${JSON.stringify(ending)}`,
          );
          session.notify("textDocument/didClose", { textDocument: { uri } });
        }
      }
    }
    assert.equal(version, 18, "every authored preprocessor and newline cohort must execute");
  } finally {
    await session.shutdown();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
