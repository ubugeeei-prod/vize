const { writeFileSync } = require("node:fs");
writeFileSync(process.env.AMBIENT_NODE_LOG, "ambient");
