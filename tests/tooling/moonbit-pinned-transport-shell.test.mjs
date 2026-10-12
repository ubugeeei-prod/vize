import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  adaptInstaller,
  pinnedVersion,
  selectArchive,
} from "../../.github/actions/setup-moonbit/pinned-artifacts.mjs";

const action = fileURLToPath(new URL("../../.github/actions/setup-moonbit/", import.meta.url));
const fixtures = new URL("../_fixtures/tooling/moonbit-pinned-transport/", import.meta.url);
const official = fs.readFileSync(new URL("unix.sh.txt", fixtures), "utf8");
const original = fs.readFileSync(path.join(action, "install-moonbit.mjs"), "utf8");

function shell(file, body) {
  fs.writeFileSync(file, `#!/bin/sh\n${body}\n`);
  fs.chmodSync(file, 0o755);
}

// These execute the complete official shell installer and current Node caller
// with a tiny controlled compiler, never representing native qualification.
void test(
  "original full shell refuses exact HTTP403; fixed expressions install original core and bundle the same targets",
  { skip: process.platform === "win32" },
  () => {
    // The existing official script invokes $exe unquoted when bundling. Keep
    // its original runner-home contract; unusual transport file paths are below.
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "moon-shell-"));
    try {
      const compiler = path.join(directory, "compiler");
      const commands = path.join(directory, "commands");
      fs.mkdirSync(path.join(compiler, "bin", "internal"), { recursive: true });
      fs.mkdirSync(path.join(compiler, "include"));
      fs.mkdirSync(path.join(compiler, "lib"));
      fs.mkdirSync(commands);
      shell(path.join(compiler, "bin", "moon"), 'printf "%s\\n" "$*" >> "$CONTROL_LOG"');
      shell(path.join(compiler, "bin", "internal", "tcc"), "exit 0");
      const archive = path.join(directory, "compiler 日本 ' .tar.gz");
      const packed = spawnSync("tar", ["-czf", archive, "-C", compiler, "."], { encoding: "utf8" });
      assert.equal(packed.status, 0, packed.stderr);
      const core = path.join(action, "assets", "core-0.10.7+bc794d341.tar.gz");
      // Refuse only the original inaccessible HTTP host; actual curl still reads
      // the fixed file URIs, including encoded Unicode/space/apostrophe paths.
      const curl = spawnSync("which", ["curl"], { encoding: "utf8" }).stdout.trim();
      shell(
        path.join(commands, "curl"),
        `for arg do case "$arg" in https://cli.moonbitlang.com/binaries/*) printf 'curl: (22) HTTP 403\\n' >&2; exit 22;; esac; done\nexec '${curl}' "$@"`,
      );
      const script = path.join(directory, "installer.sh");
      const env = {
        ...process.env,
        HOME: directory,
        MOON_HOME: path.join(directory, "moon"),
        MOONBIT_INSTALL_VERSION: pinnedVersion,
        SHELL: "/bin/bash",
        CONTROL_LOG: path.join(directory, "bundle.log"),
        PATH: `${commands}${path.delimiter}${process.env.PATH}`,
      };
      fs.writeFileSync(script, official);
      const before = spawnSync("bash", [script], { env, encoding: "utf8" });
      assert.equal(before.status, 1);
      assert.match(before.stderr, /HTTP 403/);
      assert.match(before.stderr, /0\.10\.7%2Bbc794d341/);
      assert.equal(fs.existsSync(env.CONTROL_LOG), false);
      fs.writeFileSync(script, adaptInstaller(official, process.platform, archive, core));
      const after = spawnSync("bash", [script], { env, encoding: "utf8" });
      assert.equal(after.status, 0, `${after.stderr}\n${after.stdout}`);
      assert.deepEqual(fs.readFileSync(env.CONTROL_LOG, "utf8").trim().split("\n"), [
        `-C ${env.MOON_HOME}/lib/core bundle --warn-list -a --all`,
        `-C ${env.MOON_HOME}/lib/core bundle --warn-list -a --target wasm-gc --quiet`,
      ]);
      assert.equal(fs.existsSync(path.join(env.MOON_HOME, "lib", "core", "LICENSE")), true);
      assert.equal(fs.readlinkSync(path.join(env.MOON_HOME, "bin", "moonx")), "moon");
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);

