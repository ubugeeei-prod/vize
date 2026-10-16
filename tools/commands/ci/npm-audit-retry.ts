#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { setTimeout } from "node:timers/promises";

function unsignedEnvironment(name: string, fallback: bigint, maximum: bigint): bigint {
  const value = process.env[name];
  if (!value || !/^\+?\d+$/.test(value)) return fallback;
  const parsed = BigInt(value);
  return parsed <= maximum ? parsed : fallback;
}

// Node timers overflow above 2^31 - 1 ms. Chunk configured waits instead of
// turning a long delay into an immediate retry.
async function wait(delay: bigint): Promise<void> {
  while (delay > 0n) {
    const chunk = delay > 2_147_483_647n ? 2_147_483_647n : delay;
    await setTimeout(Number(chunk));
    delay -= chunk;
  }
}

async function run(): Promise<number> {
  const attempts = Math.max(
    1,
    Number(unsignedEnvironment("VIZE_NPM_AUDIT_ATTEMPTS", 3n, 0xffff_ffffn)),
  );
  const retryDelay = unsignedEnvironment(
    "VIZE_NPM_AUDIT_RETRY_DELAY_MS",
    20_000n,
    0xffff_ffff_ffff_ffffn,
  );

  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    const executable = process.platform === "win32" ? (process.env.ComSpec ?? "cmd.exe") : "vp";
    const args = ["node", "tools/commands/ci/npm-audit.ts"];
    const commandArgs = process.platform === "win32" ? ["/d", "/s", "/c", "vp", ...args] : args;
    const result = spawnSync(executable, commandArgs, {
      stdio: "inherit",
    });
    if (result.error) {
      console.error(`npm-audit: failed to run vp: ${result.error.message}`);
      return 1;
    }
    if (result.status === 0) return 0;
    if (attempt === attempts) {
      return result.status !== null && result.status >= 0 && result.status <= 255
        ? result.status
        : 1;
    }

    const delay = retryDelay * BigInt(attempt);
    console.error(`npm-audit: attempt ${attempt} failed; retrying in ${delay / 1_000n}s...`);
    await wait(delay);
  }
  return 1;
}

process.exitCode = await run();
