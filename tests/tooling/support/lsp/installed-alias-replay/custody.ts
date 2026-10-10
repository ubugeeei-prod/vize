import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import type { VizePublicRegistryInstallAuthority } from "./authority-schema.ts";

export const digest = (bytes: Buffer | string): string =>
  createHash("sha256").update(bytes).digest("hex");
export const sourceOverrideKeys = [
  "NODE_OPTIONS",
  "VIZE_PREFER_WORKSPACE_BINDING",
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
  "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
] as const;

export function exactFile(file: string, sha256: string, owner?: string): string {
  assert.ok(path.isAbsolute(file), "authority paths must be absolute");
  assert.match(sha256, /^[a-f0-9]{64}$/u);
  assert.equal(fs.realpathSync(file), file, "authenticated file cannot redirect through a symlink");
  assert.ok(fs.statSync(file).isFile());
  if (owner)
    assert.ok(file.startsWith(`${owner}${path.sep}`), "file escaped its registry installation");
  assert.equal(digest(fs.readFileSync(file)), sha256, file);
  return file;
}

/** Observe the genuine loader return, never infer a load from resolution or an attempt. */
export function verifyNativeJournal(
  journalPath: string,
  authority: VizePublicRegistryInstallAuthority,
  expectedPid?: number,
) {
  const bytes = fs.readFileSync(journalPath);
  assert.ok(
    bytes.length > 0 && bytes.at(-1) === 10,
    "native journal must contain complete records",
  );
  const events = bytes
    .toString("utf8")
    .trimEnd()
    .split("\n")
    .map((line) => JSON.parse(line));
  assert.equal(events[0].event, "initialized");
  const pid = events[0].pid;
  assert.ok(Number.isSafeInteger(pid) && pid > 0);
  if (expectedPid !== undefined)
    assert.equal(pid, expectedPid, "journal must belong to this LSP process");
  assert.equal(events.filter((event) => event.event === "initialized").length, 1);
  assert.equal(events[0].installRoot, authority.installRoot);
  assert.equal(events[0].nativePath, authority.native.path);
  assert.equal(events[0].sha256, authority.native.sha256);
  assert.equal(events[0].node, authority.node.path);
  let returned = 0;
  let attempted = 0;
  for (const event of events) {
    assert.equal(event.schema, "vize-public-native-custody-event-v1");
    assert.equal(event.pid, pid);
    assert.ok(
      ["initialized", "attempt", "failed", "rejected", "returned", "exit"].includes(event.event),
    );
    assert.ok(
      event.event !== "failed" && event.event !== "rejected",
      "failed/rejected native load is not authority",
    );
    if (event.expectedNative === true) {
      assert.equal(event.actualPath, authority.native.path);
      assert.equal(event.sha256, authority.native.sha256);
      if (event.event === "attempt") attempted++;
      if (event.event === "returned") {
        assert.ok(attempted > returned, "a native return must follow its actual load attempt");
        assert.equal(
          event.corsaPath,
          authority.bundledCorsa.path,
          "the CLI must automatically select its public bundled Corsa before loading native",
        );
        returned++;
      }
    }
  }
  assert.ok(returned > 0, "the original expected native loader must actually return successfully");
  const exits = events.filter((event) => event.event === "exit");
  assert.ok(exits.length <= 1);
  if (exits.length) assert.equal(exits[0].code, 0);
  exactFile(authority.native.path, authority.native.sha256, authority.installRoot);
  exactFile(authority.bundledCorsa.path, authority.bundledCorsa.sha256, authority.installRoot);
  return { journalPath, sha256: digest(bytes), pid, events, successfulReturnObserved: true };
}

export function publicEnvironment(
  authority: VizePublicRegistryInstallAuthority,
  journalPath: string,
): NodeJS.ProcessEnv {
  assert.ok(
    path.isAbsolute(journalPath) && !fs.existsSync(journalPath),
    "fresh native journal required",
  );
  const env = { ...process.env };
  for (const key of sourceOverrideKeys) {
    assert.equal(process.env[key] ?? "", "", `source override ${key} must be empty`);
    env[key] = "";
  }
  assert.equal(
    process.env.VIZE_PUBLIC_NATIVE_CUSTODY ?? "",
    "",
    "outer custody configuration cannot be reused",
  );
  env.VIZE_PUBLIC_NATIVE_CUSTODY = JSON.stringify({
    schema: "vize-public-native-custody-v1",
    installRoot: authority.installRoot,
    nativePath: authority.native.path,
    nativeSha256: authority.native.sha256,
    journalPath,
  });
  return env;
}
