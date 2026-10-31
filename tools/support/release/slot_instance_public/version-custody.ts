import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { isAbsolute, join } from "node:path";
import { asRecord, parseJson } from "../../json-value.ts";
import { inside, sha256, type FileImage } from "../computed_event_custody.ts";
import { verifyJournal } from "../editor_vite_stub/custody.ts";
import { profiles, type Role } from "./contract.ts";
import { assertJournalPid, type capture } from "./process.ts";

export type VersionContext = {
  root: string;
  cli: string;
  node: string;
  hook: string;
  binary: FileImage;
};

/**
 * Both closed release roles route only --version through native Clap4.6.7
 * ArgAction::Version -> Error::exit -> std::process::exit(0). That OS exit does
 * not supply a Node exit callback. The process outcome remains independently
 * mandatory; null below means zero Node callbacks, never an unknown OS status.
 * The caller has already authenticated this native image against its capsule.
 */
export function verifyInstalledVersion(
  role: Role,
  journal: string,
  packet: ReturnType<typeof capture>,
  context: VersionContext,
) {
  const { root, cli, node, hook, binary } = context;
  for (const file of [root, cli, node, hook, binary.path]) assert(isAbsolute(file));
  assert.equal(cli, join(root, "node_modules", "vize", "bin", "vize"));
  assert(inside(root, binary.path) && binary.path.endsWith(".node"));
  assert(!inside(root, hook), "observer hook stays outside installed payloads");
  assert(Number.isSafeInteger(binary.bytes) && binary.bytes > 0);
  assert.match(binary.sha256, /^[a-f0-9]{64}$/u);
  const stdout = `vize ${profiles[role].version}\n`;
  assert(Number.isSafeInteger(packet.pid) && packet.pid > 0);
  assert.deepEqual(
    packet,
    {
      label: "installed-cli-version",
      argv: [node, "--require", hook, cli, "--version"],
      cwd: root,
      pid: packet.pid,
      status: 0,
      signal: null,
      error: null,
      stdout,
      stderr: "",
      raw: {
        stdout: { bytes: Buffer.byteLength(stdout), sha256: sha256(stdout) },
        stderr: { bytes: 0, sha256: sha256("") },
      },
    },
    "only the exact installed release --version process uses this exit law",
  );
  const raw = readFileSync(journal, "utf8");
  assertJournalPid(raw, packet.pid);
  verifyJournal(journal, { binary }, null);
  const schema = "vize-public-native-custody-event-v1",
    pid = packet.pid;
  assert.deepEqual(
    raw
      .slice(0, -1)
      .split("\n")
      .map((line) => asRecord(parseJson(line))),
    [
      {
        schema,
        pid,
        event: "initialized",
        installRoot: root,
        nativePath: binary.path,
        sha256: binary.sha256,
        node,
        argv: [node, cli, "--version"],
      },
      {
        schema,
        pid,
        event: "attempt",
        actualPath: binary.path,
        sha256: binary.sha256,
        expectedNative: true,
        corsaPath: "",
      },
      {
        schema,
        pid,
        event: "returned",
        actualPath: binary.path,
        sha256: binary.sha256,
        expectedNative: true,
        corsaPath: "",
      },
    ],
    "the complete version journal is exactly one selected native load, without synthetic exits",
  );
  return {
    role,
    version: profiles[role].version,
    pid,
    processExitStatus: packet.status,
    nodeExitCallbacks: 0,
    events: 3,
    nativeImage: binary,
    scope: "installed --version only; no diagnostic or API credit",
  };
}
