import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import {
  firstLocation,
  hoverToText,
  isDiagnosticsForUri,
  offsetToPosition,
} from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

// True multi-root workspace coverage (#3240, audit item 6 of #2971): a single
// `vize lsp` session initialized with TWO `workspaceFolders` must
//   (a) resolve diagnostics against each folder's own configuration context,
//   (b) keep the folders independent (editing a file in one root neither
//       clears nor perturbs the other root's published diagnostics, and
//       per-document versions stay monotonic), and
//   (c) answer definition/hover for documents of both roots.
// Both roots carry their own tsconfig.json; root B additionally ships a
// vize.config.json that turns `vue/require-v-for-key` off, so identical file
// contents produce a lint diagnostic in root A but none in root B. That
// asymmetry is what proves the configuration contexts are actually separate.

const LIST_SOURCE = `<script setup lang="ts">
const rows = ['alpha', 'beta']
</script>

<template>
  <ul>
    <li v-for="row in rows">{{ row }}</li>
  </ul>
</template>
`;

const V_FOR_KEY_RULE = "vue/require-v-for-key";

type WorkspaceRoot = {
  dir: string;
  fileUri: string;
};

function createRoot(
  parentDir: string,
  name: string,
  options: { strict: boolean; disableVForKeyRule: boolean },
): WorkspaceRoot {
  const dir = path.join(parentDir, name);
  fs.mkdirSync(dir, { recursive: true });

  // Each root owns a tsconfig so the fixture matches a real multi-root
  // editor session where every folder is its own TypeScript project.
  fs.writeFileSync(
    path.join(dir, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        module: "ESNext",
        moduleResolution: "bundler",
        noEmit: true,
        strict: options.strict,
        target: "ES2022",
      },
      include: ["**/*"],
    }),
    "utf8",
  );

  if (options.disableVForKeyRule) {
    fs.writeFileSync(
      path.join(dir, "vize.config.json"),
      JSON.stringify({ linter: { rules: { [V_FOR_KEY_RULE]: "off" } } }),
      "utf8",
    );
  }

  const filePath = path.join(dir, "List.vue");
  fs.writeFileSync(filePath, LIST_SOURCE, "utf8");
  return { dir, fileUri: pathToFileURL(filePath).href };
}

function hasVForKeyDiagnostic(publish: PublishDiagnosticsParams): boolean {
  return publish.diagnostics.some(
    (diagnostic) => diagnostic.source === "vize/lint" && diagnostic.code === V_FOR_KEY_RULE,
  );
}

async function openAndCollect(
  session: LspSession,
  root: WorkspaceRoot,
): Promise<PublishDiagnosticsParams> {
  session.notify("textDocument/didOpen", {
    textDocument: { uri: root.fileUri, languageId: "vue", version: 1, text: LIST_SOURCE },
  });
  return (await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
    isDiagnosticsForUri(params, root.fileUri),
  )) as PublishDiagnosticsParams;
}

