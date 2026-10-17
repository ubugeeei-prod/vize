// Drive the packaged Emacs integration through real Eglot, with every native
// packet retained by the shared transparent tap. No simulated LSP connection.
import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { isDeepStrictEqual } from "node:util";
import { pathToFileURL } from "node:url";

import { resolveRealServerPath } from "../editor-e2e/real-vue-workspace.mjs";

const repository = path.resolve(import.meta.dirname, "../../../..");
const source =
  '<script setup>\nconst count = 1;\n</script>\n<template><Widget :value.sync="count" /></template>\n';
const rule = "vue/no-deprecated-v-bind-sync";

type Packet = {
  dir?: string;
  session?: number;
  msg?: {
    id?: number | string;
    method?: string;
    result?: unknown;
    params?: {
      initializationOptions?: unknown;
      capabilities?: { workspace?: { didChangeWatchedFiles?: { dynamicRegistration?: boolean } } };
      changes?: Array<{ uri: string; type: number }>;
      registrations?: Array<{ method: string }>;
    };
  };
  event?: string;
  code?: number;
};
type Publication = {
  uri: string;
  version?: number;
  diagnostics: Array<{ code?: string | number }>;
};
type Observation = {
  phase: string;
  legacy: Publication;
  modern: Publication;
  hover: unknown;
  wireCount: number;
  authoredConfig: { file: string; source: string } | null;
};

function shellQuote(value: string) {
  return `'${value.replaceAll("'", "'\\''")}'`;
}

/** Keep the original pipe before POSIX asynchronous-list stdin defaults apply. */
export function nativeProcessWrapper(server: string, pidFile: string) {
  return `#!/bin/sh\nexec 3<&0\n${shellQuote(server)} "$@" <&3 &\nnative=$!\nexec 3<&-\nprintf '%s\\n' "$native" > ${shellQuote(pidFile)}\nwait "$native"\n`;
}

/** A zombie has terminated even when the editor killed its launcher before reaping. */
export function nativeProcessStopped(pid: number): boolean {
  assert.ok(Number.isInteger(pid) && pid > 0, "Probe the recorded native child PID");
  try {
    process.kill(pid, 0);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ESRCH") return true;
    throw error;
  }
  if (process.platform === "linux") {
    try {
      const stat = fs.readFileSync(`/proc/${pid}/stat`, "utf8");
      return ["Z", "X"].includes(stat.slice(stat.lastIndexOf(")") + 2, stat.lastIndexOf(")") + 3));
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "ENOENT") return true;
      throw error;
    }
  }
  return false;
}

function prepareWorkspace(workspace: string) {
  fs.mkdirSync(workspace, { recursive: true });
  fs.writeFileSync(path.join(workspace, "package.json"), '{"private":true}\n');
  fs.writeFileSync(
    path.join(workspace, "vite.config.mjs"),
    'import vue from "./vue-policy.mjs";\nexport default { vue };\n',
  );
  fs.writeFileSync(
    path.join(workspace, "vue-policy.mjs"),
    `export default ${JSON.stringify({
      typeChecker: { enabled: false },
      languageServer: { typecheck: false, ecosystem: false, hover: false, lint: true },
      linter: { preset: "essential", rules: { [rule]: "error" } },
    })};\n`,
  );
  fs.writeFileSync(
    path.join(workspace, "tsconfig.json"),
    JSON.stringify({
      files: [],
      references: [{ path: "./tsconfig.legacy.json" }, { path: "./tsconfig.modern.json" }],
    }),
  );
  for (const [name, target] of [
    ["Legacy", 2.7],
    ["Modern", 3],
  ] as const) {
    fs.writeFileSync(path.join(workspace, `${name}.vue`), source);
    fs.writeFileSync(
      path.join(workspace, `tsconfig.${name.toLowerCase()}.json`),
      JSON.stringify({
        compilerOptions: { noEmit: true },
        include: [`${name}.vue`],
        vueCompilerOptions: { target },
      }),
    );
  }
  // Eglot's genuine project backend discovers all watcher directories here.
  execFileSync("git", ["init", "--quiet", workspace]);
  execFileSync("git", ["-C", workspace, "add", "."]);
}

