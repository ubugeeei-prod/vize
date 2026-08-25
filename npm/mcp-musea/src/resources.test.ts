import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

test("source resource resolves through the scanned art registry", () => {
  const source = fs.readFileSync(
    path.resolve(path.dirname(fileURLToPath(import.meta.url)), "resources.ts"),
    "utf8",
  );
  const start = source.indexOf('if (uri.startsWith("musea://source/"))');
  const end = source.indexOf('if (uri.startsWith("musea://component-source/"))');
  const block = source.slice(start, end);

  assert.notEqual(start, -1, "source resource handler must exist");
  assert.notEqual(end, -1, "component-source handler must follow the source handler");
  assert.match(
    block,
    /resolveArtReference\(ctx, \{ path: relativePath \}\)/,
    "musea://source must only open files that resolve as scanned art",
  );
  assert.doesNotMatch(
    block,
    /resolveProjectPath/,
    "a project-root path check alone would serve .env and other non-art files",
  );
});
