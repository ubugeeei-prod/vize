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

test("vize lsp publishes authored script setup diagnostics without template usage", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "TypeScript 7/Corsa runtime for authored script diagnostics",
    "TypeScript 7/Corsa runtime not found; skipping LSP typecheck test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-typecheck-script-setup");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    fs.mkdirSync(path.join(workspaceDir, "src"), { recursive: true });
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
    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: true });

    const source = `<script setup lang="ts">
const a: string = 1
</script>

<template>
  <div />
</template>
`;
    const filePath = path.join(workspaceDir, "src/App.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, source, "utf8");
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });

    const publish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, uri) &&
        params.diagnostics.some((diagnostic) =>
          diagnostic.message?.includes("Type 'number' is not assignable to type 'string'"),
        ),
    )) as PublishDiagnosticsParams;
    const assignmentOffset = source.indexOf("a: string");
    assert.notEqual(assignmentOffset, -1);
    const assignmentStart = offsetToPosition(source, assignmentOffset);
    assert.deepEqual(
      publish.diagnostics.filter((diagnostic) =>
        diagnostic.message?.includes("Type 'number' is not assignable to type 'string'"),
      ),
      [
        {
          code: 2322,
          message: "Type 'number' is not assignable to type 'string'.",
          range: {
            start: assignmentStart,
            end: { line: assignmentStart.line, character: assignmentStart.character + 1 },
          },
          severity: 1,
          source: "vize/types",
        },
      ],
    );

    const fixedSource = source.replace("const a: string = 1", "const a: string = 'ok'");
    session.notify("textDocument/didChange", {
      textDocument: { uri, version: 2 },
      contentChanges: [{ text: fixedSource }],
    });
    const repaired = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, uri) &&
        params.version === 2 &&
        !params.diagnostics.some((diagnostic) =>
          diagnostic.message?.includes("Type 'number' is not assignable to type 'string'"),
        ),
    )) as PublishDiagnosticsParams;
    assert.deepEqual(
      repaired.diagnostics.filter((diagnostic) =>
        diagnostic.message?.includes("Type 'number' is not assignable to type 'string'"),
      ),
      [],
    );
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
