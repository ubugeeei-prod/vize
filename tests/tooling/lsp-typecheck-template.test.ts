import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { hoverToText, isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import { requireTypecheckDependency } from "./support/typecheck-dependency.ts";

test("vize lsp typecheck keeps DOM libs and template diagnostics", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "tsgo binary for the LSP typecheck gate",
    "tsgo binary not found; skipping LSP typecheck test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-typecheck-template");
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
      JSON.stringify(
        {
          lsp: {
            hover: true,
            lint: false,
            typecheck: true,
          },
          typeChecker: {
            corsaPath,
          },
        },
        null,
        2,
      ),
      "utf8",
    );
    fs.writeFileSync(
      path.join(workspaceDir, "tsconfig.json"),
      JSON.stringify(
        {
          compilerOptions: {
            module: "ESNext",
            moduleResolution: "bundler",
            noEmit: true,
            strict: true,
            target: "ES2022",
          },
          include: ["src/**/*"],
        },
        null,
        2,
      ),
      "utf8",
    );
    await session.initialize(workspaceDir, {
      editor: true,
      hover: true,
      lint: false,
      typecheck: true,
    });

    const source = `<script setup lang="ts">
const count: number = "oops"
const button = document.createElement("button")
</script>

<template>
  <button @click="button.focus()">{{ count.toFixed(1) }}</button>
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
          diagnostic.message?.includes("Type 'string' is not assignable to type 'number'"),
        ),
    )) as PublishDiagnosticsParams;
    const messages = publish.diagnostics.map((diagnostic) => diagnostic.message ?? "");

    assert.ok(
      messages.some((message) =>
        message.includes("Type 'string' is not assignable to type 'number'"),
      ),
      messages.join("\n"),
    );
    assert.equal(
      messages.some(
        (message) =>
          message.includes("Cannot find name 'document'") ||
          message.includes("Cannot find name 'HTMLElement'"),
      ),
      false,
      messages.join("\n"),
    );

    const hover = (await session.request("textDocument/hover", {
      textDocument: { uri },
      position: offsetToPosition(source, source.lastIndexOf("count.toFixed") + "count".length),
    })) as { contents?: unknown } | null;
    assert.match(hoverToText(hover), /count/);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

test("vize lsp publishes authored script setup diagnostics without template usage", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "tsgo binary for authored script diagnostics",
    "tsgo binary not found; skipping LSP typecheck test",
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
      JSON.stringify(
        {
          lsp: {
            lint: false,
            typecheck: true,
          },
          typeChecker: {
            corsaPath,
          },
        },
        null,
        2,
      ),
      "utf8",
    );
    fs.writeFileSync(
      path.join(workspaceDir, "tsconfig.json"),
      JSON.stringify(
        {
          compilerOptions: {
            module: "ESNext",
            moduleResolution: "bundler",
            noEmit: true,
            strict: true,
            target: "ES2022",
          },
          include: ["src/**/*"],
        },
        null,
        2,
      ),
      "utf8",
    );
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: true,
    });

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
            end: { line: assignmentStart.line, character: assignmentStart.character + "a".length },
          },
          severity: 1,
          source: "vize/types",
        },
      ],
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

test("vize lsp publishes and clears exact parent diagnostics after child prop edits", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "tsgo binary for the dependent-component LSP gate",
    "tsgo binary not found; skipping LSP typecheck test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-typecheck-dependent-component");
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

    const initialChild = `<script setup lang="ts">
defineProps<{ count: string; dataLabel?: string }>()
</script>
`;
    const changedChild = initialChild.replace("count: string", "count: number");
    const parent = `<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child data-label="😀" :count="'one'" />
</template>
`;
    const childPath = path.join(sourceDir, "Child.vue");
    const parentPath = path.join(sourceDir, "Parent.vue");
    const childUri = pathToFileURL(childPath).href;
    const parentUri = pathToFileURL(parentPath).href;
    fs.writeFileSync(childPath, initialChild, "utf8");
    fs.writeFileSync(parentPath, parent, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: { uri: childUri, languageId: "vue", version: 1, text: initialChild },
    });
    const initialChildPublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, childUri),
    )) as PublishDiagnosticsParams;
    assert.equal(initialChildPublish.version, 1);
    assert.deepEqual(initialChildPublish.diagnostics, []);
    session.notify("textDocument/didOpen", {
      textDocument: { uri: parentUri, languageId: "vue", version: 1, text: parent },
    });
    const initialParent = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, parentUri),
    )) as PublishDiagnosticsParams;
    assert.equal(initialParent.version, 1);
    assert.deepEqual(initialParent.diagnostics, []);

    session.notify("textDocument/didChange", {
      textDocument: { uri: childUri, version: 2 },
      contentChanges: [{ text: changedChild }],
    });
    const changedParent = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, parentUri) &&
        params.diagnostics.some((diagnostic) => diagnostic.message?.includes("not assignable")),
    )) as PublishDiagnosticsParams;
    const bindingOffset = parent.indexOf(":count");
    assert.notEqual(bindingOffset, -1);
    const bindingStart = offsetToPosition(parent, bindingOffset + ":".length);
    assert.equal(changedParent.version, 1);
    assert.deepEqual(changedParent.diagnostics, [
      {
        code: 2322,
        message: "Type 'string' is not assignable to type 'number'.",
        range: {
          start: bindingStart,
          end: { line: bindingStart.line, character: bindingStart.character + "count".length },
        },
        severity: 1,
        source: "vize/types",
      },
    ]);
    const changedChildPublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, childUri) && params.version === 2,
    )) as PublishDiagnosticsParams;
    assert.deepEqual(changedChildPublish.diagnostics, []);

    session.notify("textDocument/didChange", {
      textDocument: { uri: childUri, version: 3 },
      contentChanges: [{ text: initialChild }],
    });
    const repairedParent = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, parentUri) &&
        !params.diagnostics.some((diagnostic) => diagnostic.message?.includes("not assignable")),
    )) as PublishDiagnosticsParams;
    assert.equal(repairedParent.version, 1);
    assert.deepEqual(repairedParent.diagnostics, []);
    const repairedChildPublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, childUri) && params.version === 3,
    )) as PublishDiagnosticsParams;
    assert.deepEqual(repairedChildPublish.diagnostics, []);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

function resolveTsgoBinary(): string | undefined {
  const candidates = [
    process.env.CORSA_BIN,
    path.join(root, "../corsa-bind/.cache/tsgo"),
    path.join(root, "node_modules/.bin/tsgo"),
    path.join(root, "tests/node_modules/.bin/tsgo"),
  ].filter((candidate): candidate is string => candidate != null && candidate.length > 0);
  return candidates.find((candidate) => fs.existsSync(candidate));
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
