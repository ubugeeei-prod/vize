const fs = require("node:fs");
const crypto = require("node:crypto");
fs.appendFileSync(
  process.env.GITHUB_OUTPUT,
  "path=" +
    process.execPath +
    "\nsha256=" +
    crypto.createHash("sha256").update(fs.readFileSync(process.execPath)).digest("hex") +
    "\n",
);
