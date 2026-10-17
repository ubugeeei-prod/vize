// After `vp build`, OG generation and the UI docs check each read the built
// site independently, so run them together. OG rewrites each page atomically,
// and the build fails if either step fails.
import { spawn } from "node:child_process";
import { createInterface } from "node:readline";

export const finishSteps = ["generate:og-images", "check:ui-docs", "check:heading-links"] as const;

function run(script: string): Promise<number> {
  return new Promise((resolve, reject) => {
    const child = spawn("pnpm", ["run", "--silent", script], {
      cwd: import.meta.dirname + "/..",
      stdio: ["ignore", "pipe", "pipe"],
    });
    for (const [stream, sink] of [
      [child.stdout, process.stdout],
      [child.stderr, process.stderr],
    ] as const) {
      createInterface({ input: stream }).on("line", (line) => sink.write(`[${script}] ${line}\n`));
    }
    child.on("error", reject);
    child.on("close", (code, signal) => resolve(code ?? (signal ? 1 : 0)));
  });
}

const codes = await Promise.all(finishSteps.map(run));
const failed = finishSteps.filter((_, index) => codes[index] !== 0);
if (failed.length) {
  console.error(`Docs build failed: ${failed.join(", ")}`);
  process.exitCode = 1;
}
