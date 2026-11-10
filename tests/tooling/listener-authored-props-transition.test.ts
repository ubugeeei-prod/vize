import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import {
  beforeListenerAuthoredPropsSource,
  listenerAuthoredPropsRecord,
} from "../../tools/support/compat/content-mapper/listener-authored-props-transition.ts";

const root = new URL("../../", import.meta.url);
const hash = (text: string) => createHash("sha256").update(text).digest("hex");

void test("authored listener props sources restore their exact predecessors", () => {
  const record = JSON.parse(fs.readFileSync(new URL(listenerAuthoredPropsRecord, root), "utf8")) as {
    files: Array<{
      path: string;
      before: { sha256: string };
      after: { sha256: string };
    }>;
  };
  for (const file of record.files) {
    const current = fs.readFileSync(new URL(file.path, root), "utf8");
    assert.equal(hash(current), file.after.sha256, file.path);
    const previous = beforeListenerAuthoredPropsSource(file.path, current);
    assert.equal(hash(previous), file.before.sha256, file.path);
    assert.equal(beforeListenerAuthoredPropsSource(file.path, previous), previous, file.path);
  }

});
