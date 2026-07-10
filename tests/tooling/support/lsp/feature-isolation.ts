import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { firstLocation, hoverToText, isDiagnosticsForUri, offsetToPosition } from "./assertions.ts";
import { testOutputRoot } from "./paths.ts";
import type { LspInitializationOptions, PublishDiagnosticsParams } from "./protocol.ts";
import { LspSession } from "./session.ts";

export type FeatureContext = {
  publish: PublishDiagnosticsParams;
  session: LspSession;
  source: string;
  uri: string;
  workspaceDir: string;
};

type DocumentSymbol = {
  name: string;
};

type SymbolInformation = {
  name: string;
  location: { uri: string };
};

type CodeLens = {
  command?: { command?: string; title?: string };
};

type DocumentLink = {
  target?: string;
};

type InlayHint = {
  label: string | Array<{ value: string }>;
};

export const FULL_RANGE = {
  start: { line: 0, character: 0 },
  end: { line: 1000, character: 0 },
};

export async function withFeatureDocument(
  label: string,
  options: LspInitializationOptions,
  run: (ctx: FeatureContext) => Promise<void>,
): Promise<void> {
  const testRootDir = path.join(testOutputRoot, `lsp-editor-feature-isolation-${label}`);
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
      ...options,
    });

    fs.writeFileSync(
      path.join(workspaceDir, "Dep.vue"),
      `<script setup lang="ts">
defineProps<{ label: string }>()
</script>
<template><button /></template>
`,
      "utf8",
    );
    fs.writeFileSync(
      path.join(workspaceDir, "useServer.mjs"),
      "export const useServer = () => 1\n",
    );

    const source = `<script setup lang="ts">
import Dep from './Dep.vue'
import { useServer } from './useServer'
import { computed, ref } from 'vue'

const message = ref('hello')
const doubled = computed(() => message.value.length * 2)
const items = [1, 2]

function submitMessage() {
  return useServer() + message.value.length
}
</script>

<template>
  <Dep :label="message" @click="submitMessage" />
  <button :class="$style.primary" @click="submitMessage">{{ message }}</button>
  <ul>
    <li v-for="item in items">{{ item }}</li>
  </ul>
</template>

<style module>
.primary {}
</style>
`;
    const filePath = path.join(workspaceDir, "FeatureIsolation.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, source, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });

    const publish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri),
    )) as PublishDiagnosticsParams;

    await run({ publish, session, source, uri, workspaceDir });
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
}

export function templateMessagePosition(source: string): { line: number; character: number } {
  return offsetToPosition(source, source.lastIndexOf("message }}</button>") + "message".length);
}

export function completionStylePosition(source: string): { line: number; character: number } {
  return offsetToPosition(source, source.indexOf("$style.pr") + "$style.pr".length);
}

function declarationPosition(source: string): { line: number; character: number } {
  return offsetToPosition(source, source.indexOf("message = ref"));
}

function inlayLabel(hint: InlayHint): string {
  return typeof hint.label === "string"
    ? hint.label
    : hint.label.map((part) => part.value).join("");
}

export async function assertHoverWorks(ctx: FeatureContext): Promise<void> {
  const hover = (await ctx.session.request("textDocument/hover", {
    textDocument: { uri: ctx.uri },
    position: templateMessagePosition(ctx.source),
  })) as { contents?: unknown } | null;
  const text = hoverToText(hover);
  assert.match(text, /message/);
  assert.match(text, /Ref<string>|Template binding from script/);
}

export async function assertDocumentSymbolsWork(ctx: FeatureContext): Promise<void> {
  const symbols = (await ctx.session.request("textDocument/documentSymbol", {
    textDocument: { uri: ctx.uri },
  })) as DocumentSymbol[] | null;
  assert.ok(Array.isArray(symbols), JSON.stringify(symbols));
  const names = symbols.map((symbol) => symbol.name);
  assert.ok(names.includes("template"), names.join(", "));
  assert.ok(names.includes("script setup"), names.join(", "));
  assert.ok(
    names.some((name) => name.startsWith("style")),
    names.join(", "),
  );
}

