import assert from "node:assert/strict";
import { registerHooks } from "node:module";
import { test } from "node:test";
import type { TextDocumentChangeEvent, TextEditor } from "vscode";
import type { LanguageClient } from "vscode-languageclient/node.js";
import { waitForAutoInsertIdle } from "../../editors/vscode/src/auto-insert-test-state.ts";

const window = { activeTextEditor: undefined as TextEditor | undefined };
Object.assign(globalThis, { __vizeAutoInsertWindow: window });
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

function scenario() {
  const response = deferred<string | null>();
  const requested = deferred<void>();
  const requests: unknown[] = [];
  const insertions: unknown[] = [];
  const document = { version: 1, isClosed: false, uri: { toString: () => "file:///App.vue" } };
  const position = { line: 0, character: 2, isEqual: (other: unknown) => other === position };
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
    contentChanges: [{ rangeOffset: 1, rangeLength: 0, text: "{}" }],
  } as unknown as TextDocumentChangeEvent;
  const run = (next: () => Promise<void> = async () => {}) => middleware.didChange!(event, next);
  return { run, document, editor, position, state, response, requested, requests, insertions };
}

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
