const fs = require("fs");
const f = process.argv[2];
const j = JSON.parse(fs.readFileSync(f, "utf8"));
j.version = process.argv[3];
fs.writeFileSync(f, JSON.stringify(j, null, 2));
