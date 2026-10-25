import assert from "node:assert/strict";
import { test } from "node:test";
import {
  decodeNativeFrames,
  decodeStraceString,
  parseStraceTexts,
} from "./support/native-runtime-selection/lsp-strace.mjs";

const hex = (bytes) =>
  `"${[...Buffer.from(bytes)].map((byte) => `\\x${byte.toString(16).padStart(2, "0")}`).join("")}"`;
const frame = (packet) => {
  const bytes = Buffer.from(JSON.stringify(packet));
  return Buffer.concat([Buffer.from(`Content-Length: ${bytes.length}\r\n\r\n`), bytes]);
};
const exec = `1.000000 execve(${hex("/actual/compiler")}, [${hex("/actual/compiler")}, ${hex("--lsp")}, ${hex("--stdio")}], 0x1 /* 3 vars */) = 0 <0.000001>`;

void test("native syscall custody preserves complete framed bytes and original Go thread ownership", () => {
  const packet = {
    jsonrpc: "2.0",
    id: 7,
    result: { opaque: "🧭é日本語", unknown: { values: [null, 1] } },
  };
  const bytes = frame(packet);
  const owners = parseStraceTexts([
    {
      tid: 100,
      text: `${exec}\n1.100000 clone(child_stack=NULL, flags=CLONE_VM|CLONE_THREAD) = 101 <0.000001>\n2.000000 +++ exited with 0 +++\n`,
    },
    {
      tid: 101,
      text: `1.200000 write(1, ${hex(bytes)}, ${bytes.length}) = ${bytes.length} <0.000001>\n1.300000 +++ exited with 0 +++\n`,
    },
  ]);
  assert.equal(owners.length, 1);
  assert.equal(owners[0].process, 100);
  assert.deepEqual(owners[0].exit, { code: 0, signal: null });
  assert.deepEqual(owners[0].events[0].bytes, bytes);
  assert.deepEqual(decodeNativeFrames(bytes)[0].packet, packet);
  assert.deepEqual(decodeStraceString(hex(bytes)), bytes);
});

void test("native syscall custody joins only its actual unfinished call and honors partial-write counts", () => {
  const owners = parseStraceTexts([
    {
      tid: 100,
      text: `${exec}\n1.100000 read(0, <unfinished ...>\n1.200000 <... read resumed>${hex("abc")}, 3) = 3 <0.100000>\n1.300000 write(1, ${hex("abcdef")}, 6) = 3 <0.000001>\n2.000000 +++ exited with 1 +++\n`,
    },
  ]);
  assert.equal(owners[0].events[0].bytes.toString(), "abc");
  assert.equal(owners[0].events[1].bytes.toString(), "abc");
  assert.deepEqual(owners[0].exit, { code: 1, signal: null });
  assert.throws(
    () =>
      parseStraceTexts([
        { tid: 100, text: `${exec}\n1.100000 <... read resumed>${hex("abc")}, 3) = 3\n` },
      ]),
    /original entry/u,
  );
});

void test("native syscall custody refuses truncated buffers, unsupported successful IO and malformed frames", () => {
  for (const call of [
    `write(1, ${hex("abc")}..., 30) = 30`,
    `writev(1, [{iov_base=${hex("a")}, iov_len=1}], 1) = 1`,
    `read(0, ${hex("a")}, 1) = 2`,
  ]) {
    assert.throws(() =>
      parseStraceTexts([
        { tid: 100, text: `${exec}\n1.200000 ${call}\n2.000000 +++ exited with 0 +++\n` },
      ]),
    );
  }
  for (const bytes of [
    Buffer.from("Content-Length: 2\r\nContent-Length: 2\r\n\r\n{}"),
    Buffer.from("Content-Length: 4\r\n\r\n{}"),
    Buffer.from(`Content-Length: ${16 * 1024 * 1024 + 1}\r\n\r\n`),
  ])
    assert.throws(() => decodeNativeFrames(bytes));
});

void test("kernel-restarted native reads remain explicit and never manufacture input bytes", () => {
  const owners = parseStraceTexts([
    {
      tid: 100,
      text: `${exec}\n1.100000 read(0, 0x7ffff72cf5cf, 1) = ? ERESTARTSYS (To be restarted)\n1.200000 --- SIGALRM {si_signo=SIGALRM, si_code=SI_KERNEL} ---\n1.300000 read(0, ${hex("a")}, 1) = 1 <0.000001>\n2.000000 +++ exited with 0 +++\n`,
    },
  ]);
  assert.equal(owners[0].events[0].returnKind, "kernel-restart");
  assert.equal(owners[0].events[0].count, null);
  assert.equal(owners[0].events[0].bytes, undefined);
  assert.equal(owners[0].events[1].bytes.toString(), "a");
  assert.throws(
    () =>
      parseStraceTexts([
        {
          tid: 100,
          text: `${exec}\n1.100000 read(0, 0x1, 1) = ? UNKNOWN\n2.000000 +++ exited with 0 +++\n`,
        },
      ]),
    /explicit return count/u,
  );
});
