// Observe the unchanged #9349 original against the installed public executable.
// Functional edit/disk/library oracles remain in H's original Rust test.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { contained } from "./lsp_assets.ts";
import { assertBuiltSupport, assertSupportGraph } from "./lsp_cargo.ts";
import { atomicDiagnosticContract } from "./lsp_case.ts";
import { assertDecodedLspLifecycle } from "./lsp_packets.ts";
import { atomicTarget, atomicTest, sha256, supportTargets } from "./lsp_source.ts";

type Json = ReturnType<typeof JSON.parse>;
const corpus = "tests/_fixtures/differential/lsp/rename-library-refusal/8010";
const compare = (left: string, right: string) => (left < right ? -1 : Number(left > right));
export const atomicReceiptFile = "lsp-atomic-original.json";
export const atomicCaptureDirectory = "lsp-atomic-processes";
export const atomicCommandName = "cargo-atomic-original";
export const atomicArguments = (adapter: string) => [
  "test",
  "--locked",
  "--message-format=json",
  "--manifest-path",
  path.join(adapter, "Cargo.toml"),
  "--test",
  atomicTarget,
  atomicTest,
  "--",
  "--exact",
  "--nocapture",
];
function bytes(file: string) {
  const stat = fs.lstatSync(file);
  assert.ok(stat.isFile() && !stat.isSymbolicLink(), "regular atomic evidence required");
  return fs.readFileSync(file);
}
const load = (file: string) => JSON.parse(bytes(file).toString());

/** Snapshot only inputs already selected by the original fixture's native resolver. */
export function atomicInputs(preparation: Json, cli: Json, provider: Json) {
  assert.equal(preparation.atomicSourceFiles.length, 45);
  assert.equal(new Set(preparation.atomicSourceFiles.map((entry: Json) => entry.file)).size, 45);
  const sourceFiles = preparation.atomicSourceFiles.map((entry: Json) => {
    const file = contained(path.join(preparation.sourceTree, entry.file), preparation.sourceTree);
    const digest = sha256(bytes(file));
    assert.equal(digest, entry.sha256, `atomic source changed: ${entry.file}`);
    return { file: entry.file, sha256: digest };
  });
  let ancestor = provider.executable;
  let library: string | undefined;
  for (;;) {
    library = [path.join(ancestor, "lib.dom.d.ts"), path.join(ancestor, "lib/lib.dom.d.ts")].find(
      (file) => fs.existsSync(file) && fs.statSync(file).isFile(),
    );
    if (library || path.dirname(ancestor) === ancestor) break;
    ancestor = path.dirname(ancestor);
  }
  assert.ok(library, "original native DOM library required");
  library = contained(library, provider.directory);
  assert.equal(sha256(bytes(cli.executable)), cli.executableSha256);
  assert.equal(sha256(bytes(provider.executable)), provider.executableSha256);
  return {
    sourceFiles,
    cli: { executable: cli.executable, sha256: cli.executableSha256 },
    provider: { executable: provider.executable, sha256: provider.executableSha256 },
    library: { file: library, sha256: sha256(bytes(library)) },
  };
}

/** Whole printed replies supplement the unchanged original assertions, not a new golden. */
export function atomicStdout(stdout: string) {
  const pass = `test ${atomicTest} ... ok`;
  const lines = stdout.split("\n");
  assert.equal(lines.filter((line) => line === pass).length, 1, "exact original must pass once");
  assert.equal(lines.filter((line) => line.startsWith("test result:")).length, 1);
  assert.ok(
    lines.some((line) =>
      /^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; \d+ filtered out; finished in .+s$/u.test(
        line,
      ),
    ),
    "one original pass and zero ignored required",
  );
  const sessions: { runtime: string; library: string; replies: Json[] }[] = [];
  for (const line of lines) {
    const runtime =
      /^actual native runtime\/library: ("(?:[^"\\]|\\.)*") ("(?:[^"\\]|\\.)*")$/u.exec(line);
    if (runtime)
      sessions.push({
        runtime: JSON.parse(runtime[1]),
        library: JSON.parse(runtime[2]),
        replies: [],
      });
    const response =
      /^original response src\/Toggle\.vue (textDocument\/(?:rename|references)): (.+)$/u.exec(
        line,
      );
    if (response) {
      assert.ok(sessions.length > 0, "runtime authority must precede original replies");
      sessions.at(-1)!.replies.push({ method: response[1], reply: JSON.parse(response[2]) });
    }
  }
  assert.equal(sessions.length, 4, "all four original runtime contexts required");
  for (const session of sessions) assert.equal(session.replies.length, 18);
  return sessions;
}

