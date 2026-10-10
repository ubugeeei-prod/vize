// New whole parameter consumers for an unreleased source build; public qualification is separate.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../../differential/build-receipt.ts";
import { isDiagnosticsForUri } from "./lsp/assertions.ts";
import { root } from "./lsp/paths.ts";
import { LspSession } from "./lsp/session.ts";
import { errorPacket } from "./n8n-cli-config-oracle.mjs";
import { runCli, sha256, writeJson } from "./n8n-cli-config-workspace.mjs";

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
const output = path.resolve(
  process.argv[2] ?? path.join(root, "target/n8n-cli-config/slot-parameters"),
);
assert.equal(inputs.length, 24);
assert.equal(expected.length, 24);
const identity = expectedBuildIdentity(root);
const binary = path.join(root, identity.binaryPath);
const buildReceipt = JSON.parse(fs.readFileSync(binary + ".differential-build.json", "utf8"));
writeJson(path.join(output, "producer.json"), { expected: identity, buildReceipt });
validateBuildReceipt(buildReceipt, identity);
fs.mkdirSync(output, { recursive: true });
const workspace = fs.mkdtempSync(path.join(output, "workspace-"));
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
const notifications: unknown[] = [];
const responses: unknown[] = [];
const session = new LspSession({ repoRoot: root, binary });
session.notificationObservers.push((method, params) => {
  notifications.push({ method, params });
  writeJson(path.join(output, "lsp/notifications.json"), notifications);
});
session.responseObservers.push((message) => {
  responses.push(message);
  writeJson(path.join(output, "lsp/responses.json"), responses);
});
session.stderrObservers.push((stderr) =>
  fs.writeFileSync(path.join(output, "lsp/stderr.txt"), stderr),
);
writeJson(path.join(output, "lsp/process.json"), {
  binary,
  pid: session.processId,
  workspace,
  config,
});
let processFailure: unknown;
try {
  const initialized = await session.initialize(workspace, {
    editor: false,
    lint: true,
    typecheck: false,
  });
  writeJson(path.join(output, "lsp/initialize.json"), initialized);
  for (const option of ["false", "true", "disabled"]) {
    for (const input of option === "disabled" ? inputs.slice(0, 1) : inputs) {
      const uri = pathToFileURL(path.join(workspace, option, input.id + ".vue")).href;
      const diagnostics =
        option === "disabled"
          ? []
          : expected.find((record) => record.id === input.id)!.options[option].lspDiagnostics;
      for (const version of [1, 2]) {
        if (version === 1) {
          session.notify("textDocument/didOpen", {
            textDocument: { uri, languageId: "vue", version, text: input.source },
          });
        } else {
          session.notify("textDocument/didChange", {
            textDocument: { uri, version },
            contentChanges: [{ text: input.source }],
          });
        }
        let actual: unknown;
        try {
          actual = await session.waitForNotification(
            "textDocument/publishDiagnostics",
            (params) => isDiagnosticsForUri(params, uri) && params.version === version,
          );
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
      }
      session.notify("textDocument/didClose", { textDocument: { uri } });
    }
  }
} catch (error) {
  processFailure = errorPacket(error);
  writeJson(path.join(output, "lsp/failure.json"), processFailure);
} finally {
  await session.shutdown();
  fs.writeFileSync(path.join(output, "lsp/stderr.txt"), session.stderrText);
}
// All process responses and whole raw JSON-RPC streams precede qualification.
assert.equal(processFailure, undefined, "source LSP failure evidence retained");
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
for (const entry of custody)
  assert.equal(sha256(fs.readFileSync(entry.file)), entry.sha256, entry.file);
assert.deepEqual(
  expectedBuildIdentity(root),
  identity,
  "source executable custody after both consumers",
);
writeJson(path.join(output, "qualified.json"), {
  source: identity,
  cliCalls: cli.length,
  wholeLspPackets: lsp.length,
  options: [false, true],
  explicitFalseOverridesTrue: true,
  disabledRuleRemainsDisabled: true,
  rawLspSessions: "target/differential/lsp-sessions",
  qualification:
    "unreleased source parameter consumers only; independent official differences remain declared",
});
