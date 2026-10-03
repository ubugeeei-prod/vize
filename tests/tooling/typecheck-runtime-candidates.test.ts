import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { resolveTypecheckRuntime } from "./support/typecheck-dependency.ts";

const variables = ["CORSA_BIN", "CORSA_PATH", "TSGO_PATH", "TSGO_EXECUTABLE", "CORSA_EXECUTABLE"];

test("typecheck runtime discovery ignores absent, false, empty and missing candidates", () => {
  withProject((root) => {
    const first = path.join(root, "first-runtime");
    const second = path.join(root, "second-runtime");
    fs.writeFileSync(first, "original first runtime bytes");
    fs.writeFileSync(second, "original second runtime bytes");
    assert.equal(
      resolveTypecheckRuntime(root, [
        null,
        undefined,
        false,
        "",
        path.join(root, "missing"),
        first,
        second,
      ]),
      first,
    );
    assert.equal(
      resolveTypecheckRuntime(root, [null, undefined, false, "", path.join(root, "missing")]),
      undefined,
    );
  });
});

test("typecheck runtime discovery keeps configured precedence over extra physical candidates", () => {
  withProject((root) => {
    const configured = path.join(root, "configured-runtime");
    const extra = path.join(root, "extra-runtime");
    fs.writeFileSync(configured, "configured original bytes");
    fs.writeFileSync(extra, "extra original bytes");
    process.env.CORSA_BIN = "";
    process.env.CORSA_PATH = configured;
    assert.equal(resolveTypecheckRuntime(root, [extra]), configured);
    assert.equal(fs.readFileSync(configured, "utf8"), "configured original bytes");
    assert.equal(fs.readFileSync(extra, "utf8"), "extra original bytes");
  });
});

function withProject(run: (root: string) => void) {
  const prior = variables.map((name) => [name, process.env[name]] as const);
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-typecheck-runtime-candidates-"));
  try {
    for (const name of variables) delete process.env[name];
    run(root);
  } finally {
    for (const [name, value] of prior) {
      if (value === undefined) delete process.env[name];
      else process.env[name] = value;
    }
    fs.rmSync(root, { recursive: true, force: true });
  }
}
