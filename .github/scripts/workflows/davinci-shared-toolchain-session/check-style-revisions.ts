import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

function firstSource(variable: string): string | undefined {
  return (JSON.parse(readFileSync(process.env[variable]!, "utf8")) as Array<{ source: string }>)[0]
    ?.source;
}

const rows = readFileSync(process.env.VIZE_TOOLCHAIN_HISTORY!, "utf8").split("\0");
assert.equal(rows.length, 6);
assert.equal(rows[4], firstSource("VIZE_STYLED_SESSION_OUTPUT"));
assert.equal(rows[5], firstSource("VIZE_SCOPED_SESSION_OUTPUT"));
