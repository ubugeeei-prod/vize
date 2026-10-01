import { execFileSync } from "node:child_process";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("host move preserves Rust imports, conditions and literal data", () => {
  execFileSync("python3", ["tests/tooling/support/host-runtime-move.py"], {
    cwd: root,
    stdio: "pipe",
  });
});