/** Validate the exact unsaved input lifecycle and bind all original printed replies to it. */
export function assertAtomicSession(
  protocol: Json[],
  terminal: Json,
  project: string,
  originalToggle: string,
  originalApp: string,
  printed: Json[],
) {
  const uri = (file: string) => pathToFileURL(path.join(project, "src", file)).href;
  assertDecodedLspLifecycle(
    protocol,
    terminal,
    atomicDiagnosticContract(
      Object.fromEntries(
        ["NativeGuard.vue", "Toggle.vue", "App.vue"].map((file) => [file, uri(file)]),
      ),
    ),
  );
  const requests = protocol.filter((row) => row.direction === "request").map((row) => row.message);
  const responses = protocol
    .filter((row) => row.direction === "response")
    .map((row) => row.message);
  const initialize = requests[0];
  assert.equal(initialize.params.rootUri, pathToFileURL(project).href);
  assert.deepEqual(initialize.params.initializationOptions, {
    lint: true,
    typecheck: true,
    hover: true,
    crossFile: true,
  });
  const opens = requests.filter((request) => request.method === "textDocument/didOpen");
  const toggle = opens.filter((request) => request.params.textDocument.uri === uri("Toggle.vue"));
  const parents = opens.filter((request) => request.params.textDocument.uri === uri("App.vue"));
  assert.equal(toggle.length, 1);
  assert.ok(parents.length <= 1);
  const source = toggle[0].params.textDocument.text;
  const crlf = source.includes("\r\n");
  const convert = (text: string) => (crlf ? text.replaceAll("\n", "\r\n") : text);
  assert.equal(source, convert(originalToggle), "whole unchanged original source required");
  assert.equal(toggle[0].params.textDocument.version, 1);
  if (parents.length) {
    assert.equal(parents[0].params.textDocument.text, convert(originalApp));
    assert.equal(parents[0].params.textDocument.version, 1);
  }
  assert.equal(opens.length, 2 + parents.length, "only native guard and original documents open");
  const changes = requests.filter(
    (request) =>
      request.method === "textDocument/didChange" &&
      request.params.textDocument.uri === uri("Toggle.vue"),
  );
  assert.deepEqual(
    changes.map((request) => request.params),
    [
      {
        textDocument: { uri: uri("Toggle.vue"), version: 2 },
        contentChanges: [{ text: `<!-- 😀 -->${crlf ? "\r\n" : "\n"}${source}` }],
      },
      { textDocument: { uri: uri("Toggle.vue"), version: 3 }, contentChanges: [{ text: source }] },
    ],
  );
  assert.ok(
    !requests.some(
      (request) =>
        request.method === "textDocument/didSave" || request.method.startsWith("workspace/"),
    ),
    "original transaction never writes files",
  );
  const semantic = requests.filter(
    (request) =>
      request.method === "textDocument/rename" || request.method === "textDocument/references",
  );
  assert.equal(semantic.length, 18);
  assert.equal(printed.length, 18);
  assert.equal(requests.filter((request) => Object.hasOwn(request, "id")).length, 20);
  let version = 1;
  let observed = 0;
  for (const request of requests) {
    if (
      request.method === "textDocument/didChange" &&
      request.params.textDocument.uri === uri("Toggle.vue")
    )
      version = request.params.textDocument.version;
    if (!semantic.includes(request)) continue;
    const index = observed++;
    assert.equal(version, Math.floor(index / 6) + 1, "query must follow its unsaved revision");
    assert.equal(request.id, index + 2);
    assert.equal(request.jsonrpc, "2.0");
    assert.equal(request.params.textDocument.uri, uri("Toggle.vue"));
    const expectedMethod = index % 6 === 1 ? "textDocument/references" : "textDocument/rename";
    assert.equal(request.method, expectedMethod);
    if (index % 6 < 5)
      assert.deepEqual(request.params.position, { line: version === 2 ? 7 : 6, character: 9 });
    if (request.method === "textDocument/references")
      assert.deepEqual(request.params.context, { includeDeclaration: true });
    else
      assert.equal(
        request.params.newName,
        ["update", undefined, 'up"date', "up'date", "up\\date", "flipNext"][index % 6],
      );
    const reply = responses.filter((message) => message.id === request.id);
    assert.equal(reply.length, 1);
    assert.deepEqual(reply[0], { jsonrpc: "2.0", id: request.id, result: reply[0].result });
    assert.deepEqual(printed[index], { method: request.method, reply: reply[0] });
    if (index % 6 >= 2 && index % 6 <= 4) assert.equal(reply[0].result, null);
    else if (request.method === "textDocument/references") {
      assert.equal(reply[0].result.length, 3);
      assert.deepEqual(
        reply[0].result.map((site: Json) => site.uri).sort(compare),
        [uri("App.vue"), uri("Toggle.vue"), uri("Toggle.vue")].sort(compare),
      );
    } else {
      const edits = reply[0].result.changes;
      const binding = index % 6 === 5;
      assert.deepEqual(
        Object.keys(edits).sort(compare),
        binding ? [uri("Toggle.vue")] : [uri("App.vue"), uri("Toggle.vue")].sort(compare),
      );
      const all: Json[] = Object.values(edits).flat();
      assert.equal(all.length, binding ? 2 : 3);
      assert.ok(all.every((edit) => edit.newText === (binding ? "flipNext" : "update")));
    }
  }
  return {
    crlf,
    parentOpen: parents.length === 1,
    atomicRenames: 3,
    references: 3,
    unsafeRefusals: 9,
    bindingRenames: 3,
  };
}

