import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { test } from "node:test";

const acceptedPath = process.env.VIZE_NATIVE_SFC_STRICT_SETUP_CAPTURE;
const refusedPath = process.env.VIZE_NATIVE_SFC_STRICT_SETUP_REFUSAL_CAPTURE;
const accepted = acceptedPath ? JSON.parse(fs.readFileSync(acceptedPath, "utf8")) : null;
const refused = refusedPath ? JSON.parse(fs.readFileSync(refusedPath, "utf8")) : null;

function check(code: string) {
  const result = spawnSync(process.execPath, ["--input-type=module", "--check"], {
    input: code,
    encoding: "utf8",
  });
  assert.ifError(result.error);
  assert.equal(result.signal, null);
  return result;
}

test(
  "source-built strict-safe setup modules preserve valid escapes and parse in Node",
  { skip: !accepted },
  () => {
    assert.equal(accepted.length, 11);
    for (const row of accepted) {
      assert.equal(row.accepted, true);
      assert.equal(typeof row.source, "string");
      const checked = check(row.code);
      assert.equal(checked.status, 0, `${row.source}\n${checked.stderr}`);
    }
  },
);

test(
  "original source-built native refusals agree with Node strict module early errors",
  { skip: !refused },
  () => {
    assert.equal(refused.length, 39);
    for (const row of refused) {
      assert.equal(row.accepted, false);
      assert.equal(typeof row.source, "string");
      assert.equal(row.code, undefined);
      const checked = check(row.script);
      assert.equal(checked.status, 1, row.source);
      assert.match(checked.stderr, /SyntaxError:/);
    }
  },
);
