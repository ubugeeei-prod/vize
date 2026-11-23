require("node:fs").writeFileSync(
  process.argv[2],
  JSON.stringify({ execPath: process.execPath, version: process.version }) + "\n",
);
