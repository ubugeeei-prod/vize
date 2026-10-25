import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { LspSession, LspRequestError } from "./support/lsp/session.ts";
import { root } from "./support/lsp/paths.ts";
import { decodeFrames } from "../differential/lsp-wire.ts";
import { readLocalResolveSourceSession } from "../differential/lsp-source-sessions.ts";

const output = path.join(root, "target/differential/local-completion-resolve");
const fixture = readLocalResolveSourceSession(root);
const source = fixture.files.get(fixture.spec.documentPaths[0])!.toString("utf8");
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
  const folders = fixture.spec.workspaceFolders.map((name) => path.join(workspace, name));
  for (const file of fixture.initialFiles) {
    const destination = path.join(workspace, file.path);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(destination, file.bytes);
  }
  const pages = fixture.spec.documentPaths.map((name) => {
    const file = path.join(workspace, name);
    assert.equal(fs.readFileSync(file, "utf8"), source);
    return { file, uri: pathToFileURL(file).href };
  });
  observed.push({
    kind: "authored-inputs",
    caseSha256: fixture.caseSha256,
    files: fixture.initialFiles.map(({ path: name }) => ({
      path: name,
      source: fs.readFileSync(path.join(workspace, name), "utf8"),
    })),
  });
  save();
  const session = new LspSession();
  session.responseObservers.push((message) => {
    observed.push({ kind: "raw-response", message });
    save();
  });
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
  let completed = false;
  try {
    await request("initialize", {
      processId: process.pid,
      rootUri: pages.length ? pathToFileURL(folders[0]).href : null,
      workspaceFolders: folders.map((folder) => ({
        uri: pathToFileURL(folder).href,
        name: path.basename(folder),
      })),
      capabilities: fixture.capabilities,
      initializationOptions: fixture.data.initializationOptions,
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
    const expectedItem = fixture.localItem;
    const item = async (uri: string) => {
      const vector = await request("textDocument/completion", completionParams(uri));
      // This is the pinned public failure vector; membership alone would lose
      // the historical item, ordering and extra-field contract.
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

    const changed = path.join(workspace, fixture.spec.transition.target);
    fs.writeFileSync(changed, fixture.changedConfig);
    observed.push({
      kind: "configuration-write",
      path: path.relative(workspace, changed),
      source: fs.readFileSync(changed, "utf8"),
    });
    save();
    session.notify("workspace/didChangeWatchedFiles", {
      changes: [{ uri: pathToFileURL(changed).href, type: fixture.spec.transition.eventType }],
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
    completed = true;
  } finally {
    try {
      await session.shutdown();
      await session.waitForClosed();
      if (process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD === "1") {
        const captures = path.join(root, "target/differential/lsp-sessions");
        const candidates = fs
          .readdirSync(captures)
          .filter((name) => name.startsWith(`${process.pid}-`));
        assert.equal(
          candidates.length,
          1,
          "this dedicated executor owns exactly one source-bound session",
        );
        const capture = path.join(captures, candidates[0]);
        fs.cpSync(capture, path.join(output, "source-session"), { recursive: true });
        if (completed) {
          const requests = decodeFrames(
            fs.readFileSync(path.join(capture, "client.bin")),
          ).messages.filter(
            (message) => typeof message.id === "number" && typeof message.method === "string",
          );
          assert.deepEqual(
            requests.map(({ id }) => id),
            Array.from({ length: fixture.spec.plannedClientRpcCount }, (_, index) => index + 1),
          );
          assert.equal(requests[0].method, "initialize");
          assert.equal(requests.at(-1)?.method, "shutdown");
        }
      }
      if (completed) {
        const raw = observed.filter(
          (entry) => (entry as { kind?: string }).kind === "raw-response",
        ) as Array<{ message: { id: number } }>;
        assert.deepEqual(
          raw.map(({ message }) => message.id),
          Array.from({ length: fixture.spec.plannedClientRpcCount }, (_, index) => index + 1),
        );
        assert.equal(
          observed.filter((entry) =>
            ["response", "error"].includes((entry as { kind: string }).kind),
          ).length,
          fixture.spec.directWrapperRpcCount,
        );
      }
    } finally {
      save();
      fs.rmSync(workspace, { recursive: true, force: true });
    }
  }
});
