import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";

export type ProcessPacket = {
  command: string;
  cwd: string;
  args: string[];
  status: number | null;
  signal: string | null;
  error: { name: string; message: string } | null;
  stdout: string;
  stderr: string;
};
export type Message = { ruleId: string; severity: number; message: string; [key: string]: unknown };
export type LintFile = {
  file: string;
  errorCount: number;
  warningCount: number;
  messages: Message[];
  [key: string]: unknown;
};
export type Profile = {
  counters: { key: string; samples: number; total: number; min: number; max: number }[];
  truncation: { dropped_counters: number };
  [key: string]: unknown;
};
export function run(
  command: string,
  args: string[],
  cwd: string,
  env: NodeJS.ProcessEnv,
): ProcessPacket {
  const result = spawnSync(command, args, {
    cwd,
    env,
    encoding: "utf8",
    timeout: 60_000,
    maxBuffer: 16 * 1024 * 1024,
  });
  return {
    command,
    cwd,
    args,
    status: result.status,
    signal: result.signal,
    error: result.error ? { name: result.error.name, message: result.error.message } : null,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
}
export function valid(packet: ProcessPacket): void {
  assert.equal(packet.error, null);
  assert.equal(packet.signal, null);
  assert([0, 1].includes(packet.status ?? -1));
}
export function exact(actual: ProcessPacket, expected: ProcessPacket): void {
  valid(actual);
  valid(expected);
  assert.equal(actual.status, expected.status);
  assert.equal(actual.stdout, expected.stdout);
  assert.equal(actual.stderr, expected.stderr);
}
export function body(packet: ProcessPacket): LintFile[] {
  valid(packet);
  const parsed = JSON.parse(packet.stdout) as LintFile[];
  assert(Array.isArray(parsed));
  for (const file of parsed) {
    assert(Array.isArray(file.messages));
    assert.equal(typeof file.file, "string");
  }
  return parsed;
}
export function duplicates(files: LintFile[]): Message[] {
  return files
    .flatMap((file) => file.messages)
    .filter(
      (message) =>
        message.ruleId === "cross-file" &&
        message.message.includes("vize:croquis/cf/duplicate-id:"),
    );
}
export function withoutOriginalTwoGroups(files: LintFile[]): LintFile[] {
  const result = structuredClone(files);
  const removed: string[] = [];
  for (const file of result) {
    file.messages = file.messages.filter((message) => {
      if (
        message.ruleId !== "cross-file" ||
        !message.message.includes("vize:croquis/cf/duplicate-id:")
      )
        return true;
      const match =
        /Element ID '(shell|auth-page)' is used in 2 different locations across files/.exec(
          message.message,
        );
      assert(match, "Only the two original whole groups may disappear");
      assert.equal(message.severity, 1);
      removed.push(match[1]);
      file.warningCount--;
      return false;
    });
  }
  assert.deepEqual(removed.sort(), ["auth-page", "shell"]);
  return result;
}
export function profile(file: string): Profile {
  const parsed = JSON.parse(fs.readFileSync(file, "utf8")) as Profile;
  assert.equal(parsed.truncation.dropped_counters, 0);
  assert(Array.isArray(parsed.counters));
  return parsed;
}
export function scopeCounters(profile: Profile): Record<string, number> {
  const result: Record<string, number> = {};
  for (const counter of profile.counters) {
    if (!counter.key.startsWith("cli.lint.nuxt_scope.")) continue;
    assert.equal(counter.samples, 1, counter.key);
    assert.equal(counter.min, counter.total, counter.key);
    assert.equal(counter.max, counter.total, counter.key);
    result[counter.key.slice("cli.lint.nuxt_scope.".length)] = counter.total;
  }
  return result;
}
