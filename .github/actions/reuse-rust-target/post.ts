import { rmSync } from "node:fs";
import { join } from "node:path";
import { pruneRunOutputs } from "./prune-run-outputs.ts";
import { manifestName, recordSourceMtimes } from "./source-mtimes.ts";

const target = process.env.STATE_target;
if (target) {
  // Only Cargo's cache is committed. Run outputs (coverage profraw files from
  // ~19k test processes, reports, receipts) otherwise ride along and grew one
  // committed disk past what the provider still restores.
  const removed = pruneRunOutputs(target);
  console.log(`Pruned ${removed.length} run output(s) before committing the Rust target.`);
  try {
    const result = recordSourceMtimes({ target });
    console.log(
      result.recorded
        ? `Recorded ${result.files} Rust source mtimes; ${result.rewritten} changed during the run.`
        : `Rust source mtimes not recorded: ${result.reason}.`,
    );
  } catch (error) {
    rmSync(join(target, manifestName), { force: true });
    console.log(`::warning::Rust source mtimes not recorded: ${String(error)}`);
  }
}
