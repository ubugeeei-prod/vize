import assert from "node:assert/strict";
import { registerHooks } from "node:module";
import type { TextDocumentChangeEvent, TextEditor } from "vscode";
import type { LanguageClient } from "vscode-languageclient/node.js";

function emitter<T>() {
  const listeners = new Set<(event: T) => unknown>();
  return {
    listeners,
    event: (listener: (event: T) => unknown) => {
      listeners.add(listener);
      return { dispose: () => listeners.delete(listener) };
    },
    fire(event: T) {
      for (const listener of listeners) listener(event);
    },
  };
}
export const selectionChanged = emitter<{ textEditor: TextEditor; kind?: number }>();
export const editorChanged = emitter<TextEditor | undefined>();
export const documentChanged = emitter<TextDocumentChangeEvent>();
export const documentClosed = emitter<unknown>();
export const window = {
  activeTextEditor: undefined as TextEditor | undefined,
  onDidChangeTextEditorSelection: selectionChanged.event,
  onDidChangeActiveTextEditor: editorChanged.event,
};
const workspace = {
  onDidChangeTextDocument: documentChanged.event,
  onDidCloseTextDocument: documentClosed.event,
};
class CancellationTokenSource {
  private changed = emitter<void>();
  token = { isCancellationRequested: false, onCancellationRequested: this.changed.event };
  cancel() {
    this.token.isCancellationRequested = true;
    this.changed.fire();
  }
  dispose() {
    this.changed.listeners.clear();
  }
}
Object.assign(globalThis, {
  __vizeAutoInsertWindow: window,
  __vizeAutoInsertWorkspace: workspace,
  __vizeAutoInsertCancellationTokenSource: CancellationTokenSource,
});
const hooks = registerHooks({
  resolve(specifier, context, next) {
    if (specifier === "./auto-insert-test-state.js") {
      return next("./auto-insert-test-state.ts", context);
    }
    return specifier === "vscode"
      ? { url: "vize-test:auto-insert-vscode", shortCircuit: true }
      : next(specifier, context);
  },
  load(url, context, next) {
    return url === "vize-test:auto-insert-vscode"
      ? {
          format: "module",
          shortCircuit: true,
          source: `export const window = globalThis.__vizeAutoInsertWindow;
            export const workspace = globalThis.__vizeAutoInsertWorkspace;
            export const CancellationTokenSource = globalThis.__vizeAutoInsertCancellationTokenSource;
            export class SnippetString { constructor(value) { this.value = value; } }`,
        }
      : next(url, context);
  },
});
const { AUTO_INSERT_METHOD, createAutoInsertMiddleware } =
  await import("../../../editors/vscode/src/auto-insert.ts");
hooks.deregister();

export function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

export function scenario(text = "{}") {
  const response = deferred<string | null>();
  const requested = deferred<void>();
  const requests: unknown[] = [];
  const tokens: Array<{ isCancellationRequested: boolean }> = [];
  const insertions: unknown[] = [];
  const positionAt = (character: number) => ({
    line: 0,
    character,
    isEqual: (other: { line: number; character: number }) =>
      other.line === 0 && other.character === character,
  });
  const document = {
    version: 1,
    isClosed: false,
    uri: { toString: () => "file:///App.vue" },
    positionAt,
  };
  const position = positionAt(1 + (text === "{}" ? 1 : text.length));
  const editor = {
    document,
    selection: { active: position, isEmpty: true },
    selections: [{}],
    async insertSnippet(snippet: { value: string }, ...args: unknown[]) {
      insertions.push([{ value: snippet.value }, ...args]);
      return true;
    },
  };
  const client = {
    initializeResult: { capabilities: { experimental: { autoInsertionProvider: {} } } },
    sendRequest(method: string, params: unknown, token: { isCancellationRequested: boolean }) {
      tokens.push(token);
      assert.equal(method, AUTO_INSERT_METHOD);
      requests.push(params);
      requested.resolve();
      return response.promise;
    },
  };
  const state = { client: client as unknown as LanguageClient, enabled: true };
  window.activeTextEditor = editor as unknown as TextEditor;
  const middleware = createAutoInsertMiddleware(() => state.client, {
    get: <T>(_key: string, _defaultValue: T) => state.enabled as T,
    inspect: () => undefined,
  });
  const event = {
    document,
    contentChanges: [{ rangeOffset: 1, rangeLength: 0, text }],
  } as unknown as TextDocumentChangeEvent;
  const run = (next: () => Promise<void> = async () => {}, changeEvent = event) =>
    middleware.didChange!(changeEvent, next);
  return {
    run,
    event,
    document,
    editor,
    position,
    state,
    response,
    requested,
    requests,
    tokens,
    insertions,
  };
}
