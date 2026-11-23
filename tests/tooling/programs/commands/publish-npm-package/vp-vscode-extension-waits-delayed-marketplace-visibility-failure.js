const fs = require("node:fs");
const args = process.argv.slice(2);
const state = fs.existsSync(process.env.VP_STATE_PATH)
  ? JSON.parse(fs.readFileSync(process.env.VP_STATE_PATH, "utf8"))
  : { publishCalls: 0, queryCalls: 0 };
if (args[0] === "dlx" && args[3] === "vsce" && args[4] === "publish") {
  state.publishCalls += 1;
  fs.writeFileSync(process.env.VP_STATE_PATH, JSON.stringify(state));
  process.exit(1);
}
process.exit(1);
