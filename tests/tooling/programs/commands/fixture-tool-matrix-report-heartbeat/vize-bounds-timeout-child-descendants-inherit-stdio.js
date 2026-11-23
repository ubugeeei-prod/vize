import { spawn } from "node:child_process";
if (process.argv[2] === "--hold-stdio") {
  setTimeout(() => {}, 4000);
} else if (process.argv[2] !== "--version") {
  spawn(process.execPath, [process.argv[1], "--hold-stdio"], { stdio: "inherit" });
  setInterval(() => {}, 1000);
}
