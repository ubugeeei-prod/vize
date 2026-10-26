import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  CORSA_OVERRIDES,
  SOURCES,
  assertProviderFiles,
  captureConsumerInputs,
  loadNativeProviders,
  prepareNativeRuntimeConsumer,
  runtimeChildEnvironment,
  sha256,
  type ConsumerInput,
  type NativeRuntimeConsumer,
} from "./providers.ts";
import { sourceIdentity, executionAuthority } from "./lsp-source-custody.ts";
import { validateNativeLifecycle } from "./lsp-native-lifecycle.ts";
import { SourceLspWire, type WireEvent } from "./lsp-wire.ts";
import { captureNativeOwners, type NativeOwner } from "./lsp-strace.ts";

interface LspPhase {
  name: string;
  version: number;
  authoredSha256: string;
  diagnostics?: any;
  value?: any;
  doubled?: any;
  unavailable?: any;
  unavailableDiagnostics?: any;
}
interface Failure {
  message: string;
  stack: string | undefined;
}
interface LspProfile {
  kind: string;
  consumer: NativeRuntimeConsumer;
  initialInputFiles: ConsumerInput[];
  overrideEnvironment: Record<string, string | null>;
  phases: LspPhase[];
  initialize?: any;
  shutdown?: any;
  failure?: Failure;
  cleanupShutdown?: any;
  cleanupProtocolFailure?: string;
  closeFailure?: string;
  outer?: ReturnType<SourceLspWire["save"]>;
  nativeOwners?: NativeOwner[];
  syscallObservation?: Pick<
    ReturnType<typeof captureNativeOwners>,
    "syscallsComplete" | "terminalUnknownSyscalls" | "execInventory" | "qualifiedExecInventory"
  >;
  qualification?: ReturnType<typeof validateNativeLifecycle>;
  userWarnings?: WireEvent[];
  outerWarnings?: string[];
  negativeProfileWarningsQualified?: boolean;
  auditFailure?: Failure;
  functionalQualified?: boolean;
}

const write = (file: string, value: unknown) =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
const repoRoot = fs.realpathSync(
  path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../.."),
);
const result = (response: any): any => {
  assert.equal(response.error, undefined, JSON.stringify(response));
  return response.result;
};
const position = (text: string, offset: number) => {
  const lines = text.slice(0, offset).split("\n");
  return { line: lines.length - 1, character: (lines.at(-1) ?? "").length };
};
const range = (text: string, offset: number, length: number) => ({
  start: position(text, offset),
  end: position(text, offset + length),
});

function exactHover(packet: any, source: string, token: string) {
  const offset =
    token === "value" ? source.indexOf("const value") + 6 : source.indexOf("{{ doubled }}") + 3;
  const expected = {
    contents: { kind: "markdown", value: `\`\`\`typescript\nconst ${token}: number\n\`\`\`` },
    range: range(source, offset, token.length),
  };
  assert.deepEqual(result(packet), expected, `complete ${token} hover`);
}

