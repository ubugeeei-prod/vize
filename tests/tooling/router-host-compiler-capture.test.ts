import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { test } from "node:test";

const { captureCompilerDocument } = createRequire(import.meta.url)(
  "../../editors/vscode/test/suite/router-host-compiler-capture.cjs",
);
const uri = "file:///owned/Page.art.vue.setup.ts";
const source = "// 雪😀\nconst page = route.params.page\n";
const item = {
  label: "page",
  kind: 5,
  sortText: "11",
  data: {
    fileName: "/owned/Page.art.vue.setup.ts",
    name: "page",
    position: Buffer.byteLength(source.slice(0, source.lastIndexOf("page"))),
  },
};
const completion = [
  {
    ...item,
    data: {
      vizeCompletion: { item, requestUri: uri, uri: "file:///owned/Page.art.vue", revision: 1 },
    },
  },
];
const frame = (pid: number, direction: string, packet: unknown) => ({
  kind: "frame",
  pid,
  direction,
  packet,
});
const events = [
  frame(10, "client", {
    method: "textDocument/didOpen",
    params: { textDocument: { uri, version: 1, text: source } },
  }),
  frame(11, "client", {
    method: "textDocument/didOpen",
    params: { textDocument: { uri, version: 1, text: "foreign owner" } },
  }),
  frame(10, "client", {
    id: 7,
    method: "textDocument/completion",
    params: { textDocument: { uri } },
  }),
  frame(10, "server", { id: 7, result: { isIncomplete: false, items: [item] } }),
];

test("compiler capture binds the full native response to its actual open owner and raw source", () => {
  const actual = captureCompilerDocument(events, completion);
  assert.equal(actual.source, source);
  assert.equal(actual.bytes, Buffer.byteLength(source));
  assert.deepEqual(actual.observation, {
    pid: 10,
    request: 2,
    response: 3,
    snapshotFrames: [0],
    version: 1,
    requestUri: uri,
  });
  const changed = events.slice();
  changed.splice(
    2,
    0,
    frame(10, "client", {
      method: "textDocument/didChange",
      params: {
        textDocument: { uri, version: 2 },
        contentChanges: [{ text: `${source}// next\n` }],
      },
    }),
  );
  assert.equal(captureCompilerDocument(changed, completion).source, `${source}// next\n`);
});

test("compiler capture rejects missing owner/request, foreign URI, item and offset drift", () => {
  for (const mutate of [
    (copy: typeof events) => {
      copy[3].pid = 12;
    },
    (copy: typeof events) => {
      copy[2] = frame(10, "client", {
        id: 8,
        method: "textDocument/completion",
        params: { textDocument: { uri } },
      });
    },
    (copy: typeof events) => {
      copy[2] = frame(10, "client", {
        id: 7,
        method: "textDocument/completion",
        params: { textDocument: { uri: `${uri}-foreign` } },
      });
    },
    (copy: typeof events) => {
      copy[3] = frame(10, "server", {
        id: 7,
        result: {
          isIncomplete: false,
          items: [{ ...item, data: { ...item.data, position: item.data.position + 1 } }],
        },
      });
    },
    (copy: typeof events) => {
      copy[3] = frame(10, "server", {
        id: 7,
        result: { isIncomplete: false, items: [{ ...item, label: "foreign" }] },
      });
    },
  ]) {
    const copy = structuredClone(events);
    mutate(copy);
    assert.throws(() => captureCompilerDocument(copy, completion), assert.AssertionError);
  }
});

test("compiler capture refuses a closed or unsupported incremental snapshot", () => {
  for (const packet of [
    { method: "textDocument/didClose", params: { textDocument: { uri } } },
    {
      method: "textDocument/didChange",
      params: {
        textDocument: { uri, version: 2 },
        contentChanges: [
          {
            range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
            text: "drift",
          },
        ],
      },
    },
  ]) {
    const copy = events.slice();
    copy.splice(2, 0, frame(10, "client", packet));
    assert.throws(() => captureCompilerDocument(copy, completion), assert.AssertionError);
  }
});
