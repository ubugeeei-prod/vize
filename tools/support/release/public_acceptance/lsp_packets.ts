import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { contained } from "./lsp_assets.ts";
import { corpus, sha256 } from "./lsp_source.ts";

import { assertWholeCase } from "./lsp_case.ts";

type Json = ReturnType<typeof JSON.parse>;
const files = ["Child.vue", "Parent.vue", "Other.vue"];
const reply = (id: number, result: Json) => ({ jsonrpc: "2.0", id, result });

/** Validate H's decoded ledger, which records clean pipe EOF as a reader event.
 * This ledger does not retain raw stdout frames or prove header byte completeness.
 */
export function assertDecodedLspLifecycle(protocol: Json[], terminal: Json) {
  assert.equal(terminal.success, true);
  assert.equal(terminal.exitCode, 0);
  assert.equal(terminal.stdoutReaderJoined, true);
  assert.equal(terminal.stderrReaderJoined, true);
  assert.ok(protocol.length >= 7, "complete decoded lifecycle required");
  assert.deepEqual(protocol.at(-1), {
    direction: "stdoutError",
    message: { kind: "UnexpectedEof", message: "LSP stdout reached EOF" },
  });
  assert.ok(
    protocol
      .slice(0, -1)
      .every((row) => row.direction === "request" || row.direction === "response"),
    "only the final clean EOF reader event can qualify",
  );
  const requests = protocol.flatMap((row, index) =>
    row.direction === "request" ? [{ message: row.message, index }] : [],
  );
  const once = (method: string) => {
    const entries = requests.filter((row) => row.message.method === method);
    assert.equal(entries.length, 1, `one ${method} required`);
    return entries[0];
  };
  const initialize = once("initialize"),
    initialized = once("initialized"),
    shutdown = once("shutdown"),
    exit = once("exit");
  assert.equal(initialize.index, 0);
  assert.ok(Number.isSafeInteger(initialize.message.id));
  assert.ok(Number.isSafeInteger(shutdown.message.id));
  assert.notEqual(initialize.message.id, shutdown.message.id);
  const response = (id: number) => {
    const rows = protocol.flatMap((row, index) =>
      row.direction === "response" && row.message.id === id
        ? [{ message: row.message, index }]
        : [],
    );
    assert.equal(rows.length, 1, "one lifecycle response required");
    return rows[0];
  };
  const ready = response(initialize.message.id),
    stopped = response(shutdown.message.id);
  assert.equal(initialize.message.jsonrpc, "2.0");
  assert.equal(ready.message.jsonrpc, "2.0");
  assert.ok(Object.hasOwn(ready.message, "result") && !Object.hasOwn(ready.message, "error"));
  assert.deepEqual(initialized.message, { jsonrpc: "2.0", method: "initialized", params: {} });
  assert.ok(initialize.index < ready.index && ready.index < initialized.index);
  assert.ok(initialized.index < shutdown.index && shutdown.index < stopped.index);
  assert.ok(stopped.index < exit.index);
  assert.equal(exit.index, protocol.length - 2, "EOF must immediately follow exit");
  assert.deepEqual(shutdown.message, {
    jsonrpc: "2.0",
    id: shutdown.message.id,
    method: "shutdown",
  });
  assert.deepEqual(stopped.message, reply(shutdown.message.id, null));
  assert.deepEqual(exit.message, { jsonrpc: "2.0", method: "exit" });
  assert.ok(requests.every((row) => row.index <= shutdown.index || row.index === exit.index));
}

