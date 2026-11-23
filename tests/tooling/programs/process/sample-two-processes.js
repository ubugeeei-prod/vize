import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

if (process.argv[2] !== "child") {
  spawn(process.execPath, [fileURLToPath(import.meta.url), "child"], { stdio: "ignore" });
}
setTimeout(() => {}, 30_000);
