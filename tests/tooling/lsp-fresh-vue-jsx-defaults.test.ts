import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { readPolicyFixture } from "./support/lsp/fresh-capability-policy.ts";
import { withFreshVueProject } from "./support/lsp/fresh-vue-project.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import type { LspSession } from "./support/lsp/session.ts";

const expected = JSON.parse(readPolicyFixture("jsx-diagnostics.authored.json"));

async function publication(
  session: LspSession,
  uri: string,
  version: number,
): Promise<PublishDiagnosticsParams> {
  return (await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) => isDiagnosticsForUri(params, uri) && params.version === version,
  )) as PublishDiagnosticsParams;
}

function assertPublication(
  packet: PublishDiagnosticsParams,
  name: string,
  version: number,
  diagnostics: unknown,
): void {
  // Only the fresh temporary workspace URI changes; every other packet field is retained.
  assert.deepEqual(
    { ...packet, uri: `file:///<workspace>/src/${name}` },
    {
      uri: `file:///<workspace>/src/${name}`,
      version,
      diagnostics,
    },
  );
}

for (const [extension, languageId] of [
  ["tsx", "typescriptreact"],
  ["jsx", "javascriptreact"],
]) {
  test(`fresh Vue ${extension} reports the original SFC prop error and repairs it without dedicated configuration`, async (t) => {
    await withFreshVueProject(t, `fresh-${extension}-repair`, async (session, workspaceDir) => {
      const name = `Consumer.${extension}`;
      const file = path.join(workspaceDir, "src", name);
      const uri = pathToFileURL(file).href;
      const broken = readPolicyFixture(`Consumer.invalid.${extension}.txt`);
      const repaired = readPolicyFixture(`Consumer.valid.${extension}.txt`);
      fs.writeFileSync(file, broken);
      await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: true });
      session.notify("textDocument/didOpen", {
        textDocument: { uri, languageId, version: 1, text: broken },
      });
      const invalid = await publication(session, uri, 1);
      session.notify("textDocument/didChange", {
        textDocument: { uri, version: 2 },
        contentChanges: [{ text: repaired }],
      });
      const clean = await publication(session, uri, 2);
      assertPublication(invalid, name, 1, expected.invalid);
      assertPublication(clean, name, 2, expected.valid);
    });
  });

  test(`fresh Vue ${extension} accepts the original valid SFC props without dedicated configuration`, async (t) => {
    await withFreshVueProject(t, `fresh-${extension}-valid`, async (session, workspaceDir) => {
      const name = `Consumer.${extension}`;
      const file = path.join(workspaceDir, "src", name);
      const uri = pathToFileURL(file).href;
      const text = readPolicyFixture(`Consumer.valid.${extension}.txt`);
      fs.writeFileSync(file, text);
      await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: true });
      session.notify("textDocument/didOpen", {
        textDocument: { uri, languageId, version: 1, text },
      });
      assertPublication(await publication(session, uri, 1), name, 1, expected.valid);
    });
  });
}

test("fresh Vue TSX accepts the original intrinsic element control without dedicated configuration", async (t) => {
  await withFreshVueProject(t, "fresh-tsx-intrinsic", async (session, workspaceDir) => {
    const name = "Intrinsic.tsx";
    const file = path.join(workspaceDir, "src", name);
    const uri = pathToFileURL(file).href;
    const text = readPolicyFixture("Intrinsic.tsx.txt");
    fs.writeFileSync(file, text);
    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: true });
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "typescriptreact", version: 1, text },
    });
    assertPublication(await publication(session, uri, 1), name, 1, expected.intrinsic);
  });
});
