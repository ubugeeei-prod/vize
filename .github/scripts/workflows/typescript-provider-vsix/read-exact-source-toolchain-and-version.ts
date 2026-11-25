import { requiredEnv } from "../../../../tools/ci/lib/scripts/files.ts";
import { readFileSync, appendFileSync } from "node:fs";
const channel = /^channel = "([^"]+)"$/m.exec(readFileSync("rust-toolchain.toml", "utf8"))?.[1];
const { version } = JSON.parse(readFileSync("npm/native/package.json", "utf8"));
if (!channel || !/^\d+\.\d+\.\d+(?:-[\w.-]+)?$/.test(version))
  throw new Error("Invalid source toolchain/version");
appendFileSync(requiredEnv("GITHUB_OUTPUT"), `channel=${channel}\nversion=${version}\n`);