export async function observeSourceLspSelection() {
  const source = sourceIdentity(repoRoot, import.meta.filename),
    providers = loadNativeProviders();
  assert.ok(providers.frozen702, "the separately installed qualified 7.0.2 provider is mandatory");
  assert.ok(process.env.VIZE_NATIVE_STRACE && path.isAbsolute(process.env.VIZE_NATIVE_STRACE));
  const strace = fs.realpathSync(process.env.VIZE_NATIVE_STRACE);
  const output = path.join(repoRoot, "target/vize-tests/native-runtime-selection-lsp");
  fs.mkdirSync(output, { recursive: true });
  const temporary = fs.realpathSync(
    fs.mkdtempSync(path.join(process.env.RUNNER_TEMP ?? os.tmpdir(), "vize-native-lsp-consumers-")),
  );
  assert.ok(!temporary.startsWith(repoRoot + path.sep));
  const nativeSource = fs.readFileSync(
    path.join(repoRoot, "crates/vize_canon/src/lsp_client/editor_lsp/native_shutdown.rs"),
    "utf8",
  );
  const contract = /fn known_contract\(\) -> Value \{\s*serde_json::json!\(([\s\S]*?)\)\s*\}/u.exec(
    nativeSource,
  );
  assert.ok(contract);
  const knownContract = JSON.parse(contract[1]);
  const knownImage = /const LINUX_X64: &str = "([a-f0-9]{64})";/u.exec(nativeSource);
  assert.ok(knownImage);
  assert.equal(
    providers.frozen702.executable.sha256,
    knownImage[1],
    "qualified lifecycle cannot borrow an unrecognized image",
  );

  const profiles: LspProfile[] = [];
  const report = {
    source,
    strace: {
      path: strace,
      sha256: sha256(fs.readFileSync(strace)),
      version: execFileSync(strace, ["--version"], { encoding: "utf8" }),
    },
    providers,
    shutdownSourceSha256: sha256(Buffer.from(nativeSource)),
    profiles,
    scope:
      "Real source CLI/LSP selection and complete authored packets. No published/editor/cache dispatch, generic source-session, blanket API or beta clean-exit credit.",
  };
  for (const kind of [
    "canonical-beta",
    "alias-beta",
    "explicit-beta",
    "selected-config",
    "absent",
    "stale-decoys",
    "canonical-702",
    "alias-702",
  ] as const) {
    const directory = path.join(output, kind),
      consumer = prepareNativeRuntimeConsumer({
        directory: path.join(temporary, kind),
        kind,
        providers,
      });
    const inputs = captureConsumerInputs(consumer),
      environment = runtimeChildEnvironment(
        process.env,
        path.join(temporary, `${kind}-environment`),
      );
    for (const key of [
      "VIZE_TEST_DISABLE_TSGO",
      "VIZE_TEST_CONFIG_PROTOCOL_PATH",
      "VIZE_TEST_OBSERVED_SERVER_PATH",
      "VIZE_TRACE_EDITOR_PREPARATION",
    ])
      delete environment[key];
    for (const key of CORSA_OVERRIDES) assert.equal(environment[key], undefined);
    const profile: LspProfile = {
      kind,
      consumer,
      initialInputFiles: inputs,
      overrideEnvironment: Object.fromEntries(
        CORSA_OVERRIDES.map((key) => [key, environment[key] ?? null]),
      ),
      phases: [],
    };
    const wire = new SourceLspWire({
      executable: source.executable,
      strace,
      environment,
      workspace: consumer.workspace,
      directory,
    });
    const uri = pathToFileURL(consumer.entry).href,
      rootUri = pathToFileURL(consumer.workspace).href;
    try {
      assertProviderFiles(providers);
      assert.equal(sha256(fs.readFileSync(source.executable)), source.receipt.binarySha256);
      profile.initialize = await wire.request("initialize", {
        processId: process.pid,
        rootUri,
        capabilities: {
          textDocument: {
            hover: { contentFormat: ["markdown", "plaintext"] },
            publishDiagnostics: { versionSupport: true },
          },
        },
        initializationOptions: { editor: true, typecheck: true, lint: false, ecosystem: false },
        workspaceFolders: [{ uri: rootUri, name: kind }],
      });
      result(profile.initialize);
      wire.notify("initialized", {});
      for (const [name, text, version, open] of [
        ["valid", SOURCES.valid, 1, true],
        ["error", SOURCES.error, 2, false],
        ["repair", SOURCES.repair, 3, false],
        ["reopen", SOURCES.valid, 4, true],
      ] as const) {
        if (name === "reopen") wire.notify("textDocument/didClose", { textDocument: { uri } });
        const from = wire.events.length;
        if (open)
          wire.notify("textDocument/didOpen", {
            textDocument: { uri, languageId: "vue", version, text },
          });
        else
          wire.notify("textDocument/didChange", {
            textDocument: { uri, version },
            contentChanges: [{ text }],
          });
        const phase: LspPhase = { name, version, authoredSha256: sha256(Buffer.from(text)) };
        if (consumer.expectedExecutable) {
          phase.diagnostics = await wire.wait(
            (packet) =>
              packet.method === "textDocument/publishDiagnostics" &&
              packet.params?.uri === uri &&
              packet.params?.version === version,
            `complete current ${name} diagnostics`,
            from,
          );
          const diagnostics =
            name === "error"
              ? [
                  {
                    code: 2322,
                    message: "Type 'string' is not assignable to type 'number'.",
                    range: range(text, text.indexOf("const value") + 6, 5),
                    severity: 1,
                    source: "vize/types",
                  },
                ]
              : [];
          assert.deepEqual(phase.diagnostics, {
            jsonrpc: "2.0",
            method: "textDocument/publishDiagnostics",
            params: { uri, version, diagnostics },
          });
          for (const token of ["value", "doubled"] as const) {
            const offset =
              token === "value"
                ? text.indexOf("const value") + 6
                : text.indexOf("{{ doubled }}") + 3;
            phase[token] = await wire.request("textDocument/hover", {
              textDocument: { uri },
              position: position(text, offset + 1),
            });
            exactHover(phase[token], text, token);
          }
        } else if (name === "valid") {
          phase.unavailable = await wire.wait(
            (packet) =>
              packet.method === "window/showMessage" && [1, 2].includes(packet.params?.type),
            "actual missing-provider notice",
            from,
          );
          assert.deepEqual(
            phase.unavailable,
            {
              jsonrpc: "2.0",
              method: "window/showMessage",
              params: {
                message:
                  "Vize: type checking is unavailable in this workspace. Make sure tsconfig.json exists. Provide a native TypeScript compiler, for example `typescript-native@npm:typescript@7.1.0-beta`, or configure `typeChecker.corsaPath`, `CORSA_PATH`, or `TSGO_PATH`.",
                type: 2,
              },
            },
            "complete actionable user-provided compiler notice",
          );
          phase.unavailableDiagnostics = await wire.wait(
            (packet) =>
              packet.method === "textDocument/publishDiagnostics" &&
              packet.params?.uri === uri &&
              packet.params?.version === version,
            "complete unavailable-provider diagnostic",
            from,
          );
          assert.deepEqual(
            phase.unavailableDiagnostics,
            {
              jsonrpc: "2.0",
              method: "textDocument/publishDiagnostics",
              params: {
                uri,
                version,
                diagnostics: [
                  {
                    code: "typecheck-unavailable",
                    message:
                      "Type checking is unavailable in this workspace. Make sure `tsconfig.json` exists. Provide a native TypeScript compiler, for example `typescript-native@npm:typescript@7.1.0-beta`, or configure `typeChecker.corsaPath`, `CORSA_PATH`, or `TSGO_PATH`; see https://vizejs.dev/guide/static-analysis.",
                    range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
                    severity: 4,
                    source: "vize/types",
                  },
                ],
              },
            },
            "complete unavailable-provider diagnostic remains visible",
          );
        }
        assert.deepEqual(
          captureConsumerInputs(consumer),
          inputs,
          "all edits stay unsaved and authored disk remains exact",
        );
        profile.phases.push(phase);
      }
      wire.notify("textDocument/didClose", { textDocument: { uri } });
      profile.shutdown = await wire.request("shutdown");
      assert.equal(result(profile.shutdown), null);
      wire.notify("exit");
      await wire.end();
    } catch (error) {
      profile.failure = { message: String(error), stack: (error as { stack?: string }).stack };
    } finally {
      // A failed functional oracle remains failed, but its owned server still
      // receives the real shutdown protocol before administrative cleanup.
      if (!wire.status && !profile.shutdown) {
        try {
          profile.cleanupShutdown = await wire.request("shutdown");
          assert.equal(result(profile.cleanupShutdown), null);
          wire.notify("exit");
        } catch (error) {
          profile.cleanupProtocolFailure = String(error);
        }
      }
      try {
        if (!wire.status) await wire.end();
        else wire.save();
      } catch (error) {
        profile.closeFailure = String(error);
      }
      profile.outer = wire.save();
      try {
        assert.deepEqual(wire.status, { code: 0, signal: null });
        assert.equal(wire.pending.length, 0);
        assert.equal(wire.failure, null);
        assert.equal(wire.forcedTermination ?? false, false);
        const nativeCapture = captureNativeOwners(
          wire.prefix,
          path.join(directory, "native"),
          executionAuthority(repoRoot, source, consumer, environment),
        );
        profile.nativeOwners = nativeCapture.nativeOwners;
        profile.syscallObservation = {
          syscallsComplete: nativeCapture.syscallsComplete,
          terminalUnknownSyscalls: nativeCapture.terminalUnknownSyscalls,
          execInventory: nativeCapture.execInventory,
          qualifiedExecInventory: nativeCapture.qualifiedExecInventory,
        };
        profile.qualification = validateNativeLifecycle(
          profile.nativeOwners,
          consumer,
          knownContract,
        );
        profile.userWarnings = wire.events.filter(
          (event) =>
            event.direction === "server" &&
            event.packet.method === "window/showMessage" &&
            [1, 2].includes(event.packet.params?.type),
        );
        if (consumer.expectedExecutable)
          assert.deepEqual(
            profile.userWarnings,
            [],
            "a working selected compiler must not silently fall back after a provider warning",
          );
        profile.outerWarnings = Buffer.concat(wire.errors)
          .toString("utf8")
          .split("\n")
          .filter((line) => /\b(?:WARN|ERROR)\b/u.test(line));
        if (consumer.expectedExecutable)
          assert.deepEqual(
            profile.outerWarnings,
            [],
            "source warnings/errors are retained and never silently credited",
          );
        else profile.negativeProfileWarningsQualified = false;
        assertProviderFiles(providers);
        assert.equal(sha256(fs.readFileSync(source.executable)), source.receipt.binarySha256);
      } catch (error) {
        profile.auditFailure = {
          message: String(error),
          stack: (error as { stack?: string }).stack,
        };
      }
      profile.functionalQualified = Boolean(
        consumer.expectedExecutable &&
        !profile.failure &&
        !profile.closeFailure &&
        !profile.cleanupProtocolFailure &&
        !profile.auditFailure,
      );
      report.profiles.push(profile);
      write(path.join(output, "source-lsp-selection.json"), report);
    }
  }
  const failures = report.profiles.filter(
    (profile) =>
      profile.failure ||
      profile.closeFailure ||
      profile.cleanupProtocolFailure ||
      profile.auditFailure,
  );
  assert.equal(
    failures.length,
    0,
    JSON.stringify(
      failures.map((profile) => ({
        kind: profile.kind,
        failure: profile.failure,
        closeFailure: profile.closeFailure,
        cleanupProtocolFailure: profile.cleanupProtocolFailure,
        auditFailure: profile.auditFailure,
      })),
      null,
      2,
    ),
  );
  return report;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await observeSourceLspSelection();
