import { spawn } from "node:child_process";
import { writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const marker = process.argv[2];
if (marker) {
  const child = spawn(process.execPath, [fileURLToPath(import.meta.url)], { stdio: "ignore" });
  writeFileSync(marker, String(child.pid));
}
setInterval(() => {}, 1000);
