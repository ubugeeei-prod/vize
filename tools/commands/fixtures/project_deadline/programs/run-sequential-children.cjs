const { spawnSync } = require("node:child_process");
const path = require("node:path");
for (const [phase, delay] of [
  ["first", 200],
  ["second", 600],
]) {
  console.log(phase + " start");
  spawnSync(process.execPath, [path.join(__dirname, "delay.cjs"), String(delay)], {
    stdio: "inherit",
  });
  console.log(phase + " done");
}
