require("node:fs").appendFileSync(process.env.CALLS_LOG, "curl\n");
process.exit(22);
