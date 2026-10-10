// New whole parameter consumers of a reviewed public install, using the original custody adapters.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./lsp/assertions.ts";
import { root } from "./lsp/paths.ts";
import { errorPacket } from "./n8n-cli-config-oracle.mjs";
import { sha256, writeJson } from "./n8n-cli-config-workspace.mjs";
import { installedCliRuntime } from "./n8n-installed-cli.ts";
import { installedLspRuntime } from "./n8n-installed-lsp.ts";

type Input = { id: string; source: string; sourceSha256: string };
type Expected = {
  id: string;
  sourceSha256: string;
  options: Record<string, { cli: Array<Record<string, unknown>>; lspDiagnostics: unknown[] }>;
};
const fixture = path.join(root, "tests/_fixtures/differential/lint/slot-parameter-bindings-8142");
const inputs: Input[] = JSON.parse(
  fs.readFileSync(path.join(fixture, "controls.json"), "utf8"),
).cases;
const expected: Expected[] = JSON.parse(
  fs.readFileSync(path.join(fixture, "consumers.json"), "utf8"),
).records;
assert.ok(process.argv[2] && process.argv[3], "new output and reviewed installed plan required");
const output = path.resolve(process.argv[2]);
assert.equal(inputs.length, 24);
assert.equal(expected.length, 24);
const producer = installedCliRuntime(output, process.argv[3]);
const { identity, binary, runCli } = producer;
const workspace = producer.workspace("slot-parameters");
fs.mkdirSync(workspace);
fs.mkdirSync(path.join(workspace, ".git"));
fs.writeFileSync(path.join(workspace, "package.json"), '{"private":true,"type":"module"}\n');
const configValue = {
  lsp: { editor: false, lint: true, typecheck: false },
  linter: {
    enabled: true,
    preset: "incremental",
    rules: { "vue/valid-v-slot": "error" },
    ruleOptions: { "vue/valid-v-slot": { allowModifiers: true } },
  },
  entries: [
    {
      files: ["false/**/*.vue"],
      linter: { ruleOptions: { "vue/valid-v-slot": { allowModifiers: false } } },
    },
    {
      files: ["disabled/**/*.vue"],
      linter: {
        rules: { "vue/valid-v-slot": "off" },
        ruleOptions: { "vue/valid-v-slot": { allowModifiers: false } },
      },
    },
  ],
};
const configSource = JSON.stringify(configValue, null, 2) + "\n";
const config = { path: path.join(workspace, "vize.config.json"), sha256: sha256(configSource) };
fs.writeFileSync(config.path, configSource);
writeJson(path.join(output, "config.json"), {
  ...config,
  source: configSource,
  value: configValue,
});
const custody: Array<{ file: string; sha256: string }> = [
  { file: config.path, sha256: config.sha256 },
];
for (const option of ["false", "true", "disabled"]) {
  fs.mkdirSync(path.join(workspace, option));
  for (const input of inputs) {
    assert.equal(sha256(input.source), input.sourceSha256, input.id);
    const golden = expected.find((record) => record.id === input.id);
    assert.ok(golden, input.id);
    assert.equal(golden.sourceSha256, input.sourceSha256, input.id);
    const file = path.join(workspace, option, input.id + ".vue");
    fs.writeFileSync(file, input.source);
    custody.push({ file, sha256: input.sourceSha256 });
  }
}
writeJson(path.join(output, "inputs.json"), custody);
const cli: Array<{ id: string; option: string; mode: string; actual: unknown; expected: unknown }> =
  [];
