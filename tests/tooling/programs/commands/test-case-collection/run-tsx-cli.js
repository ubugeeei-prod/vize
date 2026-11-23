import { spawnSync } from "node:child_process";
const result = spawnSync(process.execPath, process.argv.slice(2), {
  stdio: "inherit",
  env: process.env,
});
process.exit(result.status ?? 1);
