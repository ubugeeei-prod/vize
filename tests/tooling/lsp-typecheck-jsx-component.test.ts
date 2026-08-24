import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import { requireTypecheckDependency } from "./support/typecheck-dependency.ts";

test("vize lsp enforces and repairs imported SFC props in TSX and checkJs JSX", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "tsgo binary for the TSX component LSP gate",
    "tsgo binary not found; skipping TSX component LSP test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-typecheck-jsx-component");
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
      JSON.stringify({
        lsp: { lint: false, typecheck: true },
        typeChecker: { corsaPath, jsxTypecheck: true },
      }),
      "utf8",
    );
    fs.writeFileSync(
      path.join(workspaceDir, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          allowJs: true,
          checkJs: true,
          jsx: "preserve",
          jsxImportSource: "vue",
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

    const counter = `<script setup lang="ts">
defineProps<{ count: number }>()
</script>
`;
    const broken = `import Counter from "./Counter.vue";
export const view = <Counter count="wrong" />;
`;
    const repaired = broken.replace('count="wrong"', "count={1}");
    const spreadBroken = `import Counter from "./Counter.vue";
const props = { count: "wrong" };
export const view = <Counter {...props} />;
`;
    const spreadRepaired = spreadBroken.replace('count: "wrong"', "count: 1");
    const counterPath = path.join(sourceDir, "Counter.vue");
    const consumerPath = path.join(sourceDir, "Consumer.tsx");
    const consumerUri = pathToFileURL(consumerPath).href;
    fs.writeFileSync(counterPath, counter, "utf8");
    fs.writeFileSync(consumerPath, broken, "utf8");

    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: true,
    });
    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: consumerUri,
        languageId: "typescriptreact",
        version: 1,
        text: broken,
      },
    });

    const invalid = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, consumerUri),
    )) as PublishDiagnosticsParams;
    const propOffset = broken.indexOf("count=");
    assert.notEqual(propOffset, -1);
    const propStart = offsetToPosition(broken, propOffset);
    assert.deepEqual(invalid.diagnostics, [
      {
        code: 2322,
        message: "Type 'string' is not assignable to type 'number'.",
        range: {
          start: propStart,
          end: { line: propStart.line, character: propStart.character + "count".length },
        },
        severity: 1,
        source: "vize/types",
      },
    ]);

    session.notify("textDocument/didChange", {
      textDocument: { uri: consumerUri, version: 2 },
      contentChanges: [{ text: repaired }],
    });
    const clean = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, consumerUri) &&
        params.version === 2 &&
        params.diagnostics.length === 0,
    )) as PublishDiagnosticsParams;
    assert.deepEqual(clean.diagnostics, []);

    const intrinsic = `export const native = <main><h1>Dashboard</h1><button disabled>Save</button></main>;
`;
    const intrinsicPath = path.join(sourceDir, "Intrinsic.tsx");
    const intrinsicUri = pathToFileURL(intrinsicPath).href;
    fs.writeFileSync(intrinsicPath, intrinsic, "utf8");
    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: intrinsicUri,
        languageId: "typescriptreact",
        version: 1,
        text: intrinsic,
      },
    });
    const intrinsicPublish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, intrinsicUri) &&
        params.version === 1 &&
        params.diagnostics.length === 0,
    )) as PublishDiagnosticsParams;
    assert.deepEqual(
      intrinsicPublish.diagnostics,
      [],
      "standalone TSX intrinsic elements stay diagnostic-free in LSP",
    );

    session.notify("textDocument/didChange", {
      textDocument: { uri: consumerUri, version: 3 },
      contentChanges: [{ text: spreadBroken }],
    });
    const spreadInvalid = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, consumerUri) &&
        params.version === 3 &&
        params.diagnostics.some((diagnostic) => diagnostic.message?.includes("not assignable")),
    )) as PublishDiagnosticsParams;
    assert.equal(spreadInvalid.diagnostics.length, 1, JSON.stringify(spreadInvalid));
    assert.match(
      spreadInvalid.diagnostics[0]?.message ?? "",
      /count.*string.*number|not assignable/,
    );
    assert.equal(spreadInvalid.diagnostics[0]?.severity, 1);
    assert.equal(spreadInvalid.diagnostics[0]?.source, "vize/types");

    session.notify("textDocument/didChange", {
      textDocument: { uri: consumerUri, version: 4 },
      contentChanges: [{ text: spreadRepaired }],
    });
    const spreadClean = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, consumerUri) &&
        params.version === 4 &&
        params.diagnostics.length === 0,
    )) as PublishDiagnosticsParams;
    assert.deepEqual(spreadClean.diagnostics, []);

    const jsxBroken = `import Counter from "./Counter.vue";
export const view = <Counter count="wrong" />;
`;
    const jsxRepaired = jsxBroken.replace('count="wrong"', "count={1}");
    const jsxPath = path.join(sourceDir, "Consumer.jsx");
    const jsxUri = pathToFileURL(jsxPath).href;
    fs.writeFileSync(jsxPath, jsxBroken, "utf8");
    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: jsxUri,
        languageId: "javascriptreact",
        version: 1,
        text: jsxBroken,
      },
    });
    const invalidJsx = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, jsxUri),
    )) as PublishDiagnosticsParams;
    assertNoIntrinsicElementDiagnostic(invalidJsx);
    const jsxPropOffset = jsxBroken.indexOf("count=");
    assert.notEqual(jsxPropOffset, -1);
    const jsxPropStart = offsetToPosition(jsxBroken, jsxPropOffset);
    assert.deepEqual(invalidJsx.diagnostics, [
      {
        code: 2322,
        message: "Type 'string' is not assignable to type 'number'.",
        range: {
          start: jsxPropStart,
          end: {
            line: jsxPropStart.line,
            character: jsxPropStart.character + "count".length,
          },
        },
        severity: 1,
        source: "vize/types",
      },
    ]);

    session.notify("textDocument/didChange", {
      textDocument: { uri: jsxUri, version: 2 },
      contentChanges: [{ text: jsxRepaired }],
    });
    const cleanJsx = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, jsxUri) &&
        params.version === 2 &&
        params.diagnostics.length === 0,
    )) as PublishDiagnosticsParams;
    assert.deepEqual(cleanJsx.diagnostics, []);
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

function assertNoIntrinsicElementDiagnostic(publish: PublishDiagnosticsParams): void {
  const messages = publish.diagnostics.map((diagnostic) => diagnostic.message ?? "");
  assert.ok(
    messages.every((message) => !message.includes("JSX.IntrinsicElements")),
    messages.join("\n"),
  );
  assert.ok(
    messages.every((message) => !message.includes("[TS7026]")),
    messages.join("\n"),
  );
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
  export interface Ref<T = any> { value: T }
  export interface ShallowRef<T = any> { value: T }
  export type DefineComponent<Props = {}> = new () => { $props: Props }
  export interface ComponentPublicInstance {
    $attrs: Record<string, unknown>
    $slots: Record<string, unknown>
    $refs: Record<string, unknown>
    $emit: (...args: any[]) => void
  }
  export interface GlobalComponents {}
}
`,
    "utf8",
  );
}
