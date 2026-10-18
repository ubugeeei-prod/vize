import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { test } from "node:test";
import {
  atomicArguments,
  atomicCaptureDirectory,
  atomicCommandName,
  atomicInputs,
  atomicReceiptFile,
  atomicStdout,
  assertAtomicOriginalReceipt,
  observeAtomicOriginal,
} from "../../../tools/support/release/public_acceptance/lsp_atomic.ts";
import {
  atomicTarget,
  atomicTest,
  readLspSourceFiles,
  sha256,
  supportTargets,
} from "../../../tools/support/release/public_acceptance/lsp_source.ts";

type Json = ReturnType<typeof JSON.parse>;
const root = fileURLToPath(new URL("../../../", import.meta.url));

// Inert custody envelopes exercise refusal paths. They do not execute the Rust
// oracle or any product and cannot establish installed/public acceptance.
function fixture() {
  const directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-atomic-custody-")));
  const consumer = path.join(directory, "consumer"),
    output = path.join(directory, "output");
  const sourceTree = path.join(directory, "source"),
    adapter = path.join(consumer, "crates/vize");
  const cases = path.join(consumer, "target/vize-tests/tests");
  for (const dir of [output, sourceTree, adapter, cases]) fs.mkdirSync(dir, { recursive: true });
  const write = (file: string, value: string | Buffer) => {
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, value);
  };
  const save = (file: string, value: unknown) => write(file, JSON.stringify(value));
  const files = readLspSourceFiles(root, "HEAD").atomicFiles;
  const atomicSourceFiles = files.map((file) => {
    const bytes = fs.readFileSync(path.join(root, file));
    write(path.join(sourceTree, file), bytes);
    return { file, sha256: sha256(bytes) };
  });
  write(path.join(sourceTree, "Cargo.lock"), fs.readFileSync(path.join(root, "Cargo.lock")));
  write(path.join(adapter, "Cargo.lock"), "inert adapter lock\n");
  const preparation = { sourceTree, adapter, atomicSourceFiles };
  const source = { head: "inert-H" },
    version = "0.999.0";
  const executable = path.join(consumer, "cli");
  write(executable, "inert public CLI identity");
  const cli = {
    executable,
    executableSha256: sha256(fs.readFileSync(executable)),
    probe: { stdoutBase64: Buffer.from(`vize ${version}\n`).toString("base64"), stderrBase64: "" },
  };
  const native = path.join(consumer, "provider/lib/tsc");
  write(native, "inert provider identity");
  write(path.join(path.dirname(native), "lib.dom.d.ts"), "inert library bytes");
  const provider = {
    directory: path.join(consumer, "provider"),
    executable: native,
    executableSha256: sha256(fs.readFileSync(native)),
  };
  const metadata: Json = {
    packages: [
      {
        id: "adapter",
        name: "vize_public_lsp_contracts",
        source: null,
        version,
        manifest_path: path.join(adapter, "Cargo.toml"),
        targets: supportTargets.map((name) => ({ name, kind: ["test"] })),
      },
    ],
    resolve: { root: "adapter", nodes: [{ id: "adapter", deps: [] }] },
  };
  const cargo: Json = {
    allowedBuildIds: ["adapter"],
    lockSha256: sha256(fs.readFileSync(path.join(adapter, "Cargo.lock"))),
  };
  const artifact = {
    reason: "compiler-artifact",
    package_id: "adapter",
    target: { name: atomicTarget, kind: ["test"] },
    filenames: ["inert-test"],
  };
  const corpus = path.join(
    sourceTree,
    "tests/_fixtures/differential/lsp/rename-library-refusal/8010",
  );
  const originalToggle = fs.readFileSync(path.join(corpus, "Toggle.vue.txt"), "utf8");
  const originalApp = fs.readFileSync(path.join(corpus, "App.vue.txt"), "utf8");
  const sessions: Json[] = [];
  for (const crlf of [false, true])
    for (const parentOpen of [false, true]) {
      const pid = 100 + sessions.length,
        project = path.join(cases, `lsp-rename-authority-${pid}`);
      const capture = path.join(output, atomicCaptureDirectory, `process-${pid}`);
      const uri = (file: string) => pathToFileURL(path.join(project, "src", file)).href;
      const convert = (text: string) => (crlf ? text.replaceAll("\n", "\r\n") : text);
      const toggle = convert(originalToggle),
        protocol: Json[] = [],
        printed: Json[] = [];
      const request = (message: Json) => protocol.push({ direction: "request", message });
      const response = (id: number, result: Json) => {
        const message = { jsonrpc: "2.0", id, result };
        protocol.push({ direction: "response", message });
        return message;
      };
      const open = (file: string, text: string) =>
        request({
          jsonrpc: "2.0",
          method: "textDocument/didOpen",
          params: { textDocument: { uri: uri(file), version: 1, text } },
        });
      const change = (file: string, text: string, version: number) =>
        request({
          jsonrpc: "2.0",
          method: "textDocument/didChange",
          params: { textDocument: { uri: uri(file), version }, contentChanges: [{ text }] },
        });
      request({
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: {
          rootUri: pathToFileURL(project).href,
          initializationOptions: { lint: true, typecheck: true, hover: true, crossFile: true },
        },
      });
      response(1, {});
      request({ jsonrpc: "2.0", method: "initialized", params: {} });
      open("NativeGuard.vue", "inert guard");
      change("NativeGuard.vue", "inert repair", 2);
      open("Toggle.vue", toggle);
      if (parentOpen) open("App.vue", convert(originalApp));
      for (const version of [1, 2, 3]) {
        if (version > 1)
          change(
            "Toggle.vue",
            version === 2 ? `<!-- 😀 -->${crlf ? "\r\n" : "\n"}${toggle}` : toggle,
            version,
          );
        for (let index = 0; index < 6; index++) {
          const id = 2 + (version - 1) * 6 + index;
          const method = index === 1 ? "textDocument/references" : "textDocument/rename";
          request({
            jsonrpc: "2.0",
            id,
            method,
            params: {
              textDocument: { uri: uri("Toggle.vue") },
              position: { line: version === 2 ? 7 : 6, character: 9 },
              ...(index === 1
                ? { context: { includeDeclaration: true } }
                : {
                    newName: ["update", undefined, 'up"date', "up'date", "up\\date", "flipNext"][
                      index
                    ],
                  }),
            },
          });
          const edit = {
            range: { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } },
            newText: index === 5 ? "flipNext" : "update",
          };
          const sites = ["App.vue", "Toggle.vue", "Toggle.vue"].map((file) => ({
            uri: uri(file),
            range: edit.range,
          }));
          const result =
            index === 1
              ? sites
              : index >= 2 && index <= 4
                ? null
                : {
                    changes: {
                      ...(index === 5 ? {} : { [uri("App.vue")]: [edit] }),
                      [uri("Toggle.vue")]: [edit, edit],
                    },
                  };
          printed.push({ method, reply: response(id, result) });
        }
      }
      request({ jsonrpc: "2.0", id: 20, method: "shutdown" });
      response(20, null);
      request({ jsonrpc: "2.0", method: "exit" });
      const process = {
        pid,
        sourceSha: source.head,
        testName: atomicTest,
        executable,
        executableSha256: cli.executableSha256,
        harnessPackageVersion: version,
        requiredCorsa: true,
        disabledNative: false,
        arguments: ["lsp"],
        cwd: project,
        versionProbe: {
          success: true,
          exitCode: 0,
          stdout: [...Buffer.from(`vize ${version}\n`)],
          stderr: [],
        },
      };
      const terminal = {
        success: true,
        exitCode: 0,
        stdoutReaderJoined: true,
        stderrReaderJoined: true,
        stderrBytes: 0,
        stderrSha256: sha256(Buffer.alloc(0)),
      };
      sessions.push({
        capture,
        process,
        terminal,
        protocol,
        printed,
        runtime: native,
        library: path.join(path.dirname(native), "lib.dom.d.ts"),
      });
    }
  let stdout = "";
  function flush() {
    save(path.join(output, "cargo-metadata.stdout"), metadata);
    cargo.metadataSha256 = sha256(fs.readFileSync(path.join(output, "cargo-metadata.stdout")));
    for (const session of sessions) {
      save(path.join(session.capture, "process.json"), session.process);
      save(path.join(session.capture, "terminal.json"), session.terminal);
      write(path.join(session.capture, "stderr.bin"), "");
      write(
        path.join(session.capture, "protocol.jsonl"),
        session.protocol.map(JSON.stringify).join("\n") + "\n",
      );
    }
    stdout =
      [
        JSON.stringify(artifact),
        JSON.stringify({ reason: "build-finished", success: true }),
        ...sessions.flatMap((session) => [
          `actual native runtime/library: ${JSON.stringify(session.runtime)} ${JSON.stringify(session.library)}`,
          ...session.printed.map(
            (row: Json) =>
              `original response src/Toggle.vue ${row.method}: ${JSON.stringify(row.reply)}`,
          ),
        ]),
        `test ${atomicTest} ... ok`,
        "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.01s",
      ].join("\n") + "\n";
    rewriteStdout(stdout);
  }
  function rewriteStdout(text: string) {
    write(path.join(output, `${atomicCommandName}.stdout`), text);
    write(path.join(output, `${atomicCommandName}.stderr`), "");
    save(path.join(output, `${atomicCommandName}.json`), {
      executable: "cargo",
      cwd: adapter,
      args: atomicArguments(adapter),
      status: 0,
      signal: null,
      stdoutSha256: sha256(Buffer.from(text)),
      stderrSha256: sha256(Buffer.alloc(0)),
    });
  }
  const args = [preparation, source, cli, provider, consumer, output, version, cargo] as const;
  save(path.join(output, "lsp-atomic-inputs.json"), atomicInputs(preparation, cli, provider));
  flush();
  return {
    directory,
    output,
    preparation,
    metadata,
    sessions,
    artifact,
    args,
    flush,
    rewriteStdout,
    stdout: () => stdout,
    observe: () => observeAtomicOriginal(...args),
    clean: () => fs.rmSync(directory, { recursive: true, force: true }),
  };
}

