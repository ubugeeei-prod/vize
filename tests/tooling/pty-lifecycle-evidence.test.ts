import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs, { type Dir } from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { commandPhase, processChannels } from "./support/pty-command.ts";

const helper = new URL("./support/pty-command.ts", import.meta.url).pathname;

void test("PTY phases identify Git transport stages without retaining argument values", () => {
  const phases = [
    ["git", "-C", "/private/credential", "pull", "origin", "main"],
    ["git", "fetch", "https://credential@private.invalid/index"],
    ["git", "remote-https", "origin", "https://credential@private.invalid/index"],
    ["/usr/lib/git-core/git-remote-https", "origin", "private-value"],
    ["ssh", "private-host", "private-command"],
    ["/private/unknown-credential", "private-value"],
    ["moon", "update"],
    ["git", "unknown-operation"],
  ].map(commandPhase);
  assert.deepEqual(phases, [
    "git:pull",
    "git:fetch",
    "git:remote-https",
    "git-remote-https",
    "ssh",
    "other",
    "moon:update",
    "git",
  ]);
  assert.doesNotMatch(JSON.stringify(phases), /private|credential@|unknown-operation/);
});

void test("PTY Git phases distinguish global option values from the actual subcommand", () => {
  const phases = [
    ["git", "config", "credential.helper", "pull"],
    ["git", "-C", "fetch", "status"],
    ["git", "-C", "fetch", "config", "credential.helper", "pull"],
    ["git", "-c", "credential.helper=fetch", "config", "credential.helper", "pull"],
    ["git", "-Cprivate", "-ccredential.helper=fetch", "--no-pager", "pull"],
    ["git", "--git-dir=fetch", "--work-tree", "pull", "remote-https"],
    ["git", "--namespace=fetch", "--config-env=credential.helper=PRIVATE_VALUE", "config"],
    ["git", "-C", "", "-C", "private", "fetch"],
    ["git", "-C"],
    ["git", "-c", ""],
    ["git", "--config-env=fetch", "pull"],
    ["git", "--unknown", "private", "pull"],
    ["git", "--", "fetch"],
    ["git", "unknown-operation", "fetch"],
    ["moon", "run", "update"],
  ].map(commandPhase);
  assert.deepEqual(phases, [
    "git:config",
    "git",
    "git:config",
    "git:config",
    "git:pull",
    "git:remote-https",
    "git:config",
    "git:fetch",
    "git",
    "git",
    "git",
    "git",
    "git",
    "git",
    "moon:run",
  ]);
  assert.doesNotMatch(JSON.stringify(phases), /private|credential|PRIVATE_VALUE|unknown/);
});

void test("PTY channel scans close their directory at both entry and emitted-channel bounds", () => {
  const observations = [false, true].map((anonymous) => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "pty-channels-"));
    try {
      fs.mkdirSync(path.join(root, "fd"));
      for (let index = 0; index < 300; index += 1) {
        const target = anonymous ? `pipe:[${index}]` : `/private/credential-${index}`;
        fs.symlinkSync(target, path.join(root, "fd", String(index)));
      }
      const count = { examined: 0, closed: false };
      const observed = (directory: string): Dir => {
        const dir = fs.opendirSync(directory);
        const readSync = dir.readSync.bind(dir);
        const closeSync = dir.closeSync.bind(dir);
        return Object.assign(dir, {
          readSync: () => {
            const entry = readSync();
            if (entry !== null) count.examined += 1;
            return entry;
          },
          closeSync: () => {
            closeSync();
            count.closed = true;
          },
        });
      };
      const channels = processChannels(root, observed);
      return { ...count, channels: Object.keys(channels).length };
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  });
  assert.deepEqual(observations, [
    { examined: 256, closed: true, channels: 0 },
    { examined: 64, closed: true, channels: 64 },
  ]);
});

void test(
  "PTY Linux evidence relates anonymous pipe and socket owners while excluding file paths",
  { skip: process.platform !== "linux" },
  async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "private-credential-"));
    const privatePath = path.join(root, "private-credential");
    const privateFile = fs.openSync(privatePath, "w");
    const server = net.createServer();
    await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
    const address = server.address();
    assert.ok(address !== null && typeof address === "object");
    const socket = net.connect(address.port, "127.0.0.1");
    await new Promise<void>((resolve) => socket.once("connect", resolve));
    try {
      const channels = processChannels(`/proc/${process.pid}`);
      const sockets = Object.values(channels).filter((target) => target.startsWith("socket:["));
      assert.ok(sockets.length > 0, JSON.stringify(channels));
      for (const target of Object.values(channels))
        assert.match(target, /^(?:pipe|socket):\[\d+\]$/u);
      assert.equal(Object.hasOwn(channels, String(privateFile)), false);
      assert.doesNotMatch(JSON.stringify(channels), /private|credential|\/tmp\//u);
    } finally {
      socket.destroy();
      server.close();
      fs.closeSync(privateFile);
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
);

void test(
  "PTY command answers its prompt through a controlling terminal",
  { skip: process.platform === "win32" },
  () => {
    const script =
      'test -t 0 || exit 9; printf "Continue? "; read answer; echo "got:$answer"; exit 3';
    const result = spawnSync(
      process.execPath,
      [helper, "Continue?", "yes\n", "/bin/sh", "-c", script],
      { encoding: "utf8", timeout: 30_000 },
    );
    assert.equal(result.error, undefined, result.stderr);
    assert.equal(result.status, 3, result.stdout + result.stderr);
    assert.match(result.stdout, /got:yes/u);
    const unanswered = spawnSync(
      process.execPath,
      [helper, "Never", "x\n", "/bin/sh", "-c", "exit 0"],
      {
        encoding: "utf8",
        timeout: 30_000,
      },
    );
    assert.equal(unanswered.status, 125, unanswered.stdout + unanswered.stderr);
  },
);
