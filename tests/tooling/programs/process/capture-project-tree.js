import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

const mode = process.argv[2];
if (mode !== "child") {
  const child = spawn(process.execPath, [fileURLToPath(import.meta.url), "child"], {
    stdio: "ignore",
  });
  console.log(JSON.stringify({ pid: process.pid, child: child.pid }));
}
if (mode === "exit") setTimeout(() => process.exit(0), 10);
else {
  process.on("SIGTERM", () => {});
  setInterval(() => {}, 1000);
}
