import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

// Run through the workspace package script so the native producer and consumer
// use the same pinned package-manager Node executable, including its exact path.
const args = process.argv.slice(2);
if (args[0] === "--") args.shift();
const profile = process.env.VIZE_PARCEL_PROFILE_DIR;
const child = spawnSync(
  process.execPath,
  [
    ...(profile ? ["--cpu-prof", `--cpu-prof-dir=${profile}`] : []),
    fileURLToPath(new URL("./host.mjs", import.meta.url)),
    ...args,
  ],
  { stdio: "inherit", env: process.env },
);
if (child.error) throw child.error;
if (child.signal) throw new Error(`Parcel qualification terminated by ${child.signal}`);
process.exitCode = child.status ?? 1;
