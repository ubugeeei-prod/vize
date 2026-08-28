import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

test("vize lsp revalidates open documents after watched declaration changes", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "TypeScript 7/Corsa runtime for the LSP watcher revalidation gate",
    "TypeScript 7/Corsa runtime not found; skipping LSP watcher revalidation test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-watcher-revalidation");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();
  let primaryError: unknown;

  try {
    const sourceDir = path.join(workspaceDir, "src");
    const declarationDir = path.join(workspaceDir, ".nuxt");
    fs.mkdirSync(sourceDir, { recursive: true });
    fs.mkdirSync(declarationDir, { recursive: true });
    const vuePackage = resolveVuePackage();
    if (vuePackage == null) {
      writeVueShim(workspaceDir);
    } else {
      linkVuePackage(workspaceDir, vuePackage);
    }
    fs.writeFileSync(
      path.join(workspaceDir, "vize.config.json"),
      JSON.stringify({ lsp: { lint: false, typecheck: true }, typeChecker: { corsaPath } }),
      "utf8",
    );
    fs.writeFileSync(
      path.join(workspaceDir, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          module: "ESNext",
          moduleResolution: "bundler",
          noEmit: true,
          strict: true,
          target: "ES2022",
        },
        include: ["src/**/*"],
      }),
      "utf8",
    );

    const declarationPath = path.join(declarationDir, "components.d.ts");
    const declarationUri = pathToFileURL(declarationPath).href;
    writeGlobalComponentDeclaration(declarationPath, "string");
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: true,
    });

    const source = `<script setup lang="ts">
/// <reference types="vue" />
const count = 1
</script>

<template>
  <NuxtCard :count="count" />
</template>
`;
    const filePath = path.join(sourceDir, "App.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, source, "utf8");
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });

    const initial = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, uri) &&
        params.version === 1 &&
        params.diagnostics.some((diagnostic) => diagnostic.message?.includes("string")),
    )) as PublishDiagnosticsParams;
    const bindingOffset = source.indexOf(":count");
    assert.notEqual(bindingOffset, -1);
    const bindingStart = offsetToPosition(source, bindingOffset + ":".length);
    assert.deepEqual(initial.diagnostics, [expectedCountDiagnostic(bindingStart, "string")]);

    writeGlobalComponentDeclaration(declarationPath, "boolean");
    session.notify("workspace/didChangeWatchedFiles", {
      changes: [{ uri: declarationUri, type: 2 }],
    });

    const revalidated = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, uri) &&
        params.version === 1 &&
        params.diagnostics.some((diagnostic) => diagnostic.message?.includes("boolean")),
    )) as PublishDiagnosticsParams;
    assert.equal(revalidated.version, 1, "the watcher must not require a document edit");
    assert.deepEqual(revalidated.diagnostics, [expectedCountDiagnostic(bindingStart, "boolean")]);
  } catch (error) {
    primaryError = error;
    throw error;
  } finally {
    await session.shutdown().catch((error: unknown) => {
      if (primaryError == null) throw error;
    });
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

function expectedCountDiagnostic(start: { line: number; character: number }, expectedType: string) {
  return {
    code: 2322,
    message: `Type 'number' is not assignable to type '${expectedType}'.`,
    range: {
      start,
      end: { line: start.line, character: start.character + "count".length },
    },
    severity: 1,
    source: "vize/types",
  };
}

function writeGlobalComponentDeclaration(filePath: string, countType: string): void {
  fs.writeFileSync(
    filePath,
    `declare module "vue" {
  export interface GlobalComponents {
    NuxtCard: import("vue").DefineComponent<{ count: ${countType} }>;
  }
}
export {};
`,
    "utf8",
  );
}

function resolveTsgoBinary(): string | undefined {
  return resolveTypecheckRuntime(root);
}

function resolveVuePackage(): string | undefined {
  const candidates = [
    path.join(root, "node_modules/vue"),
    path.join(root, "tests/node_modules/vue"),
    path.join(root, "playground/node_modules/vue"),
    path.join(root, "examples/jsx-tsx/node_modules/vue"),
  ];
  return candidates.find((candidate) => fs.existsSync(candidate));
}

function linkVuePackage(workspaceDir: string, vuePackage: string): void {
  const nodeModules = path.join(workspaceDir, "node_modules");
  fs.mkdirSync(nodeModules, { recursive: true });
  symlink(vuePackage, path.join(nodeModules, "vue"));

  const vueNamespace = path.join(path.dirname(vuePackage), "@vue");
  if (fs.existsSync(vueNamespace)) {
    symlink(vueNamespace, path.join(nodeModules, "@vue"));
  }
}

function symlink(source: string, target: string): void {
  fs.rmSync(target, { force: true, recursive: true });
  fs.symlinkSync(source, target, process.platform === "win32" ? "junction" : "dir");
}

function writeVueShim(workspaceDir: string): void {
  fs.writeFileSync(
    path.join(workspaceDir, "src/vue-shim.d.ts"),
    `declare module "vue" {
  export interface Ref<T = any> {
    value: T;
  }

  export interface ShallowRef<T = any> {
    value: T;
  }

  export type DefineComponent<Props = {}> = new () => { $props: Props };

  export interface ComponentPublicInstance {
    $attrs: Record<string, unknown>;
    $slots: Record<string, unknown>;
    $refs: Record<string, unknown>;
    $emit: (...args: any[]) => void;
  }

  export interface GlobalComponents {}
}
`,
    "utf8",
  );
}
