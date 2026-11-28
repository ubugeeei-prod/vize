import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { createOwnedScratch } from "./owned-scratch.ts";

const moduleUrl = new URL("./owned-scratch.ts", import.meta.url).href;
const host = createHash("sha256").update(os.hostname()).digest("hex").slice(0, 8);

function withBase(run: (base: string) => void): void {
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "vize-owned-scratch-"));
  try {
    run(base);
  } finally {
    fs.rmSync(base, { recursive: true, force: true });
  }
}

/** A pid that has exited: the child is reaped before spawnSync returns. */
function exitedPid(): number {
  const child = spawnSync(process.execPath, ["-e", ""]);
  assert.equal(child.status, 0);
  return child.pid;
}

void test("a scratch entry is private and named for this host and process", () => {
  withBase((base) => {
    const root = path.join(base, "nested", "root");
    const scratch = createOwnedScratch(root, "oxlint-vize-");
    assert.equal(path.dirname(scratch), root);
    assert.match(path.basename(scratch), new RegExp(`^oxlint-vize-${host}-${process.pid}-\\w{6}$`));
    if (process.platform !== "win32") {
      assert.equal(fs.statSync(root).mode & 0o777, 0o700);
      assert.equal(fs.statSync(scratch).mode & 0o777, 0o700);
    }
  });
});

void test("entries of exited runs on this host are reaped; live and foreign ones stay", () => {
  withBase((base) => {
    const root = path.join(base, "root");
    fs.mkdirSync(root, { mode: 0o700 });
    const dead = path.join(root, `oxlint-vize-${host}-${exitedPid()}-abcdef`);
    const live = path.join(root, `oxlint-vize-${host}-${process.ppid}-abcdef`);
    const foreign = path.join(root, `oxlint-vize-00000000-${exitedPid()}-abcdef`);
    const unrelated = path.join(root, "keep");
    for (const directory of [dead, live, foreign, unrelated])
      fs.mkdirSync(path.join(directory, "inside"), { recursive: true });
    const scratch = createOwnedScratch(root, "oxlint-vize-");
    assert.deepEqual(
      fs.readdirSync(root).sort(),
      [path.basename(foreign), path.basename(live), path.basename(scratch), "keep"].sort(),
    );
  });
});

void test("a killed run's scratch is removed by the next run", () => {
  withBase((base) => {
    const root = path.join(base, "root");
    const script = [
      `const { createOwnedScratch } = await import(${JSON.stringify(moduleUrl)});`,
      `const scratch = createOwnedScratch(${JSON.stringify(root)}, "");`,
      `(await import("node:fs")).writeFileSync(scratch + "/copy.vue", "<template />");`,
      `process.stdout.write(scratch);`,
      `process.kill(process.pid, "SIGKILL");`,
    ].join("\n");
    const child = spawnSync(process.execPath, ["--input-type=module", "-e", script], {
      encoding: "utf8",
    });
    assert.equal(child.signal, process.platform === "win32" ? null : "SIGKILL");
    const abandoned = child.stdout;
    assert.deepEqual(fs.readdirSync(abandoned), ["copy.vue"]);
    const scratch = createOwnedScratch(root, "");
    assert.deepEqual(fs.readdirSync(root), [path.basename(scratch)]);
  });
});

void test("a root that is a link is not adopted and nothing below it is reaped", () => {
  if (process.platform === "win32") return;
  withBase((base) => {
    const target = path.join(base, "elsewhere");
    const dead = path.join(target, `${host}-${exitedPid()}-abcdef`);
    fs.mkdirSync(dead, { recursive: true });
    const root = path.join(base, "root");
    fs.symlinkSync(target, root);
    const scratch = createOwnedScratch(root, "");
    assert.equal(path.dirname(scratch), base);
    assert.deepEqual(fs.readdirSync(target), [path.basename(dead)]);
  });
});
