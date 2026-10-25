import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { DirectSdkWire, sha256 } from "./wire.mjs";
import { fileTree, isolatedEnvironment, loadProvider, runningImage } from "./provider.mjs";

const repo = fs.realpathSync(
  path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../.."),
);
const fixtures = path.join(repo, "tests/_fixtures/runtime/native-beta-eof-contract");
const write = (file, value) => fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
const json = (file) => JSON.parse(fs.readFileSync(file, "utf8"));
const authority = json(path.join(fixtures, "authority.json"));
const expectedSourceHash = "e6bcf16051e2cbcd3c65227f9d6c8d40bf457166c4469c8a95227b7687cb2a35";
const valid =
  'import { twice } from "./support";\nexport const value: number = 2;\nexport const doubled = twice(value);\n';
const invalid = valid.replace("value: number = 2", 'value: number = "wrong"');
const support = "export const twice = (value: number): number => value * 2;\n";
const ok = (event) => {
  assert.equal(event.packet.error, undefined, JSON.stringify(event.packet));
  return event.packet.result;
};

export function bindInitialize(original, workspace, pid) {
  const request = structuredClone(original);
  assert.equal(request.jsonrpc, "2.0");
  assert.equal(request.id, 1);
  assert.equal(request.method, "initialize");
  assert.equal(request.params.workspaceFolders.length, 1);
  request.params.processId = pid;
  request.params.rootPath = workspace;
  request.params.rootUri = pathToFileURL(workspace).href;
  request.params.workspaceFolders[0] = {
    ...request.params.workspaceFolders[0],
    uri: request.params.rootUri,
    name: path.basename(workspace),
  };
  const inverse = structuredClone(request);
  for (const key of ["processId", "rootPath", "rootUri"])
    inverse.params[key] = original.params[key];
  inverse.params.workspaceFolders[0].uri = original.params.workspaceFolders[0].uri;
  inverse.params.workspaceFolders[0].name = original.params.workspaceFolders[0].name;
  assert.deepEqual(inverse, original, "only declared project/PID transport fields relocate");
  return request;
}

