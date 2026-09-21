import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { root } from "./support/lsp/paths.ts";
import {
  workspace,
  open,
  prepare,
  span,
  sourceAt,
  type Outgoing,
  type Incoming,
} from "./support/lsp/call-hierarchy.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

const functions = `// 日本語 🌱\r\nexport function 葉(value: string): string { return value }\r\nexport function caller(): string { return 葉("hello") }\r\n`;

for (const extension of ["tsx", "jsx"]) {
  test(`call hierarchy follows opted-in ${extension} calls with authored UTF-16 ranges`, async (t) => {
    const runtime = requireTypecheckDependency(
      t,
      resolveTypecheckRuntime(root),
      "native TypeScript",
      "native TypeScript unavailable",
    );
    if (!runtime) return;
    const source =
      (extension === "tsx" ? functions : functions.replaceAll(": string", "")) +
      "export const view = <span>{caller()}</span>\r\n";
    await workspace(
      runtime,
      { [`View.${extension}`]: source },
      true,
      async (session, directory) => {
        const uri = pathToFileURL(path.join(directory, `View.${extension}`)).href;
        await open(
          session,
          uri,
          source,
          extension === "tsx" ? "typescriptreact" : "javascriptreact",
        );
        const caller = await prepare(session, uri, source, "caller()");
        const calls = (await session.request("callHierarchy/outgoingCalls", {
          item: caller,
        })) as Outgoing[];
        assert.deepEqual(
          calls.map(({ to, fromRanges }) => ({
            name: to.name,
            uri: to.uri,
            range: to.selectionRange,
            fromRanges,
          })),
          [
            {
              name: "葉",
              uri,
              range: span(source, "葉(value", "葉"),
              fromRanges: [span(source, '葉("hello")', "葉")],
            },
          ],
        );
        const incoming = (await session.request("callHierarchy/incomingCalls", {
          item: calls[0].to,
        })) as Incoming[];
        assert.deepEqual(
          incoming.map(({ from, fromRanges }) => ({
            name: from.name,
            uri: from.uri,
            range: from.selectionRange,
            fromRanges,
          })),
          [
            {
              name: "caller",
              uri,
              range: caller.selectionRange,
              fromRanges: [span(source, '葉("hello")', "葉")],
            },
          ],
        );
        assert.equal(sourceAt(source, calls[0].to.selectionRange), "葉");
      },
    );
  });
}

test("call hierarchy expands unopened TS dependencies and closed Vue callers", async (t) => {
  const runtime = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "native TypeScript",
    "native TypeScript unavailable",
  );
  if (!runtime) return;
  const leaf = functions;
  const library =
    'import { 葉 } from "./leaf"\r\nexport function library() { return 葉("library") }\r\n';
  const app =
    '<script setup lang="ts">\nimport { library } from "./Library.ts"\nfunction caller() { return library() }\n</script>\n<template>{{ caller() }}</template>\n';
  const closed = app.replaceAll("caller", "closedCaller");
  await workspace(
    runtime,
    { "leaf.ts": leaf, "Library.ts": library, "App.vue": app, "Closed.vue": closed },
    false,
    async (session, directory) => {
      const uri = pathToFileURL(path.join(directory, "App.vue")).href;
      const libraryUri = pathToFileURL(path.join(directory, "Library.ts")).href;
      const leafUri = pathToFileURL(path.join(directory, "leaf.ts")).href;
      const closedUri = pathToFileURL(path.join(directory, "Closed.vue")).href;
      await open(session, uri, app);
      const caller = await prepare(session, uri, app, "caller()");
      const calls = (await session.request("callHierarchy/outgoingCalls", {
        item: caller,
      })) as Outgoing[];
      assert.ok(Array.isArray(calls), "SFC caller must have outgoing calls");
      assert.deepEqual(
        calls.map(({ to }) => ({ name: to.name, uri: to.uri, range: to.selectionRange })),
        [{ name: "library", uri: libraryUri, range: span(library, "library()", "library") }],
      );
      const nested = (await session.request("callHierarchy/outgoingCalls", {
        item: calls[0].to,
      })) as Outgoing[];
      assert.ok(Array.isArray(nested), "unopened library must expand");
      assert.deepEqual(
        nested.map(({ to, fromRanges }) => ({
          name: to.name,
          uri: to.uri,
          range: to.selectionRange,
          fromRanges,
        })),
        [
          {
            name: "葉",
            uri: leafUri,
            range: span(leaf, "葉(value", "葉"),
            fromRanges: [span(library, '葉("library")', "葉")],
          },
        ],
      );
      const parents = (await session.request("callHierarchy/incomingCalls", {
        item: calls[0].to,
      })) as Incoming[];
      assert.deepEqual(
        parents
          .map(({ from, fromRanges }) => ({ name: from.name, uri: from.uri, fromRanges }))
          .sort((a, b) => a.name.localeCompare(b.name)),
        [
          {
            name: "caller",
            uri,
            fromRanges: [span(app, "return library()", "library", "return ".length)],
          },
          {
            name: "closedCaller",
            uri: closedUri,
            fromRanges: [span(closed, "return library()", "library", "return ".length)],
          },
        ],
      );
      const leafCalls = await session.request("callHierarchy/outgoingCalls", {
        item: nested[0].to,
      });
      assert.deepEqual(
        leafCalls,
        [],
        "unopened TypeScript leaf can expand without opening an editor",
      );
    },
  );
});

