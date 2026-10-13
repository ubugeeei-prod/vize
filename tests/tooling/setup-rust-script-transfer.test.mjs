import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import https from "node:https";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

const root = path.resolve(import.meta.dirname, "../..");
const action = path.join(root, ".github/actions/setup-rust-script");
const primary =
  "https://github.com/fornwall/rust-script/releases/download/0.34.0/rust-script-x86_64-unknown-linux-gnu.tar.gz";
const api = "https://api.github.com/repos/fornwall/rust-script/releases/assets/127872817";
const pin = "0700ecfe6c768c007f3e5a7919f4fedbf0a6727d275a196791f134a1742d4d13";
const frozen = path.join(
  root,
  "tests/_fixtures/tooling/rust-script-transfer/before-download.sh.txt",
);
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
assert.equal(
  digest(fs.readFileSync(frozen)),
  "411e305f26470090d419a0eed7e2a916fafe481a03f8ada4b072ff2829f37912",
);

async function observed({ original = false, mode = "stall", alwaysFail = false } = {}) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize real TLS transfer "));
  const bin = path.join(directory, "bin");
  fs.mkdirSync(bin);
  const archive = path.join(directory, "rust-script.tar.gz");
  const sentinel = "old complete archive remains owned until replacement";
  fs.writeFileSync(archive, sentinel);
  const runner = path.join(directory, "rust-script");
  fs.writeFileSync(runner, "#!/bin/bash\nprintf 'rust-script 0.34.0\\n'\n", { mode: 0o755 });
  const fixture = path.join(directory, "fixture.tar.gz");
  const packed = spawnSync("/usr/bin/tar", [
    "--format=ustar",
    "-czf",
    fixture,
    "-C",
    directory,
    "rust-script",
  ]);
  assert.equal(packed.status, 0, packed.stderr.toString());
  const bytes = fs.readFileSync(fixture);
  const cert = path.join(directory, "cert.pem"),
    key = path.join(directory, "key.pem");
  const config = path.join(directory, "openssl.cnf");
  fs.writeFileSync(
    config,
    "[req]\nprompt=no\ndistinguished_name=dn\nx509_extensions=extensions\n[dn]\nCN=localhost\n[extensions]\nsubjectAltName=DNS:localhost,IP:127.0.0.1\n",
  );
  const certificate = spawnSync("/usr/bin/openssl", [
    "req",
    "-x509",
    "-newkey",
    "rsa:2048",
    "-nodes",
    "-days",
    "1",
    "-config",
    config,
    "-keyout",
    key,
    "-out",
    cert,
  ]);
  assert.equal(certificate.status, 0, certificate.stderr.toString());
  const requests = [],
    sockets = new Set();
  const server = https.createServer(
    { key: fs.readFileSync(key), cert: fs.readFileSync(cert) },
    (request, response) => {
      requests.push({ path: request.url, published: fs.readFileSync(archive, "utf8") });
      response.writeHead(200, { "Content-Length": bytes.length, Connection: "close" });
      if (alwaysFail || requests.length === 1) {
        response.flushHeaders();
        if (mode === "stall") response.write(bytes.subarray(0, 1));
        else response.end(bytes.subarray(0, 7));
      } else response.end(bytes);
    },
  );
  server.on("connection", (socket) => {
    sockets.add(socket);
    socket.once("close", () => sockets.delete(socket));
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const local = `https://localhost:${server.address().port}`;
  // Transport-only adapter: retain all real curl flags and production URLs in
  // the receipt, map their destination to our trusted local TLS server. No -k,
  // shorter timeout, simulated curl status, or fake output bytes are used.
  const wrapper = `#!${process.execPath}
import assert from "node:assert/strict";
import fs from "node:fs";
import { spawnSync } from "node:child_process";
const args = process.argv.slice(2), url = args.at(-1);
assert.ok([${JSON.stringify(primary)}, ${JSON.stringify(api)}].includes(url));
assert.deepEqual(args.slice(0, 7), ["--fail", "--silent", "--show-error", "--location", "--proto", "=https", "--tlsv1.2"]);
const deadlines = ["--connect-timeout", "10", "--max-time", "30", "--speed-limit", "1", "--speed-time", "5"];
if (!${original}) assert.deepEqual(args.slice(7, 15), deadlines);
const forwarded = [...args.slice(0, -1), "--cacert", ${JSON.stringify(cert)}, ${JSON.stringify(local)} + (url === ${JSON.stringify(primary)} ? "/primary" : "/api")];
const result = spawnSync("/usr/bin/curl", forwarded, { encoding: "utf8" });
fs.appendFileSync(${JSON.stringify(path.join(directory, "curl.jsonl"))}, JSON.stringify({ args, forwarded, status: result.status, signal: result.signal, stdout: result.stdout, stderr: result.stderr }) + "\\n");
process.stdout.write(result.stdout); process.stderr.write(result.stderr);
process.exit(result.status ?? 126);
`;
  fs.writeFileSync(path.join(bin, "curl"), wrapper, { mode: 0o755 });
  const source = original ? frozen : path.join(action, "download-prebuilt.sh");
  const started = Date.now();
  const child = spawn("/bin/bash", [source, archive], {
    env: { ...process.env, PATH: `${bin}:/usr/bin:/bin:/sbin` },
  });
  let stdout = "",
    stderr = "";
  child.stdout.on("data", (chunk) => {
    stdout += chunk;
  });
  child.stderr.on("data", (chunk) => {
    stderr += chunk;
  });
  const [status, signal] = await new Promise((resolve) =>
    child.once("close", (...result) => resolve(result)),
  );
  const elapsedMs = Date.now() - started;
  const calls = fs
    .readFileSync(path.join(directory, "curl.jsonl"), "utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  for (const socket of sockets) socket.destroy();
  await new Promise((resolve) => server.close(resolve));
  const result = {
    directory,
    archive,
    sentinel,
    bytes,
    requests,
    calls,
    status,
    signal,
    stdout,
    stderr,
    elapsedMs,
  };
  const capture = process.env.VIZE_RUST_SCRIPT_TRANSFER_CAPTURE_DIR;
  if (capture) {
    fs.mkdirSync(capture, { recursive: true });
    fs.writeFileSync(
      path.join(
        capture,
        `${original ? "original" : "candidate"}-${mode}-${alwaysFail}-${Date.now()}.json`,
      ),
      JSON.stringify(
        {
          ...result,
          sourceSha256: digest(fs.readFileSync(source)),
          curlSha256: digest(fs.readFileSync("/usr/bin/curl")),
          certificate: fs.readFileSync(cert, "utf8"),
          archiveBytes: fs.readFileSync(archive).toString("base64"),
          fixtureBytes: bytes.toString("base64"),
        },
        null,
        2,
      ) + "\n",
    );
  }
  return result;
}

function cleanup(result) {
  fs.rmSync(result.directory, { recursive: true, force: true });
}
function atomic(result) {
  assert.equal(result.signal, null);
  assert.equal(result.stdout, "");
  assert.ok(result.requests.every((request) => request.path === "/primary"));
  assert.ok(result.requests.every((request) => request.published === result.sentinel));
  assert.deepEqual(
    fs.readdirSync(result.directory).filter((file) => file.includes(".partial.")),
    [],
  );
  assert.ok(result.calls.every((call) => call.args.at(-1) === primary));
}

await test("real stalled HTTP200 delivery aborts and retries the same endpoint, then atomically publishes", async () => {
  const result = await observed();
  try {
    atomic(result);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.requests.length, 2);
    assert.equal(result.calls[0].status, 28);
    assert.equal(result.calls[0].stdout, "200");
    assert.match(result.calls[0].stderr, /Operation too slow|Timeout was reached|timed out/iu);
    assert.equal(result.calls[1].status, 0);
    assert.deepEqual(fs.readFileSync(result.archive), result.bytes);
    assert.match(result.stderr, /attempt=1 curl=28 HTTP=200/u);
    assert.ok(result.elapsedMs >= 5000 && result.elapsedMs < 65_000, String(result.elapsedMs));
    const destination = path.join(result.directory, "extracted");
    const extraction = spawnSync(
      "/bin/bash",
      [path.join(action, "extract-prebuilt.sh"), result.archive, destination, digest(result.bytes)],
      { encoding: "utf8" },
    );
    assert.equal(extraction.status, 0, extraction.stderr);
    assert.equal(
      fs.readFileSync(path.join(destination, "rust-script"), "utf8"),
      "#!/bin/bash\nprintf 'rust-script 0.34.0\\n'\n",
    );
  } finally {
    cleanup(result);
  }
});

await test("real partial transfer is refused by the frozen downloader and recovered by the candidate", async () => {
  for (const original of [true, false]) {
    const result = await observed({ original, mode: "partial" });
    try {
      assert.equal(result.calls[0].stdout, "200");
      assert.ok([18, 56].includes(result.calls[0].status), JSON.stringify(result.calls[0]));
      if (original) {
        assert.equal(result.status, result.calls[0].status);
        assert.equal(result.requests.length, 1);
        assert.notDeepEqual(fs.readFileSync(result.archive), result.bytes);
      } else {
        atomic(result);
        assert.equal(result.status, 0, result.stderr);
        assert.equal(result.requests.length, 2);
        assert.deepEqual(fs.readFileSync(result.archive), result.bytes);
      }
    } finally {
      cleanup(result);
    }
  }
});

await test("two actual stalled transfers fail within the bound and never replace the owned archive", async () => {
  const result = await observed({ alwaysFail: true });
  try {
    atomic(result);
    assert.equal(result.status, 28);
    assert.equal(result.requests.length, 2);
    assert.ok(result.calls.every((call) => call.status === 28 && call.stdout === "200"));
    assert.equal(fs.readFileSync(result.archive, "utf8"), result.sentinel);
    assert.match(result.stderr, /attempt=2 curl=28 HTTP=200/u);
    assert.ok(result.elapsedMs < 65_000, String(result.elapsedMs));
  } finally {
    cleanup(result);
  }
});

await test("successful network recovery still refuses the unchanged production digest before extraction", async () => {
  const result = await observed({ mode: "partial" });
  try {
    atomic(result);
    assert.equal(result.status, 0, result.stderr);
    const destination = path.join(result.directory, "refused");
    const extraction = spawnSync(
      "/bin/bash",
      [path.join(action, "extract-prebuilt.sh"), result.archive, destination, pin],
      { encoding: "utf8" },
    );
    assert.notEqual(extraction.status, 0);
    assert.match(extraction.stderr, /rust-script archive digest mismatch/u);
    assert.equal(fs.existsSync(destination), false);
    assert.deepEqual(fs.readFileSync(result.archive), result.bytes);
  } finally {
    cleanup(result);
  }
});