function readPackets(file: string): Packet[] {
  return fs.existsSync(file)
    ? fs
        .readFileSync(file, "utf8")
        .split("\n")
        .filter(Boolean)
        .map((line) => JSON.parse(line))
    : [];
}

async function main() {
  const server = resolveRealServerPath();
  const sourceSha = execFileSync("git", ["rev-parse", "HEAD"], {
    cwd: repository,
    encoding: "utf8",
  }).trim();
  if (process.env.GITHUB_SHA) assert.equal(sourceSha, process.env.GITHUB_SHA);
  const binaryHash = createHash("sha256").update(fs.readFileSync(server)).digest("hex");
  const output = path.resolve(
    process.env.VIZE_EMACS_ARTIFACT_DIR ??
      path.join(repository, "target/editor-host/emacs-native-config"),
  );
  fs.mkdirSync(output, { recursive: true });
  const session = fs.mkdtempSync(path.join(os.tmpdir(), "vize-eglot-native-"));
  const workspace = path.join(session, "workspace");
  prepareWorkspace(workspace);
  const bin = path.join(session, "bin");
  fs.mkdirSync(bin);
  const nativePidFile = path.join(output, "native.pid");
  fs.rmSync(nativePidFile, { force: true });
  const nativeWrapper = path.join(bin, "native-server");
  fs.writeFileSync(nativeWrapper, nativeProcessWrapper(server, nativePidFile), { mode: 0o755 });
  fs.writeFileSync(
    path.join(bin, "vize"),
    `#!/bin/sh\nexec ${shellQuote(process.execPath)} ${shellQuote(
      path.join(repository, "tests/editor-conformance/lsp-tap.mjs"),
    )} ${shellQuote(nativeWrapper)} "$@"\n`,
    { mode: 0o755 },
  );
  const transcript = path.join(output, "transcript.jsonl");
  const casesFile = path.join(output, "cases.json");
  for (const file of [transcript, casesFile]) fs.rmSync(file, { force: true });
  fs.writeFileSync(
    path.join(output, "identity.json"),
    JSON.stringify(
      {
        server,
        sha256: binaryHash,
        sourceSha,
        client: "Emacs Eglot using editors/emacs/vize.el",
        workspace,
      },
      null,
      2,
    ),
  );
  const child = spawn(
    process.env.VIZE_TEST_EMACS_PATH ?? "emacs",
    [
      "-Q",
      "--batch",
      "-l",
      path.join(repository, "tests/editor-host/emacs-native-config.el"),
      "-f",
      "ert-run-tests-batch-and-exit",
    ],
    {
      cwd: workspace,
      env: {
        ...process.env,
        PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
        VIZE_EMACS_REPOSITORY: repository,
        VIZE_EMACS_WORKSPACE: workspace,
        VIZE_EMACS_CASES: casesFile,
        VIZE_CONFORMANCE_TRANSCRIPT: transcript,
      },
      stdio: ["ignore", "pipe", "pipe"],
      timeout: 210_000,
    },
  );
  let stdout = "";
  let stderr = "";
  child.stdout.on("data", (chunk) => {
    stdout += chunk;
    process.stdout.write(chunk);
  });
  child.stderr.on("data", (chunk) => {
    stderr += chunk;
    process.stderr.write(chunk);
  });
  let passed = false;
  const lifecycle: {
    shutdownAcknowledged: boolean;
    exitNotificationObserved: boolean;
    nativeStopped: boolean;
    nativeExitObservable: boolean;
    nativeExitCode: number | null;
  } = {
    shutdownAcknowledged: false,
    exitNotificationObserved: false,
    nativeStopped: false,
    nativeExitObservable: false,
    nativeExitCode: null,
  };
  try {
    const code = await new Promise<number | null>((resolve, reject) => {
      child.once("error", reject);
      child.once("close", resolve);
    });
    assert.equal(code, 0, "The actual Eglot ERT scenario must pass");
    let packets = readPackets(transcript);
    const nativePid = Number(fs.readFileSync(nativePidFile, "utf8").trim());
    assert.ok(
      Number.isInteger(nativePid) && nativePid > 0,
      "Actual native child PID must be recorded",
    );
    const shutdownIndex = packets.findIndex(
      (entry) => entry.dir === "c2s" && entry.msg?.method === "shutdown",
    );
    assert.ok(shutdownIndex >= 0, "The real Eglot client must send shutdown");
    const shutdown = packets[shutdownIndex];
    assert.ok(shutdown.msg?.id != null, "The actual shutdown request must carry its JSON-RPC ID");
    const acknowledgedIndex = packets.findIndex(
      (entry, index) =>
        index > shutdownIndex &&
        entry.session === shutdown.session &&
        entry.dir === "s2c" &&
        entry.msg?.id === shutdown.msg?.id &&
        entry.msg?.method == null &&
        Object.hasOwn(entry.msg ?? {}, "result") &&
        !Object.hasOwn(entry.msg ?? {}, "error") &&
        entry.msg?.result === null,
    );
    assert.ok(
      acknowledgedIndex > shutdownIndex,
      "The native server must acknowledge shutdown with null",
    );
    lifecycle.shutdownAcknowledged = true;
    assert.ok(
      packets.some(
        (entry, index) =>
          index > acknowledgedIndex &&
          entry.session === shutdown.session &&
          entry.dir === "c2s" &&
          entry.msg?.method === "exit",
      ),
      "The actual Eglot client must send exit after its matching shutdown acknowledgment",
    );
    lifecycle.exitNotificationObserved = true;
    for (let attempt = 0; attempt < 30 && !nativeProcessStopped(nativePid); attempt++)
      await sleep(100);
    lifecycle.nativeStopped = nativeProcessStopped(nativePid);
    assert.ok(
      lifecycle.nativeStopped,
      "The recorded native PID must stop after actual Eglot shutdown",
    );
    packets = readPackets(transcript);
    // Eglot deletes its own launcher after `exit'; its process-group kill can
    // prevent the transparent tap's recorder from observing a native exit code.
    const ledger = packets.find(
      (entry) => entry.session === shutdown.session && entry.event === "exit",
    );
    lifecycle.nativeExitObservable = ledger !== undefined;
    lifecycle.nativeExitCode = ledger?.code ?? null;
    if (ledger) assert.equal(ledger.code, 0, "An observed native exit must be clean");
    const initial = packets.find(
      (entry) => entry.dir === "c2s" && entry.msg?.method === "initialize",
    );
    assert.ok(initial, "Eglot must initialize the real server");
    const initializationOptions = initial.msg!.params?.initializationOptions;
    assert.ok(
      initializationOptions == null || isDeepStrictEqual(initializationOptions, {}),
      "Command-only Eglot may send no initialization overrides, null, or an empty object",
    );
    const cases = JSON.parse(fs.readFileSync(casesFile, "utf8")) as Observation[];
    const expected = [
      ["initial-reference-targets-and-vite-hover", false, false],
      ["vite-valid-hover-enabled", false, true],
      ["vite-invalid-retains-hover-enabled", false, true],
      ["vite-valid-hover-recovery", false, false],
      ["reference-valid-target-3", true, false],
      ["reference-invalid-retains-target-3", true, false],
      ["reference-valid-target-recovery", false, false],
    ] as const;
    assert.equal(cases.length, expected.length);
    assert.equal(
      initial.msg!.params?.capabilities?.workspace?.didChangeWatchedFiles?.dynamicRegistration,
      true,
      "The actual Eglot host must advertise file watching",
    );
    const registration = packets.find(
      (entry) =>
        entry.dir === "s2c" &&
        entry.msg?.method === "client/registerCapability" &&
        entry.msg.params?.registrations?.some(
          (item) => item.method === "workspace/didChangeWatchedFiles",
        ),
    );
    assert.ok(registration, "The native server must register actual client watchers");
    assert.ok(
      packets.some(
        (entry) =>
          entry.dir === "c2s" &&
          entry.session === registration.session &&
          entry.msg?.id === registration.msg?.id &&
          Object.hasOwn(entry.msg ?? {}, "result"),
      ),
      "The genuine Eglot registration must be acknowledged",
    );
    let previousWireCount = 0;
    for (const [index, [name, deprecated, hoverEnabled]] of expected.entries()) {
      const observation = cases[index];
      assert.equal(observation.phase, name);
      assert.equal(
        observation.legacy.diagnostics.some((item) => item.code === rule),
        deprecated,
        name,
      );
      assert.ok(
        observation.modern.diagnostics.some((item) => item.code === rule),
        name,
      );
      assert.deepEqual(
        observation.modern.diagnostics,
        cases[0].modern.diagnostics,
        `${name}: the modern sibling's complete diagnostics stay stable`,
      );
      assert.equal(observation.hover !== null, hoverEnabled, name);
      const seen = packets.slice(0, observation.wireCount);
      for (const publication of [observation.legacy, observation.modern]) {
        assert.ok(
          seen.some(
            (entry) =>
              entry.dir === "s2c" &&
              entry.msg?.method === "textDocument/publishDiagnostics" &&
              isDeepStrictEqual(entry.msg.params, publication),
          ),
          `${name}: case must preserve a complete native publication`,
        );
      }
      if (index > 0) {
        assert.ok(
          observation.authoredConfig,
          `${name}: the authored config change must be retained`,
        );
        assert.equal(
          observation.authoredConfig.file,
          index <= 3 ? "vue-policy.mjs" : "tsconfig.legacy.json",
          `${name}: mutate the imported helper or referenced TS project, not the root Vite config`,
        );
        const uri = new URL(observation.authoredConfig.file, `${pathToFileURL(workspace).href}/`)
          .href;
        assert.ok(
          packets
            .slice(previousWireCount, observation.wireCount)
            .some(
              (entry) =>
                entry.dir === "c2s" &&
                entry.msg?.method === "workspace/didChangeWatchedFiles" &&
                entry.msg.params?.changes?.some((change) => change.uri === uri),
            ),
          `${name}: Eglot itself must send the corresponding file notification`,
        );
      }
      previousWireCount = observation.wireCount;
    }
    assert.ok(
      packets.some((entry) => entry.event === "spawn"),
      "A real server must start behind the tap",
    );
    passed = true;
    console.log(`Emacs Eglot native configuration passed; complete evidence: ${output}`);
  } finally {
    fs.writeFileSync(
      path.join(output, "receipt.json"),
      JSON.stringify(
        {
          passed,
          sourceSha,
          lifecycle,
          binary: {
            path: server,
            sha256: binaryHash,
            nativePid: fs.existsSync(nativePidFile)
              ? Number(fs.readFileSync(nativePidFile, "utf8").trim())
              : null,
          },
          transcript: {
            path: transcript,
            sha256: fs.existsSync(transcript)
              ? createHash("sha256").update(fs.readFileSync(transcript)).digest("hex")
              : null,
          },
          cases: fs.existsSync(casesFile) ? JSON.parse(fs.readFileSync(casesFile, "utf8")) : [],
        },
        null,
        2,
      ),
    );
    fs.writeFileSync(path.join(output, "emacs.stdout.log"), stdout);
    fs.writeFileSync(path.join(output, "emacs.stderr.log"), stderr);
    fs.cpSync(workspace, path.join(output, "fixture"), {
      recursive: true,
      filter: (name) => path.basename(name) !== ".git",
    });
    fs.rmSync(session, { recursive: true, force: true });
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  await main();
}
