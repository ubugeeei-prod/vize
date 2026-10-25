import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
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
} from "./providers.mjs";
import {
  BUILD_RECIPE,
  binaryRelativePath,
  expectedBuildIdentity,
  validateBuildReceipt,
} from "../../../differential/build-receipt.ts";
import { SourceLspWire } from "./lsp-wire.mjs";
import { captureNativeOwners } from "./lsp-strace.mjs";

const write = (file, value) => fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
const repoRoot = fs.realpathSync(
  path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../.."),
);
const result = (response) => {
  assert.equal(response.error, undefined, JSON.stringify(response));
  return response.result;
};
const position = (text, offset) => {
  const lines = text.slice(0, offset).split("\n");
  return { line: lines.length - 1, character: lines.at(-1).length };
};
const range = (text, offset, length) => ({
  start: position(text, offset),
  end: position(text, offset + length),
});

function sourceIdentity() {
  assert.equal(process.env.VIZE_REQUIRE_NATIVE_RUNTIME_SELECTION, "1");
  assert.equal(
    process.platform,
    "linux",
    "actual native syscall capture requires its Linux Actions host",
  );
  assert.equal(
    process.arch,
    "x64",
    "the qualified 7.0.2 image is the explicitly owned Linux x64 provider",
  );
  const requested = process.env.VIZE_SOURCE_CLI ?? process.env.SOURCE_CLI;
  assert.ok(requested && path.isAbsolute(requested));
  if (process.env.VIZE_SOURCE_CLI && process.env.SOURCE_CLI)
    assert.equal(
      fs.realpathSync(process.env.VIZE_SOURCE_CLI),
      fs.realpathSync(process.env.SOURCE_CLI),
    );
  const executable = fs.realpathSync(requested);
  assert.equal(
    executable,
    fs.realpathSync(path.join(repoRoot, binaryRelativePath())),
    "only the original installed canonical source artifact is admitted",
  );
  const receiptFile = `${executable}.differential-build.json`;
  const receipt = JSON.parse(fs.readFileSync(receiptFile, "utf8"));
  const expected = expectedBuildIdentity(repoRoot);
  validateBuildReceipt(receipt, expected, BUILD_RECIPE);
  const probe = spawnSync(executable, ["--version"], { encoding: null });
  assert.equal(probe.error, undefined);
  assert.equal(probe.status, 0);
  assert.equal(probe.signal, null);
  assert.equal(probe.stdout.toString().trim(), receipt.cliVersion);
  assert.equal(probe.stderr.length, 0);
  const observer = {
    file: path.relative(repoRoot, import.meta.filename),
    sha256: sha256(fs.readFileSync(import.meta.filename)),
    revision: receipt.sourceRevision,
  };
  assert.equal(
    sha256(
      execFileSync("git", ["show", `${observer.revision}:${observer.file}`], { cwd: repoRoot }),
    ),
    observer.sha256,
  );
  return {
    executable,
    receiptFile,
    receipt,
    observer,
    versionProbe: {
      stdoutBase64: probe.stdout.toString("base64"),
      stderrBase64: probe.stderr.toString("base64"),
      status: probe.status,
      signal: probe.signal,
    },
  };
}

function executionAuthority(source, consumer, environment) {
  const loader = path.join(repoRoot, "crates/vize_carton/src/config/loader");
  const rust = fs.readFileSync(path.join(loader, "js.rs"), "utf8");
  const embedded =
    /let script = \[\s*include_str!\("vite-runtime\.mjs"\),\s*include_str!\("module-runtime\.mjs"\),\s*r#"([\s\S]*?)"#,\s*\]\s*\.concat\(\);/u.exec(
      rust,
    );
  assert.ok(embedded, "exact current source config script recipe");
  assert.match(
    fs.readFileSync(path.join(loader, "lsp_snapshot.rs"), "utf8"),
    /builder\.prefix\("vize-config-compile-"\)/u,
  );
  const escape = (value) => value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  return {
    source: { path: source.executable, sha256: source.receipt.binarySha256 },
    node: {
      path: path.join(environment.PATH, "node"),
      physicalPath: fs.realpathSync(process.execPath),
      sha256: sha256(fs.readFileSync(process.execPath)),
    },
    configScript:
      fs.readFileSync(path.join(loader, "vite-runtime.mjs"), "utf8") +
      fs.readFileSync(path.join(loader, "module-runtime.mjs"), "utf8") +
      embedded[1],
    configFile: fs.realpathSync(consumer.configFile),
    configCachePattern: new RegExp(
      `^${escape(environment.TMPDIR ?? os.tmpdir())}/vize-config-compile-[A-Za-z0-9]+$`,
      "u",
    ),
    sdk: consumer.expectedExecutable,
  };
}

function exactHover(packet, source, token) {
  const offset =
    token === "value" ? source.indexOf("const value") + 6 : source.indexOf("{{ doubled }}") + 3;
  const expected = {
    contents: { kind: "markdown", value: `\`\`\`typescript\nconst ${token}: number\n\`\`\`` },
    range: range(source, offset, token.length),
  };
  assert.deepEqual(result(packet), expected, `complete ${token} hover`);
}

