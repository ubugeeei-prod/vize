const fs = require("node:fs");
if (process.cwd().endsWith("b")) fs.writeFileSync("package.json", "{");
process.exit(0);
