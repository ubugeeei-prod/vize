const fs = require("node:fs");
process.on("SIGTERM", () => {
  fs.writeFileSync("descendant-stopped", String(process.pid));
  process.exit(0);
});
fs.writeFileSync("descendant-started", String(process.pid));
setInterval(() => {}, 1000);
