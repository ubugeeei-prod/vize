import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import type { LspDiagnostic, PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

test("vize lsp anchors fallthrough attribute diagnostics on the authored template root", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-fallthrough-template-range");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    const sourceDir = path.join(workspaceDir, "src");
    fs.mkdirSync(sourceDir, { recursive: true });
    fs.writeFileSync(
      path.join(workspaceDir, "vize.config.json"),
      JSON.stringify({ lsp: { lint: false, typecheck: true } }),
      "utf8",
    );
    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: true });

    const multiRoot = `<script setup lang="ts">
const marker = 1
</script>

<template>
  <header :class="$attrs.class">top</header>
  <main>body</main>
</template>
`;
    const multiRootPath = path.join(sourceDir, "MultiRoot.vue");
    const multiRootUri = pathToFileURL(multiRootPath).href;
    fs.writeFileSync(multiRootPath, multiRoot, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: { uri: multiRootUri, languageId: "vue", version: 1, text: multiRoot },
    });
    const fallthroughPublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, multiRootUri) &&
        fallthroughDiagnostic(params as PublishDiagnosticsParams) != null,
    )) as PublishDiagnosticsParams;
    const fallthrough = fallthroughDiagnostic(fallthroughPublish);
    assert.deepEqual(fallthrough?.range?.start, { line: 5, character: 2 });
    assert.ok(
      fallthrough?.message?.includes("Multi-root component may lose fallthrough attributes"),
      fallthroughPublish.diagnostics.map((diagnostic) => diagnostic.message),
    );

    const plainFragment = multiRoot.replace(' :class="$attrs.class"', "");
    const plainFragmentPath = path.join(sourceDir, "PlainFragment.vue");
    const plainFragmentUri = pathToFileURL(plainFragmentPath).href;
    fs.writeFileSync(plainFragmentPath, plainFragment, "utf8");
    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: plainFragmentUri,
        languageId: "vue",
        version: 1,
        text: plainFragment,
      },
    });
    const plainFragmentPublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, plainFragmentUri) &&
        fallthroughDiagnostic(params as PublishDiagnosticsParams) == null,
      10_000,
    )) as PublishDiagnosticsParams;
    assert.deepEqual(plainFragmentPublish.diagnostics, []);

    const singleRoot = `<script setup lang="ts">
const marker = 1
</script>

<template>
  <main :class="$attrs.class">body</main>
</template>
`;
    const singleRootPath = path.join(sourceDir, "SingleRoot.vue");
    const singleRootUri = pathToFileURL(singleRootPath).href;
    fs.writeFileSync(singleRootPath, singleRoot, "utf8");
    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: singleRootUri,
        languageId: "vue",
        version: 1,
        text: singleRoot,
      },
    });
    const singleRootPublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, singleRootUri) &&
        fallthroughDiagnostic(params as PublishDiagnosticsParams) == null,
      10_000,
    )) as PublishDiagnosticsParams;
    assert.deepEqual(singleRootPublish.diagnostics, []);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

test("vize lsp publishes and clears fallthrough attribute diagnostics", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "TypeScript 7/Corsa runtime for the fallthrough-attribute LSP gate",
    "TypeScript 7/Corsa runtime not found; skipping LSP typecheck test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-typecheck-fallthrough-attrs");
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
    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: true });

    const child = `<script lang="ts">
export default {
  inheritAttrs: false,
  props: {
    title: String,
  },
}
</script>

<template>
  <div>{{ title }}</div>
</template>
`;
    const invalidParent = `<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child title="ok" depressed="x" />
</template>
`;
    const repairedParent = invalidParent.replace(' depressed="x"', "");
    const childPath = path.join(sourceDir, "Child.vue");
    const parentPath = path.join(sourceDir, "Parent.vue");
    const childUri = pathToFileURL(childPath).href;
    const parentUri = pathToFileURL(parentPath).href;
    fs.writeFileSync(childPath, child, "utf8");
    fs.writeFileSync(parentPath, invalidParent, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: { uri: childUri, languageId: "vue", version: 1, text: child },
    });
    session.notify("textDocument/didOpen", {
      textDocument: { uri: parentUri, languageId: "vue", version: 1, text: invalidParent },
    });
    const initialParent = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, parentUri) &&
        hasUnknownAttrDiagnostic(params as PublishDiagnosticsParams),
    )) as PublishDiagnosticsParams;
    assert.ok(
      hasUnknownAttrDiagnostic(initialParent),
      initialParent.diagnostics.map((diagnostic) => diagnostic.message),
    );

    session.notify("textDocument/didChange", {
      textDocument: { uri: parentUri, version: 2 },
      contentChanges: [{ text: repairedParent }],
    });
    const repairedPublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, parentUri) &&
        params.version === 2 &&
        !hasUnknownAttrDiagnostic(params as PublishDiagnosticsParams),
    )) as PublishDiagnosticsParams;
    assert.deepEqual(repairedPublish.diagnostics, []);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

function fallthroughDiagnostic(params: PublishDiagnosticsParams): LspDiagnostic | undefined {
  return params.diagnostics.find((diagnostic) => diagnostic.code === "fallthrough-attrs");
}

function hasUnknownAttrDiagnostic(params: PublishDiagnosticsParams): boolean {
  return params.diagnostics.some(
    (diagnostic) =>
      (diagnostic.code === 2322 || diagnostic.code === 2353) &&
      diagnostic.message?.includes("depressed"),
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
