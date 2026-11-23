const fs = require("node:fs");
const args = process.argv.slice(2);
const state = fs.existsSync(process.env.VP_STATE_PATH)
  ? JSON.parse(fs.readFileSync(process.env.VP_STATE_PATH, "utf8"))
  : { publishCalls: 0, tagChecks: 0, versionChecks: 0 };
function save() {
  fs.writeFileSync(process.env.VP_STATE_PATH, JSON.stringify(state));
}
if (args[0] === "pm" && args[1] === "view" && args[3] === "version") {
  state.versionChecks += 1;
  save();
  if (state.publishCalls === 1 && state.versionChecks >= 3) {
    process.stdout.write(JSON.stringify("1.2.3"));
    process.exit(0);
  }
  process.exit(1);
}
if (args[0] === "pm" && args[1] === "view" && args[3] === "dist-tags") {
  state.tagChecks += 1;
  save();
  if (state.publishCalls === 1) {
    process.stdout.write(JSON.stringify({ latest: "1.2.3" }));
    process.exit(0);
  }
  process.exit(1);
}
if (args[0] === "pm" && args[1] === "publish") {
  state.publishCalls += 1;
  save();
  fs.appendFileSync(process.env.NPM_FAKE_URL_LOG, "staged publish\n");
  process.stderr.write('Cannot publish over previously staged version "1.2.3".');
  process.exit(1);
}
process.exit(1);
