const fs = require("node:fs");
const args = process.argv.slice(2);
fs.writeFileSync(process.env.VP_ARGS_LOG, args.join("\n"));
fs.writeFileSync(process.env.NPM_FAKE_STATE_PATH, JSON.stringify({ published: true }));
process.exit(0);
