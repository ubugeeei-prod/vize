import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { profiledLauncher } from "../../tools/support/compat/davinci/lib/lsp-profile-launch.ts";

await test(
  "profiling exec preserves literal paths and keeps version probes outside the profile",
  {
    skip: process.platform === "win32",
  },
  () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lsp-profile-launch-"));
    try {
      const server = path.join(root, "server ' $(printf expanded)");
      const profile = path.join(root, "profile ' `printf expanded`.json");
      fs.writeFileSync(server, "#!/bin/sh\nprintf '%s\\n' \"$@\"\n", { mode: 0o700 });
      const launch = profiledLauncher(server, root, profile);
      const version = spawnSync(launch, ["--version"], { encoding: "utf8" });
      assert.equal(version.status, 0, version.stderr);
      assert.equal(version.stdout, "--version\n");
      const session = spawnSync(launch, ["lsp", "--stdio"], { encoding: "utf8" });
      assert.equal(session.status, 0, session.stderr);
      assert.equal(session.stdout, `lsp\n--stdio\n--profile-json\n${profile}\n`);
      fs.writeFileSync(profile, "existing evidence");
      assert.throws(() => profiledLauncher(server, root, profile), /new profile file/u);
      assert.equal(fs.readFileSync(profile, "utf8"), "existing evidence");
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
);
