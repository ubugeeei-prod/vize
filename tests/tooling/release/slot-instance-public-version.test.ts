import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { sha256 } from "../../../tools/support/release/computed_event_custody.ts";
import { verifyJournal } from "../../../tools/support/release/editor_vite_stub/custody.ts";
import type { capture } from "../../../tools/support/release/slot_instance_public/process.ts";
import {
  verifyInstalledVersion,
  type VersionContext,
} from "../../../tools/support/release/slot_instance_public/version-custody.ts";

const file = fileURLToPath(
  new URL("../../fixtures/release/slot-instance-public/c12-version-exit.json", import.meta.url),
);
const bytes = readFileSync(file);
assert.equal(sha256(bytes), "0a69f7cfb8aeeeadc5ab5f745cc82f050d708c9c02f61c044a3bbe4300a2e677");
const fixture = JSON.parse(bytes.toString("utf8")) as {
  role: "before";
  context: VersionContext;
  packet: ReturnType<typeof capture>;
  journal: string;
  journalSha256: string;
  semanticProcessesReached: number;
  headRoleReached: boolean;
};
const events = () =>
  fixture.journal
    .trimEnd()
    .split("\n")
    .map((row) => JSON.parse(row) as Record<string, unknown>);
function withJournal(raw: string, check: (path: string) => void) {
  const root = mkdtempSync(join(tmpdir(), "slot-version-journal-"));
  try {
    const path = join(root, "native.ndjson");
    writeFileSync(path, raw);
    check(path);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
function validate(packet = fixture.packet, raw = fixture.journal, context = fixture.context) {
  let result: ReturnType<typeof verifyInstalledVersion> | undefined;
  withJournal(raw, (path) => {
    result = verifyInstalledVersion("before", path, packet, context);
  });
  return result!;
}

test("the retained C12 native --version outcome owns exactly three events and a known OS exit0", () => {
  assert.equal(sha256(fixture.journal), fixture.journalSha256);
  assert.equal(fixture.packet.pid, 3864);
  assert.equal(fixture.packet.stdout, "vize 0.620.0\n");
  assert.deepEqual(
    events().map((value) => value.event),
    ["initialized", "attempt", "returned"],
  );
  assert.equal(fixture.semanticProcessesReached, 0);
  assert.equal(fixture.headRoleReached, false);
  const result = validate();
  assert.equal(result.pid, fixture.packet.pid);
  assert.equal(result.processExitStatus, 0);
  assert.equal(result.nodeExitCallbacks, 0);
  assert.equal(result.events, 3);
  assert.deepEqual(result.nativeImage, fixture.context.binary);
  withJournal(fixture.journal, (path) => {
    assert.throws(() =>
      verifyJournal(path, { binary: fixture.context.binary }, fixture.packet.status),
    );
    verifyJournal(path, { binary: fixture.context.binary }, null);
    assert.throws(() => verifyInstalledVersion("head", path, fixture.packet, fixture.context));
  });
});

test("only the exact version command/PID/status/raw outcome uses this role law", () => {
  for (const mutate of [
    (value: ReturnType<typeof capture>) => {
      value.label = "diagnostic-cli";
    },
    (value: ReturnType<typeof capture>) => {
      value.argv = [...value.argv.slice(0, -1), "check", "App.vue"];
    },
    (value: ReturnType<typeof capture>) => {
      value.argv[value.argv.length - 1] = "--help";
    },
    (value: ReturnType<typeof capture>) => {
      value.argv.push("extra");
    },
    (value: ReturnType<typeof capture>) => {
      value.cwd += "/foreign";
    },
    (value: ReturnType<typeof capture>) => {
      value.pid++;
    },
    (value: ReturnType<typeof capture>) => {
      value.pid = 0;
    },
    (value: ReturnType<typeof capture>) => {
      value.status = 1;
    },
    (value: ReturnType<typeof capture>) => {
      value.status = null;
    },
    (value: ReturnType<typeof capture>) => {
      value.signal = "SIGTERM";
    },
    (value: ReturnType<typeof capture>) => {
      value.error = "timeout";
    },
    (value: ReturnType<typeof capture>) => {
      value.stdout = "vize 0.621.0\n";
    },
    (value: ReturnType<typeof capture>) => {
      value.stdout += "extra\n";
    },
    (value: ReturnType<typeof capture>) => {
      value.stderr = "warning";
    },
    (value: ReturnType<typeof capture>) => {
      value.raw.stdout.sha256 = "0".repeat(64);
    },
    (value: ReturnType<typeof capture>) => {
      value.raw.stderr.bytes = 1;
    },
  ]) {
    const value = structuredClone(fixture.packet);
    mutate(value);
    assert.throws(() => validate(value));
  }
  for (const mutate of [
    (value: VersionContext) => {
      value.root += "/foreign";
    },
    (value: VersionContext) => {
      value.cli += ".foreign";
    },
    (value: VersionContext) => {
      value.node += ".foreign";
    },
    (value: VersionContext) => {
      value.hook += ".foreign";
    },
    (value: VersionContext) => {
      value.binary.sha256 = "0".repeat(64);
    },
    (value: VersionContext) => {
      value.binary.path += ".foreign";
    },
  ]) {
    const context = structuredClone(fixture.context);
    mutate(context);
    assert.throws(() => validate(fixture.packet, fixture.journal, context));
  }
});

test("the complete native load sequence refuses missing, extra, foreign and malformed journal evidence", () => {
  for (const mutate of [
    (rows: Record<string, unknown>[]) => rows.pop(),
    (rows: Record<string, unknown>[]) => rows.shift(),
    (rows: Record<string, unknown>[]) => rows.reverse(),
    (rows: Record<string, unknown>[]) => rows.push(rows[2]!),
    (rows: Record<string, unknown>[]) => rows.push({ ...rows[0], event: "exit", code: 0 }),
    (rows: Record<string, unknown>[]) => {
      rows[1]!.pid = 999;
    },
    (rows: Record<string, unknown>[]) => {
      rows[2]!.schema = "foreign";
    },
    (rows: Record<string, unknown>[]) => {
      rows[0]!.installRoot = "/foreign";
    },
    (rows: Record<string, unknown>[]) => {
      rows[0]!.argv = ["node", "--version"];
    },
    (rows: Record<string, unknown>[]) => {
      rows[0]!.node = "/foreign/node";
    },
    (rows: Record<string, unknown>[]) => {
      rows[1]!.actualPath = "/foreign.node";
    },
    (rows: Record<string, unknown>[]) => {
      rows[1]!.expectedNative = false;
    },
    (rows: Record<string, unknown>[]) => {
      rows[2]!.sha256 = "0".repeat(64);
    },
    (rows: Record<string, unknown>[]) => {
      rows[1]!.corsaPath = "/foreign/tsc";
    },
    (rows: Record<string, unknown>[]) => {
      rows[2]!.extra = true;
    },
  ]) {
    const rows = events();
    mutate(rows);
    assert.throws(() =>
      validate(fixture.packet, rows.map((value) => JSON.stringify(value)).join("\n") + "\n"),
    );
  }
  assert.throws(() => validate(fixture.packet, fixture.journal.trimEnd()));
  assert.throws(() => validate(fixture.packet, fixture.journal + "\n"));
});

test("shared diagnostic/API journal guards continue requiring actual Node exit callbacks", () => {
  for (const code of [0, 2]) {
    const rows = events();
    rows.push({ schema: rows[0]!.schema, pid: fixture.packet.pid, event: "exit", code });
    const raw = rows.map((value) => JSON.stringify(value)).join("\n") + "\n";
    withJournal(raw, (path) => {
      verifyJournal(path, { binary: fixture.context.binary }, code);
      assert.throws(() => verifyJournal(path, { binary: fixture.context.binary }, null));
      assert.throws(() => verifyInstalledVersion("before", path, fixture.packet, fixture.context));
    });
  }
});
