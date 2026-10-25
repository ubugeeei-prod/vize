import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import {
  decodeNativeFrames,
  decodeStraceString,
  parseStraceTexts,
  parseStraceCapture,
  validateExecInventory,
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

void test("native byte custody refuses cross-thread timestamp ties and overlapping syscall intervals", () => {
  const trace = (first, second) => [
    {
      tid: 100,
      text: `${exec}\n1.100000 clone(child_stack=NULL, flags=CLONE_VM|CLONE_THREAD) = 101 <0.000001>\n${second}\n3.000000 +++ exited with 0 +++\n`,
    },
    { tid: 101, text: `${first}\n2.900000 +++ exited with 0 +++\n` },
  ];
  const first = frame({ jsonrpc: "2.0", id: 1, result: null }),
    second = frame({ jsonrpc: "2.0", id: 2, result: null });
  for (const [left, right] of [
    ["2.000000", "2.000000"],
    ["2.000000", "2.000050"],
  ])
    assert.throws(
      () =>
        parseStraceTexts(
          trace(
            `${left} write(1, ${hex(first)}, ${first.length}) = ${first.length} <0.000100>`,
            `${right} write(1, ${hex(second)}, ${second.length}) = ${second.length} <0.000001>`,
          ),
        ),
      /ties\/overlaps refuse/u,
    );
  const large = parseStraceTexts(
    trace(
      `1792951393.000000001 write(1, ${hex(first)}, ${first.length}) = ${first.length} <0.000000001>`,
      `1792951393.000000100 write(1, ${hex(second)}, ${second.length}) = ${second.length} <0.000000001>`,
    ),
  );
  assert.deepEqual(
    decodeNativeFrames(Buffer.concat(large[0].events.map((event) => event.bytes))).map(
      (frame) => frame.packet.id,
    ),
    [1, 2],
  );
});

void test("exec custody retains failed probes and refuses unparsed execveat rather than hiding them", () => {
  const failed = `1.100000 execve(${hex("/isolated/npm")}, [${hex("npm")}, ${hex("root")}, ${hex("-g")}], 0x1 /* 3 vars */) = -1 ENOENT (No such file or directory) <0.000001>`;
  const capture = parseStraceCapture([
    { tid: 100, text: `${exec}\n${failed}\n2.000000 +++ exited with 0 +++\n` },
  ]);
  assert.equal(capture.attempts.length, 2);
  assert.equal(capture.attempts[1].path, "/isolated/npm");
  assert.equal(capture.attempts[1].result, -1);
  assert.throws(() => validateExecInventory(capture, {}), /failed execution\/discovery attempts/u);
  assert.throws(
    () =>
      parseStraceCapture([
        {
          tid: 100,
          text: `1.000000 execveat(3, ${hex("")}, [${hex("compiler")}], 0x1, AT_EMPTY_PATH) = 0 <0.000001>\n`,
        },
      ]),
    /every exec attempt/u,
  );
});

void test("exec custody admits only exact source, embedded config, and selected provider roles", () => {
  const directory = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-lsp-exec-custody-")),
  );
  try {
    const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
    const file = (name) => {
      const filename = path.join(directory, name);
      fs.writeFileSync(filename, name);
      return { path: filename, sha256: digest(Buffer.from(name)) };
    };
    const source = file("source"),
      node = file("node"),
      sdk = file("sdk"),
      foreign = file("foreign");
    const owner = (image, argv, process) => ({
      path: image.path,
      argv,
      process,
      exit: { code: 0, signal: null },
      rows: [],
    });
    const sourceOwner = owner(source, [source.path, "lsp", "--stdio"], 100),
      nodeOwner = owner(
        node,
        [
          "node",
          "--input-type=module",
          "-e",
          "exact-source-script",
          "/authored/vite.config.mjs",
          "/tmp/vize-config-compile-abc",
          "--observe-config-inputs",
        ],
        200,
      );
    const sdkOwner = owner(sdk, [sdk.path, "--lsp", "--stdio"], 300);
    const authority = {
      source,
      node: { ...node, physicalPath: node.path },
      sdk,
      configScript: "exact-source-script",
      configFile: "/authored/vite.config.mjs",
      configCachePattern: /^\/tmp\/vize-config-compile-[A-Za-z0-9]+$/u,
    };
    const verify = (owners, overrides = {}) =>
      validateExecInventory({ owners, attempts: [] }, { ...authority, ...overrides });
    assert.deepEqual(
      verify([sourceOwner, nodeOwner, sdkOwner]).map((owner) => owner.role),
      ["source-lsp", "source-config-node", "selected-native-lsp"],
    );
    assert.deepEqual(
      verify([sourceOwner, nodeOwner], { sdk: null }).map((owner) => owner.role),
      ["source-lsp", "source-config-node"],
    );
    for (const invalid of [
      [sourceOwner, nodeOwner, owner(foreign, [foreign.path, "--lsp", "--stdio"], 400)],
      [sourceOwner, nodeOwner, owner(sdk, [sdk.path, "--project", "tsconfig.json"], 300)],
      [
        sourceOwner,
        {
          ...nodeOwner,
          argv: nodeOwner.argv.map((value) =>
            value === "exact-source-script" ? "foreign script" : value,
          ),
        },
        sdkOwner,
      ],
      [nodeOwner, sdkOwner],
    ])
      assert.throws(() => verify(invalid));
    assert.throws(() => verify([sourceOwner, nodeOwner, sdkOwner], { sdk: null }));
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
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

void test("only proved same-group terminal interruption can retain an unknown syscall without coverage credit", () => {
  const traces = (
    unknownAt = "2.200000",
    workerTerminal = 0,
    extra = "",
    exitGroup = "2.100000 exit_group(0) = ?\n",
  ) => [
    {
      tid: 100,
      text: `${exec}\n1.100000 clone(child_stack=NULL, flags=CLONE_VM|CLONE_THREAD) = 101 <0.000001>\n${unknownAt} ???( <unfinished ...>\n${extra}2.300000 +++ exited with 0 +++\n`,
    },
    { tid: 101, text: `${exitGroup}2.210000 +++ exited with ${workerTerminal} +++\n` },
  ];
  const capture = parseStraceCapture(traces());
  assert.equal(capture.syscallsComplete, false);
  assert.equal(capture.terminalUnknownSyscalls.length, 1);
  assert.equal(
    capture.terminalUnknownSyscalls[0].classification,
    "UNKNOWN_UNCAPTURED_TERMINAL_SYSCALL",
  );
  assert.equal(capture.terminalUnknownSyscalls[0].syscallBytesCaptured, false);
  assert.match(capture.terminalUnknownSyscalls[0].raw, /\?\?\?\( <unfinished/u);
  for (const malformed of [
    traces("2.000000"),
    traces("2.200000", 1),
    traces("2.200000", 0, "", ""),
    traces("2.200000", 0, `2.250000 write(1, ${hex("a")}, 1) = 1 <0.000001>\n`),
    traces().map((trace) => (trace.tid === 101 ? { ...trace, tid: 102 } : trace)),
    [
      {
        tid: 100,
        text: `${exec}\n2.200000 execve( <unfinished ...>\n2.300000 +++ exited with 0 +++\n`,
      },
    ],
  ])
    assert.throws(() => parseStraceCapture(malformed));
});

void test("an unfinished exit entry remains uncaptured and needs its earlier same-code group exit proof", () => {
  const trace = (
    entry = "2.200000 exit_group(0 <unfinished ...>",
    later = "",
    normal = "2.100000 exit_group(0) = ?\n",
  ) => [
    {
      tid: 100,
      text: `${exec}\n1.100000 clone(child_stack=NULL, flags=CLONE_VM|CLONE_THREAD) = 101 <0.000001>\n${normal}2.300000 +++ exited with 0 +++\n`,
    },
    { tid: 101, text: `${entry}\n${later}2.310000 +++ exited with 0 +++\n` },
  ];
  const capture = parseStraceCapture(trace());
  assert.equal(capture.syscallsComplete, false);
  assert.equal(capture.terminalExitEntries.length, 1);
  assert.equal(
    capture.terminalExitEntries[0].classification,
    "UNKNOWN_UNCAPTURED_NONRETURNING_EXIT_ENTRY",
  );
  assert.equal(capture.terminalExitEntries[0].syscallBytesCaptured, false);
  assert.equal(capture.terminalExitEntries[0].raw, "2.200000 exit_group(0 <unfinished ...>");
  for (const malformed of [
    trace("2.000000 exit_group(0 <unfinished ...>"),
    trace("2.200000 exit_group(1 <unfinished ...>"),
    trace("2.200000 exit_group(0, 1 <unfinished ...>"),
    trace("2.200000 exit_group(0 <unfinished ...>", "", ""),
    trace(
      "2.200000 exit_group(0 <unfinished ...>",
      `2.250000 read(0, ${hex("a")}, 1) = 1 <0.000001>\n`,
    ),
    trace("2.200000 read(0, <unfinished ...>"),
    trace().map((row) => (row.tid === 101 ? { ...row, tid: 102 } : row)),
  ])
    assert.throws(() => parseStraceCapture(malformed));
});
