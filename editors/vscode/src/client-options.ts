import { workspace, type FileSystemWatcher, type OutputChannel } from "vscode";
import type { LanguageClient, LanguageClientOptions } from "vscode-languageclient/node.js";

import { createAutoInsertMiddleware } from "./auto-insert.js";
import { createDocumentSelector, type LspInitializationOptions } from "./extension-core.js";
import { withJsxRouting } from "./jsx-routing.js";

// vscode-languageclient hooks listeners onto synchronize.fileEvents and disposes
// only those on stop, never the watchers. One pair serves every restarted client.
let fileEvents: FileSystemWatcher[] | undefined;

export function disposeClientFileEvents(): void {
  for (const watcher of fileEvents ?? []) watcher.dispose();
  fileEvents = undefined;
}

export function createClientOptions(
  initializationOptions: LspInitializationOptions,
  getClient: () => LanguageClient | undefined,
  config: ReturnType<typeof workspace.getConfiguration>,
  outputChannel: OutputChannel,
): LanguageClientOptions {
  return {
    documentSelector: createDocumentSelector(),
    synchronize: {
      configurationSection: "vize",
      fileEvents: (fileEvents ??= [
        workspace.createFileSystemWatcher("**/*.vue"),
        workspace.createFileSystemWatcher("**/*.{html,htm}"),
      ]),
    },
    outputChannel,
    traceOutputChannel: outputChannel,
    initializationOptions,
    middleware: withJsxRouting(getClient, createAutoInsertMiddleware(getClient, config)),
  };
}
