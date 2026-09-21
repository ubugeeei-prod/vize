import assert from "node:assert/strict";
import { test } from "node:test";
import type { TextEditor } from "vscode";
import type { LanguageClient } from "vscode-languageclient/node.js";
import { waitForAutoInsertIdle } from "../../editors/vscode/src/auto-insert-test-state.ts";
import {
  scenario,
  deferred,
  window,
  selectionChanged,
  editorChanged,
  documentChanged,
  documentClosed,
} from "./support/auto-insert-scenario.ts";

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

test("changes to another document preserve the active authored request", async () => {
  const s = scenario();
  const running = s.run();
  await s.requested.promise;
  await s.run(async () => {}, {
    ...s.event,
    document: { ...s.event.document },
  });
  assert.equal(s.tokens[0].isCancellationRequested, false);
  assert.equal(s.requests.length, 1);
  s.response.resolve(" $0 ");
  await running;
  assert.equal(s.insertions.length, 1);
});

for (const reason of ["edit", "close", "editor", "selection", "superseded request"]) {
  test(`in-flight auto insertion cancels on ${reason} without waiting for the server`, async () => {
    const s = scenario();
    const running = s.run();
    await s.requested.promise;
    if (reason === "edit") {
      s.document.version++;
      documentChanged.fire(s.event);
    } else if (reason === "close") {
      s.document.isClosed = true;
      documentClosed.fire(s.document);
    } else if (reason === "editor") {
      window.activeTextEditor = undefined;
      editorChanged.fire(undefined);
    } else if (reason === "selection") {
      s.editor.selection.active = s.document.positionAt(0);
      selectionChanged.fire({ textEditor: s.editor as unknown as TextEditor });
    } else {
      // Even a non-trigger edit must cancel the previous request and still
      // deliver didChange. The old server response deliberately never arrives.
      s.state.enabled = false;
      let forwarded = false;
      await s.run(async () => {
        forwarded = true;
      });
      assert.equal(forwarded, true);
    }
    await running;
    assert.equal(s.tokens[0].isCancellationRequested, true);
    assert.deepEqual(s.insertions, []);
    for (const event of [selectionChanged, editorChanged, documentChanged, documentClosed]) {
      assert.equal(event.listeners.size, 0);
    }
    // A server that ignores cancellation cannot apply the obsolete snippet.
    s.response.resolve(" $0 ");
    await Promise.resolve();
    assert.deepEqual(s.insertions, []);
  });
}
