import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  nativeProcessStopped,
  nativeProcessWrapper,
} from "../../tools/support/compat/emacs-vize/run-real-server.ts";

test(
  "Emacs native PID wrapper preserves the real child's original stdin pipe",
  {
    skip: process.platform === "win32" ? "POSIX editor host wrapper" : false,
  },
  (t) => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-stdin-"));
    t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
    const pidFile = path.join(directory, "native.pid");
    const wrapper = path.join(directory, "native-server");
    fs.writeFileSync(wrapper, nativeProcessWrapper("/bin/cat", pidFile));
    const input = Buffer.from("The actual child must receive all pipe bytes.\n\0tail\n");
    const result = spawnSync("/bin/sh", [wrapper], { input, timeout: 5_000 });
    assert.ifError(result.error);
    assert.equal(result.status, 0, result.stderr.toString());
    assert.deepEqual(result.stdout, input);
    const nativePid = Number(fs.readFileSync(pidFile, "utf8").trim());
    assert.ok(Number.isInteger(nativePid) && nativePid > 0);
    assert.notEqual(nativePid, result.pid, "Record the native child, not the launcher shell");
    assert.equal(nativeProcessStopped(nativePid), true);
    assert.equal(
      nativeProcessStopped(process.pid),
      false,
      "A running process must not qualify as stopped",
    );
  },
);
