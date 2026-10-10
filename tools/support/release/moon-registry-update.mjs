import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

/** Keep Moon's undrained Git pull pipes below their presentation-only volume. */
export function registryUpdateEnvironment(environment = process.env) {
  const inheritedCount = environment.GIT_CONFIG_COUNT ?? "0";
  // Match Git's decimal count syntax, including empty=0 and signed zero.
  if (inheritedCount !== "" && !/^[ \t\n\r\f\v]*[+-]?\d+$/.test(inheritedCount)) {
    throw new Error("Invalid inherited GIT_CONFIG_COUNT; registry was not changed");
  }
  const count = Number(inheritedCount);
  if (!Number.isSafeInteger(count) || count < 0 || count >= 2_147_483_647) {
    throw new Error("Inherited Git configuration cannot safely accept another entry");
  }
  const result = {
    ...environment,
    GIT_CONFIG_COUNT: String(count + 1),
    [`GIT_CONFIG_KEY_${count}`]: "merge.stat",
    [`GIT_CONFIG_VALUE_${count}`]: "false",
  };
  // Git parses inherited `git -c` parameters after counted environment entries.
  // Retain their exact bytes and append one constant override, without eval or
  // parsing/logging any inherited value. Otherwise `-c merge.stat=true` wins.
  if (environment.GIT_CONFIG_PARAMETERS != null) {
    result.GIT_CONFIG_PARAMETERS = `${environment.GIT_CONFIG_PARAMETERS} 'merge.stat=false'`;
  }
  return result;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const [command, ...args] = process.argv.slice(2);
    if (!command) throw new Error("Usage: moon-registry-update.mjs COMMAND [ARG ...]");
    const result = spawnSync(command, args, {
      env: registryUpdateEnvironment(),
      stdio: "inherit",
      // CI's Windows MOON_BIN is a .cmd shim; Node cannot exec batch files.
      shell: process.platform === "win32" && /\.(cmd|bat)$/i.test(command),
    });
    if (result.error) throw result.error;
    if (result.signal) process.kill(process.pid, result.signal);
    else process.exitCode = result.status ?? 1;
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
