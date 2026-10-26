// Admit every executed image of a source LSP run and capture each native owner.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import type { Executable } from "./providers.ts";
import {
  decodeNativeFrames,
  decodeNativeIo,
  parseStraceCapture,
  sha,
  type ExecAttempt,
  type TraceOwner,
} from "./lsp-strace.ts";

/** The source, config and SDK images an LSP run may execute. */
export interface ExecAuthority {
  source: { path: string; sha256: string };
  node: { path: string; physicalPath: string; sha256: string };
  configScript: string;
  configFile: string;
  configCachePattern: RegExp;
  sdk: Pick<Executable, "path" | "sha256"> | null;
}

export function validateExecInventory<O extends Pick<TraceOwner, "path" | "argv" | "exit">>(
  capture: { owners: readonly O[]; attempts: readonly Pick<ExecAttempt, "result">[] },
  authority: ExecAuthority,
) {
  for (const attempt of capture.attempts)
    assert.equal(attempt.result, 0, "failed execution/discovery attempts remain explicit failures");
  let sourceCount = 0;
  const owners = capture.owners.map((owner) => {
    const physicalPath = fs.realpathSync(owner.path);
    const executable = {
      path: owner.path,
      physicalPath,
      sha256: sha(fs.readFileSync(physicalPath)),
    };
    let role: string;
    if (owner.path === authority.source.path) {
      assert.equal(executable.sha256, authority.source.sha256);
      assert.deepEqual(owner.argv, [authority.source.path, "lsp", "--stdio"]);
      sourceCount += 1;
      assert.equal(sourceCount, 1, "one original source CLI image");
      role = "source-lsp";
    } else if (owner.path === authority.node.path) {
      assert.equal(physicalPath, authority.node.physicalPath);
      assert.equal(executable.sha256, authority.node.sha256);
      assert.deepEqual(owner.argv.slice(0, 5), [
        "node",
        "--input-type=module",
        "-e",
        authority.configScript,
        authority.configFile,
      ]);
      assert.equal(owner.argv.length, 7, "exact source config observation recipe");
      assert.match(owner.argv[5], authority.configCachePattern);
      assert.equal(owner.argv[6], "--observe-config-inputs");
      role = "source-config-node";
    } else {
      assert.ok(authority.sdk, "negative profile permits no compiler/other helper image");
      assert.equal(
        physicalPath,
        authority.sdk.path,
        "only the selected physical provider may execute",
      );
      assert.equal(executable.sha256, authority.sdk.sha256);
      assert.deepEqual(
        owner.argv,
        [owner.path, "--lsp", "--stdio"],
        "only the exact observed native LSP recipe",
      );
      role = "selected-native-lsp";
    }
    // Selected-provider teardown has its separate exact image-specific law.
    if (role !== "selected-native-lsp")
      assert.deepEqual(
        owner.exit,
        { code: 0, signal: null },
        "every permitted source/config image closes successfully",
      );
    return { ...owner, executable, role };
  });
  assert.equal(sourceCount, 1);
  return owners;
}

export function captureNativeOwners(
  tracePrefix: string,
  directory: string,
  authority: ExecAuthority,
) {
  const parent = path.dirname(tracePrefix),
    prefix = path.basename(tracePrefix) + ".";
  const traces = fs
    .readdirSync(parent)
    .filter((file) => file.startsWith(prefix))
    .map((file) => {
      const suffix = file.slice(prefix.length);
      assert.match(suffix, /^\d+$/u);
      return { tid: Number(suffix), text: fs.readFileSync(path.join(parent, file), "utf8") };
    });
  assert.ok(traces.length > 0, "actual strace files are mandatory");
  fs.mkdirSync(directory, { recursive: true });
  const capture = parseStraceCapture(traces);
  const inventory = {
    attempts: capture.attempts,
    executions: capture.owners.map(({ rows: _rows, ...owner }) => owner),
    syscallsComplete: capture.syscallsComplete,
    terminalUnknownSyscalls: capture.terminalUnknownSyscalls,
  };
  fs.writeFileSync(
    path.join(directory, "exec-inventory.json"),
    JSON.stringify(inventory, null, 2) + "\n",
  );
  const allOwners = validateExecInventory(capture, authority);
  fs.writeFileSync(
    path.join(directory, "qualified-exec-inventory.json"),
    JSON.stringify(
      allOwners.map(({ rows: _rows, ...owner }) => owner),
      null,
      2,
    ) + "\n",
  );
  const owners = allOwners
    .filter((owner) => owner.role === "selected-native-lsp")
    .map(decodeNativeIo);
  const nativeOwners = owners.map((owner) => {
    assert.ok(owner.exit, "each native process has its own actual terminal status");
    const physicalPath = fs.realpathSync(owner.path);
    const stream = <F>(
      name: string,
      operation: string,
      fd: number,
      decode: (bytes: Buffer) => F,
    ) => {
      const bytes = Buffer.concat(
        owner.events
          .filter((event) => event.operation === operation && event.fd === fd && event.bytes)
          .map((event) => event.bytes ?? Buffer.alloc(0)),
      );
      const file = path.join(directory, `${owner.process}.${name}.bin`);
      fs.writeFileSync(file, bytes);
      return { path: file, bytes: bytes.length, sha256: sha(bytes), frames: decode(bytes) };
    };
    const streams = {
      client: stream("client", "read", 0, decodeNativeFrames),
      server: stream("server", "write", 1, decodeNativeFrames),
      stderr: stream("stderr", "write", 2, () => null),
    };
    const observation = {
      process: owner.process,
      executable: { path: owner.path, physicalPath, sha256: sha(fs.readFileSync(physicalPath)) },
      argv: owner.argv,
      exit: owner.exit,
      streams,
      ioFailures: owner.events.filter((event) => (event.count ?? 0) < 0),
      ioRestarts: owner.events.filter((event) => event.returnKind === "kernel-restart"),
      syscallsComplete: !owner.terminalUnknownSyscalls?.length,
      terminalUnknownSyscalls: owner.terminalUnknownSyscalls ?? [],
    };
    fs.writeFileSync(
      path.join(directory, `${owner.process}.json`),
      JSON.stringify(observation, null, 2) + "\n",
    );
    return observation;
  });
  return {
    nativeOwners,
    syscallsComplete: capture.syscallsComplete,
    terminalUnknownSyscalls: capture.terminalUnknownSyscalls,
    execInventory: path.join(directory, "exec-inventory.json"),
    qualifiedExecInventory: path.join(directory, "qualified-exec-inventory.json"),
  };
}

export type NativeOwner = ReturnType<typeof captureNativeOwners>["nativeOwners"][number];
