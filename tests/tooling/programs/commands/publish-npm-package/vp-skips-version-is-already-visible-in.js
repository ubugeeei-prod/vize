const fs = require("node:fs");
const args = process.argv.slice(2);
fs.appendFileSync(process.env.VP_ARGS_LOG, `${args.join(" ")}\n`);
if (args[0] === "pm" && args[1] === "view" && args[3] === "version") {
  process.stdout.write(JSON.stringify("1.2.3-beta.1"));
  process.exit(0);
}
if (args[0] === "pm" && args[1] === "view" && args[3] === "dist-tags") {
  process.stdout.write(JSON.stringify({ beta: "1.2.3-beta.1" }));
  process.exit(0);
}
if (args[0] === "pm" && args[1] === "publish") process.exit(99);
process.exit(1);
