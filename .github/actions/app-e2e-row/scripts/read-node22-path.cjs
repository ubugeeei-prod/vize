const { readFileSync } = require("node:fs");

const { version, execPath } = JSON.parse(readFileSync(process.argv[2], "utf8"));
if (!version.startsWith("v22.")) throw new Error("Nuxt preparation requires Node 22");
console.log(execPath);
