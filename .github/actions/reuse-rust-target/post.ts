import { rmSync } from "node:fs";
import { join } from "node:path";
import { manifestName, recordSourceMtimes } from "./source-mtimes.ts";

const target = process.env.STATE_target;
if (target) {
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
