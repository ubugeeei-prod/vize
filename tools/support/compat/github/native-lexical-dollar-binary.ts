import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const packet = JSON.parse(readFileSync(process.argv[2], "utf8")) as {
  testBinary: { path: string };
};
assert.ok(typeof packet.testBinary.path === "string" && packet.testBinary.path);
console.log(packet.testBinary.path);
