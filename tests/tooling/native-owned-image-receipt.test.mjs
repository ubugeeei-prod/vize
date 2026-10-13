import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { names, validatePackets } from "../../tools/support/compat/github/native-owned-image.mjs";

function fixture() {
  const runtime = {
    requireNative: "1",
    disableNative: null,
    platform: "linux",
    arch: "x64",
    nativeSHA256: "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4",
    nativeVersion: "Version 7.0.2",
    package: { name: "@typescript/typescript-linux-x64", version: "7.0.2" },
    nativeBinary: fs.realpathSync(process.execPath),
    wholeWrapper: "whole wrapper bytes",
    wholeInitialize: {
      capabilities: {
        positionEncoding: "utf-16",
        diagnosticProvider: { identifier: "typescript" },
      },
      serverInfo: { name: "typescript-go", version: "7.0.2" },
    },
  };
  const live = {
    pid: 41,
    completed: false,
    stderr: [],
    stderr_complete: false,
    stderr_error: null,
    exit_status: null,
    process_error: null,
    timed_out: false,
    kill_requested: false,
    reaped: false,
    elapsed_nanos: "0",
  };
  const changes = [
    { completed: true },
    { stderr_complete: true },
    { stderr_error: "reader failed" },
    { exit_status: 0 },
    { process_error: "PID mismatch" },
    { timed_out: true },
    { kill_requested: true },
    { reaped: true },
    { pid: 0 },
  ];
  const native = {
    pid: 73,
    wholeLiveStat: "73 (tsc) S 42 retained whole stat bytes",
    imageSHA256: runtime.nativeSHA256,
    nativePath: runtime.nativeBinary,
    wholeWrapper: runtime.wholeWrapper,
    wholeInitialize: runtime.wholeInitialize,
    wholeChangedInitialize: {
      ...runtime.wholeInitialize,
      capabilities: { ...runtime.wholeInitialize.capabilities, positionEncoding: "utf-8" },
    },
    before: { ...live, pid: 73 },
    terminal: {
      ...live,
      pid: 73,
      completed: true,
      reaped: true,
      stderr_complete: true,
      exit_status: 0,
      elapsed_nanos: "42",
    },
    knownNative: true,
    unknownInitialize: false,
    absentObserver: false,
    reapedImage: null,
  };
  const packets = [
    [
      "ownership.json",
      {
        original: live,
        wholeDeniedReceipts: changes.map((change) => ({ ...live, ...change })),
        absentObserver: false,
        selfPid: 42,
        selfImage: null,
        missingPid: 4294967295,
        missingImage: null,
      },
    ],
    [
      "life.json",
      {
        pid: 41,
        parent: 42,
        birth: 512,
        wholePositiveStat: "41 (native (exec)) S 42 0 0 0 512 retained whole bytes",
        deniedStates: ["Z", "X", "?"],
        foreignParent: 43,
        deniedBirth: 0,
        deniedPid: 42,
      },
    ],
    [
      "foreign.json",
      {
        pid: 59,
        wholeLiveStat: "59 (cat) S 42 retained whole bytes",
        imageSHA256: "f".repeat(64),
        knownNative: false,
        terminalRaw: 0,
        reapedImage: null,
        killRequested: false,
      },
    ],
    ["native.json", native],
  ];
  const stdout =
    names
      .map(
        (name) =>
          `test lsp_client::editor_lsp::native_shutdown::owned_image::tests::${name} ... ok`,
      )
      .join("\n") + "\ntest result: ok. 4 passed; 0 failed; 0 ignored; 20 filtered out;\n";
  return { runtime, packets, stdout };
}

void test("the parser accepts four complete authored control packets; toy packets grant no physical qualification", () => {
  const { packets, stdout, runtime } = fixture();
  assert.doesNotThrow(() => validatePackets(packets, stdout, runtime));
});

void test("missing duplicated and unexecuted native controls cannot qualify", () => {
  const { packets, stdout, runtime } = fixture();
  for (let index = 0; index < packets.length; index++) {
    assert.throws(() =>
      validatePackets(
        packets.filter((_, other) => other !== index),
        stdout,
        runtime,
      ),
    );
    assert.throws(() => validatePackets([...packets, packets[index]], stdout, runtime));
    assert.throws(() =>
      validatePackets(packets, stdout.replace(names[index], "unexecuted"), runtime),
    );
  }
  assert.throws(() => validatePackets(packets, "test result: ok. 0 passed; 0 failed;", runtime));
});

void test("unknown providers disabled native and partial initialize bytes refuse", () => {
  const { packets, stdout, runtime } = fixture();
  for (const changed of [
    { requireNative: "0" },
    { disableNative: "1" },
    { nativeSHA256: "f".repeat(64) },
    { platform: "darwin" },
    { arch: "arm64" },
    { nativeVersion: "Version 7.0.3" },
    { package: { ...runtime.package, version: "7.0.3" } },
  ])
    assert.throws(() => validatePackets(packets, stdout, { ...runtime, ...changed }));
  const changed = structuredClone(packets);
  delete changed[3][1].wholeInitialize.capabilities.diagnosticProvider;
  assert.throws(() => validatePackets(changed, stdout, runtime));
});

void test("nonzero stderr incomplete reaping kill timeout and changed owned PID stay fatal", () => {
  const { packets, stdout, runtime } = fixture();
  for (const changed of [
    { exit_status: 256 },
    { stderr: [99, 111, 110, 116, 101, 120, 116, 32, 99, 97, 110, 99, 101, 108, 101, 100, 10] },
    { stderr_complete: false },
    { stderr_error: "reader failed" },
    { process_error: "lost owner" },
    { reaped: false },
    { completed: false },
    { timed_out: true },
    { kill_requested: true },
    { pid: 74 },
  ]) {
    const copy = structuredClone(packets);
    Object.assign(copy[3][1].terminal, changed);
    assert.throws(() => validatePackets(copy, stdout, runtime));
  }
});

void test("whole foreign and ownership refusal controls cannot be replaced by successful-looking packets", () => {
  const { packets, stdout, runtime } = fixture();
  for (const index of [0, 1, 2]) {
    const copy = structuredClone(packets);
    copy[index][1] = { matchedCompleteExpectations: true };
    assert.throws(() => validatePackets(copy, stdout, runtime));
  }
  const copy = structuredClone(packets);
  copy[0][1].wholeDeniedReceipts[3].exit_status = null;
  assert.throws(() => validatePackets(copy, stdout, runtime));
});
