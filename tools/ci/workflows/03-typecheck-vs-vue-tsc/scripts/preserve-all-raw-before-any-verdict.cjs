const fs = require("node:fs");
const p = process.env.RUNNER_TEMP + "/comparative";
fs.mkdirSync(p, { recursive: true });
fs.writeFileSync(
  p + "/controller.json",
  JSON.stringify({
    head: process.env.GITHUB_SHA,
    ref: process.env.GITHUB_REF,
    run: process.env.GITHUB_RUN_ID,
    event: process.env.GITHUB_EVENT_NAME,
  }),
);