for (const option of ["false", "true", "disabled"]) {
  for (const input of option === "disabled" ? inputs.slice(0, 1) : inputs) {
    const packet =
      option === "disabled"
        ? [{ file: input.id + ".vue", messages: [], errorCount: 0, warningCount: 0 }]
        : expected.find((record) => record.id === input.id)!.options[option].cli;
    for (const mode of ["baseline", "repeat", "outside"]) {
      const destination = path.join(output, "cli", option, input.id, mode + ".json");
      let actual: unknown;
      try {
        actual = runCli({
          binary,
          workspace,
          packageRoot: option,
          files: [input.id + ".vue"],
          config,
          outsideCwd: mode === "outside",
          receiptPath: destination,
        });
      } catch (error) {
        actual = { failure: errorPacket(error) };
      }
      // Outside-cwd output uses the actual explicitly supplied relative path.
      const whole = packet.map((entry) => ({
        ...entry,
        file: mode === "outside" ? option + "/" + entry.file : entry.file,
      }));
      cli.push({ id: input.id, option, mode, actual, expected: whole });
      writeJson(path.join(output, "cli/complete.json"), cli);
    }
  }
}
const lsp: Array<{
  id: string;
  option: string;
  version: number;
  actual: unknown;
  expected: unknown;
}> = [];
const session = installedLspRuntime(path.join(output, "lsp"), process.argv[3]);
const { wire } = session;
let processFailure: unknown;
try {
  const initialized = await wire.request("initialize", {
    processId: process.pid,
    rootUri: pathToFileURL(workspace).href,
    capabilities: {
      textDocument: {
        completion: { completionItem: { documentationFormat: ["markdown", "plaintext"] } },
      },
    },
    initializationOptions: { editor: false, lint: true, typecheck: false },
    workspaceFolders: [{ uri: pathToFileURL(workspace).href, name: path.basename(workspace) }],
  });
  writeJson(path.join(output, "lsp/initialize.json"), initialized);
  assert.equal(initialized.error, undefined);
  assert.ok(initialized.result);
  wire.notify("initialized", {});
  for (const option of ["false", "true", "disabled"]) {
    for (const input of option === "disabled" ? inputs.slice(0, 1) : inputs) {
      const uri = pathToFileURL(path.join(workspace, option, input.id + ".vue")).href;
      const diagnostics =
        option === "disabled"
          ? []
          : expected.find((record) => record.id === input.id)!.options[option].lspDiagnostics;
      for (const version of [1, 2]) {
        if (version === 1) {
          wire.notify("textDocument/didOpen", {
            textDocument: { uri, languageId: "vue", version, text: input.source },
          });
        } else {
          wire.notify("textDocument/didChange", {
            textDocument: { uri, version },
            contentChanges: [{ text: input.source }],
          });
        }
        let actual: unknown;
        try {
          actual = (
            await wire.waitFor(
              (message) =>
                message.method === "textDocument/publishDiagnostics" &&
                isDiagnosticsForUri(message.params, uri) &&
                message.params.version === version,
            )
          ).params;
        } catch (error) {
          actual = { failure: errorPacket(error) };
        }
        lsp.push({
          id: input.id,
          option,
          version,
          actual,
          expected: { uri, version, diagnostics },
        });
        writeJson(path.join(output, "lsp/complete.json"), lsp);
        session.capture();
      }
      wire.notify("textDocument/didClose", { textDocument: { uri } });
    }
  }
} catch (error) {
  processFailure = errorPacket(error);
  writeJson(path.join(output, "lsp/failure.json"), processFailure);
} finally {
  try {
    await session.finish();
  } catch (error) {
    processFailure ??= errorPacket(error);
    writeJson(path.join(output, "lsp/failure.json"), processFailure);
  }
  session.capture();
}
// All process responses and whole raw JSON-RPC streams precede qualification.
for (const entry of custody)
  assert.equal(sha256(fs.readFileSync(entry.file)), entry.sha256, entry.file);
producer.recheck();
session.recheck();
assert.equal(processFailure, undefined, "installed LSP failure evidence retained");
assert.equal(cli.length, 147);
assert.equal(lsp.length, 98);
for (const capture of cli)
  assert.deepEqual(
    capture.actual,
    capture.expected,
    `${capture.option}/${capture.id}/${capture.mode}`,
  );
for (const capture of lsp)
  assert.deepEqual(
    capture.actual,
    capture.expected,
    `${capture.option}/${capture.id}/v${capture.version}`,
  );
writeJson(path.join(output, "qualified.json"), {
  source: identity,
  cliCalls: cli.length,
  wholeLspPackets: lsp.length,
  options: [false, true],
  explicitFalseOverridesTrue: true,
  disabledRuleRemainsDisabled: true,
  rawLspSession: path.join(output, "lsp/observation.json"),
  qualification:
    "installed public parameter consumers only; original adoption, reference differences and public H remain separate",
});
