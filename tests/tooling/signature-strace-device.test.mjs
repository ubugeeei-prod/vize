import assert from "node:assert/strict";
import { test } from "node:test";
import {
  collectPipes,
  parseTrace,
  pipeInode,
  pipeInodes,
  processClosure,
} from "../../tools/support/compat/github/signature-strace.mjs";

// Exact retained run 38193883416/artifact 11722991118, strace.7638 line 235.
// Whole trace SHA256: 02350057a014b9490751a10ada875f12e9454379cc61690298b06dd9dc492a4c.
const observed = String.raw`1791770360.959034 read(3<\x2f\x64\x65\x76\x2f\x75\x72\x61\x6e\x64\x6f\x6d<char 1:9>>, "\x6d\xaa\xd5\x5b\xc4\xcc\x1c\x50", 8) = 8`;
const trace = (line) => `${line}\n1791770361.403037 +++ exited with 100 +++\n`;

void test("real Linux char-device descriptor retains every kernel-confirmed byte", () => {
  const actual = parseTrace(7638, trace(observed))[0].io;
  assert.equal(actual.direction, "read");
  assert.equal(actual.fd, 3);
  assert.equal(actual.bytes.toString("hex"), "6daad55bc4cc1c50");
  assert.equal(
    actual.target,
    String.raw`<\x2f\x64\x65\x76\x2f\x75\x72\x61\x6e\x64\x6f\x6d<char 1:9>>`,
  );
  for (const target of ["<block 1:9>", "<char 1:9><char 1:9>", "<char 1:9", "<unknown>"])
    assert.throws(
      () => parseTrace(7638, trace(observed.replace("<char 1:9>", target))),
      /unsupported positive I\/O/,
    );
});

const hex = (value) =>
  [...Buffer.from(value)].map((byte) => `\\x${byte.toString(16).padStart(2, "0")}`).join("");

void test("actual -xx pipe names retain raw annotations and authenticate identical full bytes", () => {
  const target = `<${hex("pipe:[33129]")}>`;
  assert.equal(pipeInode(target), "33129");
  assert.equal(pipeInode("<pipe:[33129]>"), "33129");
  assert.deepEqual(pipeInodes(`pipe2([3${target}, 5${target}], O_CLOEXEC) = 0`), [
    "33129",
    "33129",
  ]);
  const write = parseTrace(
    1,
    trace(`1791770361.0 write(3${target}, "${hex("full🦀\r\n")}", 10) = 10`),
  );
  const read = parseTrace(
    2,
    trace(`1791770361.1 read(0${target}, "${hex("full🦀\r\n")}", 4096) = 10`),
  );
  assert.equal(write[0].io.target, target);
  const pipe = collectPipes([...write, ...read]).get("33129");
  assert.ok(pipe.writeBytes.equals(Buffer.from("full🦀\r\n")));
  assert.ok(pipe.readBytes.equals(pipe.writeBytes));
  for (const name of [
    "pipe:[33129]suffix",
    "pipe:[-1]",
    "file:[33129]",
    "pipe:[33129]<char 1:9>",
    "pípe:[33129]",
  ])
    assert.equal(pipeInode(`<${hex(name)}>`), undefined);
  assert.equal(pipeInode(target.slice(0, -1)), undefined);
  read[0].io.bytes[0] = 0;
  assert.throws(() => collectPipes([...write, ...read]), /writer\/reader pipe byte mismatch/);
});

void test("two original nextest lists are separate from the sole exact-case native owner", () => {
  const binary = "/archive/vize_maestro";
  const nextest = "/runtime/cargo-nextest";
  const selected = "ide::signature_help::tests::signature_help_tracks_imported_callable_edits";
  const exec = (pid, args) => ({ pid, time: 1, call: "exec", exec: [binary, binary, ...args] });
  const listing = [
    exec(2, ["--list", "--format", "terse"]),
    exec(3, ["--list", "--format", "terse", "--ignored"]),
  ];
  const actual = exec(4, ["--exact", selected, "--nocapture"]);
  const native = { pid: 5, time: 2, call: "exec", exec: ["/native/tsc"] };
  const events = [
    { pid: 1, time: 0, call: "exec", exec: [nextest] },
    ...[2, 3, 4].map((pid) => ({ pid: 1, time: 0.5, call: `clone(flags=SIGCHLD) = ${pid}` })),
    ...listing,
    ...listing.map(({ pid }) => ({ pid, time: 1.5, call: "+++ exited with 0 +++" })),
    actual,
    { pid: 4, time: 1.9, call: "clone(flags=SIGCHLD) = 5" },
    native,
  ];
  const closure = (rows) => processClosure(rows, [nextest], binary, [native], selected);
  const result = closure(events);
  assert.equal(result.test, 4);
  assert.deepEqual(
    result.listings,
    listing.map(({ pid, exec: argv }) => ({ pid, argv })),
  );
  assert.throws(
    () => closure(events.filter((row) => ![2, 3].includes(row.pid) && !/ = [23]$/.test(row.call))),
    /exact two archive listing/,
  );
  const mutate = (pid, replace) =>
    events.map((row) => (row.pid === pid && row.exec ? replace(row) : row));
  for (const args of [
    ["--exact", "foreign", "--nocapture"],
    ["--exact", selected, "--ignored"],
    ["--exact", selected],
    ["--list"],
  ])
    assert.throws(() => closure(mutate(4, () => exec(4, args))));
  assert.throws(
    () =>
      closure(
        mutate(3, (row) => ({ ...row, exec: [binary, "/foreign/argv0", ...row.exec.slice(2)] })),
      ),
    /one exact archive listing/,
  );
  assert.throws(
    () => closure(mutate(3, () => exec(3, ["--list", "--format", "terse"]))),
    /one exact archive listing/,
  );
  assert.throws(
    () =>
      closure(
        mutate(3, () => exec(3, ["--list", "--format", "terse", "--ignored", "--nocapture"])),
      ),
    /one exact archive listing/,
  );
  assert.throws(
    () => closure(events.filter((row) => row.pid !== 3 && !row.call.endsWith("= 3"))),
    /exact two archive listing/,
  );
  assert.throws(
    () =>
      closure(
        events.map((row) =>
          row.pid === 3 && row.call.includes("exited")
            ? { ...row, call: "+++ exited with 1 +++" }
            : row,
        ),
      ),
    /listing did not exit/,
  );
  assert.throws(
    () =>
      closure(
        events.map((row) => (row.call === "clone(flags=SIGCHLD) = 5" ? { ...row, pid: 2 } : row)),
      ),
    /native session does not descend/,
  );
  assert.throws(() => closure([...events, actual]), /one original archive test binary/);
});

void test("actual unknown unfinished native syscall remains an explicit completeness refusal", () => {
  const unknown =
    "1791770361.372671 ???( <unfinished ...>\n1791770361.373601 +++ exited with 0 +++\n";
  assert.throws(() => parseTrace(7838, unknown), /unfinished syscall at end of trace/);
});
