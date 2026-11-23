const fs = require("node:fs");
const args = process.argv.slice(2);
const state = fs.existsSync(process.env.VP_STATE_PATH)
  ? JSON.parse(fs.readFileSync(process.env.VP_STATE_PATH, "utf8"))
  : { published: false };
if (args[0] === "pm" && args[1] === "publish") {
  fs.writeFileSync(process.env.MANIFEST_LOG_PATH, fs.readFileSync("package.json", "utf8"));
  state.published = true;
  fs.writeFileSync(process.env.VP_STATE_PATH, JSON.stringify(state));
  process.exit(0);
}
if (args[0] === "pm" && args[1] === "view" && args[3] === "version") {
  if (state.published) {
    process.stdout.write(JSON.stringify("0.57.0"));
    process.exit(0);
  }
  process.exit(1);
}
if (args[0] === "pm" && args[1] === "view" && args[3] === "dist-tags") {
  if (state.published) {
    process.stdout.write(JSON.stringify({ latest: "0.57.0" }));
    process.exit(0);
  }
  process.exit(1);
}
process.exit(1);
