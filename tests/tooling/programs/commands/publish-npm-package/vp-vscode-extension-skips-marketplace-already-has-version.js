const fs = require("node:fs");
const args = process.argv.slice(2);
fs.appendFileSync(process.env.VP_ARGS_LOG, `${args.join(" ")}\n`);
if (args[0] === "dlx" && args[3] === "vsce" && args[4] === "publish") process.exit(99);
process.exit(1);
