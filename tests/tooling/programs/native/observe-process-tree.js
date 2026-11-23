import { spawn } from "node:child_process";
import { writeFileSync } from "node:fs";
const [mode, marker] = process.argv.slice(2);
if (mode === "parent") {
  const child = spawn(process.execPath, [process.argv[1], "child", marker], {
    detached: true,
    stdio: "ignore",
  });
  child.unref();
} else if (mode === "child") {
  writeFileSync(marker, JSON.stringify({ pid: process.pid }));
  setInterval(() => {}, 1000);
} else {
  throw new Error("expected parent or child");
}
