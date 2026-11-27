const fs = require("node:fs");
const args = process.argv.slice(2);
const calls = fs.existsSync(process.env.CALLS_PATH)
  ? JSON.parse(fs.readFileSync(process.env.CALLS_PATH, "utf8"))
  : [];
calls.push(args);
fs.writeFileSync(process.env.CALLS_PATH, JSON.stringify(calls));
const state = fs.existsSync(process.env.STATE_PATH)
  ? JSON.parse(fs.readFileSync(process.env.STATE_PATH, "utf8"))
  : { published: false };
if (
  args[0] === "dlx" &&
  args[2] === "ovsx@1.4.0" &&
  args[3] === "ovsx" &&
  args[4] === "create-namespace"
) {
  if (process.env.NAMESPACE_ERROR === "auth") {
    console.error("token is not authorized");
    process.exit(17);
  }
  console.error("Namespace already exists");
  process.exit(17);
}
if (args[0] === "dlx" && args[2] === "ovsx@1.4.0" && args[3] === "ovsx" && args[4] === "publish") {
  state.published = true;
  fs.writeFileSync(process.env.STATE_PATH, JSON.stringify(state));
  process.exit(0);
}
process.exit(1);
