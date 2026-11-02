import assert from "node:assert/strict";
import { spawnSync, type SpawnSyncOptionsWithBufferEncoding } from "node:child_process";
import { lstatSync, readFileSync, readdirSync, realpathSync, writeFileSync } from "node:fs";
import { join, sep } from "node:path";
import { sha256, type SourceFile, type State } from "./contract.ts";
import { childEnvironment } from "../public-typechecker-11317/execute.ts";

export interface Image {
  path: string;
  bytes: number;
  sha256: string;
}
export interface Recorder {
  root: string;
  json: (name: string, value: unknown) => void;
  raw: (name: string, value: Buffer) => Image;
}
export function recorder(root: string): Recorder {
  const owned = new Set<string>();
  const target = (name: string) => {
    assert(/^[A-Za-z0-9._-]+$/u.test(name));
    return join(root, name);
  };
  return {
    root,
    json(name, value) {
      const path = target(name);
      if (owned.has(name))
        assert(
          lstatSync(path).isFile() &&
            !lstatSync(path).isSymbolicLink() &&
            realpathSync(path) === path,
        );
      writeFileSync(path, JSON.stringify(value, null, 2) + "\n", {
        flag: owned.has(name) ? "w" : "wx",
      });
      owned.add(name);
    },
    raw(name, value) {
      const path = target(name);
      writeFileSync(path, value, { flag: "wx" });
      return { path, bytes: value.length, sha256: sha256(value) };
    },
  };
}
export function safeFailure(error: unknown) {
  return {
    name: error instanceof assert.AssertionError ? "AssertionError" : "Error",
    message: "owned prerequisite, process or final custody refused; private exception text omitted",
  };
}
export function inputImages(root: string, files: SourceFile[]) {
  assert(
    realpathSync(root) === root && !lstatSync(root).isSymbolicLink(),
    "canonical physical project required",
  );
  const directory = join(root, "src");
  assert(
    lstatSync(directory).isDirectory() &&
      !lstatSync(directory).isSymbolicLink() &&
      realpathSync(directory) === directory,
    "canonical physical src required",
  );
  assert.deepEqual(
    readdirSync(directory).sort(),
    files
      .filter((file) => file.path.startsWith("src/"))
      .map((file) => file.path.slice(4))
      .sort(),
    "included source population differs",
  );
  return files.map((file) => {
    const path = join(root, file.path),
      stat = lstatSync(path);
    assert(
      path.startsWith(root + sep) &&
        stat.isFile() &&
        !stat.isSymbolicLink() &&
        realpathSync(path) === path,
      "owned input must be a regular canonical physical file",
    );
    const raw = readFileSync(path);
    assert(raw.equals(Buffer.from(file.source)), "owned source bytes differ: " + file.path);
    return { path: file.path, bytes: raw.length, sha256: sha256(raw), source: file.source };
  });
}
export interface ProcessOutput {
  status: number | null;
  signal: string | null;
  pid: number | null;
  error: string | null;
  stdout: Buffer;
  stderr: Buffer;
  capacityLimited: boolean;
}
export type Runner = (argv: string[], cwd: string) => ProcessOutput;
export function spawnOptions(
  cwd: string,
  environment: NodeJS.ProcessEnv = process.env,
): SpawnSyncOptionsWithBufferEncoding {
  return {
    cwd,
    env: childEnvironment(environment),
    timeout: 600_000,
    maxBuffer: 4 * 1024 ** 2,
    encoding: "buffer",
    stdio: ["ignore", "pipe", "pipe"],
  };
}
export const runProcess: Runner = (argv, cwd) => {
  const output = spawnSync(argv[0]!, argv.slice(1), spawnOptions(cwd));
  const capacityLimited =
    output.error !== undefined && "code" in output.error && output.error.code === "ENOBUFS";
  return {
    status: output.status,
    signal: output.signal,
    pid: output.pid > 0 ? output.pid : null,
    error: output.error ? "PROCESS_ERROR_RAW_PARTIAL" : null,
    stdout: output.stdout ?? Buffer.alloc(0),
    stderr: output.stderr ?? Buffer.alloc(0),
    capacityLimited,
  };
};
function decode(raw: Buffer) {
  try {
    return new TextDecoder("utf8", { fatal: true }).decode(raw);
  } catch {
    return null;
  }
}
export function capture(
  record: Recorder,
  label: string,
  argv: string[],
  cwd: string,
  verify: () => void,
  run: Runner = runProcess,
) {
  const startedAt = new Date().toISOString();
  let executed = false,
    prerequisite: ReturnType<typeof safeFailure> | null = null;
  let output: ProcessOutput = {
    status: null,
    signal: null,
    pid: null,
    error: "PREREQUISITE_REFUSED",
    stdout: Buffer.alloc(0),
    stderr: Buffer.alloc(0),
    capacityLimited: false,
  };
  try {
    verify();
  } catch (error) {
    prerequisite = safeFailure(error);
  }
  if (!prerequisite) {
    executed = true;
    try {
      output = run(argv, cwd);
    } catch {
      output = { ...output, error: "LAUNCH_OR_CAPTURE_UNKNOWN", capacityLimited: true };
    }
  }
  // Both byte streams are persisted before decoding or any exit/semantic check.
  const stdoutImage = record.raw(label + ".stdout.raw", output.stdout),
    stderrImage = record.raw(label + ".stderr.raw", output.stderr);
  const packet = {
    argv,
    cwd,
    startedAt,
    completedAt: new Date().toISOString(),
    executed,
    prerequisite,
    status: output.status,
    signal: output.signal,
    processError: output.error,
    pid: output.pid,
    stdout: decode(output.stdout),
    stderr: decode(output.stderr),
    stdoutImage,
    stderrImage,
    wholeStreamsRetained: !output.capacityLimited && output.error === null,
    capacityLimited: output.capacityLimited,
    selectedCompilerPid: null,
    selectedCompilerImage: "UNKNOWN",
    backend: "UNKNOWN",
    qualification: false,
  };
  record.json(label + ".process.json", packet);
  return packet;
}
export type Packet = ReturnType<typeof capture>;
export function whole(packet: Packet) {
  assert(
    packet.executed && packet.prerequisite === null && packet.wholeStreamsRetained,
    "whole executed process required",
  );
  assert.equal(packet.processError, null);
  assert.equal(packet.signal, null);
  assert(
    Number.isSafeInteger(packet.status) && Number(packet.status) >= 0,
    "exit status must be observed",
  );
  assert(
    packet.stdout !== null && packet.stderr !== null,
    "valid UTF8 streams required; raw remains retained",
  );
}
export function healthy(packet: Packet, status: number) {
  whole(packet);
  assert.equal(packet.status, status);
}
export function assessTsrs(state: State, packet: Packet) {
  healthy(packet, state.expectedTsrs.status);
  if (state.expectedTsrs.stdoutKind === "OWNED_HEADER_OPAQUE_PRINTER") {
    const inputs: Record<string, string> = {
      "11390/typed-payload-wrong":
        "9135e54398a06920714a10736504841e50648be8eedb14f4c22c73ee1505ef07",
      "11391/runtime-payload-wrong":
        "d96a0cf93f878e691cd0c53d25566f8c0d2c0ce6e9319f7596d032df98e374f7",
    };
    assert(Object.hasOwn(inputs, state.id), "closed listener intent required");
    assert.equal(state.expectedTsrs.sourceInputSha256, inputs[state.id]);
    assert.equal(sha256(JSON.stringify(state.files)), inputs[state.id]);
    assert.equal(state.expectedTsrs.status, 2);
    assert.equal(state.expectedTsrs.stderr, "");
    const header = "src/Listener.ts(3,7): error TS2322";
    assert.equal(state.expectedTsrs.stdout, header);
    assert(packet.stderr === "", "whole listener stderr differs");
    const stdout = packet.stdout!;
    assert(
      stdout.startsWith(header + ": ") && stdout.endsWith("\n"),
      "one compact owned diagnostic required",
    );
    assert(
      [...stdout].every((character) => {
        const code = character.codePointAt(0)!;
        return (code === 10 || code >= 32) && code !== 127;
      }),
      "printer control bytes refuse",
    );
    const body = stdout.slice(header.length + 2, -1).split("\n");
    assert(
      body[0]!.length > 0 && body.slice(1).every((line) => /^ {2,}\S.*$/u.test(line)),
      "opaque printer record structure differs",
    );
    assert.equal(
      [...stdout.matchAll(/[^\n]*\(\d+,\d+\): (?:error|warning) TS\d+:/gu)].length,
      1,
      "additional diagnostics refuse",
    );
    return {
      state: state.id,
      ownedCodeSiteStatusAndInputsMatch: true,
      wholeRawStreamsRetained: true,
      fullPrinterTextOwned: false,
      completeTsrsVectorMatchesSourceIntent: false,
      publicAcceptance: false,
      referenceOracleOwned: false,
    };
  }
  assert.equal(state.expectedTsrs.stdoutKind, undefined);
  assert.equal(state.expectedTsrs.sourceInputSha256, undefined);
  assert(packet.stdout === state.expectedTsrs.stdout, "complete source-owned tsrs stdout differs");
  assert(packet.stderr === state.expectedTsrs.stderr, "complete source-owned tsrs stderr differs");
  return {
    state: state.id,
    completeTsrsVectorMatchesSourceIntent: true,
    publicAcceptance: false,
    referenceOracleOwned: false,
  };
}
export function finalCustodiesBeforeAssessment(
  rows: { id: string; verify: () => void }[],
  record: Recorder,
) {
  const results = rows.map((row) => {
    try {
      row.verify();
      return { id: row.id, retainedRouteAndInputs: true, failure: null };
    } catch (error) {
      return { id: row.id, retainedRouteAndInputs: false, failure: safeFailure(error) };
    }
  });
  record.json("all-final-custody.json", {
    results,
    completeNodeModulesFilesetQualified: false,
    publicAcceptance: false,
  });
  return results;
}
