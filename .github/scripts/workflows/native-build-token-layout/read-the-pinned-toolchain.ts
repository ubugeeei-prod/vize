import { readFileSync } from "node:fs";
const channel = /^channel = "([^"]+)"$/m.exec(
  readFileSync("source/rust-toolchain.toml", "utf8"),
)?.[1];
if (!channel) throw new Error("Missing source Rust toolchain");
console.log(`channel=${channel}`);
