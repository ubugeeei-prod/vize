import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { LspWire } from "../../differential/lsp-wire.ts";
import type { JsonRpcMessage } from "../../differential/lsp-types.ts";
import { root } from "./lsp/paths.ts";

export const corpus = path.join(root, "tests/_fixtures/differential/lsp/package-private-imports");
export const original = (file: string) => fs.readFileSync(path.join(corpus, file + ".txt"), "utf8");
export const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
export const uri = (file: string) => pathToFileURL(file).href;
export const captureRoot = process.env.VIZE_PRIVATE_IMPORT_CAPTURE;
export function save(name: string, value: unknown): void {
  if (!captureRoot) return;
  fs.mkdirSync(captureRoot, { recursive: true });
  fs.writeFileSync(path.join(captureRoot, name + ".json"), JSON.stringify(value, null, 2) + "\n");
}

export function diagnostic(
  code: number,
  message: string,
  line: number,
  start: number,
  end: number,
  native = false,
) {
  return {
    code,
    message,
    range: { start: { line, character: start }, end: { line, character: end } },
    severity: code === 6133 ? 4 : 1,
    source: native ? "ts" : "vize/types",
  };
}
export function expected(mode: string, native = false): Record<string, unknown[]> {
  const message = "Argument of type 'number' is not assignable to parameter of type 'string'.";
  const missing = "Cannot find module '#lib/util.ts' or its corresponding type declarations.";
  const app =
    mode === "bad-call"
      ? [diagnostic(2345, message, 3, 22, 24, native)]
      : mode === "missing-target"
        ? [diagnostic(2307, missing, 1, 22, 36, native)]
        : [];
  const main =
    mode === "bad-call"
      ? [diagnostic(2345, message, 2, 37, 39, native)]
      : mode === "missing-target"
        ? [diagnostic(2307, missing, 0, 22, 36, native)]
        : [];
  if (native)
    app.push(
      diagnostic(6133, "'message' is declared but its value is never read.", 3, 6, 13, true),
    );
  return { "src/App.vue": app, "src/main.ts": main };
}

export function prepare(directory: string, mode: string): Record<string, string> {
  const inputs: Record<string, string> = {
    "package.json": original("package.json"),
    "tsconfig.json": original("tsconfig.json"),
    "src/lib/util.ts": original("util.ts"),
    "src/App.vue": original("App.vue"),
    "src/main.ts": original("main.ts"),
  };
  if (mode === "missing-target")
    inputs["package.json"] = inputs["package.json"].replace("./src/lib/*", "./src/nope/*");
  if (mode === "bad-call") {
    inputs["src/App.vue"] = inputs["src/App.vue"].replace('greet("x")', "greet(42)");
    inputs["src/main.ts"] = inputs["src/main.ts"].replace('greet("y")', "greet(42)");
  }
  if (mode === "ordinary-package") {
    for (const file of ["src/App.vue", "src/main.ts"])
      inputs[file] = inputs[file].replace("#lib/util.ts", "ordinary");
    inputs["node_modules/ordinary/package.json"] =
      '{"name":"ordinary","type":"module","exports":"./index.ts"}';
    inputs["node_modules/ordinary/index.ts"] = original("util.ts");
  }
  for (const [file, content] of Object.entries(inputs)) {
    fs.mkdirSync(path.dirname(path.join(directory, file)), { recursive: true });
    fs.writeFileSync(path.join(directory, file), content);
  }
  const vue = fs.realpathSync(process.env.VIZE_PRIVATE_IMPORT_VUE_ROOT!);
  const vuePackage = JSON.parse(fs.readFileSync(path.join(vue, "package.json"), "utf8"));
  assert.equal(vuePackage.name, "vue");
  assert.equal(vuePackage.version, "3.5.43");
  fs.mkdirSync(path.join(directory, "node_modules"), { recursive: true });
  fs.symlinkSync(vue, path.join(directory, "node_modules/vue"), "dir");
  save(mode + "-inputs", { inputs, vue, vuePackage, sourceSha: process.env.SOURCE_SHA });
  assert.equal(fs.existsSync(path.join(directory, "vize.config.json")), false);
  return inputs;
}