test("inert observer envelopes retain all four input identities and whole joined lifecycle without mutation", (t) => {
  const f = fixture();
  t.after(f.clean);
  const before = structuredClone(f.sessions);
  const result = f.observe();
  assert.deepEqual(f.sessions, before);
  assert.deepEqual(
    [
      result.contexts,
      result.atomicRenames,
      result.references,
      result.unsafeRefusals,
      result.bindingRenames,
    ],
    [4, 12, 12, 36, 12],
  );
  fs.writeFileSync(path.join(f.output, atomicReceiptFile), JSON.stringify(result));
  assert.doesNotThrow(() => assertAtomicOriginalReceipt(...f.args));
});

test("the separate original receipt is mandatory and cannot borrow the successful original32 receipt", (t) => {
  const f = fixture();
  t.after(f.clean);
  fs.writeFileSync(
    path.join(f.output, "lsp.json"),
    JSON.stringify({ success: true, packets: { contexts: 32 } }),
  );
  assert.throws(() => assertAtomicOriginalReceipt(...f.args), /ENOENT/u);
  fs.writeFileSync(
    path.join(f.output, atomicReceiptFile),
    JSON.stringify({ ...f.observe(), contexts: 32 }),
  );
  assert.throws(() => assertAtomicOriginalReceipt(...f.args), /mandatory/u);
});

