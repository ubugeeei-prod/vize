import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { gunzipSync } from "node:zlib";
import { LspSession, LspRequestError } from "./support/lsp/session.ts";
import { root } from "./support/lsp/paths.ts";
import { readLocalResolveSourceSession } from "../differential/lsp-source-sessions.ts";
import { decodeFrames } from "../differential/lsp-wire.ts";
import { readPinnedArtifact } from "../differential/harness.mjs";

const fixture = readLocalResolveSourceSession(root);
const source = fixture.files.get(fixture.spec.documentPaths[0])!.toString("utf8");
const before = JSON.parse(
  gunzipSync(fixture.files.get("custody/public-before-observation.json.gz")!).toString("utf8"),
);
const catalog = before.find(
  (entry: { kind: string; method: string; response?: Array<{ data?: { vizeCss?: unknown } }> }) =>
    entry.kind === "response" &&
    entry.method === "textDocument/completion" &&
    Array.isArray(entry.response) &&
    entry.response.some((item) => item.data?.vizeCss),
).response;
const selected = catalog.find((item: { label: string }) => item.label === "display");
const historicalResolved = before.find(
  (entry: { kind: string; method: string; params?: { label: string } }) =>
    entry.kind === "response" &&
    entry.method === "completionItem/resolve" &&
    entry.params?.label === "display",
).response;
// Historical public-before bytes remain immutable. This separate CURRENT
// reference authors only the approved Example fence; live packets below are
// compared whole, without normalization or accepting either response form.
const resolved = JSON.parse(
  readPinnedArtifact(
    path.join(root, "tests/_fixtures/differential/lsp/css-hover-stylesheet-examples-10572"),
    {
      path: "local-completion-resolve.expected-current.json",
      sha256: "33f35ddde74f1bc0221e47d07d891d594bac2c86fdaea0c21b3ae2539a667035",
    },
  ).toString("utf8"),
);
assert.deepEqual(
  { ...resolved, documentation: historicalResolved.documentation },
  historicalResolved,
);
assert.equal(
  resolved.documentation.value,
  historicalResolved.documentation.value.replace(
    "```css\ndisplay: block;\n```",
    "```css\n.example {\n  display: block;\n}\n```",
  ),
);
const position = (offset: number) => {
  const prefix = source.slice(0, offset);
  return {
    line: prefix.split("\n").length - 1,
    character: prefix.length - prefix.lastIndexOf("\n") - 1,
  };
};

