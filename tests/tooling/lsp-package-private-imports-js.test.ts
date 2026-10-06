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
  hash,
  initialize,
  observation,
  open,
  publication,
  save,
  stockOracle,
  uri,
} from "./support/editor-jsconfig.ts";
import { cliCheck, prepare, vectors } from "./support/package-private-imports-js.ts";

test(
  "private JS imports retain JSDoc/checkJs under each authored config without Vize config",
  { timeout: 180_000 },
  async (t) => {
    const required = process.env.VIZE_PRIVATE_IMPORT_REQUIRED === "1";
    if (
      !requireTypecheckDependency(
        t,
        process.env.VIZE_PRIVATE_IMPORT_VUE_ROOT,
        "Vue 3.5.43",
        "private JS provider is qualified in the native fixture lane",
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
    save("javascript-runtime", {
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
      for (const configName of ["jsconfig.json", "tsconfig.json"])
        for (const mode of ["original", "bad-call", "missing-target", "ordinary-package"]) {
          const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-private-js-imports-"));
          const name = "javascript-" + configName + "-" + mode;
          const inputs = prepare(directory, configName, mode, name);
          let wire: LspWire | undefined;
          try {
            // CLI default jsconfig discovery is a separate provider; no inferred success credit.
            if (configName === "tsconfig.json") cliCheck(binary, directory, inputs, mode, name);
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
