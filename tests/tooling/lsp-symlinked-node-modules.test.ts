/**
 * Regression oracle for a symlinked `node_modules` under the LSP.
 *
 * `overlay_root_for_project` composes the Corsa overlay root by joining
 * `node_modules/.vize/corsa-overlay` onto the project root. When
 * `node_modules` is a symlink that path traverses the link, so the overlay is
 * written to the link target — outside the project root — and diagnostics come
 * back under a path that no longer maps to the authored SFC. The failure mode
 * is a silent absence of diagnostics, so this asserts the diagnostic is
 * present rather than asserting on any error text.
 *
 * Layouts that reach `node_modules` through a link are ordinary: pnpm-style
 * stores, monorepo hoisting shims, worktree CI lanes, and containers that
 * bind-mount dependencies.
 *
 * The batch/`vize check` half of this defect is #3320.
 */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

const repoRoot = path.resolve(import.meta.dirname, "../..");

const SOURCE = `<script setup lang="ts">
const total: number = 'not a number'
</script>

<template>
  <p>{{ total }}</p>
</template>
`;

/**
 * Build a workspace whose `node_modules` is either a real directory or a
 * symlink into an out-of-tree store. Both layouts carry the same `vue`
 * package, so the only variable is the link.
 */
function createWorkspace(caseName: string, linkNodeModules: boolean): string {
  const caseRoot = path.join(testOutputRoot, `lsp-symlinked-node-modules-${caseName}`);
  fs.rmSync(caseRoot, { recursive: true, force: true });
  const workspaceDir = path.join(caseRoot, "workspace");
  fs.mkdirSync(workspaceDir, { recursive: true });

  const vuePackage = [
    path.join(repoRoot, "node_modules", "vue"),
    path.join(repoRoot, "tests", "node_modules", "vue"),
  ].find((candidate) => fs.existsSync(candidate));
  assert.ok(vuePackage, "the repo workspace must have vue installed");

  if (linkNodeModules) {
    const store = path.join(caseRoot, "store");
    fs.mkdirSync(store, { recursive: true });
    fs.symlinkSync(vuePackage, path.join(store, "vue"), "dir");
    fs.symlinkSync(store, path.join(workspaceDir, "node_modules"), "dir");
  } else {
    const nodeModules = path.join(workspaceDir, "node_modules");
    fs.mkdirSync(nodeModules, { recursive: true });
    fs.symlinkSync(vuePackage, path.join(nodeModules, "vue"), "dir");
  }

  fs.writeFileSync(
    path.join(workspaceDir, "tsconfig.json"),
    `${JSON.stringify(
      {
        compilerOptions: {
          lib: ["ES2022", "DOM", "DOM.Iterable"],
          module: "ESNext",
          moduleResolution: "bundler",
          noEmit: true,
          skipLibCheck: true,
          strict: true,
          target: "ES2022",
          types: [],
        },
        include: ["*.vue"],
      },
      null,
      2,
    )}\n`,
  );

  return workspaceDir;
}

async function publishedDiagnosticCodes(workspaceDir: string): Promise<number[]> {
  const session = new LspSession();
  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: true,
    });

    const filePath = path.join(workspaceDir, "Mismatch.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, SOURCE, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: SOURCE },
    });

    // The type backend publishes asynchronously, so accept the first publish
    // for this URI that carries any diagnostic, and fall back to the last
    // empty one if the server insists the file is clean.
    let last: PublishDiagnosticsParams | null = null;
    for (let attempt = 0; attempt < 6; attempt += 1) {
      const params = (await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (candidate) => isDiagnosticsForUri(candidate, uri),
      )) as PublishDiagnosticsParams;
      last = params;
      if (params.diagnostics.length > 0) {
        break;
      }
    }

    return (last?.diagnostics ?? []).map((diagnostic) => Number(diagnostic.code));
  } finally {
    await session.shutdown();
  }
}

test("vize lsp publishes typecheck diagnostics when node_modules is a real directory", async () => {
  const workspaceDir = createWorkspace("real", false);
  const codes = await publishedDiagnosticCodes(workspaceDir);

  assert.ok(
    codes.includes(2322),
    `expected TS2322 for the type mismatch, got codes ${JSON.stringify(codes)}`,
  );
});

test("vize lsp publishes typecheck diagnostics when node_modules is a symlink", async () => {
  const workspaceDir = createWorkspace("symlink", true);
  const codes = await publishedDiagnosticCodes(workspaceDir);

  assert.ok(
    codes.includes(2322),
    "a symlinked node_modules must not silently suppress typecheck diagnostics; " +
      `got codes ${JSON.stringify(codes)}`,
  );
});
