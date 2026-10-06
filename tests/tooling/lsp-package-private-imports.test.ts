import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { LspWire, decodeFrames } from "../differential/lsp-wire.ts";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";
import { root } from "./support/lsp/paths.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";
import {
  captureRoot,
  change,
  cliCheck,
  corpus,
  diagnostic,
  expected,
  hash,
  initialize,
  observation,
  open,
  prepare,
  publication,
  save,
  stockOracle,
  uri,
} from "./support/package-private-imports.ts";

test(
  "original private TS imports resolve without vize.config in CLI and stdio",
  { timeout: 180_000 },
  async (t) => {
    const required = process.env.VIZE_PRIVATE_IMPORT_REQUIRED === "1";
    const provider = requireTypecheckDependency(
      t,
      process.env.VIZE_PRIVATE_IMPORT_VUE_ROOT,
      "original Vue 3.5.43 provider",
      "original package-import provider is qualified in the native fixture lane",
      required,
    );
    if (!provider) return;
    const runtime = requireTypecheckDependency(
      t,
      resolveTypecheckRuntime(root),
      "native 7.0.2",
      "native runtime unavailable",
      required,
    );
    if (!runtime) return;
    assert.ok(
      process.env.VIZE_PRIVATE_IMPORT_VUE_ROOT,
      "the original Vue 3.5.43 provider is required",
    );
    const [binary] = resolveVizeLaunchCommand(undefined, process.env.VIZE_LSP_BIN, {
      required: true,
      repoRoot: root,
    });
    const nativeVersion = spawnSync(runtime, ["--version"]);
    assert.equal(nativeVersion.status, 0);
    assert.equal(nativeVersion.stdout.toString().trim(), "Version 7.0.2");
    save("runtime", {
      sourceSha: process.env.SOURCE_SHA,
      binary,
      cliSha256: hash(fs.readFileSync(binary)),
      cliBuildReceipt: JSON.parse(fs.readFileSync(binary + ".differential-build.json", "utf8")),
      native: fs.realpathSync(runtime),
      nativeSha256: hash(fs.readFileSync(runtime)),
      nativeVersion: {
        status: nativeVersion.status,
        stdoutBase64: nativeVersion.stdout.toString("base64"),
        stderrBase64: nativeVersion.stderr.toString("base64"),
      },
    });
    const previousRuntime = process.env.CORSA_PATH;
    process.env.CORSA_PATH = runtime;
    try {
      for (const mode of ["original", "bad-call", "missing-target", "ordinary-package"]) {
        const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-private-imports-"));
        const inputs = prepare(directory, mode);
        const originals = Object.fromEntries(
          Object.keys(inputs).map((file) => [file, fs.readFileSync(path.join(directory, file))]),
        );
        let wire: LspWire | undefined;
        try {
          cliCheck(binary, directory, inputs, mode);
          if (mode === "original") originalClient(binary, directory);
          await stockOracle(directory, inputs, mode, runtime);
          wire = new LspWire(binary, ["lsp", "--stdio"], directory);
          await initialize(wire, directory, false);
          const vectors = expected(mode);
          for (const file of ["src/App.vue", "src/main.ts"]) {
            open(
              wire,
              path.join(directory, file),
              inputs[file],
              file.endsWith(".vue") ? "vue" : "typescript",
            );
          }
          for (const file of ["src/App.vue", "src/main.ts"])
            await publication(wire, path.join(directory, file), 1, vectors[file]);
          if (mode === "original") await retarget(wire, directory, inputs);
          await wire.finish();
          for (const file of ["src/App.vue", "src/main.ts"]) {
            const publications = wire.messages.filter(
              (message) =>
                message.method === "textDocument/publishDiagnostics" &&
                message.params?.uri === uri(path.join(directory, file)),
            );
            assert.deepEqual(publications.at(-1)?.params, {
              uri: uri(path.join(directory, file)),
              version: mode === "original" ? 3 : 1,
              diagnostics: vectors[file],
            });
          }
          assert.equal(fs.existsSync(path.join(directory, "vize.config.json")), false);
          for (const [file, bytes] of Object.entries(originals))
            assert.deepEqual(fs.readFileSync(path.join(directory, file)), bytes);
        } finally {
          if (wire) {
            await wire.stop().catch(() => undefined);
            save(mode + "-vize-terminal", {
              originalInputs: inputs,
              wire: observation(wire),
              allDecodedMessages: wire.messages,
              joinedThroughChildClose: wire.exitStatus !== null || wire.signal !== null,
            });
          }
          fs.rmSync(directory, { recursive: true, force: true });
        }
      }
    } finally {
      if (previousRuntime === undefined) delete process.env.CORSA_PATH;
      else process.env.CORSA_PATH = previousRuntime;
    }
  },
);