export async function assertWorkspaceSymbolsWork(ctx: FeatureContext): Promise<void> {
  const symbols = (await ctx.session.request("workspace/symbol", {
    query: "submitMessage",
  })) as SymbolInformation[] | null;
  assert.ok(Array.isArray(symbols), JSON.stringify(symbols));
  assert.ok(
    symbols.some((symbol) => symbol.name === "submitMessage" && symbol.location.uri === ctx.uri),
  );
}

export async function assertSemanticTokensWork(ctx: FeatureContext): Promise<void> {
  const tokens = (await ctx.session.request("textDocument/semanticTokens/full", {
    textDocument: { uri: ctx.uri },
  })) as { data?: number[] } | null;
  assert.ok(Array.isArray(tokens?.data), JSON.stringify(tokens));
  assert.ok(tokens.data.length > 0, JSON.stringify(tokens));
}

export async function assertFoldingRangesWork(ctx: FeatureContext): Promise<void> {
  const ranges = (await ctx.session.request("textDocument/foldingRange", {
    textDocument: { uri: ctx.uri },
  })) as unknown[] | null;
  assert.ok(Array.isArray(ranges), JSON.stringify(ranges));
  assert.ok(ranges.length > 0, JSON.stringify(ranges));
}

export async function assertInlayHintsWork(ctx: FeatureContext): Promise<void> {
  const hints = (await ctx.session.request("textDocument/inlayHint", {
    textDocument: { uri: ctx.uri },
    range: FULL_RANGE,
  })) as InlayHint[] | null;
  assert.ok(Array.isArray(hints), JSON.stringify(hints));
  const labels = hints.map(inlayLabel);
  assert.ok(labels.includes(": Ref<string>"), labels.join(", "));
  assert.ok(labels.includes(": ComputedRef<number>"), labels.join(", "));
}

export async function assertCodeLensWorks(ctx: FeatureContext): Promise<void> {
  const lenses = (await ctx.session.request("textDocument/codeLens", {
    textDocument: { uri: ctx.uri },
  })) as CodeLens[] | null;
  assert.ok(Array.isArray(lenses), JSON.stringify(lenses));
  assert.ok(lenses.some((lens) => lens.command?.command === "vize.findReferences"));
}

export async function assertDocumentLinksWork(ctx: FeatureContext): Promise<void> {
  const links = (await ctx.session.request("textDocument/documentLink", {
    textDocument: { uri: ctx.uri },
  })) as DocumentLink[] | null;
  assert.ok(Array.isArray(links), JSON.stringify(links));
  const basenames = links.map((link) =>
    path.basename(decodeURIComponent(new URL(link.target ?? "").pathname)),
  );
  assert.ok(basenames.includes("Dep.vue"), basenames.join(", "));
  assert.ok(basenames.includes("useServer.mjs"), basenames.join(", "));
}

export async function assertDefinitionWorks(ctx: FeatureContext): Promise<void> {
  const definition = (await ctx.session.request("textDocument/definition", {
    textDocument: { uri: ctx.uri },
    position: templateMessagePosition(ctx.source),
  })) as Parameters<typeof firstLocation>[0];
  const location = firstLocation(definition);
  assert.equal(location.uri, ctx.uri);
  assert.deepEqual(location.range.start, declarationPosition(ctx.source));
}

export async function assertReferencesWork(ctx: FeatureContext): Promise<void> {
  const references = (await ctx.session.request("textDocument/references", {
    textDocument: { uri: ctx.uri },
    position: templateMessagePosition(ctx.source),
    context: { includeDeclaration: true },
  })) as Array<{ uri: string; range: { start: { line: number; character: number } } }> | null;
  assert.ok(Array.isArray(references), JSON.stringify(references));
  assert.ok(
    references.some(
      (reference) =>
        reference.uri === ctx.uri &&
        reference.range.start.line === declarationPosition(ctx.source).line,
    ),
    JSON.stringify(references),
  );
}

export async function expectNull(
  ctx: FeatureContext,
  method: string,
  params: unknown,
): Promise<void> {
  const response = await ctx.session.request(method, params);
  assert.equal(response, null, `${method} should be inert`);
}