test("vize lsp resolves per-root config and navigation across two workspace folders", async (t) => {
  const testRootDir = path.join(testOutputRoot, "lsp-multi-root-workspace");
  fs.mkdirSync(testRootDir, { recursive: true });
  const parentDir = fs.mkdtempSync(path.join(testRootDir, "roots-"));
  const rootA = createRoot(parentDir, "root-a", { strict: true, disableVForKeyRule: false });
  const rootB = createRoot(parentDir, "root-b", { strict: false, disableVForKeyRule: true });
  const session = new LspSession();

  try {
    await session.initialize([rootA.dir, rootB.dir], {
      editor: true,
      lint: true,
      typecheck: false,
    });

    const publishA = await openAndCollect(session, rootA);
    const publishB = await openAndCollect(session, rootB);

    await t.test("diagnostics follow each folder's own lint configuration", () => {
      assert.equal(publishA.version, 1);
      assert.equal(publishB.version, 1);
      assert.ok(
        hasVForKeyDiagnostic(publishA),
        `root A should surface ${V_FOR_KEY_RULE}: ${JSON.stringify(publishA.diagnostics)}`,
      );
      assert.ok(
        !hasVForKeyDiagnostic(publishB),
        `root B disables ${V_FOR_KEY_RULE} in its own vize.config.json: ${JSON.stringify(
          publishB.diagnostics,
        )}`,
      );
    });

    await t.test("definition and hover work for documents of both roots", async () => {
      for (const root of [rootA, rootB]) {
        const usageOffset = LIST_SOURCE.lastIndexOf("rows") + "rows".length;
        const definition = (await session.request("textDocument/definition", {
          textDocument: { uri: root.fileUri },
          position: offsetToPosition(LIST_SOURCE, usageOffset),
        })) as
          | { uri: string; range: { start: { line: number; character: number } } }
          | Array<{ uri: string; range: { start: { line: number; character: number } } }>
          | null;
        assert.ok(definition, `definition should resolve in ${root.dir}`);
        const location = firstLocation(definition);
        assert.equal(location.uri, root.fileUri);
        assert.deepEqual(
          location.range.start,
          offsetToPosition(LIST_SOURCE, LIST_SOURCE.indexOf("rows")),
        );

        const hover = (await session.request("textDocument/hover", {
          textDocument: { uri: root.fileUri },
          position: offsetToPosition(LIST_SOURCE, usageOffset),
        })) as { contents?: unknown } | null;
        assert.match(hoverToText(hover), /rows/);
      }
    });

    await t.test("editing root A leaves root B's published diagnostics untouched", async () => {
      const changedA = LIST_SOURCE.replace("'alpha', 'beta'", "'alpha', 'beta', 'gamma'");
      session.notify("textDocument/didChange", {
        textDocument: { uri: rootA.fileUri, version: 2 },
        contentChanges: [{ text: changedA }],
      });

      const republishA = (await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) => isDiagnosticsForUri(params, rootA.fileUri) && params.version === 2,
      )) as PublishDiagnosticsParams;
      assert.ok(
        hasVForKeyDiagnostic(republishA),
        `root A keeps its lint context after the edit: ${JSON.stringify(republishA.diagnostics)}`,
      );

      // Root B must not receive any publish triggered by root A's edit: no
      // clearing, no config-context bleed, no version churn.
      await assert.rejects(
        session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) => isDiagnosticsForUri(params, rootB.fileUri),
          1500,
        ),
        /Timed out/,
        "root A's didChange must not republish root B's document",
      );

      // Root B's own next edit still resolves root B's config (rule stays
      // off) and its version advances monotonically from 1 to 2.
      const changedB = LIST_SOURCE.replace("'alpha', 'beta'", "'alpha'");
      session.notify("textDocument/didChange", {
        textDocument: { uri: rootB.fileUri, version: 2 },
        contentChanges: [{ text: changedB }],
      });
      const republishB = (await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) => isDiagnosticsForUri(params, rootB.fileUri) && params.version === 2,
      )) as PublishDiagnosticsParams;
      assert.ok(
        !hasVForKeyDiagnostic(republishB),
        `root B keeps its own lint context after edits: ${JSON.stringify(republishB.diagnostics)}`,
      );
    });
  } finally {
    await session.shutdown();
    fs.rmSync(parentDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

// The same fixture with the folder order reversed must produce the same
// per-root outcomes. This pins down that per-root behavior is derived from
// each document's OWN enclosing folder, not from whichever folder happens to
// be listed first (the classic "primary root wins" failure mode).
test("vize lsp multi-root config contexts do not depend on folder order", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-multi-root-order");
  fs.mkdirSync(testRootDir, { recursive: true });
  const parentDir = fs.mkdtempSync(path.join(testRootDir, "roots-"));
  const rootA = createRoot(parentDir, "root-a", { strict: true, disableVForKeyRule: false });
  const rootB = createRoot(parentDir, "root-b", { strict: false, disableVForKeyRule: true });
  const session = new LspSession();

  try {
    // Root B (the rule-off folder) is listed FIRST this time.
    await session.initialize([rootB.dir, rootA.dir], {
      editor: true,
      lint: true,
      typecheck: false,
    });

    const publishB = await openAndCollect(session, rootB);
    const publishA = await openAndCollect(session, rootA);

    assert.ok(
      !hasVForKeyDiagnostic(publishB),
      `root B's own config still disables ${V_FOR_KEY_RULE}: ${JSON.stringify(
        publishB.diagnostics,
      )}`,
    );
    assert.ok(
      hasVForKeyDiagnostic(publishA),
      `root A must not inherit root B's rule-off config just because B is first: ${JSON.stringify(
        publishA.diagnostics,
      )}`,
    );
  } finally {
    await session.shutdown();
    fs.rmSync(parentDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