void test(
  "full current caller stages fixed bytes before install, preserves version/update/smoke/export and cached reuse",
  { skip: process.platform === "win32" },
  () => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "moon-caller-"));
    try {
      const tree = path.join(directory, "tree");
      const location = path.join(tree, ".github", "actions", "setup-moonbit");
      const compiler = path.join(directory, "compiler");
      const commands = path.join(directory, "commands");
      const runner = path.join(directory, "runner");
      const log = path.join(directory, "calls.log");
      fs.mkdirSync(path.join(compiler, "bin", "internal"), { recursive: true });
      fs.mkdirSync(path.join(compiler, "include"));
      fs.mkdirSync(path.join(compiler, "lib"));
      fs.mkdirSync(path.join(location, "assets"), { recursive: true });
      fs.mkdirSync(commands);
      fs.mkdirSync(runner);
      shell(
        path.join(compiler, "bin", "moon"),
        'printf "%s\\n" "$*" >> "$CONTROL_LOG"\nif [ "$1" = run ]; then /bin/cat >/dev/null; fi',
      );
      shell(path.join(compiler, "bin", "moonc"), `printf 'v${pinnedVersion} (controlled)\\n'`);
      shell(path.join(compiler, "bin", "internal", "tcc"), "exit 0");
      fs.writeFileSync(
        path.join(compiler, "include", "moonbit.h"),
        "void *memcpy(void *dst, const void *src, size_t n);\n",
      );
      const archive = path.join(directory, "compiler.tar.gz");
      assert.equal(spawnSync("tar", ["-czf", archive, "-C", compiler, "."]).status, 0);
      const bytes = fs.readFileSync(archive);
      const selected = selectArchive(pinnedVersion, process.platform, process.arch);
      const helper = fs.readFileSync(path.join(action, "pinned-artifacts.mjs"), "utf8");
      const before = new RegExp(`${selected.bytes},\\s*"${selected.sha256}"`, "gu");
      assert.equal([...helper.matchAll(before)].length, 1);
      fs.writeFileSync(
        path.join(location, "pinned-artifacts.mjs"),
        helper.replace(
          before,
          `${bytes.length}, "${createHash("sha256").update(bytes).digest("hex")}"`,
        ),
      );
      fs.copyFileSync(
        path.join(action, "assets", "core-0.10.7+bc794d341.tar.gz"),
        path.join(location, "assets", "core-0.10.7+bc794d341.tar.gz"),
      );
      fs.writeFileSync(path.join(location, "install-moonbit.mjs"), original);
      fs.writeFileSync(path.join(tree, ".moonbit-version"), `${pinnedVersion}\n`);
      const preload = path.join(directory, "fetch.mjs");
      fs.writeFileSync(
        preload,
        `import fs from 'node:fs'; globalThis.fetch = async(url) => { if(url !== ${JSON.stringify(selected.url)}) throw new Error('Unexpected transport URL'); fs.appendFileSync(${JSON.stringify(log)}, 'authenticated-download\\n'); return new Response(fs.readFileSync(${JSON.stringify(archive)})); };`,
      );
      const curl = spawnSync("which", ["curl"], { encoding: "utf8" }).stdout.trim();
      const officialPath = fileURLToPath(new URL("unix.sh.txt", fixtures));
      shell(
        path.join(commands, "curl"),
        `if [ "$2" = 'https://cli.moonbitlang.com/install/unix.sh' ]; then /bin/cp '${officialPath}' "$4"; else exec '${curl}' "$@"; fi`,
      );
      const env = {
        ...process.env,
        RUNNER_TEMP: runner,
        GITHUB_ENV: path.join(directory, "github-env"),
        GITHUB_PATH: path.join(directory, "github-path"),
        CONTROL_LOG: log,
        PATH: `${commands}${path.delimiter}${process.env.PATH}`,
      };
      const caller = path.join(location, "install-moonbit.mjs");
      const cold = spawnSync(process.execPath, ["--import", preload, caller], {
        env,
        encoding: "utf8",
      });
      assert.equal(cold.status, 0, `${cold.stderr}\n${cold.stdout}`);
      assert.deepEqual(fs.readFileSync(log, "utf8").trim().split("\n"), [
        "authenticated-download",
        `-C ${runner}/moonbit/lib/core bundle --warn-list -a --all`,
        `-C ${runner}/moonbit/lib/core bundle --warn-list -a --target wasm-gc --quiet`,
        "update",
        "run -q --target native - --",
      ]);
      assert.equal(fs.readFileSync(env.GITHUB_PATH, "utf8"), `${runner}/moonbit-shims\n`);
      assert.equal(
        fs.readFileSync(env.GITHUB_ENV, "utf8"),
        `MOON_HOME=${runner}/moonbit\nMOON_BIN=${runner}/moonbit-shims/moon\n`,
      );
      fs.writeFileSync(log, "");
      const warm = spawnSync(process.execPath, ["--import", preload, caller], {
        env,
        encoding: "utf8",
      });
      assert.equal(warm.status, 0, `${warm.stderr}\n${warm.stdout}`);
      assert.equal(fs.readFileSync(log, "utf8"), "run -q --target native - --\n");
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);
