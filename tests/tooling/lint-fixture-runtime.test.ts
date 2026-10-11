import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { resolveLintEnvironmentCli } from "../_helpers/lint-fixture-runtime.ts";

test(
  "environment commands use the installed native CLI even inside a script's local vp PATH",
  { skip: process.platform === "win32" },
  () => {
    const home = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lint-runtime-"));
    try {
      const native = path.join(home, ".vite-plus", "bin");
      const local = path.join(home, "project", "node_modules", ".bin");
      for (const dir of [native, local]) fs.mkdirSync(dir, { recursive: true });
      fs.writeFileSync(
        path.join(native, "vp"),
        '#!/bin/sh\nprintf "native environment: %s" "$*"\n',
        { mode: 0o755 },
      );
      fs.writeFileSync(
        path.join(local, "vp"),
        '#!/bin/sh\necho "Command env not found" >&2\nexit 1\n',
        { mode: 0o755 },
      );
      const env = { ...process.env, PATH: `${local}${path.delimiter}${process.env.PATH ?? ""}` };
      assert.throws(() => execFileSync("vp", ["env", "exec"], { env, stdio: "pipe" }));
      const result = execFileSync(
        resolveLintEnvironmentCli(home),
        ["env", "exec", "--node", "24.14.0", "pnpm", "--version"],
        { env, encoding: "utf8" },
      );
      assert.equal(result, "native environment: env exec --node 24.14.0 pnpm --version");
    } finally {
      fs.rmSync(home, { recursive: true, force: true });
    }
  },
);

test("missing or invalid native installation never falls back to a local project CLI", () => {
  const home = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lint-runtime-"));
  try {
    assert.throws(() => resolveLintEnvironmentCli(home), /installed by setup-vp/);
    const binary = path.join(
      home,
      ".vite-plus",
      "bin",
      process.platform === "win32" ? "vp.exe" : "vp",
    );
    fs.mkdirSync(binary, { recursive: true });
    assert.throws(() => resolveLintEnvironmentCli(home), /installed file/);
  } finally {
    fs.rmSync(home, { recursive: true, force: true });
  }
});
