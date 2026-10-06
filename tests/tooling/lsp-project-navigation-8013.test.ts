import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { LspWire } from "../differential/lsp-wire.ts";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";
import { root } from "./support/lsp/paths.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";
import { hash, observation, open, publication, save, uri } from "./support/editor-jsconfig.ts";
import { carriers, references, stock } from "./support/project-navigation.ts";

const corpus = path.join(root, "tests/_fixtures/differential/lsp/project-navigation-8013");
function prepare(directory: string): Record<string, string> {
  const inputs: Record<string, string> = {};
  for (const file of [...carriers, "package.json", "tsconfig.json"]) {
    const name = carriers.includes(file) ? "src/" + file : file;
    inputs[name] = fs.readFileSync(path.join(corpus, file + ".txt"), "utf8");
    fs.mkdirSync(path.dirname(path.join(directory, name)), { recursive: true });
    fs.writeFileSync(path.join(directory, name), inputs[name]);
  }
  const vue = fs.realpathSync(process.env.VIZE_PROJECT_NAVIGATION_VUE_ROOT!);
  const provider = JSON.parse(fs.readFileSync(path.join(vue, "package.json"), "utf8"));
  assert.equal(provider.name, "vue");
  assert.equal(provider.version, "3.5.43");
  fs.mkdirSync(path.join(directory, "node_modules"));
  fs.symlinkSync(vue, path.join(directory, "node_modules/vue"), "dir");
  save("original-inputs", {
    inputs,
    hashes: Object.fromEntries(
      Object.entries(inputs).map(([file, source]) => [file, hash(Buffer.from(source))]),
    ),
    provider,
    vue,
  });
  return inputs;
}
function symbol(
  directory: string,
  file: string,
  name: string,
  kind: number,
  line = 0,
  start = 0,
  end = start,
) {
  return {
    name,
    kind,
    location: {
      uri: uri(path.join(directory, "src", file)),
      range: { start: { line, character: start }, end: { line, character: end } },
    },
  };
}
async function symbols(wire: LspWire, query: string, expected: unknown) {
  const reply = await wire.request("workspace/symbol", { query });
  save(`vize-symbol-${wire.child.pid}-${wire.nextId}`, {
    query,
    expected,
    reply,
    wire: observation(wire),
  });
  assert.deepEqual(reply, { jsonrpc: "2.0", id: wire.nextId, result: expected });
}

