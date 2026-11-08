import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { gzipSync } from "node:zlib";
import { parse } from "yaml";

const actionPath = path.resolve(".github/actions/setup-wild-http-recovery");
const original = fs.readFileSync("tests/tooling/fixtures/setup-wild-original-0bbbfa5d.yml", "utf8");
const action = fs.readFileSync(path.join(actionPath, "action.yml"), "utf8");
const helper = fs.readFileSync(path.join(actionPath, "download-linux-x64.sh"), "utf8");
const pin = "deb6ee0e5caec798053ec4aafaba042e20a8edf91f08cb4d36268571cc628d3b";
const primary =
  "https://github.com/wild-linker/wild/releases/download/0.9.0/wild-linker-0.9.0-x86_64-unknown-linux-gnu.tar.gz";
const alternative = "https://api.github.com/repos/wild-linker/wild/releases/assets/428152085";
const upstream = "wild-linker/action@0bbbfa5df4380cab8e63cb8505a1ce65e1d10203";
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");

// Harmless real USTAR/gzip bytes. Extracted files are never executed.
function archive(body = "inert Wild transport fixture\n", mode = "0000644\0") {
  const bytes = Buffer.from(body),
    header = Buffer.alloc(512);
  header.write("wild-0.9.0/wild");
  header.write(mode, 100);
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
  return gzipSync(
    Buffer.concat([
      header,
      bytes,
      Buffer.alloc((512 - (bytes.length % 512)) % 512),
      Buffer.alloc(1024),
    ]),
  );
}
function executable(file, body) {
  fs.writeFileSync(file, `#!${process.execPath}\n${body}\n`, { mode: 0o755 });
}
function snapshot(root) {
  const rows = [];
  function visit(directory) {
    for (const name of fs.readdirSync(directory).sort()) {
      const file = path.join(directory, name),
        stat = fs.lstatSync(file);
      const relative = path.relative(root, file);
      rows.push({
        path: relative,
        mode: stat.mode & 0o777,
        directory: stat.isDirectory(),
        body: stat.isFile() ? fs.readFileSync(file).toString("base64") : null,
      });
      if (stat.isDirectory()) visit(file);
    }
  }
  visit(root);
  return rows;
}
function fixture(callback) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-wild-recipe-"));
  try {
    const bin = path.join(root, "bin"),
      work = path.join(root, "work"),
      local = path.join(root, "action");
    fs.mkdirSync(bin);
    fs.mkdirSync(local);
    const realTar = spawnSync("sh", ["-c", "command -v tar"], { encoding: "utf8" });
    assert.equal(realTar.status, 0);
    const gnuSha = "/opt/homebrew/opt/coreutils/libexec/gnubin/sha256sum";
    if (process.platform === "darwin") fs.symlinkSync(gnuSha, path.join(bin, "sha256sum"));
    const shaVersion = spawnSync("sha256sum", ["--version"], {
      encoding: "utf8",
      env: { ...process.env, PATH: `${bin}${path.delimiter}${process.env.PATH}` },
    });
    assert.equal(shaVersion.status, 0);
    assert.match(shaVersion.stdout, /GNU coreutils/);
    const bytes = archive();
    fs.writeFileSync(path.join(root, "payload"), bytes);
    assert.equal(helper.split(pin).length, 2);
    const copied = helper.replace(pin, digest(bytes));
    assert.equal(copied.replace(digest(bytes), pin), helper);
    fs.writeFileSync(path.join(local, "download-linux-x64.sh"), copied);
    executable(
      path.join(bin, "curl"),
      `
import fs from 'node:fs';
const args=process.argv.slice(2), api=args.at(-1)===${JSON.stringify(alternative)};
fs.appendFileSync(process.env.CALLS, JSON.stringify(args)+'\\n');
const output=args[args.indexOf('--output')+1];
fs.writeFileSync(output, process.env[api?'API_BODY':'PRIMARY_BODY']==='valid'
  ? fs.readFileSync(process.env.PAYLOAD) : Buffer.from('wrong full archive body\\n'));
process.stderr.write(api?'api fixture stderr\\n':'primary fixture stderr\\n');
if(!api && args.includes('--write-out')) process.stdout.write(process.env.HTTP);
process.exit(Number(process.env[api?'API_EXIT':'PRIMARY_EXIT']));`,
    );
    executable(
      path.join(bin, "tar"),
      `
import fs from 'node:fs'; import {spawnSync} from 'node:child_process';
fs.appendFileSync(process.env.TAR_CALLS, JSON.stringify(process.argv.slice(2))+'\\n');
const p=spawnSync(${JSON.stringify(realTar.stdout.trim())},process.argv.slice(2),{stdio:'inherit'});
process.exit(p.status??1);`,
    );
    function run(options = {}) {
      fs.rmSync(work, { recursive: true, force: true });
      fs.mkdirSync(work);
      const home = path.join(work, "home"),
        temporary = path.join(work, "temporary");
      fs.mkdirSync(home);
      fs.mkdirSync(temporary);
      if (options.config !== undefined) {
        fs.mkdirSync(path.join(home, ".cargo"));
        fs.writeFileSync(path.join(home, ".cargo/config.toml"), options.config);
      }
      if (options.existing) fs.mkdirSync(path.join(temporary, "wild-install"));
      const calls = path.join(root, "calls"),
        tarCalls = path.join(root, "tar-calls");
      fs.writeFileSync(calls, "");
      fs.writeFileSync(tarCalls, "");
      const parsed = parse(options.original ? original : action);
      let recipe = parsed.runs.steps.find((s) => s.run).run;
      recipe = recipe
        .replaceAll("${{ inputs.wild-version }}", "0.9.0")
        .replaceAll("${{ inputs.download-retries }}", "5");
      const env = {
        ...process.env,
        PATH: `${bin}${path.delimiter}${process.env.PATH}`,
        HOME: home,
        RUNNER_TEMP: temporary,
        RUNNER_OS: "Linux",
        RUNNER_ARCH: "X64",
        GITHUB_ACTION_PATH: local,
        CALLS: calls,
        TAR_CALLS: tarCalls,
        PAYLOAD: path.join(root, "payload"),
        HTTP: "200",
        PRIMARY_EXIT: "0",
        PRIMARY_BODY: "valid",
        API_EXIT: "0",
        API_BODY: "valid",
        ...options.env,
      };
      const result = spawnSync("bash", ["--noprofile", "--norc", "-eo", "pipefail", "-c", recipe], {
        env,
        cwd: root,
        encoding: "utf8",
      });
      return {
        status: result.status,
        stdout: result.stdout,
        stderr: result.stderr,
        files: snapshot(work),
        calls: fs
          .readFileSync(calls, "utf8")
          .trim()
          .split("\n")
          .filter(Boolean)
          .map((s) => JSON.parse(s)),
        tarCalls: fs
          .readFileSync(tarCalls, "utf8")
          .trim()
          .split("\n")
          .filter(Boolean)
          .map((s) => JSON.parse(s)),
      };
    }
    callback({ run, root, local, bytes });
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}
function primaryArguments(output, writeOut = true) {
  return [
    "--fail-with-body",
    "--show-error",
    "--location",
    "--retry",
    "5",
    "--retry-all-errors",
    "--output",
    output,
    ...(writeOut ? ["--write-out", "%{http_code}"] : []),
    primary,
  ];
}
function assertBeforeExtraction(result, status, calls) {
  assert.equal(result.status, status);
  assert.equal(result.calls.length, calls);
  assert.deepEqual(result.tarCalls, []);
  assert.equal(
    result.files.some((r) => r.path.startsWith("home/.cargo")),
    false,
  );
  assert.equal(
    result.files.some((r) => r.path === "temporary/wild-install/wild"),
    false,
  );
}

export {
  archive,
  action,
  original,
  primary,
  alternative,
  upstream,
  digest,
  fixture,
  primaryArguments,
  assertBeforeExtraction,
};
