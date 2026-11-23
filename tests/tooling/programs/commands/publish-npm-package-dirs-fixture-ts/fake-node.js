const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const args = process.argv.slice(2);
if (args[0]?.replaceAll("\\", "/").endsWith("/publish_npm_package/main.ts")) {
  const packageName = path.basename(args[1]);
  const record = (phase) =>
    fs.appendFileSync(
      process.env.MOCK_PUBLISH_EVENTS,
      JSON.stringify({ phase, package: packageName, time: Date.now(), args: args.slice(2) }) + "\n",
    );
  record("start");
  setTimeout(
    () => {
      console.log("publisher stdout " + packageName);
      console.error("publisher stderr " + packageName);
      record("end");
      process.exit(packageName === process.env.MOCK_PUBLISH_FAIL_PACKAGE ? 7 : 0);
    },
    Number(process.env.MOCK_PUBLISH_DELAY_MS ?? "150"),
  );
} else {
  const result = spawnSync(process.env.REAL_NODE_BIN, args, { stdio: "inherit" });
  if (result.error) console.error(result.error);
  process.exit(result.status ?? 1);
}
