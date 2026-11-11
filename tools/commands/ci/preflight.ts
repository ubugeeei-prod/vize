#!/usr/bin/env node
// Local PR preflight: run the deterministic checks CI would run for this diff
// before a push, using CI's own planners to select them.
//
//   node tools/commands/ci/preflight.ts [--base origin/main] [--only rust,tooling] ...
//
// See `--help` and tools/support/ci/preflight-plan.ts for what each lane mirrors.
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  type PreflightOptions,
  type Step,
  planPreflight,
  parseOptions,
  usage,
} from "../../support/ci/preflight-plan.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");

type Outcome = { step: Step; status: "pass" | "fail" | "skip"; seconds: number; log?: string };

const shellQuote = (value: string) =>
  /^[\w@%+=:,./-]+$/u.test(value) ? value : `'${value.replaceAll("'", `'\\''`)}'`;

/** The command a developer can paste outside this process to reproduce one step. */
export function reproduction(step: Step, nix: boolean): string {
  if (step.repro) return step.repro;
  const env = Object.entries(step.env ?? {})
    .map(([key, value]) => `${key}=${shellQuote(value)}`)
    .join(" ");
  const command = step.command!.map(shellQuote).join(" ");
  const line = [env, command].filter(Boolean).join(" ");
  const located =
    step.cwd && step.cwd !== root
      ? `cd ${shellQuote(path.relative(root, step.cwd))} && ${line}`
      : line;
  return nix && step.toolchain ? `nix develop --command bash -c ${shellQuote(located)}` : located;
}

function hasCommand(command: string): boolean {
  return spawnSync("sh", ["-c", `command -v ${command}`], { stdio: "ignore" }).status === 0;
}

/** Plain cargo on some machines lacks the pinned toolchain; the repo flake has it. */
function reexecUnderNix(options: PreflightOptions): never | undefined {
  if (options.nix === false || process.env.IN_NIX_SHELL || process.env.VIZE_PREFLIGHT_NIX) return;
  if (!fs.existsSync(path.join(root, "flake.nix")) || !hasCommand("nix")) return;
  process.stderr.write("preflight: entering `nix develop` for the pinned Rust/Node toolchain\n");
  const result = spawnSync(
    "nix",
    ["develop", "--command", "node", fileURLToPath(import.meta.url), ...process.argv.slice(2)],
    { cwd: root, stdio: "inherit", env: { ...process.env, VIZE_PREFLIGHT_NIX: "1" } },
  );
  process.exit(result.status ?? 1);
}

function runStep(step: Step, logDir: string, timeoutSeconds: number): Promise<Outcome> {
  const started = performance.now();
  const log = path.join(logDir, `${step.id.replaceAll(/[^\w.-]+/gu, "_")}.log`);
  const done = (status: Outcome["status"]): Outcome => ({
    step,
    status,
    seconds: (performance.now() - started) / 1000,
    log,
  });
  if (step.run) {
    const lines: string[] = [];
    return step
      .run((line) => lines.push(line))
      .then(
        (ok) => {
          fs.writeFileSync(log, `${lines.join("\n")}\n`);
          return done(ok ? "pass" : "fail");
        },
        (error: unknown) => {
          fs.writeFileSync(
            log,
            `${lines.join("\n")}\n${String((error as Error)?.stack ?? error)}\n`,
          );
          return done("fail");
        },
      );
  }
  return new Promise((resolve) => {
    const fd = fs.openSync(log, "w");
    fs.writeSync(fd, `$ ${reproduction(step, false)}\n\n`);
    const [executable, ...args] = step.command!;
    const child = spawn(executable, args, {
      cwd: step.cwd ?? root,
      env: { ...process.env, ...step.env },
      stdio: ["ignore", fd, fd],
      detached: true,
    });
    const limit = step.timeoutSeconds ?? timeoutSeconds;
    const timer =
      limit > 0
        ? setTimeout(() => {
            fs.writeSync(fd, `\npreflight: timed out after ${limit}s\n`);
            try {
              process.kill(-child.pid!, "SIGKILL");
            } catch {}
          }, limit * 1000)
        : undefined;
    const finish = (status: Outcome["status"]) => {
      if (timer) clearTimeout(timer);
      fs.closeSync(fd);
      resolve(done(status));
    };
    child.on("error", (error) => {
      fs.writeSync(fd, `\npreflight: could not start ${executable}: ${error.message}\n`);
      finish("fail");
    });
    child.on("exit", (code) => finish(code === 0 ? "pass" : "fail"));
  });
}

async function pool<T>(items: readonly T[], concurrency: number, work: (item: T) => Promise<void>) {
  let next = 0;
  const workers = Array.from(
    { length: Math.max(1, Math.min(concurrency, items.length)) },
    async () => {
      while (next < items.length) await work(items[next++]);
    },
  );
  await Promise.all(workers);
}

function tail(file: string | undefined, count: number): string {
  if (!file || !fs.existsSync(file)) return "";
  const lines = fs.readFileSync(file, "utf8").trimEnd().split("\n");
  return lines.slice(-count).join("\n");
}

const formatSeconds = (seconds: number) =>
  seconds >= 60
    ? `${Math.floor(seconds / 60)}m${Math.round(seconds % 60)}s`
    : `${seconds.toFixed(1)}s`;

async function main(): Promise<number> {
  const options = parseOptions(process.argv.slice(2));
  if (options.help) {
    process.stdout.write(usage);
    return 0;
  }
  reexecUnderNix(options);
  const started = performance.now();
  const plan = planPreflight(root, options);
  process.stdout.write(plan.summary.map((line) => `${line}\n`).join(""));
  const all = plan.lanes.flatMap((lane) => lane.steps);
  if (options.list) {
    for (const lane of plan.lanes) {
      process.stdout.write(`\n[${lane.name}] ${lane.note ?? ""}\n`);
      for (const step of lane.steps)
        process.stdout.write(`  ${step.id}: ${reproduction(step, plan.nix)}\n`);
    }
    return 0;
  }
  const logDir = path.join(
    root,
    "target",
    "preflight",
    new Date().toISOString().replaceAll(/[:.]/gu, "-"),
  );
  fs.mkdirSync(logDir, { recursive: true });
  process.stdout.write(`Logs: ${path.relative(root, logDir)}\n\n`);

  const outcomes: Outcome[] = [];
  const report = (outcome: Outcome) => {
    outcomes.push(outcome);
    const label = outcome.status === "pass" ? "PASS" : outcome.status === "fail" ? "FAIL" : "SKIP";
    const elapsed = formatSeconds((performance.now() - started) / 1000);
    process.stdout.write(
      `${label} ${outcome.step.id} (${formatSeconds(outcome.seconds)}) [t+${elapsed}]\n`,
    );
  };
  let stop = false;
  const execute = async (step: Step) => {
    if (stop) {
      report({ step, status: "skip", seconds: 0 });
      return;
    }
    const outcome = await runStep(step, logDir, options.timeoutSeconds);
    report(outcome);
    if (outcome.status === "fail" && options.failFast) stop = true;
  };

  // Lanes run side by side; each lane decides its own internal concurrency.
  // Setup steps (dependency installs) gate the lanes that need them.
  for (const step of plan.setup) await execute(step);
  if (outcomes.some((outcome) => outcome.status === "fail")) stop = true;
  await Promise.all(
    plan.lanes.map(async (lane) => {
      if (lane.sequential) {
        for (const step of lane.steps) {
          // A failed build makes the rest of a sequential lane meaningless.
          if (
            lane.stopOnFailure &&
            outcomes.some((o) => o.status === "fail" && lane.steps.includes(o.step))
          ) {
            report({ step, status: "skip", seconds: 0 });
            continue;
          }
          await execute(step);
        }
      } else await pool(lane.steps, lane.concurrency ?? options.jobs, execute);
    }),
  );

  const failed = outcomes.filter((outcome) => outcome.status === "fail");
  const skipped = outcomes.filter((outcome) => outcome.status === "skip");
  const total = formatSeconds((performance.now() - started) / 1000);
  process.stdout.write(`\n${"=".repeat(72)}\nPreflight summary (${total})\n${"=".repeat(72)}\n`);
  for (const lane of plan.lanes) {
    const laneOutcomes = outcomes.filter((outcome) => lane.steps.includes(outcome.step));
    const passed = laneOutcomes.filter((outcome) => outcome.status === "pass").length;
    const laneFailed = laneOutcomes.filter((outcome) => outcome.status === "fail").length;
    const status = laneFailed ? "FAIL" : lane.steps.length ? "PASS" : "----";
    process.stdout.write(
      `${status.padEnd(5)}${lane.name.padEnd(14)}${passed}/${lane.steps.length} passed${lane.note ? `  (${lane.note})` : ""}\n`,
    );
  }
  for (const outcome of failed) {
    process.stdout.write(`\nFAIL ${outcome.step.id}: ${outcome.step.title}\n`);
    process.stdout.write(`  reproduce: ${reproduction(outcome.step, plan.nix)}\n`);
    process.stdout.write(`  log:       ${path.relative(root, outcome.log!)}\n`);
    if (outcome.step.hint) process.stdout.write(`  hint:      ${outcome.step.hint}\n`);
    const excerpt = tail(outcome.log, options.tailLines);
    if (excerpt) process.stdout.write(`${excerpt.replaceAll(/^/gmu, "  | ")}\n`);
  }
  if (plan.unmirrored.length) {
    process.stdout.write(`\nNot mirrored locally (CI still runs these):\n`);
    for (const line of plan.unmirrored) process.stdout.write(`  - ${line}\n`);
  }
  process.stdout.write(
    `\n${failed.length ? `FAILED: ${failed.length} of ${all.length + plan.setup.length} steps` : `PASSED: ${outcomes.length - skipped.length} steps`}${skipped.length ? `, ${skipped.length} skipped` : ""} in ${total}\n`,
  );
  return failed.length ? 1 : 0;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().then(
    (code) => {
      process.exitCode = code;
    },
    (error: unknown) => {
      process.stderr.write(`preflight: ${String((error as Error)?.stack ?? error)}\n`);
      process.exitCode = 2;
    },
  );
}
