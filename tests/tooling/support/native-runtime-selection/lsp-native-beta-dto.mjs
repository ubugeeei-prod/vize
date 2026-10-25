import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  decodeNativeFrames,
  decodeNativeIo,
  decodeStraceString,
  parseStraceCapture,
} from "./lsp-strace.mjs";
import { readStraceLines } from "./lsp-strace-lines.mjs";
import { sha256 } from "./providers.mjs";

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
const IMAGE = "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70";
const metadata = new Set([
  "initialize",
  "createSnapshot",
  "deriveSnapshot",
  "updateSnapshot",
  "release",
  "parseConfigFile",
  "getDefaultProjectForFile",
  "describeCapabilities",
]);
const packet = (frames) => frames.map((frame) => frame.packet);

/** Exact known SDK pair; no generic beta or 702 lifecycle admission. */
export function validateNativeBetaDtoLifecycle(owners, consumer) {
  assert.equal(consumer.kind, "selected-config");
  assert.equal(consumer.expectedExecutable.sha256, IMAGE);
  const source = fs.readFileSync(
    path.join(repo, "crates/vize_canon/src/lsp_client/editor_lsp/native_shutdown/beta_contract.rs"),
    "utf8",
  );
  const matched =
    /pub\(super\) fn initialize\(\) -> Value \{\s*serde_json::json!\(([\s\S]*?)\)\s*\}/u.exec(
      source,
    );
  assert(matched, "exact complete beta source-owned Initialize authority");
  const expected = JSON.parse(matched[1]);
  assert.equal(expected.serverInfo.version, "7.1.0-beta");
  assert(owners.length > 0);
  let nativeError = false,
    unsavedErrorText = false;
  for (const owner of owners) {
    assert.equal(owner.executable.physicalPath, consumer.expectedExecutable.path);
    assert.equal(owner.executable.sha256, IMAGE);
    assert.deepEqual(owner.argv, [owner.executable.path, "--lsp", "--stdio"]);
    assert.deepEqual(owner.exit, { code: 0, signal: null });
    assert.equal(fs.readFileSync(owner.streams.stderr.path).length, 0);
    const client = packet(owner.streams.client.frames),
      server = packet(owner.streams.server.frames);
    unsavedErrorText ||= client.some((row) =>
      row.method === "textDocument/didOpen"
        ? row.params?.textDocument?.text?.includes('"wrong"')
        : row.method === "textDocument/didChange" &&
          row.params?.contentChanges?.some((change) => change.text?.includes('"wrong"')),
    );
    nativeError ||= server.some((row) => row.result?.items?.some((item) => item.code === 2322));
    const init = client.filter((row) => row.method === "initialize");
    assert.equal(init.length, 1);
    assert.deepEqual(
      server.find((row) => row.id === init[0].id && !row.method),
      { jsonrpc: "2.0", id: init[0].id, result: expected },
    );
    const close = client.filter((row) => row.method === "shutdown");
    assert.equal(close.length, 1);
    assert.deepEqual(
      server.find((row) => row.id === close[0].id && !row.method),
      { jsonrpc: "2.0", id: close[0].id, result: null },
    );
    assert(
      server.some(
        (row) =>
          row.method === "window/logMessage" &&
          row.params?.type === 3 &&
          row.params.message.startsWith(`handled method 'shutdown' (${close[0].id}) in `),
      ),
    );
    assert.equal(
      client.some((row) => row.method === "exit"),
      false,
    );
  }
  assert(unsavedErrorText, "actual selected native receives unsaved error bytes");
  assert(nativeError, "actual selected native supplies genuine semantic2322");
  const api = captureRetainedApi(owners);
  return {
    selectedProvider: true,
    nativeCleanExitQualified: true,
    semanticCredit: false,
    scope:
      "Exact A183 source Maestro selected-config packets and retained attached DerivedSnapshots refusal; API configuration metadata is independently owned and provides no LS overlay-view credit.",
    betaContractSourceSha256: sha256(Buffer.from(source)),
    retainedApi: api,
  };
}

