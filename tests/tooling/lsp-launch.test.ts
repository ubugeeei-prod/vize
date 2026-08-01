import assert from "node:assert/strict";
import { test } from "node:test";

import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";

test("LSP tests ignore a globally installed vize binary", () => {
  const probed: string[] = [];
  const command = resolveVizeLaunchCommand((candidate) => {
    probed.push(candidate);
    return candidate === "vize";
  }, "");

  assert.ok(!probed.includes("vize"));
  assert.deepEqual(command, ["cargo", "run", "-q", "-p", "vize", "--", "lsp"]);
});
