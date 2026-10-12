import assert from "node:assert/strict";
import { test } from "node:test";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import {
  collectPipes,
  framedMessages,
  nativeSessions,
  parseTrace,
  processClosure,
  quotedBytes,
} from "../../tools/support/compat/github/signature-strace.mjs";
import {
  artifact,
  originalTest,
  providerRevision,
  requireArtifact,
  selectedBinary,
  sha256,
  source,
} from "../../tools/support/compat/github/signature-archive-capture.mjs";
import { validateCapture } from "../../tools/support/compat/github/signature-capture-integrity.mjs";

const hex = (bytes) =>
  [...Buffer.from(bytes)].map((byte) => `\\x${byte.toString(16).padStart(2, "0")}`).join("");
const trace = (call) => `1.000000 ${call}\n2.000000 +++ exited with 0 +++\n`;
const frame = (body) => Buffer.from(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
const metadata = () => ({
  id: artifact.id,
  name: artifact.name,
  size_in_bytes: artifact.size,
  digest: artifact.digest,
  expired: false,
  workflow_run: { id: artifact.run, head_sha: source },
});

void test("literal archive rejects source/run/id/digest/expiration drift", () => {
  requireArtifact(metadata());
  for (const key of ["id", "name", "size_in_bytes", "digest", "expired"]) {
    assert.throws(() => requireArtifact({ ...metadata(), [key]: "changed" }));
  }
  for (const key of ["id", "head_sha"]) {
    const value = metadata();
    value.workflow_run[key] = "changed";
    assert.throws(() => requireArtifact(value));
  }
});

void test("exact case selection refuses missing, extra or different archive binaries", () => {
  const list = {
    "rust-suites": {
      id: {
        "package-name": "vize_maestro",
        "binary-name": "vize_maestro",
        "binary-path": "/workspace/target/ci/deps/vize_maestro-fixed",
        testcases: {
          [originalTest]: { "filter-match": { status: "matches" } },
          unrelated: { "filter-match": { status: "mismatch" } },
        },
      },
    },
  };
  assert.equal(selectedBinary(list), "/workspace/target/ci/deps/vize_maestro-fixed");
  list["rust-suites"].id.testcases.unrelated["filter-match"].status = "matches";
  assert.throws(() => selectedBinary(list), /additional test/);
  delete list["rust-suites"].id.testcases[originalTest];
  assert.throws(() => selectedBinary(list), /one archive binary/);
});

void test("byte decoding retains CRLF, Unicode and NUL without normalization", () => {
  const bytes = Buffer.from("🦀\r\n\0日本語");
  assert.ok(quotedBytes(hex(bytes)).equals(bytes));
  assert.throws(() => quotedBytes("\\x41..."));
  assert.throws(() => quotedBytes("\\n"));
});

void test("partial writes retain only kernel-confirmed bytes and pair full pipe reads", () => {
  const writes = parseTrace(10, trace(`write(3<pipe:[42]>, "${hex("abcdef")}", 6) = 3`));
  const reads = parseTrace(11, trace(`read(0<pipe:[42]>, "${hex("abc")}", 12) = 3`));
  const pipe = collectPipes([...writes, ...reads]).get("42");
  assert.equal(pipe.writeBytes.toString(), "abc");
  assert.ok(pipe.writeBytes.equals(pipe.readBytes));
  reads[0].io.bytes = Buffer.from("abd");
  assert.throws(() => collectPipes([...writes, ...reads]), /mismatch/);
  assert.throws(() => collectPipes(reads), /unobserved pipe writer/);
});

void test("resumed reads and complete vectored writes retain every returned byte", () => {
  const read = parseTrace(
    11,
    `1.0 read(0<pipe:[42]>, <unfinished ...>\n1.1 <... read resumed>"${hex("abcd")}", 9) = 4\n2.0 +++ exited with 0 +++\n`,
  );
  const write = parseTrace(
    10,
    trace(
      `writev(3<pipe:[42]>, [{iov_base="${hex("ab")}", iov_len=2}, {iov_base="${hex("cd")}", iov_len=2}], 2) = 4`,
    ),
  );
  assert.equal(
    collectPipes([...write, ...read])
      .get("42")
      .readBytes.toString(),
    "abcd",
  );
});

void test("missing lifecycle, abbreviated payload and ambiguous pipe order explicitly refuse", () => {
  for (const text of [
    "1.0 read(0<pipe:[42]>, <unfinished ...>\n",
    `1.0 write(1<pipe:[42]>, "${hex("x")}"..., 20) = 20\n2.0 +++ exited with 0 +++\n`,
    `1.0 write(1<pipe:[42]>, "${hex("x")}", 20) = 20\n2.0 +++ exited with 0 +++\n`,
    '1.0 <... read resumed>"", 1) = 0\n2.0 +++ exited with 0 +++\n',
  ]) {
    assert.throws(() => parseTrace(1, text));
  }
  const a = parseTrace(
    10,
    `1.0 write(3<pipe:[42]>, "${hex("a")}", 1 <unfinished ...>\n1.5 <... write resumed>) = 1\n2.0 +++ exited with 0 +++\n`,
  );
  const b = parseTrace(11, trace(`write(3<pipe:[42]>, "${hex("b")}", 1) = 1`));
  assert.throws(() => collectPipes([...a, ...b]), /ambiguous concurrent/);
  for (const time of [1.0, 1.5]) {
    b[0].time = time;
    b[0].end = time;
    assert.throws(() => collectPipes([...a, ...b]), /ambiguous concurrent/);
  }
  b[0].time = 1.500001;
  b[0].end = 1.500001;
  assert.equal(
    collectPipes([...a, ...b])
      .get("42")
      .writeBytes.toString(),
    "ab",
  );
});

void test("whole JSON-RPC packets retain errors and null separately and reject truncation", () => {
  const bytes = Buffer.concat([
    frame('{"jsonrpc":"2.0","id":1,"result":null}'),
    frame('{"jsonrpc":"2.0","id":2,"error":{"code":-1,"message":"full 🦀 reason"}}'),
  ]);
  const messages = framedMessages(bytes);
  assert.equal(messages[0].message.result, null);
  assert.equal(messages[1].message.error.message, "full 🦀 reason");
  assert.throws(() => framedMessages(bytes.subarray(0, bytes.length - 1)), /unfinished/);
  assert.throws(
    () => framedMessages(Buffer.from("Content-Length: 4\r\nContent-Length: 4\r\n\r\nnull")),
    /one Content-Length/,
  );
  assert.throws(() => framedMessages(Buffer.alloc(0)), /empty RPC/);
});

void test("native mode comes from actual observed argv, unsupported msgpack stays refusal", () => {
  const event = { exec: ["/native/tsc", "/native/tsc", "--api", "--async", "--cwd", "/project"] };
  assert.deepEqual(nativeSessions([event], "/native/tsc"), [event]);
  for (const extra of ["--pipe", "--pipe=/tmp/socket", "--lsp"]) {
    assert.throws(
      () => nativeSessions([{ exec: [...event.exec, extra] }], "/native/tsc"),
      /unsupported/,
    );
  }
  assert.throws(() => nativeSessions([event], "/foreign/tsc"), /no actual pinned/);
  event.exec = ["/native/tsc", "/native/tsc", "--api", "--cwd", "/project"];
  assert.throws(
    () => nativeSessions([event], "/native/tsc"),
    /unsupported observed native protocol/,
  );
});

void test("complete passive observation keeps original FAIL and rejects missing custody/closure", () => {
  const directory = mkdtempSync(join(tmpdir(), "vize-signature-capture-law-"));
  const put = (name, value) => writeFileSync(join(directory, name), value);
  const putJson = (name, value) => put(name, JSON.stringify(value));
  try {
    mkdirSync(join(directory, "bodies"));
    const binary = "/archive/test-binary";
    const provider = "/native/tsc";
    const nextest = "/archive/cargo-nextest";
    const originalMetadata = "/home/runner/_work/vize/vize/target/nextest/binaries-metadata.json";
    const dependencyPaths = ["/lib/loader.so", "/lib/libc.so"];
    const providerPackage = "/native/package.json";
    const commands = ["/runtime/cargo", "/runtime/rustup", "/runtime/toolchain/cargo"];
    const tree = "1".repeat(40);
    const bodies = [
      binary,
      provider,
      nextest,
      originalMetadata,
      ...commands,
      providerPackage,
      ...dependencyPaths,
    ].map((path, index) => {
      const bytes = Buffer.from(
        path === providerPackage
          ? JSON.stringify({ version: "7.0.2", gitHead: providerRevision })
          : `complete synthetic body ${index}`,
      );
      const retained = `bodies/${index}.bin`;
      put(retained, bytes);
      return { path, retained, bytes: bytes.length, sha256: sha256(bytes) };
    });
    putJson("source.json", {
      source,
      tree,
      binary,
      provider,
      providerPackage,
      providerRevision,
      originalTest,
      bodies,
      environment: {
        requireTsgo: "1",
        disableTsgo: null,
        nuxtIterations: "100",
        rustTestThreads: "4",
      },
    });
    putJson("source-unchanged.json", { source, tree, unchanged: true });
    putJson("artifact.json", metadata());
    put("archive-zip.sha256", `${artifact.digest.slice(7)}  archive.zip\n`);
    put("driver-sha.txt", "2".repeat(40));
    for (const name of [
      "driver-workflow.yml",
      "signature-archive-capture.mjs",
      "signature-archive-libdirs.mjs",
      "signature-strace.mjs",
      "signature-capture-integrity.mjs",
      "strace-version.txt",
      "test.stdout",
    ])
      put(name, "whole synthetic driver evidence");
    put("test.stderr", "original assertion failed");
    putJson("library-runtime.json", {
      cwd: "/home/runner/_work/vize/vize",
      variables: {},
      nextest,
      nextestCommand: nextest,
      nextestSha256: bodies[2].sha256,
      version: "cargo-nextest 0.9.146",
      ...Object.fromEntries(
        ["cargo", "rustup", "toolchainCargo"].flatMap((key, index) => [
          [key, commands[index]],
          [`${key}Sha256`, bodies[index + 4].sha256],
        ]),
      ),
    });
    for (const name of [
      "selected-tests.json",
      "derived-selected-tests.json",
      "derived-binaries-metadata.json",
    ])
      putJson(name, { complete: name });
    const hashJson = (name) => sha256(JSON.stringify({ complete: name }));
    const mapping = {
      libraryEnvironmentEqual: true,
      inverseWholeObjectExact: true,
      archiveListSha256: hashJson("selected-tests.json"),
      derivedListSha256: hashJson("derived-selected-tests.json"),
      derivedSha256: hashJson("derived-binaries-metadata.json"),
      originalSha256: bodies[3].sha256,
      dependencies: bodies
        .slice(8)
        .map((body) => ({ loaderReportedPath: body.path, path: body.path, sha256: body.sha256 })),
    };
    putJson("library-mapping.json", mapping);
    putJson("archive-receipt.json", {
      schemaVersion: 3,
      source,
      sha: source,
      tree,
      workspaceRoot: "/home/runner/_work/vize/vize",
      requireTsgo: "1",
      disableTsgo: null,
      nextestVersion: "0.9.146",
      cargoProfile: "ci",
    });
    putJson("instrumented-outcome.json", { exitCode: 100, elapsedSeconds: 1 });
    put(
      "junit.xml",
      `<testsuite tests="1"><testcase name="${originalTest}"><failure>original None</failure></testcase></testsuite>`,
    );
    const request = frame('{"jsonrpc":"2.0","id":1,"method":"textDocument/signatureHelp"}');
    const response = frame('{"jsonrpc":"2.0","id":1,"result":null}');
    put(
      "strace.9",
      `0.8 execve("${hex(nextest)}", ["${hex(nextest)}"], 0x1 /* 20 vars */) = 0\n0.9 clone(flags=SIGCHLD) = 10\n0.9 clone(flags=SIGCHLD) = 11\n0.9 clone(flags=SIGCHLD) = 12\n2.0 +++ exited with 100 +++\n`,
    );
    put(
      "strace.10",
      `1.0 execve("${hex(binary)}", ["${hex(binary)}", "${hex("--exact")}", "${hex(originalTest)}", "${hex("--nocapture")}"], 0x1 /* 20 vars */) = 0\n1.01 pipe2([3<pipe:[42]>, 5<pipe:[42]>], O_CLOEXEC) = 0\n1.02 pipe2([4<pipe:[43]>, 6<pipe:[43]>], O_CLOEXEC) = 0\n1.1 clone(flags=SIGCHLD) = 20\n1.2 write(3<pipe:[42]>, "${hex(request)}", ${request.length}) = ${request.length}\n1.5 read(4<pipe:[43]>, "${hex(response)}", 4096) = ${response.length}\n2.0 +++ exited with 101 +++\n`,
    );
    put(
      "strace.20",
      `1.11 execve("${hex(provider)}", ["${hex(provider)}", "${hex("--lsp")}"], 0x1 /* 20 vars */) = 0\n1.3 read(0<pipe:[42]>, "${hex(request)}", 4096) = ${request.length}\n1.4 write(1<pipe:[43]>, "${hex(response)}", ${response.length}) = ${response.length}\n2.0 +++ exited with 0 +++\n`,
    );
    for (const pid of [11, 12])
      put(
        `strace.${pid}`,
        `0.91 execve("${hex(binary)}", ["${hex(binary)}", "${hex("--list")}", "${hex("--format")}", "${hex("terse")}"${pid === 12 ? `, "${hex("--ignored")}"` : ""}], 0x1 /* 20 vars */) = 0\n0.99 +++ exited with 0 +++\n`,
      );
    const result = validateCapture(directory);
    assert.equal(result.captureComplete, true);
    assert.equal(result.instrumentedTestPassed, false);
    assert.equal(result.originalAcceptanceCredit, false);
    assert.equal(result.performanceCredit, false);
    const events = [
      { pid: 9, time: 0.8, call: "exec", exec: [nextest] },
      { pid: 9, time: 0.9, call: "clone(flags=SIGCHLD) = 10" },
      { pid: 10, time: 1, call: "exec", exec: [binary] },
      { pid: 10, time: 1.1, call: "clone(flags=SIGCHLD) = 20" },
      { pid: 20, time: 1.11, call: "exec", exec: [provider] },
    ];
    const closure = (rows) => processClosure(rows, [nextest], binary, [rows.at(-1)]);
    assert.equal(closure(events).test, 10);
    assert.throws(() => closure(events.filter((_, i) => i !== 3)), /orphan/);
    assert.throws(
      () => closure([...events.slice(0, 3), { ...events[3], pid: 9 }, events[4]]),
      /native session/,
    );
    assert.throws(
      () => closure([{ ...events[0], time: 2 }, ...events.slice(1)]),
      /test does not descend/,
    );
    assert.throws(
      () => closure([...events.slice(0, 4), events[3], events[4]]),
      /reused\/conflicting/,
    );
    assert.equal(closure(events).root, 9);
    put("test.stdout", "");
    assert.equal(validateCapture(directory).captureComplete, true);
    rmSync(join(directory, "test.stdout"));
    assert.throws(() => validateCapture(directory), /ENOENT/);
    put("test.stdout", "");
    putJson("library-mapping.json", { ...mapping, libraryEnvironmentEqual: false });
    assert.throws(() => validateCapture(directory));
    putJson("library-mapping.json", mapping);
    putJson("instrumented-outcome.json", { exitCode: 0, elapsedSeconds: 1 });
    assert.throws(() => validateCapture(directory), /exit\/result mismatch/);
    putJson("instrumented-outcome.json", { exitCode: 100, elapsedSeconds: 1 });
    put("bodies/1.bin", "changed provider");
    assert.throws(() => validateCapture(directory));
    put("bodies/1.bin", "complete synthetic body 1");
    rmSync(join(directory, "strace.20"));
    assert.throws(() => validateCapture(directory), /no retained lifecycle trace/);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
