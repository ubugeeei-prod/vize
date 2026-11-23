const fs = require("node:fs");
const path = require("node:path");
const args = process.argv.slice(2);
const state = fs.existsSync(process.env.VP_STATE_PATH)
  ? JSON.parse(fs.readFileSync(process.env.VP_STATE_PATH, "utf8"))
  : { published: {}, order: [] };
const packageJson = path.join(process.cwd(), "package.json");
if (args[0] === "pm" && args[1] === "publish") {
  const pkg = JSON.parse(fs.readFileSync(packageJson, "utf8"));
  state.published[pkg.name] = pkg.version;
  state.order.push(pkg.name);
  fs.writeFileSync(process.env.VP_STATE_PATH, JSON.stringify(state));
  process.exit(0);
}
if (args[0] === "pm" && args[1] === "view" && args[3] === "version") {
  const spec = args[2];
  const splitAt = spec.lastIndexOf("@");
  const name = spec.slice(0, splitAt);
  const version = state.published[name];
  if (version) {
    process.stdout.write(JSON.stringify(version));
    process.exit(0);
  }
  process.exit(1);
}
if (args[0] === "pm" && args[1] === "view" && args[3] === "dist-tags") {
  const name = args[2];
  const version = state.published[name];
  if (version) {
    process.stdout.write(JSON.stringify({ latest: version }));
    process.exit(0);
  }
  process.exit(1);
}
process.exit(1);
