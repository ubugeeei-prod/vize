import assert from "node:assert/strict";
import { registerHooks } from "node:module";
import { test } from "node:test";
import type { TextDocumentChangeEvent, TextEditor } from "vscode";
import type { LanguageClient } from "vscode-languageclient/node.js";
import { waitForAutoInsertIdle } from "../../editors/vscode/src/auto-insert-test-state.ts";

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
const selectionChanged = emitter<{ textEditor: TextEditor; kind?: number }>();
const editorChanged = emitter<TextEditor | undefined>();
const documentChanged = emitter<TextDocumentChangeEvent>();
const documentClosed = emitter<unknown>();
const window = {
  activeTextEditor: undefined as TextEditor | undefined,
  onDidChangeTextEditorSelection: selectionChanged.event,
  onDidChangeActiveTextEditor: editorChanged.event,
};
const workspace = {
  onDidChangeTextDocument: documentChanged.event,
  onDidCloseTextDocument: documentClosed.event,
};
Object.assign(globalThis, { __vizeAutoInsertWindow: window, __vizeAutoInsertWorkspace: workspace });
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
            export class SnippetString { constructor(value) { this.value = value; } }`,
        }
      : next(url, context);
  },
});
const { AUTO_INSERT_METHOD, createAutoInsertMiddleware } =
  await import("../../editors/vscode/src/auto-insert.ts");
hooks.deregister();

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

function scenario(text = "{}") {
  const response = deferred<string | null>();
  const requested = deferred<void>();
  const requests: unknown[] = [];
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
    sendRequest(method: string, params: unknown) {
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
  const run = (next: () => Promise<void> = async () => {}) => middleware.didChange!(event, next);
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
    insertions,
  };
}

test("ordinary insertion triggers wait for the end of the inserted UTF-16 text", async () => {
  for (const text of ["=", ">", "/", "value", "\u{10400}value"]) {
    const s = scenario(text);
    s.editor.selection.active = s.document.positionAt(1);
    const running = s.run();
    await Promise.resolve();
    assert.deepEqual(s.requests, []);
    s.editor.selection.active = s.position;
    selectionChanged.fire({ textEditor: s.editor as unknown as TextEditor });
    await s.requested.promise;
    s.response.resolve(null);
    await running;
    assert.deepEqual(s.requests, [
      {
        textDocument: { uri: "file:///App.vue" },
        selection: { line: 0, character: 1 + text.length },
        change: { rangeOffset: 1, rangeLength: 0, text },
      },
    ]);
  }
});

test("non-trigger edits do not subscribe or wait for a caret update", async () => {
  for (const text of ["", " ", "\n", "a\nb"]) {
    const s = scenario(text);
    s.editor.selection.active = s.document.positionAt(0);
    await s.run();
    assert.deepEqual(s.requests, []);
    for (const event of [selectionChanged, editorChanged, documentChanged, documentClosed]) {
      assert.equal(event.listeners.size, 0);
    }
  }
});

for (const reason of ["edit", "close", "editor", "selection", "command", "timeout"]) {
  test(`selection settlement cancels on ${reason} without a stale request`, async (t) => {
    t.mock.timers.enable({ apis: ["setTimeout"] });
    const s = scenario();
    s.editor.selection.active = s.document.positionAt(1);
    const running = s.run();
    await Promise.resolve();
    if (reason === "edit") {
      s.document.version++;
      documentChanged.fire(s.event);
    } else if (reason === "close") {
      s.document.isClosed = true;
      documentClosed.fire(s.document);
    } else if (reason === "editor") {
      window.activeTextEditor = undefined;
      editorChanged.fire(undefined);
    } else if (reason === "selection" || reason === "command") {
      s.editor.selection.active = s.document.positionAt(reason === "command" ? 3 : 0);
      selectionChanged.fire({
        textEditor: s.editor as unknown as TextEditor,
        kind: reason === "command" ? 3 : undefined,
      });
    } else {
      t.mock.timers.tick(1_000);
    }
    await running;
    assert.deepEqual(s.requests, []);
    assert.deepEqual(s.insertions, []);
    for (const event of [selectionChanged, editorChanged, documentChanged, documentClosed]) {
      assert.equal(event.listeners.size, 0);
    }
  });
}

test("auto insertion waits for the authored caret even beyond the next timer turn", async (t) => {
  t.mock.timers.enable({ apis: ["setTimeout"] });
  const s = scenario();
  s.editor.selection.active = s.document.positionAt(1);
  const running = s.run();
  try {
    await Promise.resolve();
    t.mock.timers.tick(1);
    await Promise.resolve();
    assert.deepEqual(s.requests, [], "the pre-typing caret must never reach the server");
    s.editor.selection.active = s.document.positionAt(3);
    selectionChanged.fire({ textEditor: s.editor as unknown as TextEditor });
    assert.equal(selectionChanged.listeners.size, 1, "the implicit edit-end caret is intermediate");
    assert.deepEqual(s.requests, []);
    s.editor.selection.active = s.position;
    selectionChanged.fire({ textEditor: s.editor as unknown as TextEditor });
    await s.requested.promise;
    s.response.resolve(" $0 ");
    await running;
    assert.deepEqual(s.requests, [
      {
        textDocument: { uri: "file:///App.vue" },
        selection: { line: 0, character: 2 },
        change: { rangeOffset: 1, rangeLength: 0, text: "{}" },
      },
    ]);
    assert.equal(s.insertions.length, 1);
    for (const event of [selectionChanged, editorChanged, documentChanged, documentClosed]) {
      assert.equal(event.listeners.size, 0, "selection wait listeners must be disposed");
    }
  } finally {
    s.response.resolve(null);
    t.mock.timers.tick(1_000);
    await running;
  }
});

test("auto insertion uses the exact unchanged authored snapshot", async () => {
  const s = scenario();
  const running = s.run();
  await s.requested.promise;
  s.response.resolve(" $0 ");
  await running;
  assert.deepEqual(s.requests, [
    {
      textDocument: { uri: "file:///App.vue" },
      selection: { line: 0, character: 2 },
      change: { rangeOffset: 1, rangeLength: 0, text: "{}" },
    },
  ]);
  assert.deepEqual(s.insertions, [
    [
      { value: " $0 " },
      s.position,
      {
        undoStopBefore: false,
        undoStopAfter: false,
      },
    ],
  ]);
});

test("edits superseded while didChange is forwarded send no stale request", async () => {
  const s = scenario();
  const forwarded = deferred<void>();
  const running = s.run(() => forwarded.promise);
  s.document.version++;
  s.response.resolve(" $0 ");
  forwarded.resolve();
  await running;
  assert.deepEqual(s.requests, []);
  assert.deepEqual(s.insertions, []);
});

test("host barrier waits for response handling and the snippet application promise", async () => {
  const previous = process.env.VIZE_TEST_ENABLE_HOST_COMMANDS;
  process.env.VIZE_TEST_ENABLE_HOST_COMMANDS = "1";
  const s = scenario();
  const inserting = deferred<void>();
  const applied = deferred<void>();
  s.editor.insertSnippet = async () => {
    inserting.resolve();
    await applied.promise;
    return true;
  };
  try {
    const running = s.run();
    await s.requested.promise;
    let idle = false;
    const barrier = waitForAutoInsertIdle().then(() => {
      idle = true;
    });
    await Promise.resolve();
    assert.equal(idle, false, "response is still pending");
    s.response.resolve(" $0 ");
    await inserting.promise;
    assert.equal(idle, false, "snippet application is still pending");
    applied.resolve();
    await running;
    await barrier;
    assert.equal(idle, true);
  } finally {
    applied.resolve();
    s.response.resolve(null);
    if (previous === undefined) delete process.env.VIZE_TEST_ENABLE_HOST_COMMANDS;
    else process.env.VIZE_TEST_ENABLE_HOST_COMMANDS = previous;
  }
});

for (const [name, invalidate] of Object.entries({
  "document revision": (s: ReturnType<typeof scenario>) => {
    s.document.version++;
  },
  "closed document": (s: ReturnType<typeof scenario>) => {
    s.document.isClosed = true;
  },
  "active editor": () => {
    window.activeTextEditor = undefined;
  },
  "expanded selection": (s: ReturnType<typeof scenario>) => {
    s.editor.selection.isEmpty = false;
  },
  "multiple selections": (s: ReturnType<typeof scenario>) => {
    s.editor.selections.push({});
  },
  "server replacement": (s: ReturnType<typeof scenario>) => {
    s.state.client = {} as LanguageClient;
  },
  "disabled feature": (s: ReturnType<typeof scenario>) => {
    s.state.enabled = false;
  },
})) {
  test(`auto insertion discards a response after a change to ${name}`, async () => {
    const s = scenario();
    const running = s.run();
    await s.requested.promise;
    invalidate(s);
    s.response.resolve(" $0 ");
    await running;
    assert.equal(s.requests.length, 1);
    assert.deepEqual(s.insertions, []);
  });
}
