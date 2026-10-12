import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { gzipSync } from "node:zlib";

import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

const pinnedDigest = "0700ecfe6c768c007f3e5a7919f4fedbf0a6727d275a196791f134a1742d4d13";
const actionPath = path.resolve(".github/actions/setup-rust-script");
const helper = path.join(actionPath, "extract-prebuilt.sh");
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

type Entry = { name: string; type?: string; body?: string; link?: string };

// Author real USTAR inputs; the production extractor uses the system tar.
function archive(entries: Entry[]): Buffer {
  const blocks: Buffer[] = [];
  for (const entry of entries) {
    const body = Buffer.from(entry.body ?? "");
    const header = Buffer.alloc(512);
    header.write(entry.name);
    header.write("0000755\0", 100);
    header.write("0000000\0", 108);
    header.write("0000000\0", 116);
    header.write(`${body.length.toString(8).padStart(11, "0")}\0`, 124);
    header.write("00000000000\0", 136);
    header.fill(32, 148, 156);
    header.write(entry.type ?? "0", 156);
    header.write(entry.link ?? "", 157);
    header.write("ustar\0", 257);
    header.write("00", 263);
    const checksum = header.reduce((sum, byte) => sum + byte, 0);
    header.write(`${checksum.toString(8).padStart(6, "0")}\0 `, 148);
    blocks.push(header, body, Buffer.alloc((512 - (body.length % 512)) % 512));
  }
  return gzipSync(Buffer.concat([...blocks, Buffer.alloc(1024)]));
}

function executable(file: string, body: string): void {
  fs.writeFileSync(file, body, { mode: 0o755 });
}

function fixtureRunner(version: string, status = 0): string {
  return `#!/bin/bash
printf '%s\\n' "$*" >> "$RUNNER_LOG"
printf 'rust-script ${version}\\n'
exit ${status}
`;
}

function extraction(bytes: Buffer, expected = digest(bytes)) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize rust-script archive "));
  const source = path.join(root, "runner.tar.gz");
  const destination = path.join(root, "extracted");
  const bin = path.join(root, "bin");
  fs.mkdirSync(bin);
  executable(
    path.join(bin, "tar"),
    `#!/bin/bash\nprintf '%s\\n' "$*" >> "$TAR_LOG"\nexec /usr/bin/tar "$@"\n`,
  );
  fs.writeFileSync(source, bytes);
  const result = spawnSync("/bin/bash", [helper, source, destination, expected], {
    encoding: "utf8",
    env: {
      ...process.env,
      PATH: `${bin}:/usr/bin:/bin:/sbin`,
      TAR_LOG: path.join(root, "tar.log"),
    },
  });
  return { root, destination, result };
}

type InstallerOptions = {
  os?: string;
  arch?: string;
  version?: string;
  versionStatus?: number;
  curlStatus?: number;
  fixtureDigest?: boolean;
  initialVersion?: string;
};

