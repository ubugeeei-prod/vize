const fs = require("node:fs");
const args = process.argv.slice(2);
fs.writeFileSync(process.env.CURL_CALLS_PATH, JSON.stringify([args]));
const targetPlatform = args.at(-1).includes("/universal/") ? "universal" : "darwin-x64";
process.stdout.write(JSON.stringify({ version: "0.57.0", targetPlatform }));
