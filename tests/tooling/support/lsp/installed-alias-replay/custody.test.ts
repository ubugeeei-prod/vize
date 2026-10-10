import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import type { VizePublicRegistryInstallAuthority } from "./authority-schema.ts";
import { digest, verifyNativeJournal } from "./custody.ts";

/** Plain file/journal falsification only; these controls never load a native/provider. */
function control() {
  const installRoot = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "native-journal-control-")),
  );
  const nativePath = path.join(installRoot, "plain-control.node");
  fs.writeFileSync(nativePath, "not executable native bytes");
  const sha256 = digest(fs.readFileSync(nativePath));
  const corsaPath = path.join(installRoot, "plain-corsa-control");
  fs.writeFileSync(corsaPath, "not executable runtime bytes");
  const journalPath = path.join(installRoot, "events.ndjson");
  const authority = {
    installRoot,
    node: { path: process.execPath },
    native: { path: nativePath, sha256 },
    bundledCorsa: { path: corsaPath, sha256: digest(fs.readFileSync(corsaPath)) },
  } as VizePublicRegistryInstallAuthority;
  const common = { schema: "vize-public-native-custody-event-v1", pid: 123 };
  const initialized = {
    ...common,
    event: "initialized",
    installRoot,
    nativePath,
    sha256,
    node: process.execPath,
  };
  const attempt = {
    ...common,
    event: "attempt",
    actualPath: nativePath,
    sha256,
    expectedNative: true,
    corsaPath,
  };
  const returned = { ...attempt, event: "returned" };
  const exit = { ...common, event: "exit", code: 0, corsaPath };
  const write = (events: unknown[]) =>
    fs.writeFileSync(journalPath, `${events.map((event) => JSON.stringify(event)).join("\n")}\n`);
  return {
    installRoot,
    nativePath,
    journalPath,
    authority,
    initialized,
    attempt,
    returned,
    exit,
    write,
  };
}

test("native path resolution or a loader attempt never implies a successful loader return", () => {
  const c = control();
  try {
    for (const events of [
      [c.initialized],
      [c.initialized, c.attempt],
      [c.initialized, c.attempt, { ...c.attempt, event: "failed" }],
    ]) {
      c.write(events);
      assert.throws(() => verifyNativeJournal(c.journalPath, c.authority, 123));
    }
  } finally {
    fs.rmSync(c.installRoot, { recursive: true });
  }
});

test("a recorded return with the wrong process, native identity or changed actual bytes fails", () => {
  const c = control();
  try {
    c.write([c.initialized, c.attempt, c.returned, c.exit]);
    assert.throws(() => verifyNativeJournal(c.journalPath, c.authority, 456), /this LSP process/);
    c.write([
      c.initialized,
      c.attempt,
      { ...c.returned, actualPath: path.join(c.installRoot, "foreign.node") },
    ]);
    assert.throws(() => verifyNativeJournal(c.journalPath, c.authority, 123));
    c.write([c.initialized, c.attempt, { ...c.returned, sha256: "0".repeat(64) }]);
    assert.throws(() => verifyNativeJournal(c.journalPath, c.authority, 123));
    c.write([c.initialized, c.attempt, c.returned, c.exit]);
    fs.writeFileSync(c.nativePath, "changed after a recorded return");
    assert.throws(() => verifyNativeJournal(c.journalPath, c.authority, 123));
  } finally {
    fs.rmSync(c.installRoot, { recursive: true });
  }
});

test("rejection or an incomplete journal invalidates any other recorded successful event", () => {
  const c = control();
  try {
    c.write([c.initialized, c.attempt, c.returned, { ...c.attempt, event: "rejected" }]);
    assert.throws(
      () => verifyNativeJournal(c.journalPath, c.authority, 123),
      /rejected native load/,
    );
    c.write([c.initialized, c.attempt, c.returned, c.exit]);
    fs.appendFileSync(c.journalPath, "{");
    assert.throws(() => verifyNativeJournal(c.journalPath, c.authority, 123), /complete records/);
  } finally {
    fs.rmSync(c.installRoot, { recursive: true });
  }
});

test("a successful return must witness its attempt and automatically selected bundled Corsa", () => {
  const c = control();
  try {
    c.write([c.initialized, c.attempt, c.returned]);
    assert.doesNotThrow(() => verifyNativeJournal(c.journalPath, c.authority, 123));
    c.write([c.initialized, c.returned]);
    assert.throws(
      () => verifyNativeJournal(c.journalPath, c.authority, 123),
      /actual load attempt/,
    );
    c.write([c.initialized, c.attempt, { ...c.returned, corsaPath: "/unowned/source/tsc" }]);
    assert.throws(
      () => verifyNativeJournal(c.journalPath, c.authority, 123),
      /public bundled Corsa/,
    );
  } finally {
    fs.rmSync(c.installRoot, { recursive: true });
  }
});
