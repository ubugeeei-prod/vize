const fs = require("node:fs");
const args = process.argv.slice(2);
const output = args[args.indexOf("--output") + 1];
if (args.at(-1).includes("static.crates.io")) {
  fs.writeFileSync(output, "changed Rust API");
  process.stdout.write("200");
  process.exit(0);
}
const mode = process.env.SEMVER_REGISTRY_MODE;
if (mode === "404" || mode === "403") {
  fs.writeFileSync(output, "missing");
  process.stdout.write(mode);
  process.exit(22);
}
fs.writeFileSync(
  output,
  mode === "malformed"
    ? "bad json"
    : JSON.stringify({ name: "vize_l1_to_l2", vers: "0.428.1", yanked: false }),
);
process.stdout.write("200");
