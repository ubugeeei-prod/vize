import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const directory = join(resolve(process.argv[2]), "actual-native-handshake");
mkdirSync(directory);
const project = mkdtempSync(join(directory, "project-"));
writeFileSync(join(project, "tsconfig.json"), '{"compilerOptions":{"strict":true},"files":[]}\n');
const captureProvider = process.argv[3] ? JSON.parse(readFileSync(process.argv[3], "utf8")) : null;
if (captureProvider) {
  assert.equal(
    captureProvider.sha256,
    "77dd575e73107c00cc95cbccc3764ab7210089b5634f75e219b61632bd5d230b",
  );
  assert.equal(captureProvider.version, "7.1.0-dev");
  assert.equal(captureProvider.serverName, "typescript");
  assert.equal(captureProvider.upstream, "d6c4afddb2c55f4a9dea7b59293a99a8fdea1799");
}
const executable = realpathSync(
  captureProvider?.binary ?? "node_modules/@typescript/typescript-linux-x64/lib/tsc",
);
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
assert.equal(
  sha256(readFileSync(executable)),
  captureProvider?.sha256 ?? "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4",
);
const shutdownSource = readFileSync(
  "crates/vize_canon/src/lsp_client/editor_lsp/native_shutdown.rs",
  "utf8",
);
const contractMatch =
  /fn known_contract\(\) -> Value \{\s*serde_json::json!\(([\s\S]*?)\)\s*\}/.exec(shutdownSource);
assert(contractMatch, "the complete known contract must remain an explicit JSON value");
const expectedInitialize = captureProvider ? null : JSON.parse(contractMatch[1]);
const rootUri = pathToFileURL(project).href;
// Mirror diagnostics_lsp::initialize_lsp_params without changing logging.
// Preserve these complete parameters and the original source hash as custody.
const params = {
  processId: process.pid,
  rootPath: project,
  rootUri,
  capabilities: {
    textDocument: {
      hover: { contentFormat: ["markdown", "plaintext"] },
      completion: { completionItem: { documentationFormat: ["markdown", "plaintext"] } },
      signatureHelp: { signatureInformation: { documentationFormat: ["markdown", "plaintext"] } },
      diagnostic: {
        dynamicRegistration: false,
        relatedDocumentSupport: true,
        relatedInformation: true,
      },
    },
    workspace: {
      didChangeWatchedFiles: { dynamicRegistration: true, relativePatternSupport: true },
      diagnostic: { refreshSupport: true },
    },
  },
  initializationOptions: {
    userPreferences: {
      tsserver: { automaticTypeAcquisition: { enabled: false } },
      inlayHints: { variableTypes: { enabled: true } },
    },
  },
  workspaceFolders: [{ uri: rootUri, name: project.split("/").at(-1) }],
};

const child = spawn(executable, ["--lsp", "--stdio"], { cwd: project, stdio: "pipe" });
const sent = [];
const stdout = [];
const stderr = [];
const packets = [];
const waiters = new Set();
let pending = Buffer.alloc(0);
let failure;
let cursor = 0;
let exitObservation;
const notify = (error) => {
  failure ??= error;
  for (const wake of waiters) wake();
};
const exit = new Promise((accept) =>
  child.once("close", (code, signal) => {
    exitObservation = { code, signal };
    accept(exitObservation);
    notify();
  }),
);
child.on("error", notify);
child.stdin.on("error", notify);
child.stderr.on("data", (bytes) => stderr.push(Buffer.from(bytes)));

function send(packet) {
  const body = Buffer.from(JSON.stringify(packet));
  const frame = Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
  sent.push(frame);
  packets.push({ direction: "sent", wholePacket: packet });
  child.stdin.write(frame);
}

child.stdout.on("data", (bytes) => {
  stdout.push(Buffer.from(bytes));
  pending = Buffer.concat([pending, bytes]);
  try {
    while (true) {
      const boundary = pending.indexOf("\r\n\r\n");
      if (boundary < 0) break;
      const headers = pending.subarray(0, boundary).toString("ascii").split("\r\n");
      const lengths = headers.filter((header) => /^content-length:/i.test(header));
      assert.equal(lengths.length, 1);
      const length = Number(lengths[0].slice(lengths[0].indexOf(":") + 1));
      assert(Number.isSafeInteger(length) && length >= 0);
      if (pending.length < boundary + 4 + length) break;
      const packet = JSON.parse(
        pending.subarray(boundary + 4, boundary + 4 + length).toString("utf8"),
      );
      pending = pending.subarray(boundary + 4 + length);
      packets.push({ direction: "received", wholePacket: packet });
      if (packet.method && packet.id !== undefined) {
        // The original responder uses positional null configuration entries
        // and null for other server-initiated requests.
        const result =
          packet.method === "workspace/configuration"
            ? (packet.params?.items ?? []).map(() => null)
            : null;
        send({ jsonrpc: "2.0", id: packet.id, result });
      }
    }
    notify();
  } catch (error) {
    notify(error);
  }
});

