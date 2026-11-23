import { spawnSync } from "node:child_process";

const option = process.argv[2];
const debug = (process.env.usage_debug || option) === "true" || option === "--debug";
const args = ["install", "--path", "crates/vize", "--force", "--locked"];
if (debug) args.push("--debug");
const result = spawnSync("cargo", args, { stdio: "inherit" });
if (result.status === 0)
  console.log("Installed vize CLI (" + (debug ? "debug" : "release") + " build)");
if (result.error) console.error(result.error.message);
process.exitCode = result.status ?? 1;
