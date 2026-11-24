import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const rows = readFileSync(process.env.VIZE_TOOLCHAIN_HISTORY!, "utf8").split("\0");
const output = JSON.parse(readFileSync(process.env.VIZE_STYLED_SESSION_OUTPUT!, "utf8")) as Array<{
  source: string;
}>;
assert.equal(rows.length, 5);
assert.equal(rows[4], output[0]?.source);