function installer(options: InstallerOptions = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize rust-script installer "));
  const bin = path.join(root, "bin");
  const cargoHome = path.join(root, "cargo");
  const temporary = path.join(root, "temporary");
  fs.mkdirSync(bin);
  fs.mkdirSync(temporary);
  const body = fixtureRunner(options.version ?? "0.34.0", options.versionStatus);
  const bytes = archive([{ name: "rust-script", body }]);
  fs.writeFileSync(path.join(root, "fixture.tar.gz"), bytes);
  if (options.initialVersion)
    executable(path.join(bin, "rust-script"), fixtureRunner(options.initialVersion));
  executable(
    path.join(bin, "curl"),
    `#!/bin/bash
set -euo pipefail
printf '%s\\n' "$@" > "$CURL_LOG"
if [ "$CURL_STATUS" != 0 ]; then exit "$CURL_STATUS"; fi
while [ "$#" -gt 0 ]; do
  if [ "$1" = --output ]; then cp "$FIXTURE_ARCHIVE" "$2"; exit 0; fi
  shift
done
exit 65
`,
  );
  // Only the positive Mac fixture substitutes its digest at the helper boundary.
  // Record/assert the actual producer pin; production refusal uses it unchanged.
  executable(
    path.join(bin, "bash"),
    `#!/bin/bash
set -euo pipefail
printf '%s\\n' "$@" > "$HELPER_LOG"
if [ "$FIXTURE_DIGEST" = true ]; then
  exec /bin/bash "$1" "$2" "$3" "$ARCHIVE_DIGEST"
fi
exec /bin/bash "$@"
`,
  );
  executable(
    path.join(bin, "cargo"),
    `#!/bin/bash
set -euo pipefail
printf '%s\\n' "$*" > "$CARGO_LOG"
mkdir -p "$CARGO_HOME/bin"
cp "$FIXTURE_RUNNER" "$CARGO_HOME/bin/rust-script"
chmod +x "$CARGO_HOME/bin/rust-script"
`,
  );
  executable(path.join(root, "fixture-runner"), fixtureRunner("0.34.0"));
  executable(
    path.join(bin, "tar"),
    `#!/bin/bash\nprintf '%s\\n' "$*" >> "$TAR_LOG"\nexec /usr/bin/tar "$@"\n`,
  );
  const action = parse(readRepoFile(".github", "actions", "setup-rust-script", "action.yml")) as {
    runs: { steps: Array<{ name?: string; run?: string }> };
  };
  const script = action.runs.steps.find((step) => step.name === "Install rust-script")?.run;
  assert.ok(script);
  const result = spawnSync("/bin/bash", ["-c", script], {
    encoding: "utf8",
    env: {
      ...process.env,
      PATH: `${bin}:/usr/bin:/bin:/sbin`,
      CARGO_HOME: cargoHome,
      RUNNER_TEMP: temporary,
      RUNNER_OS: options.os ?? "Linux",
      RUNNER_ARCH: options.arch ?? "X64",
      GITHUB_ACTION_PATH: actionPath,
      GITHUB_PATH: path.join(root, "github-path"),
      FIXTURE_ARCHIVE: path.join(root, "fixture.tar.gz"),
      FIXTURE_RUNNER: path.join(root, "fixture-runner"),
      FIXTURE_DIGEST: String(options.fixtureDigest ?? true),
      ARCHIVE_DIGEST: digest(bytes),
      CURL_STATUS: String(options.curlStatus ?? 0),
      CURL_LOG: path.join(root, "curl.log"),
      HELPER_LOG: path.join(root, "helper.log"),
      CARGO_LOG: path.join(root, "cargo.log"),
      RUNNER_LOG: path.join(root, "runner.log"),
      TAR_LOG: path.join(root, "tar.log"),
      RUST_SCRIPT_SHA256: digest(bytes),
    },
  });
  const read = (name: string) =>
    fs.existsSync(path.join(root, name)) ? fs.readFileSync(path.join(root, name), "utf8") : "";
  return { root, temporary, cargoHome, body, result, read };
}

void test("Linux X64 uses the pinned producer tuple and installs the exact benign fixture", () => {
  const run = installer({ initialVersion: "0.33.0" });
  try {
    assert.equal(run.result.status, 0, run.result.stderr);
    assert.equal(run.read("cargo.log"), "");
    const helperArgs = run.read("helper.log").trim().split("\n");
    assert.equal(helperArgs[0], helper);
    assert.equal(helperArgs.at(-1), pinnedDigest);
    assert.deepEqual(run.read("curl.log").trim().split("\n"), [
      "--fail",
      "--silent",
      "--show-error",
      "--location",
      "--proto",
      "=https",
      "--tlsv1.2",
      "--output",
      helperArgs[1],
      "--write-out",
      "%{http_code}",
      "https://github.com/fornwall/rust-script/releases/download/0.34.0/rust-script-x86_64-unknown-linux-gnu.tar.gz",
    ]);
    assert.equal(fs.readFileSync(path.join(run.cargoHome, "bin/rust-script"), "utf8"), run.body);
    assert.equal(fs.statSync(path.join(run.cargoHome, "bin/rust-script")).mode & 0o777, 0o755);
    assert.equal(run.read("github-path"), `${run.cargoHome}/bin\n`);
    assert.deepEqual(fs.readdirSync(run.temporary), []);
    assert.match(run.result.stdout, /^rust-script 0\.34\.0$/m);
  } finally {
    fs.rmSync(run.root, { recursive: true, force: true });
  }
});

