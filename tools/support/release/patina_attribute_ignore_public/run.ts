/** Main-only entry point; pull requests execute pure falsification controls. */
import assert from "node:assert/strict";
import { existsSync, realpathSync, statfsSync } from "node:fs";
import { join, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { bootstrap, preserve } from "./bootstrap.ts";
import { request, role, write } from "./contract.ts";
import { runNative } from "./native.ts";
import { runCli } from "./cli.ts";
import { runEslint, runOxlint } from "./plugin.ts";

export async function run(args: string[]): Promise<void> {
  assert.equal(args.length, 4);
  const [version, candidate, producerRun, outputArg] = args;
  assert(version && candidate && producerRun && outputArg);
  request(version, candidate, producerRun, process.env);
  assert.equal(process.version, "v24.14.0");
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  const header = (process.report.getReport() as { header?: { glibcVersionRuntime?: string } })
    .header;
  assert(header?.glibcVersionRuntime, "an actual GNU native provider is required");
  const temporary = realpathSync(process.env.RUNNER_TEMP!),
    output = resolve(outputArg);
  assert(output.startsWith(temporary + sep));
  assert(!existsSync(output));
  const disk = statfsSync(temporary);
  assert(
    disk.bavail * disk.bsize >= 3 * 1024 ** 3,
    "one GiB floor plus two GiB installation forecast",
  );
  let phase = "producer-and-install";
  try {
    const context = await bootstrap(version, candidate, producerRun, output);
    for (const [name, execute] of [
      ["native", () => runNative(context)],
      ["cli", () => runCli(context)],
      ["eslint", () => runEslint(context)],
      ["oxlint", () => runOxlint(context)],
    ] as const) {
      phase = name;
      context.record.json("progress.json", { role, phase, complete: false });
      await execute();
      preserve(context);
    }
    write(join(context.proof, "complete.json"), {
      role,
      complete: true,
      version,
      candidate,
      producerRun,
      native: { inputs: 9, forms: 2, packets: 63, refusals: 6, unselected: 3 },
      cli: { inputs: 9, carriers: 2, packets: 55 },
      eslint: { inputs: 9, wholePackets: true },
      oxlint: { inputs: 9, packets: 18, semanticAndRetainedPresentationOnly: true },
      wholeIssue6097Qualified: false,
      wholeIssue10822Qualified: false,
      limits:
        "Finite literal attribute-ignore acceptance only; original reference and target provider identities remain separate. No Event/Boolean/formatter/JSX, direct Oxlint precise-range/fixer, unobserved worker-exit, or performance credit.",
    });
  } catch (error) {
    if (existsSync(join(output, "proof")))
      write(join(output, "proof/failure.json"), {
        role,
        complete: false,
        phase,
        error:
          error instanceof Error
            ? { name: error.name, message: error.message, stack: error.stack }
            : error,
      });
    throw error;
  }
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await run(process.argv.slice(2));
