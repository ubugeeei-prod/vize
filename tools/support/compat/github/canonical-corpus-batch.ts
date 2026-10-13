// Run independent command chains concurrently behind a synchronous call.
// Canonical corpus capture runs the same read-only Git commands in each of
// ~150 fixture repositories, and the finalizer validates three independent
// observer artifacts; one at a time they dominated those steps. A chain runs
// its commands in order, chains run concurrently, and every result keeps its
// complete stdout, stderr, exit status and signal so callers apply exactly the
// checks they applied to execFileSync before.
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { closeSync, mkdtempSync, openSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export type Command = {
  /** The executable; Git when omitted. */
  readonly file?: string;
  readonly cwd: string;
  readonly args: readonly string[];
  readonly input?: string;
};
export type CommandResult = {
  readonly command: Command;
  readonly status: number | null;
  readonly signal: string | null;
  readonly error: { readonly code?: string; readonly message: string } | null;
  readonly stdout: Buffer;
  readonly stderr: Buffer;
};
type Outcome = Omit<CommandResult, "command" | "stdout" | "stderr">;

export const defaultConcurrency = 32;
const helper = fileURLToPath(import.meta.url);

/** Run one chain in the calling process, in order. */
export function runChain(chain: readonly Command[]): CommandResult[] {
  return chain.map((command) => {
    const result = spawnSync(command.file ?? "git", command.args, {
      cwd: command.cwd,
      input: command.input,
      maxBuffer: 1024 * 1024 * 1024,
    });
    const error: (Error & { code?: string }) | undefined = result.error;
    return {
      command,
      status: result.status,
      signal: result.signal,
      error: error ? { code: error.code, message: error.message } : null,
      stdout: result.stdout ?? Buffer.alloc(0),
      stderr: result.stderr ?? Buffer.alloc(0),
    };
  });
}

/** Run every chain, up to `concurrency` at once, and wait for all of them. */
export function runChainsSync(
  chains: readonly (readonly Command[])[],
  concurrency = defaultConcurrency,
): CommandResult[][] {
  if (chains.length <= 1 || concurrency <= 1) return chains.map(runChain);
  const scratch = mkdtempSync(join(tmpdir(), "canonical-batch-"));
  try {
    writeFileSync(join(scratch, "chains.json"), JSON.stringify(chains));
    const child = spawnSync(process.execPath, [helper, scratch, String(concurrency)], {
      stdio: ["ignore", "inherit", "inherit"],
    });
    assert(!child.error && child.status === 0, "Concurrent canonical command batch failed");
    const outcomes = JSON.parse(
      readFileSync(join(scratch, "outcomes.json"), "utf8"),
    ) as Outcome[][];
    assert.equal(outcomes.length, chains.length, "Incomplete canonical command batch");
    return chains.map((chain, index) => {
      const row = outcomes[index] as Outcome[];
      assert.equal(row.length, chain.length, "Incomplete canonical command chain");
      return chain.map((command, step) => ({
        ...(row[step] as Outcome),
        command,
        stdout: readFileSync(join(scratch, `${index}-${step}.out`)),
        stderr: readFileSync(join(scratch, `${index}-${step}.err`)),
      }));
    });
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
}

type CommandError = Error & {
  status: number | null;
  signal: string | null;
  stdout: Buffer;
  stderr: Buffer;
  code?: string;
};

/** The stdout execFileSync would have returned, or the error it would throw. */
export function commandOutput(result: CommandResult): Buffer;
export function commandOutput(result: CommandResult, encoding: "utf8"): string;
export function commandOutput(result: CommandResult, encoding?: "utf8") {
  if (result.error || result.signal !== null || result.status !== 0) {
    const command = [result.command.file ?? "git", ...result.command.args].join(" ");
    const error = new Error(
      result.error
        ? `${result.error.message} (${command} in ${result.command.cwd})`
        : `Command failed: ${command}\n${result.stderr.toString("utf8")}`,
    ) as CommandError;
    error.status = result.status;
    error.signal = result.signal;
    error.stdout = result.stdout;
    error.stderr = result.stderr;
    if (result.error?.code) error.code = result.error.code;
    throw error;
  }
  return encoding ? result.stdout.toString(encoding) : result.stdout;
}

function runCommand(command: Command, prefix: string): Promise<Outcome> {
  const stdout = openSync(`${prefix}.out`, "w");
  const stderr = openSync(`${prefix}.err`, "w");
  return new Promise((resolvePromise) => {
    let settled = false;
    const finish = (outcome: Outcome) => {
      if (settled) return;
      settled = true;
      closeSync(stdout);
      closeSync(stderr);
      resolvePromise(outcome);
    };
    const child = spawn(command.file ?? "git", command.args, {
      cwd: command.cwd,
      stdio: [command.input === undefined ? "ignore" : "pipe", stdout, stderr],
    });
    child.on("error", (error: NodeJS.ErrnoException) =>
      finish({ status: null, signal: null, error: { code: error.code, message: error.message } }),
    );
    child.on("close", (status, signal) => finish({ status, signal, error: null }));
    if (command.input !== undefined) child.stdin?.end(command.input);
  });
}

async function main(scratch: string, concurrency: number): Promise<void> {
  const chains = JSON.parse(readFileSync(join(scratch, "chains.json"), "utf8")) as Command[][];
  const outcomes: Outcome[][] = chains.map(() => []);
  let next = 0;
  const worker = async () => {
    while (next < chains.length) {
      const index = next++;
      const chain = chains[index] as Command[];
      for (const [step, command] of chain.entries())
        (outcomes[index] as Outcome[]).push(
          await runCommand(command, join(scratch, `${index}-${step}`)),
        );
    }
  };
  await Promise.all(Array.from({ length: Math.min(concurrency, chains.length) }, worker));
  writeFileSync(join(scratch, "outcomes.json"), JSON.stringify(outcomes));
}

if (process.argv[1] && resolve(process.argv[1]) === helper) {
  const [scratch, concurrency] = process.argv.slice(2);
  assert(scratch && Number(concurrency) > 0, "Expected canonical batch SCRATCH CONCURRENCY");
  await main(scratch, Number(concurrency));
}
