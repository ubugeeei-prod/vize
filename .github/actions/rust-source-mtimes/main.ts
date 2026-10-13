import { appendFileSync, rmSync } from "node:fs";
import { join } from "node:path";
import { manifestName, restoreSourceMtimes } from "./source-mtimes.ts";

const input = (name: string) => process.env[`INPUT_${name.toUpperCase()}`] ?? "";
const record = input("record") === "true";
const target = input("target-path");
const started = performance.now();
try {
  if (!target) throw new Error("target-path is required");
  const result = restoreSourceMtimes({
    target,
    secondary: input("secondary-target-path"),
    record,
  });
  // The post step records the final tree only for the run that may commit it.
  if (record && result.applied && process.env.GITHUB_STATE)
    appendFileSync(process.env.GITHUB_STATE, `target=${target}\n`);
  const elapsed = Math.round(performance.now() - started);
  console.log(
    result.applied
      ? `Rust source mtimes (${result.reason}): ${result.restored} restored, ${result.changed} newer than ${result.floor} in ${elapsed} ms.`
      : `Rust source mtimes left as checked out: ${result.reason}.`,
  );
} catch (error) {
  // Checked-out mtimes are always safe; only a stale record could be harmful.
  if (record && target) rmSync(join(target, manifestName), { force: true });
  console.log(`::warning::Rust source mtimes left as checked out: ${String(error)}`);
}
