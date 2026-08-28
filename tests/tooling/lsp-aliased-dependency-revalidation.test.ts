import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

const cleanDependency = `export const value = 1
`;
const brokenDependency = `export const value = 'one'
`;
const appSource = `<script setup lang="ts">
import { value } from '@store'

const count: number = value
</script>

<template>
  <div>{{ count }}</div>
</template>
`;

// A `paths` alias routes the component through the session-private Canon
// mirror, so the checker reads the dependency from a generated copy instead of
// the authored file. The editor process that answers standard tsgo diagnostics
// is reused across requests and keeps the mirror it already parsed, so a
// dependency edit only reaches it when the mirror refresh retires that session
// (#3955).
test("vize lsp refreshes dependent diagnostics after an aliased dependency edit", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "TypeScript 7/Corsa runtime for the aliased dependency revalidation gate",
    "TypeScript 7/Corsa runtime not found; skipping aliased dependency revalidation test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-aliased-dependency-revalidation");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();
  let primaryError: unknown;

  try {
    const sourceDir = path.join(workspaceDir, "src");
    fs.mkdirSync(sourceDir, { recursive: true });
    linkVuePackages(workspaceDir);
    writeWorkspaceConfig(workspaceDir, corsaPath);

    const dependencyPath = path.join(sourceDir, "store.ts");
    const appPath = path.join(sourceDir, "App.vue");
    const dependencyUri = pathToFileURL(dependencyPath).href;
    const appUri = pathToFileURL(appPath).href;
    fs.writeFileSync(dependencyPath, cleanDependency, "utf8");
    fs.writeFileSync(appPath, appSource, "utf8");

    await session.initialize(workspaceDir, {
      editor: true,
      hover: true,
      lint: false,
      typecheck: true,
    });
    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: dependencyUri,
        languageId: "typescript",
        version: 1,
        text: cleanDependency,
      },
    });
    session.notify("textDocument/didOpen", {
      textDocument: { uri: appUri, languageId: "vue", version: 1, text: appSource },
    });

    const clean = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, appUri) &&
        params.version === 1 &&
        params.diagnostics.length === 0,
    )) as PublishDiagnosticsParams;
    assert.deepEqual(clean.diagnostics, []);

    fs.writeFileSync(dependencyPath, brokenDependency, "utf8");
    session.notify("textDocument/didChange", {
      textDocument: { uri: dependencyUri, version: 2 },
      contentChanges: [{ text: brokenDependency }],
    });

    const revalidated = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, appUri) &&
        params.version === 1 &&
        params.diagnostics.length > 0,
    )) as PublishDiagnosticsParams;
    assert.equal(revalidated.version, 1, "the dependent must not require an edit of its own");
    assert.deepEqual(revalidated.diagnostics, [
      {
        code: 2322,
        message: "Type 'string' is not assignable to type 'number'.",
        range: {
          start: { line: 3, character: 6 },
          end: { line: 3, character: 11 },
        },
        severity: 1,
        source: "vize/types",
      },
    ]);
  } catch (error) {
    primaryError = error;
    throw error;
  } finally {
    await session.shutdown().catch((error: unknown) => {
      if (primaryError == null) throw error;
    });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

function writeWorkspaceConfig(workspaceDir: string, corsaPath: string): void {
  fs.writeFileSync(
    path.join(workspaceDir, "vize.config.json"),
    JSON.stringify({
      lsp: { editor: true, hover: true, lint: false, typecheck: true },
      typeChecker: { corsaPath },
    }),
    "utf8",
  );
  fs.writeFileSync(
    path.join(workspaceDir, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        module: "ESNext",
        moduleResolution: "bundler",
        noEmit: true,
        paths: { "@store": ["./src/store.ts"] },
        skipLibCheck: true,
        strict: true,
        target: "ES2022",
      },
      include: ["src/**/*"],
    }),
    "utf8",
  );
}

function resolveTsgoBinary(): string | undefined {
  return resolveTypecheckRuntime(root);
}

function linkVuePackages(workspaceDir: string): void {
  const nodeModules = path.join(workspaceDir, "node_modules");
  fs.mkdirSync(nodeModules, { recursive: true });
  for (const name of ["vue", "@vue"]) {
    const source = path.join(root, "node_modules", name);
    if (!fs.existsSync(source)) continue;
    const target = path.join(nodeModules, name);
    fs.rmSync(target, { force: true, recursive: true });
    fs.symlinkSync(source, target, process.platform === "win32" ? "junction" : "dir");
  }
}