/** Decode only observed accepted API socket lifetimes, never unrelated FD reuse. */
function captureRetainedApi(owners) {
  const directory = path.dirname(owners[0].streams.client.path);
  const parent = path.dirname(directory),
    prefix = "syscalls.";
  const capture = parseStraceCapture(
    fs
      .readdirSync(parent)
      .filter((name) => name.startsWith(prefix))
      .map((name) => ({
        tid: Number(name.slice(prefix.length)),
        lines: readStraceLines(path.join(parent, name)),
      })),
  );
  const result = [];
  for (const observed of owners) {
    const raw = capture.owners.find((owner) => owner.process === observed.process);
    assert(raw, "same actual process owns API socket bytes");
    const stdio = decodeNativeIo({ ...raw });
    assert(
      stdio.events.some((row) => row.operation === "read" && row.fd === 0 && row.count === 0),
      "actual native stdin EOF read(0)=0; terminal unknown bytes cannot substitute",
    );
    const requests = packet(observed.streams.client.frames),
      responses = packet(observed.streams.server.frames);
    for (const attach of requests.filter((row) => row.method === "custom/initializeAPISession")) {
      const response = responses.find((row) => row.id === attach.id && !row.method);
      assert.equal(response.error, undefined);
      const socket = response.result.pipe;
      assert.equal(typeof socket, "string");
      const binds = raw.rows
        .filter(
          (row) =>
            row.call.startsWith("bind(") &&
            row.call.includes("sun_path=") &&
            row.call.match(/\)\s+=\s+0\b/u),
        )
        .filter((row) => {
          const encoded = /sun_path=("(?:\\x[0-9a-fA-F]{2})*")/u.exec(row.call);
          return encoded && decodeStraceString(encoded[1]).toString("utf8") === socket;
        });
      assert.equal(binds.length, 1, "one actual listener binds the returned API path");
      const listener = Number(/^bind\((\d+),/u.exec(binds[0].call)[1]);
      const accepts = raw.rows.filter(
        (row) =>
          new RegExp(`^accept4?\\(${listener},`, "u").test(row.call) &&
          /\)\s+=\s+\d+\b/u.test(row.call),
      );
      assert.equal(accepts.length, 1, "one accepted attachment per returned socket");
      const accepted = accepts[0],
        fd = Number(/\)\s+=\s+(\d+)\b/u.exec(accepted.call)[1]);
      const closes = raw.rows.filter(
        (row) =>
          new RegExp(`^close\\(${fd}\\)\\s+=\\s+0\\b`, "u").test(row.call) &&
          BigInt(row.interval.startNanoseconds) > BigInt(accepted.interval.endNanoseconds),
      );
      assert(closes.length > 0, "actual accepted socket close must be observed");
      closes.sort((a, b) =>
        BigInt(a.interval.startNanoseconds) < BigInt(b.interval.startNanoseconds) ? -1 : 1,
      );
      const closed = closes[0];
      const rows = raw.rows.filter(
        (row) =>
          BigInt(row.interval.startNanoseconds) > BigInt(accepted.interval.endNanoseconds) &&
          BigInt(row.interval.endNanoseconds) < BigInt(closed.interval.startNanoseconds),
      );
      for (const row of rows) {
        const alias = /^(dup|dup2|dup3|fcntl)\((\d+)(?:,\s*([^,)]*))?/u.exec(row.call);
        if (alias) {
          const source = Number(alias[2]);
          const target = /^(dup2|dup3)$/u.test(alias[1]) ? Number(alias[3]) : null;
          assert(
            !(source === fd && (alias[1] !== "fcntl" || /F_DUPFD/u.test(alias[3]))) &&
              target !== fd,
            "API socket descriptor aliasing remains unsupported",
          );
        }
        assert(
          !row.call.startsWith("close_range("),
          "range closure could invalidate accepted API lifetime",
        );
        assert(
          !row.call.includes("SCM_RIGHTS"),
          "descriptor passing is outside accepted API lifetime coverage",
        );
        const allocation =
          /^(?:openat|socket|accept|accept4|dup|dup2|dup3)\(.*\)\s+=\s+(\d+)\b/u.exec(row.call);
        assert(
          !allocation || Number(allocation[1]) !== fd,
          "accepted API descriptor must not be reused before its observed close",
        );
      }
      const decoded = decodeNativeIo({ ...raw, rows }, [fd]);
      const stream = (operation) =>
        Buffer.concat(
          decoded.events
            .filter((row) => row.operation === operation && row.bytes)
            .map((row) => row.bytes),
        );
      const client = stream("read"),
        server = stream("write");
      const input = packet(decodeNativeFrames(client)),
        output = packet(decodeNativeFrames(server));
      for (const [requests, replies] of [
        [input, output],
        [output, input],
      ]) {
        const ids = requests
          .filter((row) => row.id !== undefined && row.method)
          .map((row) => `${typeof row.id}:${row.id}`);
        const answers = replies
          .filter((row) => row.id !== undefined && !row.method)
          .map((row) => `${typeof row.id}:${row.id}`);
        assert.equal(new Set(ids).size, ids.length);
        assert.deepEqual(
          ids.sort(),
          answers.sort(),
          "complete bidirectional API request/reply identity",
        );
      }
      const calls = input.filter((row) => row.id !== undefined && row.method);
      assert(calls.length > 0, "actual API handshake and owned metadata are observed");
      for (const call of calls) {
        assert(
          metadata.has(call.method),
          `attached semantic API query is forbidden: ${call.method}`,
        );
        assert.equal(
          output.filter((row) => row.id === call.id && !row.method).length,
          1,
          "whole matched API response",
        );
      }
      const init = calls.find((row) => row.method === "initialize");
      assert(init);
      assert.equal(
        output.find((row) => row.id === init.id && !row.method).result.caseSensitivity,
        1,
        "actual DerivedSnapshots handshake",
      );
      const stem = `${observed.process}-api-${fd}`;
      fs.writeFileSync(path.join(directory, `${stem}.client.bin`), client);
      fs.writeFileSync(path.join(directory, `${stem}.server.bin`), server);
      result.push({
        process: observed.process,
        session: response.result,
        fd,
        accepted,
        closed,
        methods: calls.map((row) => row.method),
        client: { sha256: sha256(client), bytes: client.length, packets: input },
        server: { sha256: sha256(server), bytes: server.length, packets: output },
        semanticApiQueries: 0,
      });
    }
  }
  assert(
    result.length > 0,
    "real workspace source must retain an actual same-owner API attachment",
  );
  fs.writeFileSync(
    path.join(directory, "retained-api-10731.json"),
    JSON.stringify(result, null, 2) + "\n",
  );
  return result;
}

/** Unsaved dependency reference offsets change; exact authored geometry matters. */
export async function observeBetaNavigation(wire, consumer, _campaign, source) {
  const uri = pathToFileURL(path.join(consumer.configRoot, "src/support.ts")).href;
  const declaration = source.indexOf("value: number"),
    use = source.lastIndexOf("value");
  const location = (offset) => ({
    uri,
    range: { start: { line: 0, character: offset }, end: { line: 0, character: offset + 5 } },
  });
  const request = { textDocument: { uri }, position: { line: 0, character: declaration + 1 } };
  const definition = await wire.request("textDocument/definition", request);
  assert.deepEqual(definition, {
    jsonrpc: "2.0",
    id: definition.id,
    result: location(declaration),
  });
  const references = await wire.request("textDocument/references", {
    ...request,
    context: { includeDeclaration: true },
  });
  assert.deepEqual(references, {
    jsonrpc: "2.0",
    id: references.id,
    result: [location(declaration), location(use)],
  });
  return { dependencyText: source, definition, references };
}
