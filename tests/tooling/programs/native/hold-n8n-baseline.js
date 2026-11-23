import fs from "node:fs";
import { join } from "node:path";

const [directory, status] = process.argv.slice(2);
fs.writeFileSync(join(directory, "baseline.started"), "started");
console.log("whole baseline stdout");
console.error("whole baseline stderr");
const timer = setInterval(() => {
  if (!fs.existsSync(join(directory, "baseline.release"))) return;
  clearInterval(timer);
  fs.writeFileSync(join(directory, "baseline.done"), "joined vectors and reference");
  process.exitCode = Number(status);
}, 10);
