import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { LspSession, LspRequestError } from "./support/lsp/session.ts";
import { root } from "./support/lsp/paths.ts";

const output = path.join(root, "target/differential/local-completion-resolve");
const source = `<script setup lang="ts">
const stable: number = 1
const route = useRoute()
route.params.id
</script>
<template><div>{{ stable }}</div></template>
<style>.x { display: flex; }</style>
`;
type Json = null | boolean | number | string | Json[] | { [key: string]: Json };
const position = (offset: number) => {
  const prefix = source.slice(0, offset);
  return {
    line: prefix.split("\n").length - 1,
    character: prefix.length - prefix.lastIndexOf("\n") - 1,
  };
};
const nativeMarker = [null, {}, { uri: false }, { uri: "not a URI" }];

test("local and catalog completion resolve survives real multi-root configuration retirement", async () => {
  fs.mkdirSync(output, { recursive: true });
  const workspace = fs.mkdtempSync(path.join(output, "workspace-"));
  const observed: unknown[] = [];
  const save = () =>
    fs.writeFileSync(
      path.join(output, "observations.json"),
      `${JSON.stringify(observed, null, 2)}\n`,
    );
  const folders = ["a", "b"].map((name) => path.join(workspace, name));
  const config = (hover: boolean) =>
    `export default ${JSON.stringify({ vue: { languageServer: { hover, completion: true, ecosystem: true, typecheck: false, lint: false }, linter: { enabled: false } } })};\n`;
  const pages = folders.map((folder) => {
    fs.mkdirSync(path.join(folder, "src/pages/products"), { recursive: true });
    fs.writeFileSync(path.join(folder, "package.json"), '{"private":true,"type":"module"}\n');
    fs.writeFileSync(path.join(folder, "vite.config.mjs"), config(true));
    fs.writeFileSync(
      path.join(folder, "tsconfig.json"),
      '{"compilerOptions":{"strict":true},"include":["src/**/*"]}\n',
    );
    const file = path.join(folder, "src/pages/products/[id].vue");
    fs.writeFileSync(file, source);
    return { file, uri: pathToFileURL(file).href };
  });
  const session = new LspSession();
  session.notificationObservers.push((method, params) => {
    observed.push({ kind: "notification", method, params });
    save();
  });
  const request = async (method: string, params: unknown): Promise<Json> => {
    const started = performance.now();
    try {
      const response = await session.request(method, params);
      observed.push({
        kind: "response",
        method,
        params,
        response,
        elapsedMs: performance.now() - started,
      });
      save();
      return response as Json;
    } catch (error) {
      observed.push({
        kind: "error",
        method,
        params,
        error:
          error instanceof LspRequestError
            ? { code: error.code, message: error.message, data: error.data }
            : String(error),
        elapsedMs: performance.now() - started,
      });
      save();
      throw error;
    }
  };
  const publish = (uri: string) =>
    session.waitForNotification(
      "textDocument/publishDiagnostics",
      (packet) => (packet as { uri: string }).uri === uri,
    );
  try {
    await request("initialize", {
      processId: process.pid,
      rootUri: pages.length ? pathToFileURL(folders[0]).href : null,
      workspaceFolders: folders.map((folder) => ({
        uri: pathToFileURL(folder).href,
        name: path.basename(folder),
      })),
      capabilities: {
        textDocument: {
          completion: {
            completionItem: {
              documentationFormat: ["markdown"],
              resolveSupport: { properties: ["documentation"] },
            },
          },
        },
      },
      initializationOptions: { typecheck: false, lint: false },
    });
    session.notify("initialized", {});
    for (const { uri } of pages) {
      session.notify("textDocument/didOpen", {
        textDocument: { uri, languageId: "vue", version: 1, text: source },
      });
      assert.deepEqual(await publish(uri), { diagnostics: [], uri, version: 1 });
    }
    const completionParams = (uri: string) => ({
      textDocument: { uri },
      position: position(source.indexOf("route.params.id") + "route.params.".length),
    });
    const hoverParams = (uri: string) => ({
      textDocument: { uri },
      position: position(source.indexOf("const stable") + "const st".length),
    });
    const expectedItem = {
      label: "id",
      kind: 10,
      detail: "Vue Router file route param: string",
      sortText: "0",
    };
    const item = async (uri: string) => {
      const vector = await request("textDocument/completion", completionParams(uri));
      assert.deepEqual(vector, [expectedItem]);
      return (vector as Json[])[0];
    };
    const initialItems = [];
    for (const { uri } of pages) {
      const value = await item(uri);
      initialItems.push(value);
      assert.deepEqual(await request("completionItem/resolve", value), value);
    }
    const siblingHover = await request("textDocument/hover", hoverParams(pages[1].uri));
    assert.notEqual(siblingHover, null);
    const catalog = (await request("textDocument/completion", {
      textDocument: { uri: pages[1].uri },
      position: position(source.indexOf("display:") + 4),
    })) as Array<Record<string, Json>>;
    const selected = catalog.find((entry) => entry.label === "display");
    assert.ok(selected);
    assert.deepEqual(selected.data, {
      vizeCss: { kind: "property", name: "display", property: null },
    });
    const initialStyle = (await request("completionItem/resolve", selected)) as Record<
      string,
      Json
    >;
    assert.ok(initialStyle.documentation);
    const insertion = { ...initialStyle };
    delete insertion.documentation;
    assert.deepEqual(insertion, selected);

    const changed = path.join(folders[0], "vite.config.mjs");
    fs.writeFileSync(changed, config(false));
    session.notify("workspace/didChangeWatchedFiles", {
      changes: [{ uri: pathToFileURL(changed).href, type: 2 }],
    });
    assert.deepEqual(await publish(pages[0].uri), {
      diagnostics: [],
      uri: pages[0].uri,
      version: 1,
    });
    assert.equal(await request("textDocument/hover", hoverParams(pages[0].uri)), null);
    assert.deepEqual(await request("textDocument/hover", hoverParams(pages[1].uri)), siblingHover);
    for (const { uri } of pages) {
      const value = await item(uri);
      assert.deepEqual(await request("completionItem/resolve", value), value);
    }
    for (const value of initialItems)
      assert.deepEqual(await request("completionItem/resolve", value), value);
    assert.deepEqual(await request("completionItem/resolve", selected), initialStyle);
    for (const marker of nativeMarker) {
      const value = { ...expectedItem, data: { vizeCompletion: marker } };
      await assert.rejects(
        request("completionItem/resolve", value),
        (error: unknown) => error instanceof LspRequestError && error.code === -32801,
      );
    }
    await assert.rejects(
      request("completionItem/resolve", {
        ...selected,
        data: { ...(selected.data as Record<string, Json>), vizeCompletion: null },
      }),
      (error: unknown) => error instanceof LspRequestError && error.code === -32801,
    );
  } finally {
    save();
    await session.shutdown();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
