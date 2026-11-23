const fs = require("node:fs");
const [source, ready] = process.argv.slice(2);
fs.writeFileSync(ready, String(process.pid));
setInterval(() => fs.writeFileSync(source, "broken"), 10);
