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

const source = `<script setup lang="tsx">
const label: string = "ready"
const vnode = <button class="primary">{label}</button>
const mismatch: string = 1
void vnode
void mismatch
</script>
`;

test("SFC TSX script typecheck reports script errors without intrinsic JSX noise", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "TypeScript 7/Corsa runtime for the SFC TSX typecheck gate",
    "TypeScript 7/Corsa runtime not found; skipping SFC TSX typecheck test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-sfc-tsx-script-typecheck");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    fs.mkdirSync(path.join(workspaceDir, "src"), { recursive: true });
    linkVuePackage(workspaceDir);
    fs.writeFileSync(
      path.join(workspaceDir, "vize.config.json"),
      JSON.stringify({ lsp: { lint: false, typecheck: true }, typeChecker: { corsaPath } }),
      "utf8",
    );
    fs.writeFileSync(
      path.join(workspaceDir, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          jsx: "preserve",
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

    const filePath = path.join(workspaceDir, "src/App.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, source, "utf8");
    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: true });
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
      120_000,
    )) as PublishDiagnosticsParams;
    const messages = publish.diagnostics.map((diagnostic) => diagnostic.message ?? "");
    assert.equal(
      messages.some((message) => message.includes("JSX.IntrinsicElements")),
      false,
      messages.join("\n"),
    );

    const mismatchOffset = source.indexOf("mismatch: string");
    assert.notEqual(mismatchOffset, -1);
    assert.deepEqual(publish.diagnostics, [
      {
        code: 2322,
        message: "Type 'number' is not assignable to type 'string'.",
        range: {
          start: offsetToPosition(source, mismatchOffset),
          end: offsetToPosition(source, mismatchOffset + "mismatch".length),
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

function linkVuePackage(workspaceDir: string): void {
  const vuePackage = [
    path.join(root, "node_modules/vue"),
    path.join(root, "tests/node_modules/vue"),
  ].find((candidate) => fs.existsSync(candidate));
  assert.ok(vuePackage, "Vue package is required for SFC TSX typecheck");
  const nodeModules = path.join(workspaceDir, "node_modules");
  fs.mkdirSync(nodeModules, { recursive: true });
  fs.symlinkSync(vuePackage, path.join(nodeModules, "vue"), dirLinkType());
  const vueNamespace = path.join(path.dirname(vuePackage), "@vue");
  if (fs.existsSync(vueNamespace)) {
    fs.symlinkSync(vueNamespace, path.join(nodeModules, "@vue"), dirLinkType());
  }
}

function dirLinkType(): fs.symlink.Type {
  return process.platform === "win32" ? "junction" : "dir";
}
