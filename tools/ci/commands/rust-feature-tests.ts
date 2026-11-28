// Runs the experimental Cargo feature rows in tools/support/ci/rust-feature-tests.ts.
//
//   node tools/ci/commands/rust-feature-tests.ts matrix   # every row, packed into GitHub matrix jobs
//   node tools/ci/commands/rust-feature-tests.ts run "<row>, <row>, ..."   # one job's rows, in order
import { spawnSync } from "node:child_process";
import { appendFileSync, rmSync } from "node:fs";
import {
  allRustFeatureRows,
  rustFeatureCommands,
  rustFeatureRow,
} from "../../support/ci/rust-feature-tests.ts";
import { rustFeatureJobRows } from "../../support/ci/rust-feature-batches.ts";
import { runEntrypoint } from "../../support/command-runtime.ts";

function run(command: string, args: readonly string[], env: NodeJS.ProcessEnv): void {
  console.log(`::group::${[command, ...args].join(" ")}`);
  const result = spawnSync(command, args, { stdio: "inherit", env });
  console.log("::endgroup::");
  if (result.error) throw result.error;
  if (result.status !== 0)
    throw new Error(
      `${[command, ...args].join(" ")} exited with ${result.status ?? result.signal}`,
    );
}

function runRow(id: string): void {
  const row = rustFeatureRow(id);
  const env = { ...process.env, ...row.env };
  if (row.runtime) {
    // The provider receipt describes this run only; a reused target may hold an older one.
    rmSync("target/nextest/full/native-runtime-provider.json", { force: true });
    const head = spawnSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).stdout.trim();
    run("vp", ["exec", "node", "tools/support/compat/github/full-rust-runtime.ts"], {
      ...env,
      GITHUB_SHA: head,
    });
  }
  for (const args of rustFeatureCommands(row)) run("cargo", args, env);
}

// A job runs its rows in order. A failing row does not stop the later ones, so
// every row still reports, and the job fails if any row failed.
function runRows(ids: readonly string[]): void {
  for (const id of ids) rustFeatureRow(id);
  const failed: string[] = [];
  for (const id of ids) {
    console.log(`Feature row ${id}`);
    try {
      runRow(id);
    } catch (error) {
      failed.push(id);
      console.log(`::error title=Rust feature row ${id}::${(error as Error).message}`);
    }
  }
  if (failed.length > 0) throw new Error(`Failed feature rows: ${failed.join(", ")}`);
}

runEntrypoint((): number => {
  const [mode, id, ...rest] = process.argv.slice(2);
  if (mode === "matrix" && id === undefined) {
    const output = `matrix=${JSON.stringify(allRustFeatureRows())}\n`;
    if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, output);
    else process.stdout.write(output);
    return 0;
  }
  if (mode === "run" && id !== undefined && rest.length === 0) {
    runRows(rustFeatureJobRows(id));
    return 0;
  }
  throw new Error('Usage: rust-feature-tests.ts matrix | run "<row>, <row>, ..."');
});
