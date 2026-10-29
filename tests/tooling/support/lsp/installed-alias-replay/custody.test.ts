import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import type { VizePublicRegistryInstallAuthority } from "./authority-schema.ts";
import { explicitProviderMode } from "./authority.ts";
import {
  digest,
  publicEnvironment,
  sourceOverrideKeys,
  verifyNativeJournal,
  verifySourceOverrideReceipt,
} from "./custody.ts";

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

test("a successful return must witness its attempt and authenticated checker authority", () => {
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
      /reviewed checker authority mode/,
    );
  } finally {
    fs.rmSync(c.installRoot, { recursive: true });
  }
});

test("explicit replay can bind the authenticated provider without changing receipt semantics", () => {
  const c = control();
  try {
    c.authority.resolutionMode = "explicit-user-provider";
    c.write([c.initialized, c.attempt, c.returned]);
    assert.throws(() => verifyNativeJournal(c.journalPath, c.authority, 123));
    assert.doesNotThrow(() =>
      verifyNativeJournal(c.journalPath, c.authority, 123, c.authority.bundledCorsa.path),
    );
  } finally {
    fs.rmSync(c.installRoot, { recursive: true });
  }
});

test("explicit replay environment names only the authenticated installed provider", () => {
  const c = control();
  const keys = [...sourceOverrideKeys, "VIZE_PUBLIC_NATIVE_CUSTODY"] as const;
  const prior = Object.fromEntries(keys.map((key) => [key, process.env[key]]));
  try {
    c.authority.resolutionMode = "explicit-user-provider";
    for (const key of keys) delete process.env[key];
    const env = publicEnvironment(c.authority, path.join(c.installRoot, "fresh-journal.ndjson"));
    assert.equal(env.CORSA_PATH, c.authority.bundledCorsa.path);
    assert.deepEqual(JSON.parse(env.VIZE_PUBLIC_NATIVE_CUSTODY ?? "null"), {
      schema: "vize-public-native-custody-v1",
      installRoot: c.authority.installRoot,
      nativePath: c.authority.native.path,
      nativeSha256: c.authority.native.sha256,
      journalPath: path.join(c.installRoot, "fresh-journal.ndjson"),
      checkerPath: c.authority.bundledCorsa.path,
      checkerSha256: c.authority.bundledCorsa.sha256,
    });
    for (const key of sourceOverrideKeys) {
      if (key !== "CORSA_PATH") assert.equal(env[key], "");
    }
  } finally {
    for (const key of keys) {
      const value = prior[key];
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
    fs.rmSync(c.installRoot, { recursive: true });
  }
});

test("legacy receipts may omit only the later TSRS override fields", () => {
  const complete = Object.fromEntries(sourceOverrideKeys.map((key) => [key, ""]));
  const legacy: Record<string, string> = { ...complete };
  delete legacy.TSRS_PATH;
  delete legacy.TSRS_EXECUTABLE;
  assert.doesNotThrow(() => verifySourceOverrideReceipt(legacy, false));
  assert.throws(() => verifySourceOverrideReceipt(legacy, true));
  assert.throws(() => verifySourceOverrideReceipt({ ...complete, TSRS_PATH: "/ambient" }, false));
});

test("receipt resolution modes are closed to legacy or explicit user authority", () => {
  assert.equal(explicitProviderMode(undefined), false);
  assert.equal(explicitProviderMode("explicit-user-provider"), true);
  assert.throws(() => explicitProviderMode("future-provider"), /unsupported runtime resolution/u);
});
