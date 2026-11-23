const assert = require("node:assert/strict");
const { loadConfigFromFile } = require("node:module").createRequire(
  require("node:path").resolve("package.json"),
)("vite-plus");
(async () => {
  const loaded = await loadConfigFromFile({ command: "build", mode: "production" });
  assert(loaded && loaded.config && loaded.config.run && loaded.config.run.tasks);
  process.stdout.write(JSON.stringify(loaded.config.run.tasks));
})().catch((e) => {
  console.error(e);
  process.exitCode = 1;
});
