const fs = require("node:fs");
const args = process.argv.slice(2);
fs.writeFileSync(process.env.NPX_ARGS_LOG, args.join("\n"));
process.exit(0);
