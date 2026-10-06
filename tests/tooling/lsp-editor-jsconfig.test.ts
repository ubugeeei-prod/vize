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
import {
  diagnostic,
  hash,
  initialize,
  observation,
  open,
  publication,
  save,
  stockOracle,
  uri,
} from "./support/editor-jsconfig.ts";

const corpus = path.join(root, "tests/_fixtures/differential/lsp/editor-jsconfig");
const original = (file: string) => fs.readFileSync(path.join(corpus, file + ".txt"), "utf8");
const argumentMessage =
  "Argument of type 'number' is not assignable to parameter of type 'string'.";

function vectors(mode: string, native = false): Record<string, unknown[]> {
  const app =
    mode === "wrong-argument"
      ? [diagnostic(2345, argumentMessage, 3, 22, 24, native)]
      : mode === "missing-target"
        ? [
            diagnostic(
              2307,
              "Cannot find module './missing.js' or its corresponding type declarations.",
              1,
              22,
              36,
              native,
            ),
          ]
        : [];
  const main =
    mode === "wrong-argument"
      ? [diagnostic(2345, argumentMessage, 3, 29, 31, native)]
      : mode === "missing-target"
        ? [
            diagnostic(
              2307,
              "Cannot find module './missing.js' or its corresponding type declarations.",
              0,
              22,
              36,
              native,
            ),
          ]
        : [];
  if (native)
    app.push(
      diagnostic(6133, "'message' is declared but its value is never read.", 3, 6, 13, true),
    );
  return { "src/App.vue": app, "src/main.js": main };
}

function prepare(
  directory: string,
  configName: string,
  mode: string,
  name: string,
  implicit: boolean,
): Record<string, string> {
  const inputs: Record<string, string> = {
    [configName]: original(implicit ? "jsconfig-implicit.json" : "jsconfig.json"),
    "package.json": original("package.json"),
    "src/util.js": original("util.js"),
    "src/App.vue": original("App.vue"),
    "src/main.js": original("main.js"),
  };
  if (mode === "wrong-argument") {
    inputs["src/App.vue"] = inputs["src/App.vue"].replace('greet("x")', "greet(42)");
    inputs["src/main.js"] = inputs["src/main.js"].replace('greet("y")', "greet(42)");
  }
  if (mode === "missing-target")
    for (const file of ["src/App.vue", "src/main.js"])
      inputs[file] = inputs[file].replace("./util.js", "./missing.js");
  if (configName === "tsconfig.json")
    inputs["jsconfig.json"] = original("jsconfig.json").replace(
      '"checkJs": true',
      '"checkJs": false',
    );
  for (const [file, bytes] of Object.entries(inputs)) {
    fs.mkdirSync(path.dirname(path.join(directory, file)), { recursive: true });
    fs.writeFileSync(path.join(directory, file), bytes);
  }
  const vue = fs.realpathSync(process.env.VIZE_EDITOR_CONFIG_VUE_ROOT!);
  const provider = JSON.parse(fs.readFileSync(path.join(vue, "package.json"), "utf8"));
  assert.equal(provider.name, "vue");
  assert.equal(provider.version, "3.5.43");
  fs.mkdirSync(path.join(directory, "node_modules"));
  fs.symlinkSync(vue, path.join(directory, "node_modules/vue"), "dir");
  save(name + "-inputs", {
    inputs,
    vue,
    provider,
    sourceSha: process.env.SOURCE_SHA,
  });
  assert.equal(fs.existsSync(path.join(directory, "vize.config.json")), false);
  return inputs;
}

test(
  "nearest JS config preserves JSDoc/checkJs for ordinary JS and setup-JS without Vize config",
  { timeout: 180_000 },
  async (t) => {
    const required = process.env.VIZE_EDITOR_CONFIG_REQUIRED === "1";
    if (
      !requireTypecheckDependency(
        t,
        process.env.VIZE_EDITOR_CONFIG_VUE_ROOT,
        "Vue 3.5.43",
        "JS config provider is qualified in the native fixture lane",
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
      cliSha256: hash(fs.readFileSync(binary)),
      cliBuildReceipt: JSON.parse(fs.readFileSync(binary + ".differential-build.json", "utf8")),
      native: fs.realpathSync(runtime),
      nativeSha256: hash(fs.readFileSync(runtime)),
      nativeVersion: {
        status: version.status,
        stdoutBase64: version.stdout.toString("base64"),
        stderrBase64: version.stderr.toString("base64"),
      },
    });
    const previous = process.env.CORSA_PATH;
    process.env.CORSA_PATH = runtime;
    try {
      for (const [configName, implicit] of [
        ["jsconfig.json", false],
        ["tsconfig.json", false],
        ["jsconfig.json", true],
      ] as const)
        for (const mode of ["clean", "wrong-argument", "missing-target"]) {
          const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-editor-jsconfig-"));
          const name = configName + (implicit ? "-implicit" : "") + "-" + mode;
          const inputs = prepare(directory, configName, mode, name, implicit);
          let wire: LspWire | undefined;
          try {
            const source = inputs["src/App.vue"];
            const bare = source.slice(source.indexOf(">") + 1, source.indexOf("</script>"));
            await stockOracle(
              directory,
              configName,
              inputs,
              bare,
              "js",
              "src/main.js",
              vectors(mode, true),
              name,
              runtime,
            );
            wire = new LspWire(binary, ["lsp", "--stdio"], directory);
            await initialize(wire, directory, false);
            for (const file of ["src/App.vue", "src/main.js"])
              open(
                wire,
                path.join(directory, file),
                inputs[file],
                file.endsWith(".vue") ? "vue" : "javascript",
              );
            for (const file of ["src/App.vue", "src/main.js"])
              await publication(wire, path.join(directory, file), 1, vectors(mode)[file]);
            await wire.finish();
            for (const file of ["src/App.vue", "src/main.js"]) {
              const publications = wire.messages.filter(
                (message) =>
                  message.method === "textDocument/publishDiagnostics" &&
                  message.params?.uri === uri(path.join(directory, file)),
              );
              assert.deepEqual(publications.at(-1), {
                jsonrpc: "2.0",
                method: "textDocument/publishDiagnostics",
                params: {
                  uri: uri(path.join(directory, file)),
                  version: 1,
                  diagnostics: vectors(mode)[file],
                },
              });
            }
            for (const [file, bytes] of Object.entries(inputs))
              assert.equal(fs.readFileSync(path.join(directory, file), "utf8"), bytes);
            assert.equal(fs.existsSync(path.join(directory, "vize.config.json")), false);
          } finally {
            if (wire) {
              await wire.stop().catch(() => undefined);
              save(name + "-vize-terminal", {
                inputs,
                expected: vectors(mode),
                wire: observation(wire),
                allDecodedMessages: wire.messages,
                joinedThroughChildClose: wire.exitStatus !== null || wire.signal !== null,
              });
            }
            fs.rmSync(directory, { recursive: true, force: true });
          }
        }
    } finally {
      if (previous === undefined) delete process.env.CORSA_PATH;
      else process.env.CORSA_PATH = previous;
    }
  },
);
