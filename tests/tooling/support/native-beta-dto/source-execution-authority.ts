import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import type { MergedTraceAuthority } from "./merged-trace.ts";

export interface SourceExecutionAuthority {
  source: string;
  executable: string;
  executableSha256: string;
  command: string;
  commandSha256: string;
  commandVersion: string;
  args: string[];
  trace: MergedTraceAuthority;
  status: number | null;
  signal: NodeJS.Signals | null;
  error: string | null;
}

const hash = (value: Uint8Array) => createHash("sha256").update(value).digest("hex");

/** Authenticate the exact successful collector invocation before consuming its trace. */
export function verifySourceExecution(
  value: unknown,
  root: string,
  source: string,
): SourceExecutionAuthority {
  const execution = value as SourceExecutionAuthority;
  assert.equal(execution.source, source);
  assert.equal(execution.status, 0);
  assert.equal(execution.signal, null);
  assert.equal(execution.error, null);
  assert.equal(hash(fs.readFileSync(execution.executable)), execution.executableSha256);
  assert.equal(fs.realpathSync(execution.command), execution.command);
  assert.equal(hash(fs.readFileSync(execution.command)), execution.commandSha256);
  assert.match(execution.commandVersion, /^strace -- version [0-9]/u);
  const selector =
    "lsp_client::editor_lsp::snapshot_source::tests::native_beta::actual_sdk_semantic_error_repair_navigation_and_two_file_scope_are_complete";
  assert.deepEqual(execution.args, [
    "-f",
    "-ttt",
    "-s",
    "16777216",
    "-xx",
    "-e",
    "trace=read,write,readv,writev,process",
    "-o",
    path.join(root, "syscalls.merged"),
    execution.executable,
    selector,
    "--ignored",
    "--exact",
    "--nocapture",
  ]);
  return execution;
}
