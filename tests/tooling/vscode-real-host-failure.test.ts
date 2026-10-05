import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { retainRealHostFailure } from "../../editors/vscode/test/real-host-failure.mjs";

test("real host failure preserves full log and controlled source bytes before cleanup", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-host-failure-"));
  const workspace = path.join(directory, "workspace");
  const destination = path.join(directory, "evidence");
  const log = Buffer.from("whole watched event\nwhole refusal stamp\n");
  const source = Buffer.from("<template>é\r\n<div>全体</div></template>\n");
  fs.mkdirSync(path.join(workspace, "node_modules", ".vize"), {
    recursive: true,
  });
  fs.mkdirSync(path.join(workspace, "src"));
  fs.writeFileSync(path.join(workspace, "node_modules", ".vize", "lsp.log"), log);
  fs.writeFileSync(path.join(workspace, "src", "Scenario.vue"), source);
  fs.writeFileSync(path.join(workspace, "node_modules", "unrelated-large-file"), "not evidence");
  try {
    const error = new Error("Content modified");
    const result = retainRealHostFailure(workspace, destination, error);
    fs.rmSync(workspace, { force: true, recursive: true });
    assert.deepEqual(
      fs.readFileSync(path.join(destination, "node_modules/.vize/lsp.log.txt")),
      log,
    );
    assert.deepEqual(fs.readFileSync(path.join(destination, "src/Scenario.vue.txt")), source);
    assert.deepEqual(result.captured, [
      {
        file: "node_modules/.vize/lsp.log",
        output: "node_modules/.vize/lsp.log.txt",
        bytes: log.length,
        sha256: crypto.createHash("sha256").update(log).digest("hex"),
      },
      {
        file: "src/Scenario.vue",
        output: "src/Scenario.vue.txt",
        bytes: source.length,
        sha256: crypto.createHash("sha256").update(source).digest("hex"),
      },
    ]);
    assert.equal(result.error.message, "Content modified");
    assert.ok(result.missing.includes("src/ContractChild.vue"));
    assert.equal(fs.existsSync(path.join(destination, "node_modules/unrelated-large-file")), false);
    assert.deepEqual(
      JSON.parse(fs.readFileSync(path.join(destination, "manifest.json"), "utf8")),
      result,
    );
  } finally {
    fs.rmSync(directory, { force: true, recursive: true });
  }
});

test("real host failure refuses a controlled path redirected outside the fixture", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-host-failure-path-"));
  const workspace = path.join(directory, "workspace");
  fs.mkdirSync(path.join(workspace, "src"), { recursive: true });
  const outside = path.join(directory, "outside.vue");
  fs.writeFileSync(outside, "outside data");
  fs.symlinkSync(outside, path.join(workspace, "src", "Scenario.vue"));
  try {
    assert.throws(
      () =>
        retainRealHostFailure(
          workspace,
          path.join(directory, "evidence"),
          new Error("original failure"),
        ),
      /escapes the workspace/,
    );
  } finally {
    fs.rmSync(directory, { force: true, recursive: true });
  }
});
