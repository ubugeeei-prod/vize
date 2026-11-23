import fs from "node:fs";
import path from "node:path";
const root = process.env.FIXTURE_ROOT;
process.stdout.write("complete remaining stdout\n");
process.stderr.write("complete remaining stderr\n");
const timer = setInterval(() => {
  if (!["0.ready.json", "1.ready.json"].every((name) => fs.existsSync(path.join(root, name))))
    return;
  clearInterval(timer);
  fs.writeFileSync(path.join(root, "remaining.started"), "started");
  if (process.env.FIXTURE_MODE === "parent-cancel") setInterval(() => {}, 1000);
  else
    setTimeout(() => {
      fs.writeFileSync(path.join(root, "remaining.done"), "complete");
    }, 150);
}, 10);
