const fs = require("node:fs");
const state = JSON.parse(fs.readFileSync(process.env.VP_STATE_PATH, "utf8"));
state.publishCalls++;
state.invocations.push({ arguments: process.argv.slice(2), cwd: process.cwd() });
if (process.env.WRAP_GENERIC === "1" && state.publishCalls === 3) state.published = true;
fs.writeFileSync(process.env.VP_STATE_PATH, JSON.stringify(state));
if (state.published) process.exit(0);
process[process.env.WRAP_STREAM].write(fs.readFileSync(process.env.WRAP_DETAIL));
process.exit(1);
