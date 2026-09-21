import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { root } from "./support/lsp/paths.ts";
import {
  workspace,
  open,
  prepare,
  span,
  type Incoming,
  type Outgoing,
} from "./support/lsp/call-hierarchy.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

const leafFunction = "export function 葉(value: string): string { return value }";
const leaf = `<script lang="ts">\r\n// 日本語 🌱\r\n${leafFunction}\r\n</script>\r\n<template><span /></template>\r\n`;
const localFunction = "function local(value: string): string { return 葉(value) }";
const libraryFunction = "export function library(): string { return local(suffix) }";
const library = `<script lang="ts">\r\nimport { 葉 } from "./Leaf.vue"\r\nconst suffix = "library"; /* 🌱 */ ${localFunction}\r\n${libraryFunction}\r\n</script>\r\n<template><span /></template>\r\n`;
const app =
  '<script setup lang="ts">\nimport { library } from "./Library.vue"\nfunction caller() { return library() }\n</script>\n<template>{{ caller() }}</template>\n';
const closed = app.replaceAll("caller", "closedCaller");

for (const opened of [false, true]) {
  test(`named Vue functions retain call hierarchy through ${opened ? "open" : "unopened"} SFC dependencies`, async (t) => {
    const runtime = requireTypecheckDependency(
      t,
      resolveTypecheckRuntime(root),
      "native TypeScript",
      "native TypeScript unavailable",
    );
    if (!runtime) return;
    await workspace(
      runtime,
      { "Leaf.vue": leaf, "Library.vue": library, "App.vue": app, "Closed.vue": closed },
      false,
      async (session, directory) => {
        const uri = (file: string) => pathToFileURL(path.join(directory, file)).href;
        if (opened) {
          await open(session, uri("Leaf.vue"), leaf);
          await open(session, uri("Library.vue"), library);
        }
        await open(session, uri("App.vue"), app);
        const caller = await prepare(session, uri("App.vue"), app, "caller()");
        const calls = (await session.request("callHierarchy/outgoingCalls", {
          item: caller,
        })) as Outgoing[];
        assert.deepEqual(
          calls.map(({ to, fromRanges }) => ({
            name: to.name,
            uri: to.uri,
            range: to.range,
            selection: to.selectionRange,
            fromRanges,
          })),
          [
            {
              name: "library",
              uri: uri("Library.vue"),
              range: span(library, libraryFunction, libraryFunction),
              selection: span(library, "library()", "library"),
              fromRanges: [span(app, "return library()", "library", 7)],
            },
          ],
        );
        const nested = (await session.request("callHierarchy/outgoingCalls", {
          item: calls[0].to,
        })) as Outgoing[];
        assert.deepEqual(
          nested.map(({ to, fromRanges }) => ({
            name: to.name,
            uri: to.uri,
            range: to.range,
            selection: to.selectionRange,
            fromRanges,
          })),
          [
            {
              name: "local",
              uri: uri("Library.vue"),
              range: span(library, localFunction, localFunction),
              selection: span(library, "local(value", "local"),
              fromRanges: [span(library, "local(suffix)", "local")],
            },
          ],
        );
        const leaves = (await session.request("callHierarchy/outgoingCalls", {
          item: nested[0].to,
        })) as Outgoing[];
        assert.deepEqual(
          leaves.map(({ to, fromRanges }) => ({
            name: to.name,
            uri: to.uri,
            range: to.range,
            selection: to.selectionRange,
            fromRanges,
          })),
          [
            {
              name: "葉",
              uri: uri("Leaf.vue"),
              range: span(leaf, leafFunction, leafFunction),
              selection: span(leaf, "葉(value", "葉"),
              fromRanges: [span(library, "return 葉(value)", "葉", 7)],
            },
          ],
        );
        assert.deepEqual(
          await session.request("callHierarchy/outgoingCalls", {
            item: leaves[0].to,
          }),
          [],
        );
        const incoming = (await session.request("callHierarchy/incomingCalls", {
          item: calls[0].to,
        })) as Incoming[];
        assert.deepEqual(
          incoming
            .map(({ from, fromRanges }) => ({
              name: from.name,
              uri: from.uri,
              selection: from.selectionRange,
              fromRanges,
            }))
            .sort((a, b) => a.name.localeCompare(b.name)),
          [
            {
              name: "caller",
              uri: uri("App.vue"),
              selection: span(app, "caller()", "caller"),
              fromRanges: [span(app, "return library()", "library", 7)],
            },
            {
              name: "closedCaller",
              uri: uri("Closed.vue"),
              selection: span(closed, "closedCaller()", "closedCaller"),
              fromRanges: [span(closed, "return library()", "library", 7)],
            },
          ],
        );
        if (opened) {
          const direct = await prepare(session, uri("Library.vue"), library, "library()");
          assert.deepEqual(direct.selectionRange, calls[0].to.selectionRange);
          assert.deepEqual(direct.range, calls[0].to.range);
        }
      },
    );
  });
}
