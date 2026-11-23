const args = process.argv.slice(2);
require("node:fs").appendFileSync(process.env.CALLS_LOG, args[4] + "\n");
process.exit(args[4] === "publish" ? 0 : 1);