test("call hierarchy leaves JSX untouched when its typechecker opt-in is disabled", async (t) => {
  const runtime = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "native TypeScript",
    "native TypeScript unavailable",
  );
  if (!runtime) return;
  const source = functions + "export const view = <span>{caller()}</span>\r\n";
  await workspace(runtime, { "View.tsx": source }, false, async (session, directory) => {
    const uri = pathToFileURL(path.join(directory, "View.tsx")).href;
    await open(session, uri, source, "typescriptreact");
    assert.equal(
      await session.request("textDocument/prepareCallHierarchy", {
        textDocument: { uri },
        position: offsetToPosition(source, source.indexOf("caller()")),
      }),
      null,
    );
  });
});

test("call hierarchy rejects stale items and uses unsaved source after prepare and repair", async (t) => {
  const runtime = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "native TypeScript",
    "native TypeScript unavailable",
  );
  if (!runtime) return;
  const source = `<script setup lang="ts">
${functions}
</script>
<template>{{ caller() }}</template>
`.replaceAll("export function", "function");
  await workspace(runtime, { "App.vue": source }, false, async (session, directory) => {
    const uri = pathToFileURL(path.join(directory, "App.vue")).href;
    await open(session, uri, source);
    const stale = await prepare(session, uri, source, "caller()");
    const edited = source.replace("function caller()", "// unsaved 日本語 🌱\nfunction caller()");
    session.notify("textDocument/didChange", {
      textDocument: { uri, version: 2 },
      contentChanges: [{ text: edited }],
    });
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri) && params.version === 2,
      60_000,
    );
    assert.equal(
      await session.request("callHierarchy/outgoingCalls", { item: stale }),
      null,
      "old coordinates cannot query a changed source",
    );
    const current = await prepare(session, uri, edited, "caller()");
    const changed = (await session.request("callHierarchy/outgoingCalls", {
      item: current,
    })) as Outgoing[];
    assert.deepEqual(
      changed.map(({ to, fromRanges }) => ({ name: to.name, uri: to.uri, fromRanges })),
      [{ name: "葉", uri, fromRanges: [span(edited, '葉("hello")', "葉")] }],
    );
    session.notify("textDocument/didChange", {
      textDocument: { uri, version: 3 },
      contentChanges: [{ text: source }],
    });
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri) && params.version === 3,
      60_000,
    );
    const repaired = await prepare(session, uri, source, "caller()");
    const calls = (await session.request("callHierarchy/outgoingCalls", {
      item: repaired,
    })) as Outgoing[];
    assert.deepEqual(
      calls.map(({ to, fromRanges }) => ({ name: to.name, uri: to.uri, fromRanges })),
      [{ name: "葉", uri, fromRanges: [span(source, '葉("hello")', "葉")] }],
    );
  });
});
