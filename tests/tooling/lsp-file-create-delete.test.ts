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

test("vize lsp refreshes global components after declaration create and delete events", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "TypeScript 7/Corsa runtime for the LSP declaration event gate",
    "TypeScript 7/Corsa runtime not found; skipping LSP declaration event test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-file-create-delete");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    const sourceDir = path.join(workspaceDir, "src");
    fs.mkdirSync(sourceDir, { recursive: true });
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
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: true,
    });

    // The authored reference directive keeps unresolved tags delegated to Vue's
    // ambient GlobalComponents type instead of the permissive `any` fallback.
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
      (params) => isDiagnosticsForUri(params, uri) && params.version === 1,
    )) as PublishDiagnosticsParams;
    assert.equal(initial.version, 1);
    assert.deepEqual(initial.diagnostics, []);

    const declarationDir = path.join(workspaceDir, ".nuxt");
    const declarationPath = path.join(declarationDir, "components.d.ts");
    const declarationUri = pathToFileURL(declarationPath).href;
    fs.mkdirSync(declarationDir, { recursive: true });
    fs.writeFileSync(
      declarationPath,
      `declare module "vue" {
  export interface GlobalComponents {
    NuxtCard: import("vue").DefineComponent<{ count: string }>;
  }
}
export {};
`,
      "utf8",
    );
    session.notify("workspace/didCreateFiles", { files: [{ uri: declarationUri }] });
    session.notify("textDocument/didChange", {
      textDocument: { uri, version: 2 },
      contentChanges: [{ text: source }],
    });

    const created = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, uri) &&
        params.version === 2 &&
        params.diagnostics.some((diagnostic) => diagnostic.message?.includes("not assignable")),
    )) as PublishDiagnosticsParams;
    assert.equal(created.version, 2);
    const bindingOffset = source.indexOf(":count");
    assert.notEqual(bindingOffset, -1);
    const bindingStart = offsetToPosition(source, bindingOffset + ":".length);
    assert.deepEqual(created.diagnostics, [
      {
        code: 2322,
        message: "Type 'number' is not assignable to type 'string'.",
        range: {
          start: bindingStart,
          end: { line: bindingStart.line, character: bindingStart.character + "count".length },
        },
        severity: 1,
        source: "vize/types",
      },
    ]);

    // Deliberately omit a create notification for this second declaration. A
    // handled delete must invalidate the old path set and discover this file;
    // retaining the deleted path would instead erase the component diagnostic.
    const replacementPath = path.join(declarationDir, "replacement.d.ts");
    fs.writeFileSync(
      replacementPath,
      `declare module "vue" {
  export interface GlobalComponents {
    NuxtCard: import("vue").DefineComponent<{ count: boolean }>;
  }
}
export {};
`,
      "utf8",
    );
    fs.rmSync(declarationPath);
    session.notify("workspace/didDeleteFiles", { files: [{ uri: declarationUri }] });
    session.notify("textDocument/didChange", {
      textDocument: { uri, version: 3 },
      contentChanges: [{ text: source }],
    });

    const deleted = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, uri) &&
        params.version === 3 &&
        params.diagnostics.some((diagnostic) => diagnostic.message?.includes("not assignable")),
    )) as PublishDiagnosticsParams;
    assert.equal(deleted.version, 3);
    assert.deepEqual(deleted.diagnostics, [
      {
        code: 2322,
        message: "Type 'number' is not assignable to type 'boolean'.",
        range: {
          start: bindingStart,
          end: { line: bindingStart.line, character: bindingStart.character + "count".length },
        },
        severity: 1,
        source: "vize/types",
      },
    ]);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

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
