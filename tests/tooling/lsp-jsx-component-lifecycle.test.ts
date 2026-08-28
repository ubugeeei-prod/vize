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

type WorkspaceEdit = {
  changes?: Record<string, Array<{ newText: string }>>;
  documentChanges?: Array<{
    textDocument?: { uri?: string };
    edits?: Array<{ newText: string }>;
  }>;
} | null;

test("vize lsp revalidates a TSX SFC import across create, edit, rename, and delete", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "TypeScript 7/Corsa runtime for the TSX SFC lifecycle gate",
    "TypeScript 7/Corsa runtime not found; skipping TSX SFC lifecycle test",
  );
  if (corsaPath == null) return;

  const outputDir = path.join(testOutputRoot, "lsp-jsx-component-lifecycle");
  fs.mkdirSync(outputDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(outputDir, "workspace-"));
  const sourceDir = path.join(workspaceDir, "src");
  fs.mkdirSync(sourceDir, { recursive: true });
  const session = new LspSession();

  try {
    writeProjectConfig(workspaceDir, corsaPath);
    const consumer = `import Counter from "./Counter.vue";
export const view = <Counter count={1} />;
`;
    const renamedConsumer = consumer.replace("./Counter.vue", "./Renamed.vue");
    const stringCounter = `<script setup lang="ts">
defineProps<{ count: string }>()
</script>
`;
    const numberCounter = stringCounter.replace("count: string", "count: number");
    const consumerPath = path.join(sourceDir, "Consumer.tsx");
    const counterPath = path.join(sourceDir, "Counter.vue");
    const renamedPath = path.join(sourceDir, "Renamed.vue");
    const consumerUri = pathToFileURL(consumerPath).href;
    const counterUri = pathToFileURL(counterPath).href;
    const renamedUri = pathToFileURL(renamedPath).href;
    fs.writeFileSync(consumerPath, consumer, "utf8");

    await session.initialize(workspaceDir, {
      editor: true,
      fileRename: true,
      lint: false,
      typecheck: true,
    });
    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: consumerUri,
        languageId: "typescriptreact",
        version: 1,
        text: consumer,
      },
    });
    await waitForCode(session, consumerUri, 1, 2307);

    fs.writeFileSync(counterPath, stringCounter, "utf8");
    session.notify("workspace/didCreateFiles", { files: [{ uri: counterUri }] });
    const created = await waitForCode(session, consumerUri, 1, 2322);
    assert.match(created.diagnostics[0]?.message ?? "", /number.*string|not assignable/);

    session.notify("textDocument/didOpen", {
      textDocument: { uri: counterUri, languageId: "vue", version: 1, text: stringCounter },
    });
    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, counterUri),
    );
    fs.writeFileSync(counterPath, numberCounter, "utf8");
    session.notify("textDocument/didChange", {
      textDocument: { uri: counterUri, version: 2 },
      contentChanges: [{ text: numberCounter }],
    });
    await waitForClean(session, consumerUri, 1);
    session.notify("textDocument/didClose", { textDocument: { uri: counterUri } });

    const renameEdit = (await session.request("workspace/willRenameFiles", {
      files: [{ oldUri: counterUri, newUri: renamedUri }],
    })) as WorkspaceEdit;
    assert.deepEqual(editTextsForUri(renameEdit, consumerUri), ["./Renamed.vue"]);

    fs.renameSync(counterPath, renamedPath);
    session.notify("workspace/didRenameFiles", {
      files: [{ oldUri: counterUri, newUri: renamedUri }],
    });
    await waitForCode(session, consumerUri, 1, 2307);

    fs.writeFileSync(consumerPath, renamedConsumer, "utf8");
    session.notify("textDocument/didChange", {
      textDocument: { uri: consumerUri, version: 2 },
      contentChanges: [{ text: renamedConsumer }],
    });
    await waitForClean(session, consumerUri, 2);

    fs.rmSync(renamedPath);
    session.notify("workspace/didDeleteFiles", { files: [{ uri: renamedUri }] });
    await waitForCode(session, consumerUri, 2, 2307);

    fs.writeFileSync(renamedPath, numberCounter, "utf8");
    session.notify("workspace/didCreateFiles", { files: [{ uri: renamedUri }] });
    await waitForClean(session, consumerUri, 2);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});

async function waitForCode(
  session: LspSession,
  uri: string,
  version: number,
  code: number,
): Promise<PublishDiagnosticsParams> {
  return (await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) =>
      isDiagnosticsForUri(params, uri) &&
      params.version === version &&
      params.diagnostics.some((diagnostic) => diagnostic.code === code),
    10_000,
  )) as PublishDiagnosticsParams;
}

async function waitForClean(
  session: LspSession,
  uri: string,
  version: number,
): Promise<PublishDiagnosticsParams> {
  return (await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) =>
      isDiagnosticsForUri(params, uri) &&
      params.version === version &&
      params.diagnostics.length === 0,
    10_000,
  )) as PublishDiagnosticsParams;
}

function editTextsForUri(edit: WorkspaceEdit, uri: string): string[] {
  const texts = (edit?.changes?.[uri] ?? []).map((textEdit) => textEdit.newText);
  for (const change of edit?.documentChanges ?? []) {
    if (change.textDocument?.uri === uri) {
      texts.push(...(change.edits ?? []).map((textEdit) => textEdit.newText));
    }
  }
  return texts;
}

function resolveTsgoBinary(): string | undefined {
  return resolveTypecheckRuntime(root);
}

function writeProjectConfig(workspaceDir: string, corsaPath: string): void {
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
