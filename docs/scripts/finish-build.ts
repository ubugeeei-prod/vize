// After `vp build`, each phase reads the built site independently. Launch their
// Node entrypoints directly: concurrent package-manager commands can race while
// reinstalling dependencies. OG rewrites pages atomically; every phase must pass.
import { spawn } from "node:child_process";
import { createInterface } from "node:readline";

export const finishSteps = ["generate:og-images", "check:ui-docs", "check:heading-links"] as const;
const finishEntrypoints = {
  "generate:og-images": "./scripts/generate-og-images.ts",
  "check:ui-docs": "./previews/ui/check-site.ts",
  "check:heading-links": "./scripts/check-heading-hash.ts",
} as const;

function run(script: (typeof finishSteps)[number]): Promise<number> {
  return new Promise((resolve, reject) => {
    const child = spawn(process.execPath, [finishEntrypoints[script]], {
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