// A separate session preserves every request of the original 23-RPC public
// failure scenario while exercising the newly found mixed-owner regression.
test("only exact CSS catalog identities bypass a genuinely retired multi-root owner", async () => {
  const output = path.join(root, "target/differential/local-completion-resolve/ownership");
  fs.mkdirSync(output, { recursive: true });
  const workspace = fs.mkdtempSync(path.join(output, "workspace-"));
  const observed: Array<Record<string, unknown>> = [];
  const raw: Array<{ id?: number; error?: unknown }> = [];
  const save = () =>
    fs.writeFileSync(
      path.join(output, "observations.json"),
      `${JSON.stringify(observed, null, 2)}\n`,
    );
  for (const input of fixture.initialFiles) {
    const file = path.join(workspace, input.path);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, input.bytes);
  }
  observed.push({
    kind: "authored-inputs",
    caseSha256: fixture.caseSha256,
    files: fixture.initialFiles.map(({ path: name }) => ({
      path: name,
      source: fs.readFileSync(path.join(workspace, name), "utf8"),
    })),
  });
  save();
  const folders = fixture.spec.workspaceFolders.map((name) => path.join(workspace, name));
  const pages = fixture.spec.documentPaths.map(
    (name) => pathToFileURL(path.join(workspace, name)).href,
  );
  const session = new LspSession();
  session.responseObservers.push((message) => {
    raw.push(message);
    observed.push({ kind: "raw-response", message });
    save();
  });
  session.notificationObservers.push((method, params) => {
    observed.push({ kind: "notification", method, params });
    save();
  });
  let calls = 0;
  const request = async (method: string, params: unknown) => {
    calls++;
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
      return response;
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
      rootUri: pathToFileURL(folders[0]).href,
      workspaceFolders: folders.map((folder) => ({
        uri: pathToFileURL(folder).href,
        name: path.basename(folder),
      })),
      capabilities: fixture.capabilities,
      initializationOptions: fixture.data.initializationOptions,
    });
    session.notify("initialized", {});
    for (const uri of pages) {
      session.notify("textDocument/didOpen", {
        textDocument: { uri, languageId: "vue", version: 1, text: source },
      });
      assert.deepEqual(await publish(uri), { diagnostics: [], uri, version: 1 });
      assert.deepEqual(
        await request("textDocument/completion", {
          textDocument: { uri },
          position: position(source.indexOf("route.params.id") + "route.params.".length),
        }),
        [fixture.localItem],
      );
    }
    assert.deepEqual(
      await request("textDocument/completion", {
        textDocument: { uri: pages[1] },
        position: position(source.indexOf("display:") + 4),
      }),
      catalog,
    );
    assert.deepEqual(await request("completionItem/resolve", selected), resolved);
    const changed = path.join(workspace, fixture.spec.transition.target);
    fs.writeFileSync(changed, fixture.changedConfig);
    observed.push({
      kind: "configuration-write",
      path: fixture.spec.transition.target,
      source: fs.readFileSync(changed, "utf8"),
    });
    save();
    session.notify("workspace/didChangeWatchedFiles", {
      changes: [{ uri: pathToFileURL(changed).href, type: 2 }],
    });
    assert.deepEqual(await publish(pages[0]), { diagnostics: [], uri: pages[0], version: 1 });
    assert.equal(
      await request("textDocument/hover", {
        textDocument: { uri: pages[0] },
        position: position(source.indexOf("const stable") + "const st".length),
      }),
      null,
    );
    assert.deepEqual(await request("completionItem/resolve", fixture.localItem), fixture.localItem);
    assert.deepEqual(await request("completionItem/resolve", selected), resolved);
    const css = selected.data.vizeCss;
    const variants = [
      { ...selected, data: { ...selected.data, unknownOwner: { uri: pages[1] } } },
      { ...selected, data: { ...selected.data, unknownOwner: null } },
      { ...selected, data: { vizeCss: { ...css, unknownOwner: { uri: pages[1] } } } },
      ...[
        null,
        {},
        { kind: "property", name: "display" },
        { ...css, property: false },
        { ...css, property: "color" },
      ].map((vizeCss) => ({ ...selected, data: { vizeCss } })),
      { ...selected, label: "foreign" },
      { ...selected, kind: 3 },
      { ...selected, data: { ...selected.data, vizeCompletion: null } },
      { ...selected, data: { unknownOwner: { uri: pages[1] } } },
    ];
    for (const item of variants) {
      await assert.rejects(
        request("completionItem/resolve", item),
        (error: unknown) => error instanceof LspRequestError && error.code === -32801,
      );
      const response = raw.at(-1)!;
      assert.deepEqual(response, {
        jsonrpc: "2.0",
        id: response.id,
        error: { code: -32801, message: "Content modified" },
      });
    }
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
        assert.equal(candidates.length, 1);
        const capture = path.join(captures, candidates[0]);
        fs.cpSync(capture, path.join(output, "source-session"), { recursive: true });
        if (completed) {
          const requests = decodeFrames(
            fs.readFileSync(path.join(capture, "client.bin")),
          ).messages.filter(
            (packet) => typeof packet.id === "number" && typeof packet.method === "string",
          );
          assert.equal(requests.length, calls + 1);
          assert.equal(requests.at(-1)?.method, "shutdown");
          assert.deepEqual(
            raw.map(({ id }) => id),
            requests.map(({ id }) => id),
          );
        }
      }
    } finally {
      save();
      fs.rmSync(workspace, { recursive: true, force: true });
    }
  }
});
