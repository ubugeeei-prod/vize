/** Bounded real-stdio worker probes; historical Python source/images stay immutable. */
import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import {
  closeSync,
  existsSync,
  mkdirSync,
  openSync,
  readFileSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { constants } from "node:os";
import { dirname, join, resolve } from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { pathToFileURL } from "node:url";
import { parseArgs } from "node:util";
import {
  DEPTHS,
  STACKS,
  TIMEOUT_SECONDS,
  OUTPUT_LIMIT_BYTES,
  RSS_LIMIT_BYTES,
  frames,
  hashFile,
  largerControlStack,
  readTrialResult,
  requireFullWorkerPool,
  requireStackFailure,
  workerPoolReceipt,
  witness,
  type Profile,
  type Shape,
  type TrialResult,
} from "./content-mapper-worker-stack-core.ts";

export interface Options {
  binary: string;
  out: string;
  sourceSha: string;
  profile: Profile;
  remainingVSlotOnly: boolean;
  fixedWorkerBudget: boolean;
}

interface Report {
  sourceSha: string;
  binarySha256: string;
  profile: Profile;
  depths: readonly number[];
  candidateInheritedStacks: readonly number[];
  timeoutSeconds: number;
  perFileOutputLimitBytes: number;
  processTreeRssLimitBytes: number;
  remainingVSlotOnly: boolean;
  fixedWorkerBudget: boolean;
  trials: TrialResult[];
  explicitWorkerStackBytes?: number;
  qualifiedDepth?: number;
  confirmedInheritedStacks?: number[];
  complete?: true;
}

function missingProcess(error: unknown): boolean {
  return (
    error !== null &&
    typeof error === "object" &&
    "code" in error &&
    (error.code === "ENOENT" || error.code === "ESRCH")
  );
}

export function processTreeRss(
  pid: number,
  pageSize: number,
  readText: (path: string) => string = (path) => readFileSync(path, "utf8"),
): number {
  const pending = [pid];
  let total = 0;
  while (pending.length) {
    const current = pending.pop()!;
    try {
      const resident = Number(readText(`/proc/${current}/statm`).split(/\s+/)[1]);
      assert(Number.isSafeInteger(resident) && resident >= 0, "invalid process RSS");
      total += resident * pageSize;
      for (const token of readText(`/proc/${current}/task/${current}/children`)
        .trim()
        .split(/\s+/)
        .filter(Boolean)) {
        const child = Number(token);
        assert(Number.isSafeInteger(child) && child > 0, "invalid child process id");
        pending.push(child);
      }
    } catch (error) {
      if (!missingProcess(error)) throw error;
    }
  }
  return total;
}

export function boundedCommand(
  binary: string,
  directory: string,
  timeoutSeconds: number,
): string[] {
  // prlimit execs time with the same inherited soft/hard limits as setrlimit.
  return [
    `--core=0:0`,
    `--fsize=${OUTPUT_LIMIT_BYTES}:${OUTPUT_LIMIT_BYTES}`,
    `--cpu=${timeoutSeconds}:${timeoutSeconds + 1}`,
    "--",
    "/usr/bin/time",
    "-v",
    "-o",
    join(directory, "resources.txt"),
    binary,
    "content-mapper",
  ];
}

function makeExclusive(directory: string): void {
  mkdirSync(dirname(directory), { recursive: true });
  mkdirSync(directory);
}

type Outcome =
  | { kind: "error"; error: Error }
  | { kind: "exit"; code: number | null; signal: NodeJS.Signals | null };

async function trial(
  binary: string,
  directory: string,
  shape: Shape,
  depth: number,
  stack: number,
  pageSize: number,
  timeoutSeconds = TIMEOUT_SECONDS,
): Promise<TrialResult> {
  makeExclusive(directory);
  writeFileSync(join(directory, "stdin.wire"), frames(witness(shape, depth)));
  const started = performance.now(),
    descriptors: number[] = [];
  let timedOut = false,
    rssLimitExceeded = false,
    peakRss = 0,
    returncode: number | undefined;
  let abort: (() => Promise<void>) | undefined;
  const owned = (path: string, mode: "r" | "w") => {
    const descriptor = openSync(path, mode);
    descriptors.push(descriptor);
    return descriptor;
  };
  try {
    const child = spawn("/usr/bin/prlimit", boundedCommand(binary, directory, timeoutSeconds), {
      stdio: [
        owned(join(directory, "stdin.wire"), "r"),
        owned(join(directory, "stdout.wire"), "w"),
        owned(join(directory, "stderr.txt"), "w"),
      ],
      env: {
        ...process.env,
        RUST_MIN_STACK: String(stack),
        VIZE_CONTENT_MAPPER_WORKER_CAPTURE: join(directory, "worker-pool.json"),
      },
      detached: true,
      shell: false,
    });
    let closed = false;
    const completed = new Promise<Outcome>((finish) => {
      child.once("error", (error) => {
        closed = true;
        finish({ kind: "error", error });
      });
      child.once("close", (code, signal) => {
        closed = true;
        finish({ kind: "exit", code, signal });
      });
    });
    const pid = child.pid;
    abort = async () => {
      if (!closed && pid !== undefined) {
        try {
          process.kill(-pid, "SIGKILL");
        } catch (error) {
          if (!missingProcess(error)) throw error;
        }
      }
      await completed;
    };
    if (pid !== undefined) {
      while (!closed) {
        peakRss = Math.max(peakRss, processTreeRss(pid, pageSize));
        timedOut = (performance.now() - started) / 1000 >= timeoutSeconds;
        rssLimitExceeded = peakRss > RSS_LIMIT_BYTES;
        if (timedOut || rssLimitExceeded) {
          try {
            process.kill(-pid, "SIGKILL");
          } catch (error) {
            if (!missingProcess(error)) throw error;
          }
          break;
        }
        await delay(50);
      }
    }
    const outcome = await completed;
    if (outcome.kind === "error") throw outcome.error;
    if (outcome.code !== null) returncode = outcome.code;
    else {
      assert(outcome.signal !== null);
      const signal = constants.signals[outcome.signal];
      assert(typeof signal === "number");
      returncode = -signal;
    }
  } catch (error) {
    await abort?.();
    throw error;
  } finally {
    for (const descriptor of descriptors.reverse()) closeSync(descriptor);
  }
  assert(returncode !== undefined);
  const result = readTrialResult(directory, shape, depth, stack, {
    returncode,
    timedOut,
    timeoutSeconds,
    rssLimitExceeded,
    peakProcessTreeRssBytes: peakRss,
    wallSeconds: (performance.now() - started) / 1000,
  });
  const poolPath = join(directory, "worker-pool.json");
  if (existsSync(poolPath))
    result.workerPool = workerPoolReceipt(JSON.parse(readFileSync(poolPath, "utf8")));
  writeFileSync(join(directory, "result.json"), JSON.stringify(result, null, 2) + "\n");
  console.log(JSON.stringify(result));
  if (result.accepted) requireFullWorkerPool(result.workerPool);
  return result;
}

export async function runProbe(options: Options): Promise<void> {
  const binary = realpathSync(resolve(options.binary)),
    out = resolve(options.out);
  makeExclusive(out);
  const report: Report = {
    sourceSha: options.sourceSha,
    binarySha256: hashFile(binary),
    profile: options.profile,
    depths: DEPTHS,
    candidateInheritedStacks: STACKS,
    timeoutSeconds: TIMEOUT_SECONDS,
    perFileOutputLimitBytes: OUTPUT_LIMIT_BYTES,
    processTreeRssLimitBytes: RSS_LIMIT_BYTES,
    remainingVSlotOnly: options.remainingVSlotOnly,
    fixedWorkerBudget: options.fixedWorkerBudget,
    trials: [],
  };
  try {
    assert(!(options.fixedWorkerBudget && options.remainingVSlotOnly));
    const pageSize = Number(
      execFileSync("/usr/bin/getconf", ["PAGESIZE"], { encoding: "utf8" }).trim(),
    );
    assert(Number.isInteger(pageSize) && pageSize > 0);
    if (options.fixedWorkerBudget) {
      const depth = options.profile === "ci-opt" ? 4094 : 2048;
      Object.assign(report, {
        explicitWorkerStackBytes: 64 * 1024 ** 2,
        qualifiedDepth: depth,
        depths: [depth],
        candidateInheritedStacks: [256 * 1024, 512 * 1024],
        timeoutSeconds: 90,
      });
      for (const shape of ["v-for", "v-slot"] as const) {
        let reference: string | undefined;
        for (const inherited of report.candidateInheritedStacks) {
          const result = await trial(
            binary,
            join(out, `${shape}-${depth}-${inherited}`),
            shape,
            depth,
            inherited,
            pageSize,
            90,
          );
          report.trials.push(result);
          assert(result.accepted, "explicit worker budget must survive the tiny inherited stack");
          if (reference !== undefined)
            assert.equal(
              result.stdoutSha256,
              reference,
              "inherited stack changed complete response",
            );
          reference = result.stdoutSha256;
        }
      }
    } else if (options.remainingVSlotOnly) {
      assert.equal(options.profile, "ci-opt", "focused follow-up uses the optimized source binary");
      Object.assign(report, {
        depths: [4094],
        candidateInheritedStacks: [32 * 1024 ** 2, 64 * 1024 ** 2, 128 * 1024 ** 2],
        timeoutSeconds: 90,
      });
      const accepted: TrialResult[] = [];
      for (const stack of report.candidateInheritedStacks) {
        const result = await trial(
          binary,
          join(out, `v-slot-4094-${stack}`),
          "v-slot",
          4094,
          stack,
          pageSize,
          90,
        );
        report.trials.push(result);
        assert(!result.rssLimitExceeded, "bounded RSS limit reached; stop the probe");
        if (result.accepted) {
          if (accepted.length)
            assert.equal(
              result.stdoutSha256,
              accepted[0].stdoutSha256,
              "stack changed complete response",
            );
          accepted.push(result);
        } else if (!result.timedOut)
          assert(result.stackOverflow, "non-stack failure is not a stack-budget observation");
      }
      assert(
        accepted.length >= 2,
        "require complete output plus an independent larger-stack control",
      );
      report.confirmedInheritedStacks = accepted.map((result) => result.inheritedMinStackBytes);
    } else {
      for (const shape of ["v-for", "v-slot"] as const) {
        let candidate = 0;
        for (const depth of DEPTHS) {
          let result: TrialResult | undefined;
          while (candidate < STACKS.length) {
            const stack = STACKS[candidate];
            result = await trial(
              binary,
              join(out, `${shape}-${depth}-${stack}`),
              shape,
              depth,
              stack,
              pageSize,
            );
            report.trials.push(result);
            if (result.accepted) break;
            requireStackFailure(result);
            candidate++;
          }
          assert(candidate < STACKS.length && result !== undefined, "no candidate stack completed");
          const stack = largerControlStack(candidate);
          const control = await trial(
            binary,
            join(out, `${shape}-${depth}-control`),
            shape,
            depth,
            stack,
            pageSize,
          );
          report.trials.push(control);
          assert(control.accepted, "larger stack control must complete");
          assert.equal(
            control.stdoutSha256,
            result.stdoutSha256,
            "stack changed complete response",
          );
        }
      }
    }
    report.complete = true;
  } finally {
    writeFileSync(join(out, "report.json"), JSON.stringify(report, null, 2) + "\n");
  }
}

class ArgumentError extends Error {}

async function main(): Promise<void> {
  const { values } = parseArgs({
    options: {
      binary: { type: "string" },
      out: { type: "string" },
      "source-sha": { type: "string" },
      profile: { type: "string" },
      "remaining-v-slot-only": { type: "boolean", default: false },
      "fixed-worker-budget": { type: "boolean", default: false },
      help: { type: "boolean", short: "h" },
    },
  });
  if (values.help) {
    console.log(
      "Bounded content-mapper worker probes: --binary PATH --out PATH --source-sha SHA --profile ci|ci-opt [--fixed-worker-budget|--remaining-v-slot-only]",
    );
    return;
  }
  const binary = values.binary,
    out = values.out,
    sourceSha = values["source-sha"],
    profile = values.profile;
  if (
    binary === undefined ||
    out === undefined ||
    sourceSha === undefined ||
    (profile !== "ci" && profile !== "ci-opt")
  )
    throw new ArgumentError("--binary, --out, --source-sha and --profile ci|ci-opt are required");
  await runProbe({
    binary,
    out,
    sourceSha,
    profile,
    remainingVSlotOnly: values["remaining-v-slot-only"],
    fixedWorkerBudget: values["fixed-worker-budget"],
  });
}

if (
  process.argv[1] !== undefined &&
  import.meta.url === pathToFileURL(resolve(process.argv[1])).href
) {
  main().catch((error: unknown) => {
    console.error(error);
    process.exitCode =
      error instanceof ArgumentError ||
      (error instanceof TypeError &&
        "code" in error &&
        String(error.code).startsWith("ERR_PARSE_ARGS"))
        ? 2
        : 1;
  });
}