function validateNativeLifecycle(owners, consumer, knownContract) {
  const positive = consumer.expectedExecutable !== null;
  if (!positive) {
    assert.equal(owners.length, 0, "no undeclared, bundled, cached or global compiler may run");
    return { selectedProvider: false, nativeCleanExitQualified: false, semanticCredit: false };
  }
  assert.ok(
    owners.length > 0,
    "real compiler use must be observed independently of outer responses",
  );
  const sdk = consumer.expectedExecutable;
  for (const owner of owners) {
    assert.equal(owner.executable.physicalPath, sdk.path);
    assert.equal(owner.executable.sha256, sdk.sha256);
    assert.equal(owner.exit.signal, null);
  }
  const lsp = owners.filter((owner) => owner.argv.includes("--lsp"));
  assert.ok(lsp.length > 0, "the selected actual provider must serve LSP");
  const packets = (owner) => ({
    client: owner.streams.client.frames.map((frame) => frame.packet),
    server: owner.streams.server.frames.map((frame) => frame.packet),
  });
  let realWrongBuffer = false,
    realTypeError = false;
  for (const owner of lsp) {
    const { client, server } = packets(owner);
    realWrongBuffer ||= client.some(
      (packet) =>
        (packet.method === "textDocument/didOpen" &&
          packet.params?.textDocument?.text?.includes('"wrong"')) ||
        (packet.method === "textDocument/didChange" &&
          packet.params?.contentChanges?.some((change) => change.text?.includes('"wrong"'))),
    );
    realTypeError ||= server.some((packet) =>
      packet.result?.items?.some((diagnostic) => diagnostic.code === 2322),
    );
    const initialized = server.filter(
      (packet) =>
        packet.id === client.find((request) => request.method === "initialize")?.id &&
        !packet.method,
    );
    assert.equal(initialized.length, 1);
    assert.equal(initialized[0].error, undefined);
    if (consumer.kind.endsWith("702")) {
      assert.deepEqual(
        initialized[0].result,
        knownContract,
        "exact qualified image retains its complete source-owned Initialize contract",
      );
      const shutdown = client.filter((packet) => packet.method === "shutdown");
      assert.equal(shutdown.length, 1);
      assert.deepEqual(
        server.find((packet) => !packet.method && packet.id === shutdown[0].id),
        { jsonrpc: "2.0", id: shutdown[0].id, result: null },
      );
      assert.ok(
        server.some(
          (packet) =>
            packet.method === "window/logMessage" &&
            packet.params?.type === 3 &&
            packet.params?.message?.startsWith(`handled method 'shutdown' (${shutdown[0].id}) in `),
        ),
        "actual current shutdown completion",
      );
      assert.equal(
        client.some((packet) => packet.method === "exit"),
        false,
        "qualified EOF closure does not borrow the standard Exit path",
      );
      assert.deepEqual(owner.exit, { code: 0, signal: null });
      assert.equal(fs.readFileSync(owner.streams.stderr.path).length, 0);
    } else {
      const stderr = fs.readFileSync(owner.streams.stderr.path, "utf8");
      assert.ok(
        (owner.exit.code === 0 && stderr === "") ||
          (owner.exit.code === 1 && stderr === "context canceled\n") ||
          (owner.exit.code === 0 && stderr === "error handling method 'exit': EOF\n"),
        `unrecognized beta terminal behavior: ${owner.exit.code} ${JSON.stringify(stderr)}`,
      );
    }
  }
  assert.ok(realWrongBuffer, "actual selected LSP receives the genuine unsaved error text");
  assert.ok(realTypeError, "actual selected native LSP returns the genuine type error");
  const apiErrors = owners
    .filter((owner) => owner.argv.includes("--api"))
    .flatMap((owner) =>
      owner.streams.server.frames
        .filter((frame) => frame.packet.error)
        .map((frame) => frame.packet),
    );
  for (const owner of owners.filter((owner) => owner.argv.includes("--api"))) {
    assert.deepEqual(owner.exit, { code: 0, signal: null });
    assert.equal(fs.readFileSync(owner.streams.stderr.path).length, 0);
  }
  return {
    selectedProvider: true,
    nativeCleanExitQualified: consumer.kind.endsWith("702"),
    betaStandardExitFrontier: !consumer.kind.endsWith("702"),
    apiCompatibilityCredit: false,
    actualApiErrors: apiErrors,
    actualNativeSemanticTraffic: true,
    semanticCredit: false,
  };
}

export async function observeSourceLspSelection() {
  const source = sourceIdentity(),
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

  const report = {
    source,
    strace: {
      path: strace,
      sha256: sha256(fs.readFileSync(strace)),
      version: execFileSync(strace, ["--version"], { encoding: "utf8" }),
    },
    providers,
    shutdownSourceSha256: sha256(Buffer.from(nativeSource)),
    profiles: [],
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
  ]) {
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
    const profile = {
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
      ]) {
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
        const phase = { name, version, authoredSha256: sha256(Buffer.from(text)) };
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
          for (const token of ["value", "doubled"]) {
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
      profile.failure = { message: String(error), stack: error.stack };
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
          executionAuthority(source, consumer, environment),
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
        profile.auditFailure = { message: String(error), stack: error.stack };
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
