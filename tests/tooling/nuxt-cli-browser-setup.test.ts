import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

const workflow = parse(readRepoFile(".github", "workflows", "nuxt3-module-build.yml")) as {
  jobs: Record<string, { steps: Array<{ name?: string; run?: string }> }>;
};
const setup = workflow.jobs["nuxt3-module-build"].steps.find(
  (step) => step.name === "Download Chromium dependencies while building the current CLI",
)?.run;
assert.ok(setup);

function probe(downloadStatus: number, cliStatus: number, installStatus = 0, cancel = false) {
  const dir = mkdtempSync(path.join(tmpdir(), "vize-nuxt-cli-browser-"));
  try {
    for (const name of ["bin", "tests", "config", "cache/archives"]) {
      mkdirSync(path.join(dir, name), { recursive: true });
    }
    const original = path.join(dir, "config", "99-vize-nuxt-archives-original");
    const target = path.join(dir, "unowned-target");
    writeFileSync(original, "original configuration\n");
    writeFileSync(target, "unowned symlink target\n");
    symlinkSync(target, path.join(dir, "config", "98-vize-nuxt-download-only-original"));
    writeFileSync(path.join(dir, "cache", "archives", "default.deb"), "default archive\n");
    const script = path.join(dir, "setup.sh");
    writeFileSync(script, `printf '%s' "$$" > "$PROBE_DIR/parent-pid"\n${setup}`);
    assert.equal(spawnSync("mkfifo", [path.join(dir, "ready"), path.join(dir, "gate")]).status, 0);

    const shim = (name: string, source: string) => {
      writeFileSync(path.join(dir, "bin", name), `#!${process.execPath}\n${source}\n`, {
        mode: 0o755,
      });
    };
    shim(
      "sudo",
      String.raw`
const { spawnSync } = require("node:child_process");
const path = require("node:path");
const [command, ...args] = process.argv.slice(2);
const translated = args.map((arg) => arg.replace(/^\/etc\/apt\/apt\.conf\.d\//, process.env.PROBE_DIR + "/config/").replace(/^\/var\/cache\/apt\//, process.env.PROBE_DIR + "/cache/"));
const result = spawnSync(command, translated, { stdio: "inherit" });
process.exit(result.status ?? 1);
`,
    );
    shim(
      "vize-ci-apt-retry",
      String.raw`
const { spawnSync } = require("node:child_process");
const result = spawnSync(process.argv[2], process.argv.slice(3), { stdio: "inherit" });
process.exit(result.status ?? 1);
`,
    );
    shim(
      "cargo",
      String.raw`
const fs = require("node:fs");
const { spawn } = require("node:child_process");
const dir = process.env.PROBE_DIR;
const record = (value) => fs.appendFileSync(dir + "/events", value + "\n");
if (JSON.stringify(process.argv.slice(2)) !== JSON.stringify(["build", "--profile", "ci", "-p", "vize"])) process.exit(99);
const child = spawn(process.execPath, ["-e", "setInterval(() => {}, 1000)"], { stdio: "inherit" });
fs.writeFileSync(dir + "/child-pid", String(child.pid));
let stopping = false;
const stop = (status) => {
  if (stopping) return;
  stopping = true;
  child.once("close", () => { record("child-closed"); record("cli-done"); process.exit(status); });
  child.kill("SIGTERM");
};
process.on("SIGTERM", () => stop(143));
record("cli-start");
fs.writeFileSync(dir + "/ready", "ready\n");
fs.createReadStream(dir + "/gate").once("data", () => stop(Number(process.env.CLI_STATUS)));
`,
    );
    shim(
      "vp",
      String.raw`
const assert = require("node:assert/strict");
const fs = require("node:fs");
const { spawnSync } = require("node:child_process");
const dir = process.env.PROBE_DIR;
const names = fs.readdirSync(dir + "/config");
const ownCache = names.filter((name) => name.startsWith("99-vize-nuxt-archives-") && !name.endsWith("original"));
assert.equal(ownCache.length, 1);
const config = fs.readFileSync(dir + "/config/" + ownCache[0], "utf8");
const archive = /Dir::Cache::archives "([^"]+)";/.exec(config)[1];
const hook = /APT::Update::Post-Invoke \{ "([^"]+)"; \};/.exec(config)[1];
const record = (value) => fs.appendFileSync(dir + "/events", value + "\n");
const update = () => {
  // The existing Docker-clean fixed-path hook remains active before our observer.
  fs.rmSync(dir + "/cache/archives/default.deb", { force: true });
  assert.equal(spawnSync("sh", ["-c", hook], { stdio: "inherit" }).status, 0);
};
const args = process.argv.slice(2);
const ownDownload = names.filter((name) => name.startsWith("98-vize-nuxt-download-only-") && !name.endsWith("original"));
if (JSON.stringify(args) === JSON.stringify(["exec", "playwright", "install-deps", "chromium"])) {
  assert.equal(ownDownload.length, 1);
  assert.equal(fs.readFileSync(dir + "/config/" + ownDownload[0], "utf8"), 'APT::Get::Download-Only "true";\n');
  assert.equal(fs.readFileSync(dir + "/ready", "utf8"), "ready\n");
  record("download-start");
  update();
  fs.writeFileSync(archive + "/package.deb", "verified package archive\n");
  record("download-done");
  if (process.env.CANCEL === "1") {
    process.kill(Number(fs.readFileSync(dir + "/parent-pid", "utf8")), "SIGTERM");
    process.exit(0);
  }
  fs.writeFileSync(dir + "/gate", "done\n");
  process.exit(Number(process.env.DOWNLOAD_STATUS));
}
assert.deepEqual(args, ["exec", "playwright", "install", "--with-deps", "chromium"]);
assert.equal(ownDownload.length, 0);
assert.ok(fs.readFileSync(dir + "/events", "utf8").includes("cli-done\n"));
record("install-start");
update();
assert.equal(fs.readFileSync(archive + "/package.deb", "utf8"), "verified package archive\n");
record("archive-reused");
process.exit(Number(process.env.INSTALL_STATUS));
`,
    );
    const result = spawnSync("bash", ["-e", "-o", "pipefail", script], {
      cwd: dir,
      encoding: "utf8",
      timeout: 8000,
      env: {
        ...process.env,
        PATH: `${path.join(dir, "bin")}:${process.env.PATH}`,
        PROBE_DIR: dir,
        DOWNLOAD_STATUS: String(downloadStatus),
        CLI_STATUS: String(cliStatus),
        INSTALL_STATUS: String(installStatus),
        CANCEL: cancel ? "1" : "0",
      },
    });
    assert.ifError(result.error);
    assert.equal(result.signal, null, result.stderr);
    assert.equal(readFileSync(original, "utf8"), "original configuration\n");
    assert.equal(readFileSync(target, "utf8"), "unowned symlink target\n");
    assert.deepEqual(
      spawnSync("find", [path.join(dir, "cache"), "-maxdepth", "1", "-type", "d"], {
        encoding: "utf8",
      })
        .stdout.trim()
        .split("\n")
        .sort(),
      [path.join(dir, "cache"), path.join(dir, "cache", "archives")].sort(),
    );
    assert.equal(
      spawnSync("find", [path.join(dir, "config"), "-type", "f"], {
        encoding: "utf8",
      }).stdout.trim(),
      original,
    );
    const events = readFileSync(path.join(dir, "events"), "utf8").trim().split("\n");
    assert.ok(events.includes("child-closed"));
    assert.ok(events.includes("cli-done"));
    return { status: result.status, events, output: result.stdout };
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

const linux = { skip: process.platform !== "linux" };

test(
  "Nuxt setup joins both commands, retains private archives through update hooks, and preserves unrelated configuration",
  linux,
  () => {
    const result = probe(0, 0);
    assert.equal(result.status, 0);
    assert.ok(result.events.indexOf("download-start") < result.events.indexOf("cli-done"));
    assert.ok(result.events.indexOf("install-start") > result.events.indexOf("cli-done"));
    assert.ok(result.events.includes("archive-reused"));
    assert.equal(
      result.output.match(/Nuxt3 archive inventory after package-index update/g)?.length,
      2,
    );
  },
);

test(
  "Nuxt setup preserves either command failure, reaps the CLI, and refuses installation",
  linux,
  () => {
    for (const [download, cli, expected] of [
      [23, 0, 23],
      [0, 11, 11],
      [23, 11, 23],
    ]) {
      const result = probe(download, cli);
      assert.equal(result.status, expected);
      assert.ok(!result.events.includes("install-start"));
    }
  },
);

test("Nuxt setup preserves the original install failure after both commands finish", linux, () => {
  const result = probe(0, 0, 29);
  assert.equal(result.status, 29);
  assert.ok(result.events.includes("archive-reused"));
});

test(
  "Nuxt setup interruption terminates and reaps the owned CLI process group and cleans only owned paths",
  linux,
  () => {
    const result = probe(0, 0, 0, true);
    assert.equal(result.status, 143);
    assert.ok(!result.events.includes("install-start"));
  },
);
