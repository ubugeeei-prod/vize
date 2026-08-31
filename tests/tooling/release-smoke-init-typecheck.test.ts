import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";

import { isPathInsideOrEqual } from "../../legacy-tools/npm/smoke-release-init-typecheck.mjs";

test("init typecheck smoke rejects only package roots inside the repository", () => {
  assert.equal(
    isPathInsideOrEqual("D:\\a\\vize\\vize", "D:\\a\\vize\\vize\\node_modules\\vize", path.win32),
    true,
  );
  assert.equal(
    isPathInsideOrEqual("D:\\a\\vize\\vize", "D:\\a\\vize\\other\\node_modules\\vize", path.win32),
    false,
  );
  assert.equal(
    isPathInsideOrEqual(
      "D:\\a\\vize\\vize",
      "C:\\Users\\runneradmin\\AppData\\Local\\Temp\\vize-release-smoke-AAk6yG\\install\\node_modules\\vize",
      path.win32,
    ),
    false,
  );
  assert.equal(isPathInsideOrEqual("/tmp/vize", "/tmp/vize/node_modules/vize", path.posix), true);
  assert.equal(
    isPathInsideOrEqual("/tmp/vize", "/tmp/vize-other/node_modules/vize", path.posix),
    false,
  );
});

test("init typecheck smoke treats .. as a path segment", () => {
  assert.equal(
    isPathInsideOrEqual(
      "D:\\a\\vize\\vize",
      "D:\\a\\vize\\vize\\..cache\\node_modules\\vize",
      path.win32,
    ),
    true,
  );
  assert.equal(isPathInsideOrEqual("D:\\a\\vize\\vize", "D:\\a\\vize", path.win32), false);
  assert.equal(
    isPathInsideOrEqual("/tmp/vize", "/tmp/vize/..cache/node_modules/vize", path.posix),
    true,
  );
  assert.equal(isPathInsideOrEqual("/tmp/vize", "/tmp", path.posix), false);
});
