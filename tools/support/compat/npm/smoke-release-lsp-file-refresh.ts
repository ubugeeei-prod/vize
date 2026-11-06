import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

interface WireMessage {
  method?: string;
  params?: unknown;
}

interface PublishedDiagnostics {
  diagnostics: Array<{ code?: unknown; message?: unknown }>;
  uri: string;
  version?: number | null;
}

interface FileRefreshContext {
  appPath: string;
  appText: string;
  appUri: string;
  notifications: readonly WireMessage[];
  record(name: string, result: unknown): void;
  send(message: object): void;
}

type WatchedFileChange = { uri: string; type: 1 | 2 | 3 };

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

function diagnostics(message: WireMessage, uri: string): PublishedDiagnostics | undefined {
  if (message.method !== "textDocument/publishDiagnostics") return undefined;
  if (message.params === null || typeof message.params !== "object") return undefined;
  const params = message.params as Partial<PublishedDiagnostics>;
  if (typeof params.uri !== "string" || !Array.isArray(params.diagnostics)) return undefined;
  const left = path.resolve(fileURLToPath(params.uri));
  const right = path.resolve(fileURLToPath(uri));
  const matches =
    process.platform === "win32" ? left.toLowerCase() === right.toLowerCase() : left === right;
  return matches ? (params as PublishedDiagnostics) : undefined;
}

const hasCode = (published: PublishedDiagnostics, code: number) =>
  published.diagnostics.some((diagnostic) => String(diagnostic.code) === String(code));

async function waitForDiagnostics(
  context: FileRefreshContext,
  name: string,
  uri: string,
  after: number,
  accept: (published: PublishedDiagnostics) => boolean,
): Promise<PublishedDiagnostics> {
  const deadline = performance.now() + 60_000;
  let cursor = after;
  let latest: PublishedDiagnostics | undefined;
  while (performance.now() < deadline) {
    while (cursor < context.notifications.length) {
      const published = diagnostics(context.notifications[cursor++], uri);
      if (!published) continue;
      latest = published;
      if (accept(published)) {
        context.record(name, published);
        return published;
      }
    }
    await sleep(25);
  }
  throw new Error(`${name} timed out; latest diagnostics: ${JSON.stringify(latest)}`);
}

async function afterWatchedFiles(
  context: FileRefreshContext,
  name: string,
  uri: string,
  changes: WatchedFileChange[],
  mutate: () => void,
  accept: (published: PublishedDiagnostics) => boolean,
) {
  const after = context.notifications.length;
  mutate();
  context.send({ method: "workspace/didChangeWatchedFiles", params: { changes } });
  return waitForDiagnostics(context, name, uri, after, accept);
}

/**
 * Proves the packed server invalidates open importers for SFC and TypeScript
 * file creation, rename, deletion, and recovery using watched-file events.
 */
