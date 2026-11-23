const fs = require("node:fs");
const args = process.argv.slice(2);
const state = fs.existsSync(process.env.VP_STATE_PATH)
  ? JSON.parse(fs.readFileSync(process.env.VP_STATE_PATH, "utf8"))
  : { publishCalls: 0, versionVisible: false };
if (args[0] === "pm" && args[1] === "view" && args[3] === "version") {
  if (state.versionVisible) {
    process.stdout.write(JSON.stringify("1.2.3"));
    process.exit(0);
  }
  process.exit(1);
}
if (args[0] === "pm" && args[1] === "view" && args[3] === "dist-tags") {
  if (state.versionVisible) {
    process.stdout.write(JSON.stringify({ latest: "1.2.3" }));
    process.exit(0);
  }
  process.exit(1);
}
if (args[0] === "pm" && args[1] === "publish") {
  state.publishCalls += 1;
  state.versionVisible = true;
  fs.writeFileSync(process.env.VP_STATE_PATH, JSON.stringify(state));
  process.exit(1);
}
process.exit(1);
