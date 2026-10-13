import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";

import {
  trustedServerPath,
  workspaceDevServerPaths,
} from "../../editors/vscode/src/server-trust.ts";

const repository = path.join(path.sep, "work", "cloned");
const setting = {
  globalValue: "",
  workspaceValue: "./bin/evil",
  workspaceFolderValue: " ./tools/evil ",
};

test("an untrusted workspace cannot choose the server through its settings", () => {
  assert.equal(trustedServerPath(setting, false), undefined);
  assert.equal(
    trustedServerPath({ ...setting, globalValue: "/usr/bin/vize" }, false),
    "/usr/bin/vize",
  );
});

test("a trusted workspace may choose the server through its settings", () => {
  assert.equal(trustedServerPath(setting, true), "./bin/evil");
  assert.equal(trustedServerPath({ workspaceFolderValue: " ./tools/vize " }, true), "./tools/vize");
  assert.equal(
    trustedServerPath({ globalValue: "/opt/vize", workspaceValue: "./x" }, true),
    "/opt/vize",
  );
  assert.equal(trustedServerPath(undefined, true), undefined);
});

test("an untrusted workspace contributes no development server binaries", () => {
  assert.deepEqual(workspaceDevServerPaths([repository], "vize", false), []);
  assert.deepEqual(workspaceDevServerPaths([repository], "vize", true), [
    path.join(repository, "target", "release", "vize"),
    path.join(repository, "target", "debug", "vize"),
  ]);
});
