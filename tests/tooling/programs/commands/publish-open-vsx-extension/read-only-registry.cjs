const fs = require("node:fs");
const state = JSON.parse(fs.readFileSync(process.env.STATE_PATH, "utf8"));
const tool = require("node:path").basename(process.argv[1]);
const args = process.argv.slice(2);
const name = tool === "curl" ? "curl" : args[4];
state.calls.push(name);
const refusals = {
  "create-namespace": Number(process.env.NAMESPACE_REFUSALS),
  publish: Number(process.env.PUBLISH_REFUSALS),
};
const seen = state.calls.filter((call) => call === name).length;
if (name in refusals && seen <= refusals[name]) {
  console.error("❌  Registry is in read-only mode.");
  process.exitCode = 1;
} else if (name === "create-namespace") {
  console.error("Namespace already exists");
  process.exitCode = 1;
} else if (name === "publish") {
  state.published = true;
  console.log("published");
} else if (state.published) process.stdout.write(JSON.stringify({ version: "0.57.0" }));
else process.exitCode = 22;
fs.writeFileSync(process.env.STATE_PATH, JSON.stringify(state));
