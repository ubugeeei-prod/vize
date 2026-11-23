const fs = require("node:fs");
const args = process.argv.slice(2);
const calls = fs.existsSync(process.env.CURL_CALLS_PATH)
  ? JSON.parse(fs.readFileSync(process.env.CURL_CALLS_PATH, "utf8"))
  : [];
calls.push(args);
fs.writeFileSync(process.env.CURL_CALLS_PATH, JSON.stringify(calls));
const state = fs.existsSync(process.env.STATE_PATH)
  ? JSON.parse(fs.readFileSync(process.env.STATE_PATH, "utf8"))
  : { published: false };
if (!state.published) process.exit(22); // curl --fail on HTTP 404
process.stdout.write(JSON.stringify({ version: "0.57.0" }));
