import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { gzipSync } from "node:zlib";
import { parse } from "yaml";

const actionPath = path.resolve(".github/actions/setup-rust-script");
const primary =
  "https://github.com/fornwall/rust-script/releases/download/0.34.0/rust-script-x86_64-unknown-linux-gnu.tar.gz";
const alternative = "https://api.github.com/repos/fornwall/rust-script/releases/assets/127872817";
const pin = "0700ecfe6c768c007f3e5a7919f4fedbf0a6727d275a196791f134a1742d4d13";
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");

// Real USTAR/gzip bytes; system tar and SHA256 validate the actual extractor.
function archive(entries) {
  const blocks = [];
  for (const [name, body] of entries) {
    const bytes = Buffer.from(body),
      header = Buffer.alloc(512);
    header.write(name);
    header.write("0000755\0", 100);
    header.write("0000000\0", 108);
    header.write("0000000\0", 116);
    header.write(bytes.length.toString(8).padStart(11, "0") + "\0", 124);
    header.write("00000000000\0", 136);
    header.fill(32, 148, 156);
    header.write("0", 156);
    header.write("ustar\0", 257);
    header.write("00", 263);
    const sum = header.reduce((a, b) => a + b, 0);
    header.write(sum.toString(8).padStart(6, "0") + "\0 ", 148);
    blocks.push(header, bytes, Buffer.alloc((512 - (bytes.length % 512)) % 512));
  }
  return gzipSync(Buffer.concat([...blocks, Buffer.alloc(1024)]));
}
function executable(file, lines) {
  fs.writeFileSync(file, lines.join("\n") + "\n", { mode: 0o755 });
}
function installScript(original) {
  let body = fs.readFileSync(path.join(actionPath, "action.yml"), "utf8");
  if (original) {
    const replacement = '          . "$GITHUB_ACTION_PATH/download-prebuilt.sh" "$archive"';
    assert.equal(body.split(replacement).length, 2);
    body = body.replace(
      replacement,
      [
        "          curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 \\",
        '            --output "$archive" \\',
        '            "' + primary + '"',
      ].join("\n"),
    );
    assert.equal(digest(body), "57c321c3ea4604a871184480fe1fd438cf297cd72f787a7563cd3a1a20772d33");
  }
  return parse(body).runs.steps.find((s) => s.name === "Install rust-script").run;
}
function run(options = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize rust-script transport "));
  const bin = path.join(root, "bin"),
    temporary = path.join(root, "temporary");
  const cargoHome = path.join(root, "cargo");
  fs.mkdirSync(bin);
  fs.mkdirSync(temporary);
  const runner =
    [
      "#!/bin/bash",
      'printf "%s\\n" "$*" >> "$RUNNER_LOG"',
      'printf "rust-script ' + (options.version ?? "0.34.0") + '\\n"',
    ].join("\n") + "\n";
  const entries = [["rust-script", runner]];
  if (options.extraMember) entries.push(["extra", "unowned"]);
  const bytes = archive(entries),
    bad = Buffer.from(bytes);
  bad[bad.length - 1] ^= 1;
  fs.writeFileSync(path.join(root, "fixture.tar.gz"), bytes);
  fs.writeFileSync(path.join(root, "changed.tar.gz"), bad);
  fs.writeFileSync(path.join(root, "partial.tar.gz"), bytes.subarray(0, 7));
  fs.writeFileSync(path.join(root, "json.tar.gz"), "{}");
  executable(path.join(bin, "curl"), [
    "#!/bin/bash",
    "set -euo pipefail",
    'count=0; [ ! -f "$CURL_COUNT" ] || count="$(cat "$CURL_COUNT")"',
    'count=$((count + 1)); printf "%s" "$count" > "$CURL_COUNT"',
    'printf "%s\\n" "$@" > "$CURL_LOG.$count"',
    'url=""; output=""; writeout=false',
    'while [ "$#" -gt 0 ]; do',
    '  case "$1" in',
    '    --output) output="$2"; shift 2 ;;',
    '    --write-out) [ "$2" = "%{http_code}" ]; writeout=true; shift 2 ;;',
    "    --header|--proto) shift 2 ;;",
    "    --*) shift ;;",
    '    *) url="$1"; shift ;;',
    "  esac",
    "done",
    'if [ "$url" = "$PRIMARY_URL" ]; then',
    '  status="$PRIMARY_STATUS"; http="$PRIMARY_HTTP"; body="$FIXTURE_ARCHIVE"',
    'elif [ "$url" = "$API_URL" ]; then',
    '  status="$API_STATUS"; http="$API_HTTP"; body="$API_BODY"',
    "else exit 65; fi",
    'cp "$body" "$output"',
    '$writeout && printf "%s" "$http"',
    'if [ "$status" -ne 0 ]; then',
    '  printf "curl fixture: original failure %s HTTP %s\\n" "$status" "$http" >&2',
    "fi",
    'exit "$status"',
  ]);
  // The production pin is asserted before a benign Mac-only extractor fixture
  // substitutes its own full digest. No downloaded Linux runner is executed.
  executable(path.join(bin, "bash"), [
    "#!/bin/bash",
    "set -euo pipefail",
    'printf "%s\\n" "$@" > "$HELPER_LOG"',
    '[ "$1" = "$EXTRACTOR" ] && [ "$4" = "$PRODUCTION_PIN" ]',
    'if [ "$FIXTURE_DIGEST" = true ]; then',
    '  exec /bin/bash "$1" "$2" "$3" "$FIXTURE_SHA"',
    "fi",
    'exec /bin/bash "$@"',
  ]);
  executable(path.join(bin, "cargo"), [
    "#!/bin/bash",
    'printf "%s\\n" "$*" >> "$CARGO_LOG"',
    "exit 73",
  ]);
  executable(path.join(bin, "tar"), [
    "#!/bin/bash",
    'printf "%s\\n" "$*" >> "$TAR_LOG"',
    'exec /usr/bin/tar "$@"',
  ]);
  const script = installScript(options.original);
  const result = spawnSync("/bin/bash", ["-c", script], {
    encoding: "utf8",
    timeout: 10_000,
    env: {
      ...process.env,
      PATH: bin + ":/usr/bin:/bin:/sbin",
      RUNNER_OS: "Linux",
      RUNNER_ARCH: "X64",
      RUNNER_TEMP: temporary,
      CARGO_HOME: cargoHome,
      GITHUB_ACTION_PATH: actionPath,
      GITHUB_PATH: path.join(root, "github-path"),
      PRIMARY_URL: primary,
      API_URL: alternative,
      PRIMARY_STATUS: String(options.primaryStatus ?? 22),
      PRIMARY_HTTP: options.primaryHttp ?? "503",
      API_STATUS: String(options.apiStatus ?? 0),
      API_HTTP: options.apiHttp ?? "200",
      API_BODY: path.join(
        root,
        options.changed
          ? "changed.tar.gz"
          : options.partial
            ? "partial.tar.gz"
            : options.jsonBody
              ? "json.tar.gz"
              : "fixture.tar.gz",
      ),
      FIXTURE_ARCHIVE: path.join(root, "fixture.tar.gz"),
      FIXTURE_SHA: digest(bytes),
      FIXTURE_DIGEST: String(options.fixtureDigest ?? true),
      EXTRACTOR: path.join(actionPath, "extract-prebuilt.sh"),
      PRODUCTION_PIN: pin,
      CURL_COUNT: path.join(root, "curl.count"),
      CURL_LOG: path.join(root, "curl.log"),
      HELPER_LOG: path.join(root, "helper.log"),
      RUNNER_LOG: path.join(root, "runner.log"),
      CARGO_LOG: path.join(root, "cargo.log"),
      TAR_LOG: path.join(root, "tar.log"),
    },
  });
  const read = (name) =>
    fs.existsSync(path.join(root, name)) ? fs.readFileSync(path.join(root, name), "utf8") : "";
  const calls = Array.from({ length: Number(read("curl.count")) }, (_, i) =>
    read("curl.log." + (i + 1))
      .trimEnd()
      .split("\n"),
  );
  return { root, temporary, cargoHome, runner, result, read, calls };
}
function retained(run, count) {
  assert.equal(run.result.error, undefined);
  assert.equal(run.result.signal, null);
  assert.equal(run.calls.length, count);
  assert.equal(run.read("cargo.log"), "");
  assert.deepEqual(fs.readdirSync(run.temporary), []);
  assert.equal(run.calls[0].at(-1), primary);
  assert.deepEqual(run.calls[0].slice(0, 7), [
    "--fail",
    "--silent",
    "--show-error",
    "--location",
    "--proto",
    "=https",
    "--tlsv1.2",
  ]);
  assert.deepEqual(run.calls[0].slice(-3), ["--write-out", "%{http_code}", primary]);
  if (count === 2) {
    const output = run.calls[0][run.calls[0].indexOf("--output") + 1];
    assert.deepEqual(run.calls[1], [
      "--fail",
      "--silent",
      "--show-error",
      "--location",
      "--proto",
      "=https",
      "--tlsv1.2",
      "--header",
      "Accept: application/octet-stream",
      "--header",
      "X-GitHub-Api-Version: 2022-11-28",
      "--output",
      output,
      alternative,
    ]);
  }
}
function accepted(run, count) {
  retained(run, count);
  assert.equal(run.result.status, 0, run.result.stderr);
  assert.equal(fs.readFileSync(path.join(run.cargoHome, "bin/rust-script"), "utf8"), run.runner);
  assert.equal(fs.statSync(path.join(run.cargoHome, "bin/rust-script")).mode & 0o777, 0o755);
  assert.equal(run.read("github-path"), run.cargoHome + "/bin\n");
  assert.equal(run.read("helper.log").trimEnd().split("\n").at(-1), pin);
}
function withRun(options, verify) {
  const observed = run(options);
  try {
    verify(observed);
  } finally {
    fs.rmSync(observed.root, { recursive: true, force: true });
  }
}
void test("the entire original action refuses all four 5xx before extraction", () => {
  for (const http of ["500", "502", "503", "504"]) {
    withRun({ primaryHttp: http, original: true }, (observed) => {
      assert.equal(observed.result.status, 22);
      assert.equal(observed.result.error, undefined);
      assert.equal(observed.result.signal, null);
      assert.equal(observed.calls.length, 1);
      assert.equal(observed.calls[0].at(-1), primary);
      for (const name of ["helper.log", "runner.log", "cargo.log", "github-path"])
        assert.equal(observed.read(name), "");
      assert.deepEqual(fs.readdirSync(observed.temporary), []);
    });
  }
});
for (const http of ["500", "502", "503", "504"]) {
  void test("only curl22 HTTP" + http + " uses one same-byte API route", () => {
    withRun({ primaryHttp: http }, (observed) => {
      accepted(observed, 2);
      assert.match(observed.result.stderr, /curl fixture: original failure 22 HTTP/);
      assert(observed.result.stderr.includes("HTTP=" + http + " URL=" + primary));
    });
  });
}
void test("primary HTTP200 never requests the alternative", () => {
  withRun({ primaryStatus: 0, primaryHttp: "200" }, (observed) => {
    accepted(observed, 1);
    assert.equal(observed.result.stderr, "");
  });
});
void test("HTTP authentication, missing asset and rate limiting keep original failure", () => {
  for (const http of ["401", "403", "404", "429"]) {
    withRun({ primaryHttp: http }, (observed) => {
      retained(observed, 1);
      assert.equal(observed.result.status, 22);
      assert.equal(observed.read("helper.log"), "");
      assert.equal(observed.read("runner.log"), "");
      assert(observed.result.stderr.includes("original failure 22 HTTP " + http));
    });
  }
});
void test("network, timeout and TLS failures never recover even with a 5xx status", () => {
  for (const status of [7, 28, 35, 60, 77]) {
    withRun({ primaryStatus: status }, (observed) => {
      retained(observed, 1);
      assert.equal(observed.result.status, status);
      assert.equal(observed.read("helper.log"), "");
      assert.equal(observed.read("runner.log"), "");
    });
  }
});
void test("missing and malformed HTTP status cannot enter recovery", () => {
  for (const http of ["", "000", "503\n", "HTTP503", "503503"]) {
    withRun({ primaryHttp: http }, (observed) => {
      retained(observed, 1);
      assert.equal(observed.result.status, 22);
      assert.equal(observed.read("helper.log"), "");
      assert.equal(observed.read("runner.log"), "");
    });
  }
});
void test("API failures never produce a third download or reach extraction", () => {
  for (const [apiStatus, apiHttp] of [
    [22, "500"],
    [22, "403"],
    [28, "000"],
  ]) {
    withRun({ apiStatus, apiHttp }, (observed) => {
      retained(observed, 2);
      assert.equal(observed.result.status, apiStatus);
      assert.equal(observed.read("helper.log"), "");
      assert.equal(observed.read("runner.log"), "");
    });
  }
});
void test("changed or partial API bodies refuse full SHA before tar or execution", () => {
  for (const options of [
    { changed: true },
    { partial: true },
    { jsonBody: true },
    { fixtureDigest: false },
  ]) {
    withRun(options, (observed) => {
      retained(observed, 2);
      assert.notEqual(observed.result.status, 0);
      assert.match(observed.result.stderr, /archive digest mismatch/);
      assert.equal(observed.read("tar.log"), "");
      assert.equal(observed.read("runner.log"), "");
      assert.equal(observed.read("github-path"), "");
      assert.equal(observed.read("helper.log").trimEnd().split("\n").at(-1), pin);
    });
  }
});
void test("an authenticated whole body with an extra member keeps original refusal", () => {
  withRun({ extraMember: true }, (observed) => {
    retained(observed, 2);
    assert.notEqual(observed.result.status, 0);
    assert.match(observed.result.stderr, /expected only the rust-script archive member/);
    assert.equal(observed.read("runner.log"), "");
    assert.equal(observed.read("github-path"), "");
  });
});
void test("API success still requires the original installed version", () => {
  withRun({ version: "0.33.0" }, (observed) => {
    retained(observed, 2);
    assert.notEqual(observed.result.status, 0);
    assert.match(observed.result.stderr, /expected rust-script 0\.34\.0/);
    assert.equal(observed.read("helper.log").trimEnd().split("\n").at(-1), pin);
  });
});