export async function initialize(wire: LspWire, directory: string, stock: boolean): Promise<void> {
  const initialized = await wire.request("initialize", {
    processId: process.pid,
    rootUri: uri(directory),
    capabilities: {},
    ...(stock
      ? {
          initializationOptions: {
            userPreferences: { tsserver: { automaticTypeAcquisition: { enabled: false } } },
          },
        }
      : {}),
  });
  assert.equal(initialized.error, undefined);
  assert.equal(typeof initialized.result, "object");
  wire.notify("initialized", {});
  if (stock) {
    const registration = await wire.waitFor(
      (message) => message.method === "client/registerCapability",
    );
    assert.deepEqual(registration, {
      jsonrpc: "2.0",
      id: "ts1",
      method: "client/registerCapability",
      params: {
        registrations: [
          {
            id: "typescript-config-watch-id",
            method: "workspace/didChangeConfiguration",
            registerOptions: { section: ["js/ts", "typescript", "javascript", "editor"] },
          },
        ],
      },
    });
    wire.send({ jsonrpc: "2.0", id: "ts1", result: null } as unknown as JsonRpcMessage);
  }
}
export function open(wire: LspWire, file: string, text: string, languageId: string): void {
  wire.notify("textDocument/didOpen", {
    textDocument: { uri: uri(file), languageId, version: 1, text },
  });
}
export function change(wire: LspWire, file: string, text: string, version: number): void {
  wire.notify("textDocument/didChange", {
    textDocument: { uri: uri(file), version },
    contentChanges: [{ text }],
  });
}
export async function publication(
  wire: LspWire,
  file: string,
  version: number,
  diagnostics: unknown[],
): Promise<void> {
  const response = await wire.waitFor(
    (message) =>
      message.method === "textDocument/publishDiagnostics" &&
      message.params?.uri === uri(file) &&
      message.params?.version === version,
  );
  assert.deepEqual(response, {
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: { uri: uri(file), version, diagnostics },
  });
}
export function observation(wire: LspWire) {
  const observed = wire.observation();
  return {
    ...observed,
    clientWireSha256: hash(Buffer.from(observed.clientWireBase64, "base64")),
    serverWireSha256: hash(Buffer.from(observed.serverWireBase64, "base64")),
  };
}

export async function stockOracle(
  directory: string,
  inputs: Record<string, string>,
  mode: string,
  runtime: string,
): Promise<void> {
  const source = inputs["src/App.vue"];
  const bare = source.slice(source.indexOf(">") + 1, source.indexOf("</script>"));
  const file = path.join(directory, "src/Oracle.ts");
  fs.writeFileSync(file, bare);
  const wire = new LspWire(runtime, ["--lsp", "--stdio"], directory);
  try {
    await initialize(wire, directory, true);
    const vectors: Record<string, unknown> = {};
    for (const [target, bytes] of [
      [file, bare],
      [path.join(directory, "src/main.ts"), inputs["src/main.ts"]],
    ]) {
      open(wire, target, bytes, "typescript");
      const result = await wire.request("textDocument/diagnostic", {
        textDocument: { uri: uri(target) },
      });
      vectors[target] = result;
      save(mode + "-native-current", {
        originalInputs: inputs,
        stockSource: bare,
        requestedConfig: inputs["tsconfig.json"],
        vectors,
        wire: observation(wire),
      });
      assert.deepEqual(result, {
        jsonrpc: "2.0",
        id: wire.nextId,
        result: {
          kind: "full",
          items: expected(mode, true)[target === file ? "src/App.vue" : "src/main.ts"],
        },
      });
    }
    const shutdown = await wire.request("shutdown");
    assert.deepEqual(shutdown, { jsonrpc: "2.0", id: wire.nextId, result: null });
    wire.child.stdin.end();
    await wire.stop(false);
    assert.equal(wire.exitStatus, 0);
    assert.equal(wire.signal, null);
    assert.equal(wire.processError, null);
    assert.equal(Buffer.concat(wire.stderrChunks).length, 0);
    assert.deepEqual(
      wire.messages.filter((message) => message.method === "textDocument/publishDiagnostics"),
      [
        {
          jsonrpc: "2.0",
          method: "textDocument/publishDiagnostics",
          params: { uri: uri(path.join(directory, "tsconfig.json")), diagnostics: [] },
        },
      ],
    );
  } finally {
    await wire.stop().catch(() => undefined);
    save(mode + "-native-terminal", {
      wire: observation(wire),
      joinedThroughChildClose: wire.exitStatus !== null || wire.signal !== null,
    });
    fs.rmSync(file, { force: true });
  }
}

export function cliCheck(
  binary: string,
  directory: string,
  inputs: Record<string, string>,
  mode: string,
): void {
  const command = spawnSync(binary, ["check", "--format", "json"], {
    cwd: directory,
    maxBuffer: 16 * 1024 * 1024,
  });
  save(mode + "-cli", {
    argv: ["check", "--format", "json"],
    cwd: directory,
    status: command.status,
    signal: command.signal,
    error: command.error?.message ?? null,
    stdoutBase64: command.stdout.toString("base64"),
    stderrBase64: command.stderr.toString("base64"),
    originalInputs: inputs,
  });
  const vectors = expected(mode);
  const files = ["src/App.vue", "src/lib/util.ts", "src/main.ts"];
  const diagnostics = Object.fromEntries(
    files.map((file) => [
      file,
      (vectors[file] ?? []).map((value) => {
        const item = value as ReturnType<typeof diagnostic>;
        return `error:${item.range.start.line + 1}:${item.range.start.character + 1} [TS${item.code}] ${item.message}`;
      }),
    ]),
  );
  assert.equal(command.status, mode === "bad-call" || mode === "missing-target" ? 1 : 0);
  assert.equal(command.signal, null);
  assert.equal(command.error, undefined);
  assert.deepEqual(JSON.parse(command.stdout.toString()), {
    files: files.map((file) => ({ file, diagnostics: diagnostics[file] })),
    programs: [
      {
        root: ".",
        tsconfig: "tsconfig.json",
        compilerOptions: JSON.parse(inputs["tsconfig.json"]).compilerOptions,
        files,
      },
    ],
    errorCount: mode === "bad-call" || mode === "missing-target" ? 2 : 0,
    warningCount: 0,
    fileCount: 3,
  });
}
