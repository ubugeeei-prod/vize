import assert from "node:assert/strict";
import type { MergeDescriptor } from "./model.ts";
import type { GithubRead } from "./authority.ts";
import { bytes, gitBlob, object, text, sourcePath } from "./value.ts";
export async function authenticateFixCustody(
  fix: MergeDescriptor,
  selected: string,
  candidate: string,
  read: GithubRead,
): Promise<void> {
  assert(fix.paths.length > 0);
  for (const path of fix.paths) {
    sourcePath(path);
    let expected: string | undefined;
    for (const ref of [fix.originalHead, fix.merge, selected, candidate]) {
      const file = object(await read(`contents/${path}?ref=${ref}`));
      assert.equal(file.path, path);
      assert.equal(file.type, "file");
      assert.equal(file.encoding, "base64");
      const raw = bytes(text(file.content).replace(/\n/gu, ""));
      assert.equal(gitBlob(raw), file.sha);
      if (expected === undefined) expected = String(file.sha);
      else
        assert.equal(
          file.sha,
          expected,
          "actual reviewed H/M/S/C production or owned fixture bytes differ; reviewed successor required",
        );
    }
  }
}
