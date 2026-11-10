import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import {
  applyListenerReturnParityIn,
  beforeListenerReturnParity,
  listenerReturnParityRecord,
} from "../../tools/support/compat/content-mapper/listener-return-parity-transition.ts";

const root = new URL("../../", import.meta.url);
const hash = (text: string) => createHash("sha256").update(text).digest("hex");

void test("listener return parity sources restore their exact predecessors", () => {
  const record = JSON.parse(fs.readFileSync(new URL(listenerReturnParityRecord, root), "utf8")) as {
    files: Array<{
      path: string;
      before: { sha256: string };
      after: { sha256: string };
    }>;
  };
  for (const file of record.files) {
    const current = fs.readFileSync(new URL(file.path, root), "utf8");
    assert.equal(hash(current), file.after.sha256, file.path);
    const previous = beforeListenerReturnParity(file.path, current);
    assert.equal(hash(previous), file.before.sha256, file.path);
    if (file.path.endsWith("shared-listener-args.d.ts.txt"))
      assert.equal(applyListenerReturnParityIn(previous), current, file.path);
    assert.equal(beforeListenerReturnParity(file.path, previous), previous, file.path);
  }
});
