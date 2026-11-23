const [status, message] = process.argv.slice(2);
if (message) require("node:fs").writeSync(2, message);
process.exit(Number(status));