void test("the production digest refuses changed bytes before install or execution", () => {
  const run = installer({ fixtureDigest: false });
  try {
    assert.notEqual(run.result.status, 0);
    assert.match(run.result.stderr, /archive digest mismatch/);
    assert.equal(run.read("runner.log"), "");
    assert.equal(run.read("tar.log"), "");
    assert.equal(run.read("cargo.log"), "");
    assert.equal(run.read("github-path"), "");
    assert.equal(fs.existsSync(path.join(run.cargoHome, "bin/rust-script")), false);
    assert.deepEqual(fs.readdirSync(run.temporary), []);
  } finally {
    fs.rmSync(run.root, { recursive: true, force: true });
  }
});

void test("a pinned existing Linux runner retains the original fast path", () => {
  const run = installer({ initialVersion: "0.34.0" });
  try {
    assert.equal(run.result.status, 0, run.result.stderr);
    for (const name of ["cargo.log", "curl.log", "helper.log", "github-path"])
      assert.equal(run.read(name), "");
  } finally {
    fs.rmSync(run.root, { recursive: true, force: true });
  }
});

void test("every other platform retains the original locked Cargo recipe", () => {
  for (const [osName, arch] of [
    ["Linux", "ARM64"],
    ["macOS", "X64"],
    ["Windows", "X64"],
  ]) {
    const run = installer({ os: osName, arch });
    try {
      assert.equal(run.result.status, 0, run.result.stderr);
      assert.equal(
        run.read("cargo.log"),
        "install rust-script --version 0.34.0 --locked --force\n",
      );
      assert.equal(run.read("curl.log"), "");
      assert.equal(run.read("helper.log"), "");
    } finally {
      fs.rmSync(run.root, { recursive: true, force: true });
    }
  }
});

void test("download and installed-version failures remain fatal with cleanup", () => {
  for (const options of [{ curlStatus: 77 }, { version: "0.33.0" }, { versionStatus: 69 }]) {
    const run = installer(options);
    try {
      assert.notEqual(run.result.status, 0);
      assert.equal(run.read("cargo.log"), "");
      assert.equal(run.read("github-path"), options.curlStatus ? "" : `${run.cargoHome}/bin\n`);
      assert.deepEqual(fs.readdirSync(run.temporary), []);
      if (options.version) assert.match(run.result.stderr, /expected rust-script 0\.34\.0/);
    } finally {
      fs.rmSync(run.root, { recursive: true, force: true });
    }
  }
});

void test("complete archive membership and file type reject aliases and traversal", () => {
  const cases: Entry[][] = [
    [{ name: "rust-script", type: "2", link: "/tmp/outside" }],
    [{ name: "rust-script", type: "1", link: "outside" }],
    [{ name: "rust-script", type: "5" }],
    [{ name: "../rust-script", body: "outside" }],
    [
      { name: "rust-script", body: "one" },
      { name: "extra", body: "two" },
    ],
    [
      { name: "rust-script", body: "one" },
      { name: "rust-script", body: "two" },
    ],
  ];
  for (const entries of cases) {
    const run = extraction(archive(entries));
    try {
      assert.notEqual(run.result.status, 0, JSON.stringify(entries));
      assert.equal(fs.existsSync(run.destination), false, JSON.stringify(entries));
    } finally {
      fs.rmSync(run.root, { recursive: true, force: true });
    }
  }
});

void test("the extractor preserves all regular-file bytes and refuses damaged input", () => {
  const bytes = archive([{ name: "rust-script", body: "exact\0whole\nbytes" }]);
  for (const [input, expected, success] of [
    [bytes, digest(bytes), true],
    [bytes, "0".repeat(64), false],
    [Buffer.from("broken gzip"), digest(Buffer.from("broken gzip")), false],
  ] as const) {
    const run = extraction(input, expected);
    try {
      if (success) {
        assert.equal(run.result.status, 0, run.result.stderr);
        assert.equal(
          fs.readFileSync(path.join(run.destination, "rust-script")).toString(),
          "exact\0whole\nbytes",
        );
      } else {
        assert.notEqual(run.result.status, 0);
        assert.equal(fs.existsSync(run.destination), false);
        if (expected === "0".repeat(64))
          assert.equal(fs.existsSync(path.join(run.root, "tar.log")), false);
      }
    } finally {
      fs.rmSync(run.root, { recursive: true, force: true });
    }
  }
});
