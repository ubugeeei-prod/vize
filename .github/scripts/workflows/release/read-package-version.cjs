const { readFileSync } = require("node:fs");
console.log(JSON.parse(readFileSync(process.argv[2], "utf8")).version);