async function retarget(
  wire: LspWire,
  directory: string,
  inputs: Record<string, string>,
): Promise<void> {
  const manifest = path.join(directory, "package.json");
  const original = fs.statSync(manifest);
  const alternate = path.join(directory, "src/alt/util.ts");
  fs.mkdirSync(path.dirname(alternate), { recursive: true });
  const source = "export const greet = (name: string): number => name.length;\n";
  fs.writeFileSync(alternate, source);
  const retargeted = inputs["package.json"].replace("./src/lib/*", "./src/alt/*");
  assert.equal(Buffer.byteLength(retargeted), Buffer.byteLength(inputs["package.json"]));
  save("same-session-retarget-inputs", {
    original: inputs,
    alternateSource: source,
    retargetedManifest: retargeted,
  });
  for (const [version, contents] of [
    [2, retargeted],
    [3, inputs["package.json"]],
  ] as const) {
    fs.writeFileSync(manifest, contents);
    fs.utimesSync(manifest, original.atime, original.mtime);
    wire.notify("workspace/didChangeWatchedFiles", { changes: [{ uri: uri(manifest), type: 2 }] });
    for (const file of ["src/App.vue", "src/main.ts"])
      change(wire, path.join(directory, file), inputs[file], version);
    await publication(wire, path.join(directory, "src/App.vue"), version, []);
    await publication(
      wire,
      path.join(directory, "src/main.ts"),
      version,
      version === 2
        ? [diagnostic(2322, "Type 'number' is not assignable to type 'string'.", 2, 13, 20)]
        : [],
    );
  }
}

function originalClient(binary: string, directory: string): void {
  assert.ok(captureRoot, "original full-frame capture is required");
  const output = path.join(captureRoot, "original-reported-client.json");
  fs.mkdirSync(path.join(directory, "node_modules/.bin"), { recursive: true });
  fs.symlinkSync(binary, path.join(directory, "node_modules/.bin/vize"));
  const helper = path.join(directory, "lsp-diag.mjs");
  fs.copyFileSync(path.join(corpus, "lsp-diag.mjs.txt"), helper);
  const observer = path.join(root, "tests/tooling/support/package-private-imports-observer.mjs");
  const result = spawnSync(
    process.execPath,
    ["--import", observer, helper, directory, "src/App.vue", "src/main.ts"],
    {
      cwd: directory,
      env: { ...process.env, VIZE_PRIVATE_IMPORT_ORIGINAL_CAPTURE: output },
      timeout: 45_000,
      maxBuffer: 16 * 1024 * 1024,
    },
  );
  save("original-reported-client-parent", {
    helperBase64: fs.readFileSync(helper).toString("base64"),
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdoutBase64: result.stdout.toString("base64"),
    stderrBase64: result.stderr.toString("base64"),
  });
  assert.equal(result.status, 0);
  assert.equal(result.signal, null);
  assert.equal(result.error, undefined);
  assert.equal(result.stdout.toString(), "");
  assert.equal(result.stderr.toString(), "");
  const observation = JSON.parse(fs.readFileSync(output, "utf8"));
  assert.equal(observation.status, 0);
  assert.equal(observation.signal, null);
  const client = decodeFrames(Buffer.from(observation.clientWireBase64, "base64")).messages;
  const server = decodeFrames(Buffer.from(observation.serverWireBase64, "base64")).messages;
  assert.deepEqual(client[0].params, {
    processId: observation.clientPid,
    rootUri: uri(directory),
    capabilities: {},
  });
  for (const file of ["src/App.vue", "src/main.ts"]) {
    const diagnostics = server.filter(
      (message) =>
        message.method === "textDocument/publishDiagnostics" &&
        message.params?.uri === uri(path.join(directory, file)),
    );
    assert.deepEqual(diagnostics.at(-1)?.params, {
      uri: uri(path.join(directory, file)),
      version: 1,
      diagnostics: [],
    });
  }
}