export function assertProcess(
  packet: Json,
  source: Json,
  cli: Json,
  provider: Json,
  consumer: string,
  output: string,
  version: string,
) {
  const trace = contained(packet.rawTraceDir, path.join(output, "lsp-processes"));
  const load = (file: string) => JSON.parse(fs.readFileSync(path.join(trace, file), "utf8"));
  const process = load("process.json"),
    terminal = load("terminal.json");
  assert.equal(process.sourceSha, source.head);
  assert.equal(process.executable, cli.executable);
  assert.equal(process.executableSha256, cli.executableSha256);
  assert.equal(process.harnessPackageVersion, version);
  assert.deepEqual(process.arguments, ["lsp"]);
  assert.equal(process.versionProbe.success, true);
  assert.equal(process.versionProbe.exitCode, 0);
  assert.deepEqual(
    Buffer.from(process.versionProbe.stdout),
    Buffer.from(cli.probe.stdoutBase64, "base64"),
  );
  assert.deepEqual(
    Buffer.from(process.versionProbe.stderr),
    Buffer.from(cli.probe.stderrBase64, "base64"),
  );
  // The original TempDir has already been dropped after its full packet was captured.
  const project = path.resolve(process.cwd);
  assert.equal(
    path.dirname(project),
    fs.realpathSync(path.join(consumer, "target/vize-tests/tests")),
  );
  assert.match(path.basename(project), /^lsp-(named-model|bound-event)-[A-Za-z0-9]+$/u);
  const stderr = fs.readFileSync(path.join(trace, "stderr.bin"));
  assert.equal(stderr.length, terminal.stderrBytes);
  assert.equal(sha256(stderr), terminal.stderrSha256);
  const configuration = JSON.parse(packet.vizeConfig);
  assert.deepEqual(configuration, {
    experimentals: { patternedTemplate: false },
    typeChecker: {
      corsaPath: provider.executable,
      checkFallthroughAttrs: false,
      optionsApi: false,
    },
    lsp: { lint: false, typecheck: true, hover: true, crossFile: true },
  });
  assert.equal(
    sha256(fs.readFileSync(configuration.typeChecker.corsaPath)),
    provider.executableSha256,
  );
  assert.equal(packet.sourceSha, source.head);
  assert.equal(packet.cliBinary, cli.executable);
  assert.equal(packet.requireTsgo, "1");
  assert.equal(packet.disableTsgo, null);
  const protocol = fs
    .readFileSync(path.join(trace, "protocol.jsonl"), "utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  assertDecodedLspLifecycle(protocol, terminal);
  const responses = protocol
    .filter((row) => row.direction === "response")
    .map((row) => row.message);
  assert.deepEqual(
    responses.filter((message) => message.method === "textDocument/publishDiagnostics"),
    packet.actual.allDiagnosticPublications,
  );
  for (const observation of packet.actual.observations) {
    const replies = observation.replies
      ? Object.values(observation.replies)
      : [observation.referencesReply, observation.definitionReply, observation.renameReply];
    for (const expected of replies)
      assert.equal(
        responses.filter((message) => {
          try {
            assert.deepEqual(message, expected);
            return true;
          } catch {
            return false;
          }
        }).length,
        1,
        "whole reply present once in original decoded process stream",
      );
  }
  const requests = protocol.filter((row) => row.direction === "request").map((row) => row.message);
  const initialize = requests.find((message) => message.method === "initialize");
  assert.equal(initialize.params.rootUri, pathToFileURL(project).href);
  assert.deepEqual(requests.at(-1), { jsonrpc: "2.0", method: "exit" });
  const shutdown = requests.find((message) => message.method === "shutdown");
  assert.deepEqual(
    responses.find((message) => message.id === shutdown.id),
    reply(shutdown.id, null),
  );
  return {
    trace,
    process,
    terminal,
    protocolSha256: sha256(fs.readFileSync(path.join(trace, "protocol.jsonl"))),
    uris: Object.fromEntries(
      [...files, "NativeGuard.vue"].map((file) => [
        file,
        pathToFileURL(path.join(project, "src", file)).href,
      ]),
    ),
  };
}

export function assertAllLspPackets(
  preparation: Json,
  source: Json,
  cli: Json,
  provider: Json,
  consumer: string,
  output: string,
  version: string,
) {
  const results = [];
  const seenTraces = new Set<string>();
  for (const [directory, namespace, model, count] of [
    ["named-model", "named-model-transactions", true, 12],
    ["bound-emitter", "bound-event-casing-transactions", false, 20],
  ] as const) {
    const literal = JSON.parse(
      fs.readFileSync(
        path.join(preparation.sourceTree, corpus, directory, "cases.json.txt"),
        "utf8",
      ),
    );
    assert.equal(literal.cases.length, count);
    const capture = path.join(output, "lsp", namespace);
    const entries = fs.readdirSync(capture).sort();
    assert.equal(entries.length, count);
    const remaining = [...literal.cases];
    let cursors = 0;
    for (const file of entries) {
      assert.ok(file.endsWith(".json"));
      const bytes = fs.readFileSync(path.join(capture, file));
      const packet = JSON.parse(bytes.toString());
      const index = remaining.findIndex((oracle) => {
        try {
          assert.deepEqual(packet.oracle, oracle);
          return true;
        } catch {
          return false;
        }
      });
      assert.ok(index >= 0, "duplicate, missing or foreign literal context");
      const oracle = remaining.splice(index, 1)[0];
      assert.equal(
        packet.tsconfig,
        fs.readFileSync(
          path.join(
            preparation.sourceTree,
            "crates/vize/tests/fixtures/content_mapper_project/tsconfig.json",
          ),
          "utf8",
        ),
      );
      const process = assertProcess(packet, source, cli, provider, consumer, output, version);
      assert.ok(!seenTraces.has(process.trace));
      seenTraces.add(process.trace);
      const observations = assertWholeCase(packet, oracle, process.uris, model);
      cursors += observations.length;
      results.push({
        namespace,
        file,
        context: packet.context,
        sha256: sha256(bytes),
        cursors: observations.length,
        process,
      });
    }
    assert.equal(remaining.length, 0);
    assert.equal(cursors, model ? 224 : 660);
  }
  assert.equal(fs.readdirSync(path.join(output, "lsp-processes")).length, 32);
  return {
    contexts: 32,
    namedModelCursors: 224,
    namedModelResponses: 896,
    boundEventCursorTriples: 660,
    contractedBoundDefinitions: 660,
    observedUncontractedBoundDefinitions: 0,
    results,
    stockHost:
      "Unchanged stock Content Mapper null/refusal laws are not functional stock-host rename proof.",
  };
}