export async function runReleaseLspFileRefresh(context: FileRefreshContext): Promise<void> {
  const sourceDirectory = path.dirname(context.appPath);
  const componentsDirectory = path.join(sourceDirectory, "components");
  const componentPath = path.join(componentsDirectory, "ReleaseRefreshProbe.vue");
  const renamedComponentPath = path.join(componentsDirectory, "ReleaseRefreshProbeRenamed.vue");
  const modulePath = path.join(componentsDirectory, "release-refresh-source.ts");
  const renamedModulePath = path.join(componentsDirectory, "release-refresh-source-renamed.ts");
  const componentUri = pathToFileURL(componentPath).href;
  const renamedComponentUri = pathToFileURL(renamedComponentPath).href;
  const moduleUri = pathToFileURL(modulePath).href;
  const renamedModuleUri = pathToFileURL(renamedModulePath).href;
  const lifecycleApp = [
    '<script setup lang="ts">',
    'import ReleaseRefreshProbe from "./components/ReleaseRefreshProbe.vue";',
    "</script>",
    "",
    "<template>",
    "  <ReleaseRefreshProbe />",
    "</template>",
    "",
  ].join("\n");
  const component = [
    '<script setup lang="ts">',
    'import { refreshValue } from "./release-refresh-source";',
    "const rendered: number = refreshValue;",
    "</script>",
    "",
    "<template>",
    "  <span>{{ rendered }}</span>",
    "</template>",
    "",
  ].join("\n");
  const module = "export const refreshValue = 1;\n";
  let childOpen = false;
  let appVersion = 1;

  for (const candidate of [componentPath, renamedComponentPath, modulePath, renamedModulePath]) {
    assert.equal(fs.existsSync(candidate), false, `release LSP probe already exists: ${candidate}`);
  }
  fs.mkdirSync(componentsDirectory, { recursive: true });

  try {
    let after = context.notifications.length;
    context.send({
      method: "textDocument/didChange",
      params: {
        textDocument: { uri: context.appUri, version: ++appVersion },
        contentChanges: [{ text: lifecycleApp }],
      },
    });
    await waitForDiagnostics(
      context,
      "missing SFC is diagnosed before watched creation",
      context.appUri,
      after,
      (published) => hasCode(published, 2307),
    );

    await afterWatchedFiles(
      context,
      "created SFC clears its open importer",
      context.appUri,
      [{ uri: componentUri, type: 1 }],
      () => fs.writeFileSync(componentPath, component),
      (published) => published.diagnostics.length === 0,
    );

    after = context.notifications.length;
    context.send({
      method: "textDocument/didOpen",
      params: {
        textDocument: { uri: componentUri, languageId: "vue", version: 1, text: component },
      },
    });
    childOpen = true;
    await waitForDiagnostics(
      context,
      "created SFC reports its missing TypeScript module",
      componentUri,
      after,
      (published) => hasCode(published, 2307),
    );

    await afterWatchedFiles(
      context,
      "created TypeScript module clears the SFC",
      componentUri,
      [{ uri: moduleUri, type: 1 }],
      () => fs.writeFileSync(modulePath, module),
      (published) => published.diagnostics.length === 0,
    );
    await afterWatchedFiles(
      context,
      "renamed TypeScript module invalidates the SFC",
      componentUri,
      [
        { uri: moduleUri, type: 3 },
        { uri: renamedModuleUri, type: 1 },
      ],
      () => fs.renameSync(modulePath, renamedModulePath),
      (published) => hasCode(published, 2307),
    );
    await afterWatchedFiles(
      context,
      "restored TypeScript module clears rename diagnostics",
      componentUri,
      [
        { uri: renamedModuleUri, type: 3 },
        { uri: moduleUri, type: 1 },
      ],
      () => fs.renameSync(renamedModulePath, modulePath),
      (published) => published.diagnostics.length === 0,
    );
    await afterWatchedFiles(
      context,
      "deleted TypeScript module invalidates the SFC",
      componentUri,
      [{ uri: moduleUri, type: 3 }],
      () => fs.rmSync(modulePath),
      (published) => hasCode(published, 2307),
    );
    await afterWatchedFiles(
      context,
      "recreated TypeScript module clears delete diagnostics",
      componentUri,
      [{ uri: moduleUri, type: 1 }],
      () => fs.writeFileSync(modulePath, module),
      (published) => published.diagnostics.length === 0,
    );

    context.send({
      method: "textDocument/didClose",
      params: { textDocument: { uri: componentUri } },
    });
    childOpen = false;
    await afterWatchedFiles(
      context,
      "renamed SFC invalidates its open importer",
      context.appUri,
      [
        { uri: componentUri, type: 3 },
        { uri: renamedComponentUri, type: 1 },
      ],
      () => fs.renameSync(componentPath, renamedComponentPath),
      (published) => hasCode(published, 2307),
    );
    await afterWatchedFiles(
      context,
      "restored SFC clears rename diagnostics",
      context.appUri,
      [
        { uri: renamedComponentUri, type: 3 },
        { uri: componentUri, type: 1 },
      ],
      () => fs.renameSync(renamedComponentPath, componentPath),
      (published) => published.diagnostics.length === 0,
    );
    await afterWatchedFiles(
      context,
      "deleted SFC invalidates its open importer",
      context.appUri,
      [{ uri: componentUri, type: 3 }],
      () => fs.rmSync(componentPath),
      (published) => hasCode(published, 2307),
    );
    await afterWatchedFiles(
      context,
      "recreated SFC clears delete diagnostics",
      context.appUri,
      [{ uri: componentUri, type: 1 }],
      () => fs.writeFileSync(componentPath, component),
      (published) => published.diagnostics.length === 0,
    );

    after = context.notifications.length;
    context.send({
      method: "textDocument/didChange",
      params: {
        textDocument: { uri: context.appUri, version: ++appVersion },
        contentChanges: [{ text: context.appText }],
      },
    });
    await waitForDiagnostics(
      context,
      "restored authored app has no stale lifecycle diagnostics",
      context.appUri,
      after,
      (published) => published.diagnostics.length === 0,
    );
  } finally {
    if (childOpen) {
      context.send({
        method: "textDocument/didClose",
        params: { textDocument: { uri: componentUri } },
      });
    }
    for (const candidate of [componentPath, renamedComponentPath, modulePath, renamedModulePath]) {
      fs.rmSync(candidate, { force: true });
    }
  }
}
