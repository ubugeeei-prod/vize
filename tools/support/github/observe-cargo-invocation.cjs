const fs = require("node:fs");
const { spawnSync } = require("node:child_process");
const { invocation, originalCargo, realRustc, originalPath } = JSON.parse("@@PROGRAM_SETTINGS@@");
const argv = process.argv.slice(2);
const observed = {
  argv,
  cwd: process.cwd(),
  token: process.env.VIZE_NATIVE_BUILD_TOKEN,
  cargo: originalCargo,
  rustc: realRustc,
};
fs.writeFileSync(invocation, JSON.stringify(observed, null, 2) + "\n", { flag: "wx" });
const result = spawnSync("cargo", argv, {
  env: { ...process.env, PATH: originalPath, RUSTC: realRustc },
  stdio: "inherit",
});
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
