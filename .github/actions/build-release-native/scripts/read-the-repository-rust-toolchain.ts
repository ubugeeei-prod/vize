import { readFileSync } from "node:fs";
const channel = /^channel = "([^"]+)"$/m.exec(readFileSync("rust-toolchain.toml", "utf8"))?.[1];
if (!channel) throw new Error("Missing repository Rust toolchain channel");
console.log(`channel=${channel}`);