test(
  "issue 8013 original project references and unopened exports ignore the cross-file lint switch",
  { timeout: 180_000 },
  async (t) => {
    const required = process.env.VIZE_PROJECT_NAVIGATION_REQUIRED === "1";
    if (
      !requireTypecheckDependency(
        t,
        process.env.VIZE_PROJECT_NAVIGATION_VUE_ROOT,
        "Vue 3.5.43",
        "project navigation provider unavailable",
        required,
      )
    )
      return;
    const runtime = requireTypecheckDependency(
      t,
      resolveTypecheckRuntime(root),
      "native 7.0.2",
      "native runtime unavailable",
      required,
    );
    if (!runtime) return;
    const [binary] = resolveVizeLaunchCommand(undefined, process.env.VIZE_LSP_BIN, {
      required: true,
      repoRoot: root,
    });
    const version = spawnSync(runtime, ["--version"]);
    assert.equal(version.status, 0);
    assert.equal(version.stdout.toString().trim(), "Version 7.0.2");
    save("runtime", {
      sourceSha: process.env.SOURCE_SHA,
      binary,
      vizeSha256: hash(fs.readFileSync(binary)),
      buildReceipt: JSON.parse(fs.readFileSync(binary + ".differential-build.json", "utf8")),
      native: fs.realpathSync(runtime),
      nativeSha256: hash(fs.readFileSync(runtime)),
      nativeVersion: {
        status: version.status,
        stdoutBase64: version.stdout.toString("base64"),
        stderrBase64: version.stderr.toString("base64"),
      },
    });
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-project-navigation-"));
    const stockDirectory = fs.mkdtempSync(path.join(os.tmpdir(), "native-project-navigation-"));
    const previous = process.env.CORSA_PATH;
    process.env.CORSA_PATH = runtime;
    const inputs = prepare(directory);
    try {
      await stock(stockDirectory, inputs, runtime);
      for (const crossFile of [false, true]) {
        const wire = new LspWire(binary, ["lsp", "--stdio"], directory);
        const events: unknown[] = [];
        try {
          const initialized = await wire.request("initialize", {
            processId: process.pid,
            rootUri: uri(directory),
            capabilities: {},
            initializationOptions: {
              editor: true,
              typecheck: true,
              lint: false,
              crossFile,
            },
          });
          assert.equal(initialized.error, undefined);
          assert.equal(typeof initialized.result, "object");
          wire.notify("initialized", {});
          const app = path.join(directory, "src/App.vue");
          open(wire, app, inputs["src/App.vue"], "vue");
          await publication(wire, app, 1, []);
          // The original unopened query runs before any other carrier is opened.
          await symbols(wire, "Banner", [symbol(directory, "Banner.vue", "Banner", 5)]);
          await symbols(wire, "useToast", [
            symbol(directory, "useToast.ts", "useToast", 12, 0, 16, 24),
          ]);
          await references(wire, directory, true);
          await references(wire, directory, false);
          // Original reproduction: all four public files open. Same whole vectors.
          for (const file of carriers.filter((file) => file !== "App.vue"))
            open(
              wire,
              path.join(directory, "src", file),
              inputs["src/" + file],
              file.endsWith(".vue") ? "vue" : "typescript",
            );
          for (const file of carriers.filter((file) => file !== "App.vue"))
            await publication(wire, path.join(directory, "src", file), 1, []);
          await references(wire, directory, true);
          await references(wire, directory, false);
          // A same-spelling setup-local binding must retain its local namespace.
          const local = await wire.request("textDocument/references", {
            textDocument: { uri: uri(app) },
            position: { line: 4, character: 10 },
            context: { includeDeclaration: true },
          });
          const expectedLocal = [
            {
              uri: uri(app),
              range: { start: { line: 4, character: 8 }, end: { line: 4, character: 12 } },
            },
            {
              uri: uri(app),
              range: { start: { line: 9, character: 18 }, end: { line: 9, character: 22 } },
            },
          ];
          events.push({ local, expectedLocal });
          assert.deepEqual(local, { jsonrpc: "2.0", id: wire.nextId, result: expectedLocal });
          const notify = path.join(directory, "src/notify.ts");
          const dirty = inputs["src/notify.ts"].replace(
            "notifier = useToast()",
            "notifier = () => useToast()",
          );
          events.push({ notify, dirty });
          wire.notify("textDocument/didChange", {
            textDocument: { uri: uri(notify), version: 2 },
            contentChanges: [{ text: dirty }],
          });
          await publication(wire, notify, 2, []);
          await references(wire, directory, true, false, true);
          wire.notify("textDocument/didClose", { textDocument: { uri: uri(notify) } });
          await references(wire, directory, true);
          const exported = path.join(directory, "src/useToast.ts");
          const renamed = inputs["src/useToast.ts"].replace("useToast", "useSnack");
          events.push({ exported, renamed });
          wire.notify("textDocument/didChange", {
            textDocument: { uri: uri(exported), version: 2 },
            contentChanges: [{ text: renamed }],
          });
          await symbols(wire, "useToast", null);
          await symbols(wire, "useSnack", [
            symbol(directory, "useToast.ts", "useSnack", 12, 0, 16, 24),
          ]);
          wire.notify("textDocument/didClose", { textDocument: { uri: uri(exported) } });
          await symbols(wire, "useToast", [
            symbol(directory, "useToast.ts", "useToast", 12, 0, 16, 24),
          ]);
          await wire.finish();
        } finally {
          await wire.stop();
          save(`vize-terminal-cross-file-${crossFile}`, {
            inputs,
            crossFile,
            events,
            wire: observation(wire),
          });
        }
      }
    } finally {
      if (previous === undefined) delete process.env.CORSA_PATH;
      else process.env.CORSA_PATH = previous;
      fs.rmSync(directory, { recursive: true, force: true });
      fs.rmSync(stockDirectory, { recursive: true, force: true });
    }
  },
);