test("a build-script row survives the atomic receipt JSON seal without losing artifact custody", (t) => {
  const f = fixture();
  t.after(f.clean);
  // Cargo build-script messages do not carry compiler-artifact target/filenames.
  f.rewriteStdout(
    JSON.stringify({ reason: "build-script-executed", package_id: "adapter" }) + "\n" + f.stdout(),
  );
  const observed = f.observe();
  assert.deepEqual(observed.built, [
    { packageId: "adapter" },
    { packageId: "adapter", target: f.artifact.target, filenames: f.artifact.filenames },
  ]);
  const receipt = path.join(f.output, atomicReceiptFile);
  fs.writeFileSync(receipt, JSON.stringify(observed));
  assert.deepEqual(JSON.parse(fs.readFileSync(receipt, "utf8")), f.observe());
  assert.doesNotThrow(() => assertAtomicOriginalReceipt(...f.args));
  const tampered = JSON.parse(fs.readFileSync(receipt, "utf8"));
  tampered.built[1].filenames = ["foreign-test"];
  fs.writeFileSync(receipt, JSON.stringify(tampered));
  assert.throws(() => assertAtomicOriginalReceipt(...f.args), /receipt is mandatory/u);
});

const corruptions: Record<string, (f: ReturnType<typeof fixture>) => void> = {
  "wrong executable identity": (f) => {
    f.sessions[0].process.executableSha256 = "wrong";
    f.flush();
  },
  "wrong test identity": (f) => {
    f.sessions[0].process.testName = "other";
    f.flush();
  },
  "wrong source identity": (f) => {
    f.sessions[0].process.sourceSha = "other-H";
    f.flush();
  },
  "wrong provider authority": (f) => {
    f.sessions[0].runtime = "/foreign/tsc";
    f.flush();
  },
  "missing context": (f) => {
    const row = f.sessions.pop();
    fs.rmSync(row.capture, { recursive: true });
    f.flush();
  },
  "extra process capture": (f) => {
    fs.mkdirSync(path.join(f.output, atomicCaptureDirectory, "process-999"));
  },
  "duplicate context": (f) => {
    Object.assign(
      f.sessions[1],
      structuredClone({ ...f.sessions[0], capture: f.sessions[1].capture }),
    );
    f.sessions[1].process.pid = 101;
    f.flush();
  },
  "missing clean terminal": (f) => {
    f.sessions[0].terminal.stdoutReaderJoined = false;
    f.flush();
  },
  "legacy reader error": (f) => {
    f.sessions[0].protocol.push({ direction: "stdoutError", message: { kind: "UnexpectedEof" } });
    f.flush();
  },
  "changed unsaved Unicode bytes": (f) => {
    f.sessions[0].protocol.find(
      (row: Json) =>
        row.message.params?.textDocument?.version === 2 &&
        row.message.params.textDocument.uri.endsWith("Toggle.vue"),
    ).message.params.contentChanges[0].text = "changed";
    f.flush();
  },
  "unsafe partial transaction": (f) => {
    f.sessions[0].printed[2].reply.result = { changes: {} };
    f.flush();
  },
  "null atomic rename": (f) => {
    f.sessions[0].printed[0].reply.result = null;
    f.flush();
  },
  "missing binding site": (f) => {
    Object.values<Json[]>(f.sessions[0].printed[5].reply.result.changes)[0].pop();
    f.flush();
  },
  "extra foreign reference": (f) => {
    f.sessions[0].printed[1].reply.result.push({ uri: "file:///foreign" });
    f.flush();
  },
  "source file replacement": (f) => {
    fs.appendFileSync(
      path.join(f.preparation.sourceTree, f.preparation.atomicSourceFiles[0].file),
      "changed",
    );
  },
  "library bytes replaced": (f) => {
    fs.appendFileSync(f.sessions[0].library, "changed");
  },
  "absent original compiler artifact": (f) => {
    f.artifact.target.name = "other";
    f.flush();
  },
  "product compiler artifact": (f) => {
    f.artifact.package_id = "vize_canon";
    f.flush();
  },
  "product through normal build graph": (f) => {
    f.metadata.packages.push({ id: "product", name: "vize_canon", source: null });
    f.metadata.resolve.nodes[0].deps.push({ pkg: "product", dep_kinds: [{ kind: null }] });
    f.flush();
  },
};
for (const [name, mutate] of Object.entries(corruptions))
  test(`atomic original refuses ${name}`, (t) => {
    const f = fixture();
    t.after(f.clean);
    mutate(f);
    assert.throws(f.observe);
  });

test("zero, ignored, duplicate or differently named original tests cannot qualify", (t) => {
  const f = fixture();
  t.after(f.clean);
  for (const stdout of [
    f.stdout().replace("1 passed; 0 failed; 0 ignored", "0 passed; 0 failed; 1 ignored"),
    f.stdout().replace(`test ${atomicTest} ... ok`, "test other ... ok"),
    f.stdout() + `test ${atomicTest} ... ok\n`,
    f.stdout().replace("1 passed; 0 failed; 0 ignored", "0 passed; 0 failed; 0 ignored"),
  ]) {
    f.rewriteStdout(stdout);
    assert.throws(f.observe);
    assert.throws(() => atomicStdout(stdout));
  }
});