/** Recomputed at run completion and again at the final public acceptance seal. */
export function observeAtomicOriginal(
  preparation: Json,
  source: Json,
  cli: Json,
  provider: Json,
  consumer: string,
  output: string,
  version: string,
  cargo: Json,
) {
  const inputs = atomicInputs(preparation, cli, provider);
  assert.deepEqual(
    load(path.join(output, "lsp-atomic-inputs.json")),
    inputs,
    "atomic input/library bytes changed during installed execution",
  );
  const command = load(path.join(output, `${atomicCommandName}.json`));
  const stdout = bytes(path.join(output, `${atomicCommandName}.stdout`));
  const stderr = bytes(path.join(output, `${atomicCommandName}.stderr`));
  assert.equal(command.executable, "cargo");
  assert.equal(command.cwd, preparation.adapter);
  assert.deepEqual(command.args, atomicArguments(preparation.adapter));
  assert.equal(command.status, 0);
  assert.equal(command.signal, null);
  assert.equal(command.error, undefined);
  assert.equal(command.stdoutSha256, sha256(stdout));
  assert.equal(command.stderrSha256, sha256(stderr));
  assert.equal(sha256(bytes(path.join(preparation.adapter, "Cargo.lock"))), cargo.lockSha256);
  const metadata = bytes(path.join(output, "cargo-metadata.stdout"));
  assert.equal(sha256(metadata), cargo.metadataSha256);
  const graph = JSON.parse(metadata.toString());
  const allowed = assertSupportGraph(
    graph,
    preparation.sourceTree,
    path.join(preparation.sourceTree, "Cargo.lock"),
    supportTargets,
  );
  assert.deepEqual(allowed, cargo.allowedBuildIds);
  const built = assertBuiltSupport(stdout.toString(), allowed);
  const testArtifacts = built.filter(
    (entry) => entry.target?.name === atomicTarget && entry.target.kind.includes("test"),
  );
  assert.equal(testArtifacts.length, 1, "actual original adapter target must be built");
  assert.equal(testArtifacts[0].packageId, graph.resolve.root);
  const printed = atomicStdout(stdout.toString());
  const capture = path.join(output, atomicCaptureDirectory);
  const entries = fs.readdirSync(capture).sort();
  assert.equal(entries.length, 4, "exactly four separate original processes required");
  const original = (file: string) =>
    bytes(path.join(preparation.sourceTree, corpus, file)).toString();
  const contexts = new Set<string>();
  const results = entries.map((entry) => {
    assert.match(entry, /^process-\d+$/u);
    const directory = contained(path.join(capture, entry), capture);
    assert.ok(fs.lstatSync(path.join(capture, entry)).isDirectory());
    const process = load(path.join(directory, "process.json"));
    const terminal = load(path.join(directory, "terminal.json"));
    assert.equal(entry, `process-${process.pid}`);
    assert.equal(process.sourceSha, source.head);
    assert.equal(process.testName, atomicTest);
    assert.equal(process.executable, cli.executable);
    assert.equal(process.executableSha256, cli.executableSha256);
    assert.equal(process.harnessPackageVersion, version);
    assert.equal(process.requiredCorsa, true);
    assert.equal(process.disabledNative, false);
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
    assert.equal(
      path.dirname(process.cwd),
      fs.realpathSync(path.join(consumer, "target/vize-tests/tests")),
    );
    assert.match(path.basename(process.cwd), /^lsp-rename-authority-[A-Za-z0-9]+$/u);
    const protocolBytes = bytes(path.join(directory, "protocol.jsonl"));
    const protocol = protocolBytes
      .toString()
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line));
    const stderrBytes = bytes(path.join(directory, "stderr.bin"));
    assert.equal(stderrBytes.length, terminal.stderrBytes);
    assert.equal(sha256(stderrBytes), terminal.stderrSha256);
    const toggleUri = pathToFileURL(path.join(process.cwd, "src/Toggle.vue")).href;
    const matched = printed.filter(
      (session) => session.replies[0].reply.result?.changes?.[toggleUri],
    );
    assert.equal(matched.length, 1, "one whole printed original session must join each process");
    assert.equal(matched[0].runtime, provider.executable);
    assert.equal(fs.realpathSync(matched[0].library), inputs.library.file);
    const observed = assertAtomicSession(
      protocol,
      terminal,
      process.cwd,
      original("Toggle.vue.txt"),
      original("App.vue.txt"),
      matched[0].replies,
    );
    const key = JSON.stringify([observed.crlf, observed.parentOpen]);
    assert.ok(!contexts.has(key), "duplicate or missing original context");
    contexts.add(key);
    return {
      trace: directory,
      process,
      terminal,
      protocolSha256: sha256(protocolBytes),
      ...observed,
    };
  });
  assert.equal(contexts.size, 4);
  return {
    schema: "vize-public-atomic-original-v1",
    source,
    version,
    target: atomicTarget,
    test: atomicTest,
    inputs,
    command,
    built,
    contexts: 4,
    atomicRenames: 12,
    references: 12,
    unsafeRefusals: 36,
    bindingRenames: 12,
    results,
    success: true,
    scope:
      "Unchanged original Rust edit/disk/library assertions plus decoded lifecycle custody; no raw framing completeness claim.",
  };
}

export function assertAtomicOriginalReceipt(...args: Parameters<typeof observeAtomicOriginal>) {
  const stored = load(path.join(args[5], atomicReceiptFile));
  assert.deepEqual(
    stored,
    observeAtomicOriginal(...args),
    "separate atomic-original receipt is mandatory at the final seal",
  );
}
