import assert from "node:assert/strict";
import { test } from "node:test";

import { parseCliArgs } from "./cli.js";

void test("parses directory, force, and package-manager options", () => {
  assert.deepEqual(parseCliArgs(["dashboard", "--force", "--pm", "bun"]), {
    directory: "dashboard",
    force: true,
    packageManager: "bun",
    help: false,
    version: false,
  });
  assert.equal(parseCliArgs(["--package-manager=yarn"]).packageManager, "yarn");
});

void test("defaults to a conventional directory and rejects ambiguous input", () => {
  assert.equal(parseCliArgs([]).directory, "fresco-app");
  assert.throws(() => parseCliArgs(["one", "two"]), /CREATE_FRESCO_EXTRA_ARGUMENT/u);
  assert.throws(() => parseCliArgs(["--pm", "deno"]), /CREATE_FRESCO_PACKAGE_MANAGER/u);
  assert.throws(() => parseCliArgs(["--unknown"]), /CREATE_FRESCO_UNKNOWN_OPTION/u);
});
