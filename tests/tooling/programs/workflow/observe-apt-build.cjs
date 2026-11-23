const fs = require("node:fs");
const { spawn } = require("node:child_process");
const dir = process.env.PROBE_DIR;
const record = (value) => fs.appendFileSync(dir + "/events", value + "\n");
if (
  JSON.stringify(process.argv.slice(2)) !==
  JSON.stringify(["build", "--profile", "ci", "-p", "vize"])
)
  process.exit(99);
const child = spawn(process.execPath, [process.env.HELD_PROCESS], { stdio: "inherit" });
fs.writeFileSync(dir + "/child-pid", String(child.pid));
let stopping = false;
let exitStatus = 143;
child.once("close", () => {
  record("child-closed");
  record("cli-done");
  process.exit(exitStatus);
});
const stop = (status) => {
  if (stopping) return;
  stopping = true;
  exitStatus = status;
  child.kill("SIGTERM");
};
process.on("SIGTERM", () => stop(143));
record("cli-start");
fs.writeFileSync(dir + "/ready", "ready\n");
fs.createReadStream(dir + "/gate").once("data", () => {
  if (process.env.CANCEL !== "1") stop(Number(process.env.CLI_STATUS));
});