async function receive(predicate, operation, timeout) {
  const deadline = performance.now() + timeout;
  while (true) {
    if (failure) throw failure;
    while (cursor < packets.length) {
      const event = packets[cursor++];
      if (event.direction === "received" && predicate(event.wholePacket)) return event.wholePacket;
    }
    assert.equal(exitObservation, undefined, `native exited before ${operation}`);
    const remaining = deadline - performance.now();
    assert(remaining > 0, `${operation} timed out after ${timeout} ms`);
    await new Promise((accept, reject) => {
      const timer = setTimeout(() => {
        waiters.delete(wake);
        reject(new Error(`${operation} timed out after ${timeout} ms`));
      }, remaining);
      const wake = () => {
        clearTimeout(timer);
        waiters.delete(wake);
        accept();
      };
      waiters.add(wake);
    });
  }
}

async function closeOwnedProcess() {
  child.stdin.end();
  let timer;
  try {
    return await Promise.race([
      exit,
      new Promise((_, reject) => {
        timer = setTimeout(
          () => reject(new Error("native process close timed out after 2000 ms")),
          2000,
        );
      }),
    ]);
  } finally {
    clearTimeout(timer);
    if (!exitObservation) {
      child.kill("SIGKILL");
      await exit;
    }
  }
}

let observation = {};
try {
  send({ jsonrpc: "2.0", id: 1, method: "initialize", params });
  const initialized = await receive(
    (packet) => packet.id === 1 && !packet.method,
    "initialize",
    30000,
  );
  observation = { ...observation, initialized };
  if (captureProvider) {
    assert.equal(initialized.jsonrpc, "2.0");
    assert.equal(initialized.id, 1);
    assert.equal(initialized.error, undefined);
    assert.equal(initialized.result?.serverInfo?.name, captureProvider.serverName);
    assert.equal(initialized.result?.serverInfo?.version, captureProvider.version);
    assert(initialized.result?.capabilities && typeof initialized.result.capabilities === "object");
  } else {
    assert.deepEqual(initialized, { jsonrpc: "2.0", id: 1, result: expectedInitialize });
  }
  observation = {
    ...observation,
    completeInitializeMatchesKnownContract: !captureProvider,
    completeInitializeCaptured: true,
    productAdmissionChanged: false,
    initialized,
  };
  send({ jsonrpc: "2.0", method: "initialized", params: {} });
  send({ jsonrpc: "2.0", id: 2, method: "shutdown" });
  const acknowledgement = await receive(
    (packet) => packet.id === 2 && !packet.method,
    "shutdown acknowledgement",
    2000,
  );
  assert.deepEqual(acknowledgement, { jsonrpc: "2.0", id: 2, result: null });
  const completion = await receive(
    (packet) =>
      packet.method === "window/logMessage" &&
      packet.params?.type === 3 &&
      /^handled method 'shutdown' \(2\) in /.test(packet.params?.message),
    "shutdown completion",
    2000,
  );
  observation = { ...observation, acknowledgement, completion };
} finally {
  try {
    observation.exit = await closeOwnedProcess();
  } finally {
    writeFileSync(join(directory, "stdin.bin"), Buffer.concat(sent));
    writeFileSync(join(directory, "stdout.bin"), Buffer.concat(stdout));
    writeFileSync(join(directory, "stderr.bin"), Buffer.concat(stderr));
    writeFileSync(join(directory, "whole-packets.json"), JSON.stringify(packets, null, 2) + "\n");
    writeFileSync(
      join(directory, "observation.json"),
      JSON.stringify(
        {
          source: process.env.SOURCE_SHA,
          captureProvider,
          executable: {
            path: executable,
            sha256: sha256(readFileSync(executable)),
            argv: ["--lsp", "--stdio"],
          },
          initializationSourceSha256: sha256(
            readFileSync("crates/vize_canon/src/lsp_client/diagnostics_lsp.rs"),
          ),
          shutdownSourceSha256: sha256(shutdownSource),
          params,
          expectedInitialize,
          observation,
        },
        null,
        2,
      ) + "\n",
    );
  }
}
assert.deepEqual(observation.exit, { code: 0, signal: null });
assert.equal(pending.length, 0);
assert.equal(Buffer.concat(stderr).length, 0, "complete native handshake stderr must remain empty");
console.log("authenticated native Initialize + Shutdown completion + EOF: pass");