export async function observe() {
  assert.equal(process.env.VIZE_REQUIRE_NATIVE_PROVIDER_CONTRACT, "1");
  assert.ok(process.env.VIZE_NATIVE_PROVIDER_ROOT);
  assert.equal(authority.productionInitializerSha256, expectedSourceHash);
  assert.equal(
    sha256(fs.readFileSync(path.join(repo, authority.productionInitializerPath))),
    expectedSourceHash,
  );
  for (const [name, digest] of Object.entries(authority.sealedFiles))
    assert.equal(sha256(fs.readFileSync(path.join(fixtures, name))), digest);
  const originalRequest = json(path.join(fixtures, "initialize-request.json"));
  const expectedResponse = json(path.join(fixtures, "initialize-response.json"));
  const evidence = path.join(repo, "target/vize-tests/native-beta-eof-contract");
  assert.equal(fs.existsSync(evidence), false, "each observation starts without earlier output");
  fs.mkdirSync(evidence, { recursive: true });
  const provider = loadProvider(process.env.VIZE_NATIVE_PROVIDER_ROOT, authority, evidence);
  const temporary = fs.realpathSync(
    fs.mkdtempSync(path.join(process.env.RUNNER_TEMP ?? os.tmpdir(), "native-beta-eof-")),
  );
  const report = {
    scope:
      "Independent direct exact Linux A183 SDK lifecycle; no source CLI, DTO, mapper or generic beta qualification",
    auditor: {
      node: process.execPath,
      nodeSha256: sha256(fs.readFileSync(process.execPath)),
      script: fileURLToPath(import.meta.url),
      revision: execFileSync("git", ["rev-parse", "HEAD"], { cwd: repo, encoding: "utf8" }).trim(),
      sourceInitializerSha256: expectedSourceHash,
      helperHashes: Object.fromEntries(
        ["observe.mjs", "provider.mjs", "wire.mjs"].map((name) => [
          name,
          sha256(fs.readFileSync(path.join(path.dirname(fileURLToPath(import.meta.url)), name))),
        ]),
      ),
    },
    authority,
    providerBefore: provider.receipt,
    sessions: [],
  };
  write(path.join(evidence, "report.json"), report);
  for (const mode of ["ack-info-eof-only", "standard-exit-observation"]) {
    const directory = path.join(evidence, mode),
      workspace = path.join(temporary, mode);
    fs.mkdirSync(path.join(workspace, "src"), { recursive: true });
    fs.mkdirSync(path.join(workspace, "node_modules"));
    fs.symlinkSync(provider.metaRoot, path.join(workspace, "node_modules/typescript"), "dir");
    fs.writeFileSync(
      path.join(workspace, "package.json"),
      '{"private":true,"devDependencies":{"typescript":"7.1.0-beta"}}\n',
    );
    fs.writeFileSync(
      path.join(workspace, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          strict: true,
          target: "ES2022",
          module: "ESNext",
          moduleResolution: "Bundler",
          skipLibCheck: true,
          noEmit: true,
        },
        include: ["src/**/*.ts"],
      }) + "\n",
    );
    const file = path.join(workspace, "src/main.ts"),
      uri = pathToFileURL(file).href;
    fs.writeFileSync(file, valid);
    fs.writeFileSync(path.join(workspace, "src/support.ts"), support);
    const inputs = fileTree(workspace),
      environment = isolatedEnvironment(path.join(temporary, mode + "-environment"));
    const session = {
      mode,
      workspace,
      authoredFiles: { valid, invalid, repair: valid, support },
      inputTree: inputs,
      childEnvironment: { HOME: environment.HOME, PATH: environment.PATH },
      phases: [],
    };
    const wire = new DirectSdkWire({
      executable: provider.executable,
      environment,
      workspace,
      directory,
    });
    try {
      session.providerBefore = provider.check();
      const request = bindInitialize(originalRequest, workspace, process.pid);
      session.initialize = await wire.requestExact(request);
      assert.deepEqual(
        session.initialize.packet,
        expectedResponse,
        "whole original A183 Initialize response",
      );
      session.runningImageBeforeWork = runningImage(wire.child.pid, provider);
      wire.notify("initialized", {});
      for (const [name, text, version] of [
        ["valid", valid, 1],
        ["error", invalid, 2],
        ["repair", valid, 3],
      ]) {
        if (version === 1)
          wire.notify("textDocument/didOpen", {
            textDocument: { uri, languageId: "typescript", version, text },
          });
        else
          wire.notify("textDocument/didChange", {
            textDocument: { uri, version },
            contentChanges: [{ text }],
          });
        const readiness = await wire.request("textDocument/documentSymbol", {
          textDocument: { uri },
        });
        ok(readiness);
        const response = await wire.request("textDocument/diagnostic", { textDocument: { uri } });
        const expected =
          name === "error"
            ? [
                {
                  range: { start: { line: 1, character: 13 }, end: { line: 1, character: 18 } },
                  severity: 1,
                  code: 2322,
                  source: "ts",
                  message: "Type 'string' is not assignable to type 'number'.",
                },
              ]
            : [];
        ok(response);
        assert.deepEqual(
          response.packet,
          { jsonrpc: "2.0", id: wire.nextId, result: { kind: "full", items: expected } },
          "complete genuine native diagnostic packet",
        );
        session.phases.push({
          name,
          version,
          textSha256: sha256(Buffer.from(text)),
          readiness,
          diagnostics: response,
        });
      }
      wire.notify("textDocument/didClose", { textDocument: { uri } });
      const beforeShutdown = wire.events.length;
      session.shutdown = await wire.request("shutdown");
      assert.deepEqual(session.shutdown.packet, { jsonrpc: "2.0", id: wire.nextId, result: null });
      session.currentInfo = await wire.wait(
        (packet) =>
          packet.method === "window/logMessage" &&
          packet.params?.type === 3 &&
          packet.params?.message?.startsWith(
            `handled method 'shutdown' (${session.shutdown.packet.id}) in `,
          ),
        "matching current shutdown Info",
        beforeShutdown,
      );
      assert.ok(
        session.currentInfo.index > session.shutdown.index,
        "Info is observed after the actual ACK",
      );
      assert.equal(wire.status, null);
      session.runningImageImmediatelyBeforeEof = runningImage(wire.child.pid, provider);
      assert.equal(wire.child.stdin.writableEnded, false);
      assert.equal(wire.child.stdout.readableEnded, false);
      if (mode === "standard-exit-observation") wire.notify("exit");
      await wire.end(mode);
      const observation = wire.save();
      assert.equal(observation.forcedKill, false);
      assert.equal(observation.status.signal, null);
      assert.equal(observation.failure, null);
      assert.equal(observation.pendingBytes, 0);
      assert.equal(observation.streams.stdin.framing, "complete");
      assert.equal(observation.streams.stdout.framing, "complete");
      for (const kind of [
        "stdin-finish",
        "stdin-close",
        "stdout-end",
        "stderr-end",
        "process-exit",
        "process-close",
      ])
        assert.equal(
          observation.events.filter((event) => event.kind === kind).length,
          1,
          `actual single ${kind}`,
        );
      const packets = observation.streams.stdin.frames.map((frame) => frame.packet);
      const serverPackets = observation.streams.stdout.frames.map((frame) => frame.packet);
      assert.equal(
        serverPackets.filter((packet) => packet.id === session.shutdown.packet.id && !packet.method)
          .length,
        1,
      );
      assert.equal(
        serverPackets.filter(
          (packet) =>
            packet.method === "window/logMessage" &&
            packet.params?.type === 3 &&
            packet.params?.message?.startsWith(
              `handled method 'shutdown' (${session.shutdown.packet.id}) in `,
            ),
        ).length,
        1,
      );
      const ended = observation.events.filter((event) => event.kind === "stdin-end-called");
      assert.equal(ended.length, 1);
      assert.ok(ended[0].index > session.currentInfo.index);
      assert.ok(
        observation.events.find((event) => event.kind === "stdout-end").index > ended[0].index,
      );
      assert.equal(packets.filter((packet) => packet.method === "shutdown").length, 1);
      assert.equal(
        packets.filter((packet) => packet.method === "exit").length,
        mode === "ack-info-eof-only" ? 0 : 1,
      );
      assert.deepEqual(fileTree(workspace), inputs);
      if (mode === "ack-info-eof-only") {
        assert.deepEqual(observation.status, { code: 0, signal: null });
        assert.equal(observation.streams.stderr.bytes, 0, "whole stderr is empty");
        session.strictEofOnlyImageContractQualified = true;
      } else session.closePolicyCredit = false;
    } catch (error) {
      session.failure = { message: String(error), stack: error.stack };
    } finally {
      try {
        if (wire.status === null) await wire.end("FAILED-observation-cleanup");
      } catch (error) {
        session.closeFailure = String(error);
      }
      session.raw = wire.save();
      try {
        session.providerAfter = provider.check();
      } catch (error) {
        session.providerFailure = String(error);
      }
      report.sessions.push(session);
      write(path.join(evidence, "report.json"), report);
    }
  }
  report.providerAfter = provider.check();
  write(path.join(evidence, "report.json"), report);
  assert.equal(
    report.sessions.filter(
      (session) => session.failure || session.closeFailure || session.providerFailure,
    ).length,
    0,
    JSON.stringify(
      report.sessions.map(({ mode, failure, closeFailure, providerFailure }) => ({
        mode,
        failure,
        closeFailure,
        providerFailure,
      })),
      null,
      2,
    ),
  );
  assert.equal(report.sessions[0].strictEofOnlyImageContractQualified, true);
  return report;
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await observe();
