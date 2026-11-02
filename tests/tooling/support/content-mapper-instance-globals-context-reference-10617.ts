import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { performance } from "node:perf_hooks";
import path from "node:path";
import { isDeepStrictEqual } from "node:util";
import type { Packet } from "./content-mapper-instance-globals-10617.ts";

export type Ts59ContextInputs = {
  command: string;
  args: string[];
  cwd: string;
  config: string;
  globals: string;
  context: string;
};

const oracleConfig = {
  compilerOptions: {
    module: "preserve",
    moduleResolution: "bundler",
    strict: true,
    noEmit: true,
    jsx: "preserve",
    jsxImportSource: "vue",
    lib: ["ESNext", "DOM"],
    types: [],
  },
  files: ["src/globals.d.ts", "oracle/Context.ts"],
};

// This fixture's independent project reads these two files, Vue's locked
// declarations and the compiler libraries. It never reads Globals.vue. Reject
// unfamiliar dependencies/options rather than assuming they are variant-free.
export function isFlatTs59Context(input: Ts59ContextInputs): boolean {
  if (
    !path.isAbsolute(input.command) ||
    !path.isAbsolute(input.cwd) ||
    !path.isAbsolute(input.args[0] ?? "") ||
    !isDeepStrictEqual(input.args.slice(1), [
      "--noEmit",
      "--pretty",
      "false",
      "-p",
      "oracle.tsconfig.json",
    ])
  )
    return false;
  try {
    if (!isDeepStrictEqual(JSON.parse(input.config), oracleConfig)) return false;
  } catch {
    return false;
  }
  return [input.globals, input.context].every((source) => {
    const rest = source
      .replace(/\bimport\s*\(\s*(["'])vue\1\s*\)/gu, "")
      .replace(/\bexport\s*\{\s*\}\s*;?/gu, "")
      .replace(/\bdeclare\s+module\s*(["'])vue\1/gu, "");
    return !/\b(?:import|export|require|module)\b|<reference\b/u.test(rest);
  });
}

// Scope this table to the one immutable-provider LF/CRLF loop. Distinct cases
// still execute independently, including valid/negative/repaired cases with
// equal bytes. Return the genuine original packet; never invent a second run.
export function createTs59ContextReferences() {
  const references = new Map<string, { inputs: string; packet: Packet }>();
  return (caseId: string, input: Ts59ContextInputs, execute: () => Packet): Packet => {
    if (!isFlatTs59Context(input)) {
      references.clear();
      return execute();
    }
    const identity = JSON.stringify(input);
    const previous = references.get(caseId);
    if (previous?.inputs === identity) return previous.packet;
    const packet: Packet = execute();
    if (
      packet.error === null &&
      packet.signal === null &&
      packet.stderr === "" &&
      (packet.status === 0 || packet.status === 2)
    )
      references.set(caseId, { inputs: identity, packet });
    else references.delete(caseId);
    return packet;
  };
}

// Original process capture, shared without changing any command or failure law.
export function createInstanceGlobalsRunner(recordProcess: (packet: Packet) => void) {
  return (label: string, command: string, args: string[], cwd: string, input?: Buffer) => {
    const started = performance.now();
    const raw = spawnSync(command, args, {
      cwd,
      input,
      encoding: "utf8",
      timeout: 120_000,
      maxBuffer: 32 * 1024 * 1024,
      env: { ...process.env, LANG: "C", LC_ALL: "C" },
    });
    const packet: Packet = {
      label,
      command,
      args,
      cwd,
      status: raw.status,
      signal: raw.signal,
      error: raw.error?.message ?? null,
      stdout: raw.stdout ?? "",
      stderr: raw.stderr ?? "",
      elapsedMs: performance.now() - started,
    };
    recordProcess(packet);
    assert.equal(packet.error, null, label);
    assert.equal(packet.signal, null, label);
    return packet;
  };
}
