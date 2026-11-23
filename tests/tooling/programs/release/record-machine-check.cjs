const { writeFileSync } = require("node:fs");
writeFileSync(
  "machine-invocation.json",
  JSON.stringify({ argv: process.argv.slice(2), corsa: process.env.CORSA_PATH ?? null }),
);
process.stdout.write(JSON.stringify({ files: [], errorCount: 0, warningCount: 0, fileCount: 0 }));
