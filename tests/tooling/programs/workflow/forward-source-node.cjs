const { spawnSync } = require("node:child_process");
const [env, exec, node, ...args] = process.argv.slice(2);
if (env !== "env" || exec !== "exec" || node !== "node") process.exit(19);
const result = spawnSync(process.env.TEST_PRODUCER_NODE, args, { stdio: "inherit" });
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
