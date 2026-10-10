import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
const { sourceJudgeFixture } = await import("./bench-source-harness-fixture.mjs");
export const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
export const comparePath = path.join(repoRoot, "tools", "commands", "davinci", "bench-compare.rs");

export function sourceCompare(f: ReturnType<typeof sourceJudgeFixture>, args: string[] = []) {
  return spawnSync(
    "rust-script",
    [comparePath, ...f.args, "--source-harness-pair", f.pair, ...args],
    { cwd: repoRoot, encoding: "utf8", env: { ...process.env, DAVINCI_BASELINE_REFRESH: "0" } },
  );
}
