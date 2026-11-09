import { spawnSync } from "node:child_process";
import { isEntrypoint } from "../../../support/command-runtime.ts";

const commands = [
  ["check", "-p", "vize_maestro", "--no-default-features"],
  ["test", "-p", "vize_maestro", "--no-default-features", "--test", "non_native_structural"],
  ["check", "-p", "vize_maestro", "--no-default-features", "--features", "glyph"],
  [
    "test",
    "-p",
    "vize_maestro",
    "--no-default-features",
    "--features",
    "glyph",
    "--test",
    "non_native_structural",
  ],
] as const;

export function checkMaestroFeatureContract(arguments_: readonly string[]): number {
  if (arguments_.length > 0) {
    console.error(
      `Usage: node tools/commands/ci/github/check-maestro-feature-contract.ts; unexpected argument ${arguments_[0]}`,
    );
    return 1;
  }
  for (const arguments__ of commands) {
    const result = spawnSync("cargo", arguments__, {
      env: { ...process.env, RUSTFLAGS: "-D warnings" },
      stdio: "inherit",
    });
    if (result.error) {
      console.error(`failed to run cargo ${arguments__.join(" ")}: ${result.error.message}`);
      return 1;
    }
    if (result.status !== 0) return result.status ?? 1;
  }
  return 0;
}

if (isEntrypoint(import.meta.url)) {
  process.exitCode = checkMaestroFeatureContract(process.argv.slice(2));
}
