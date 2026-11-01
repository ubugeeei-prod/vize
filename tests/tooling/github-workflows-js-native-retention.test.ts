import assert from "node:assert/strict";
import fs from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { mock, test } from "node:test";
import {
  beginProcessObservation,
  finishProcessObservation,
  processPacketLimit,
  processRetention,
  retainProcessPacket,
} from "../../.github/actions/test-js-packages-with-history/native-fault-retention.ts";
import { observeDescendants } from "../../.github/actions/test-js-packages-with-history/native-fault-support.ts";
import {
  main,
  observers,
} from "../../.github/actions/test-js-packages-with-history/native-fault-evidence.ts";

// Synthetic whole packets only; no manager, native addon or generated product is executed.
const packet = (index: number) => ({
  pid: 100,
  startTicks: "123",
  observedAt: 1_700_000_000 + index,
  tasks: [{ tid: 100 + index, tgid: 100, startTicks: String(index) }],
  maps: "the complete synthetic map packet " + index,
});

test("long observations retain the initial whole packet and the latest127 within128 slots", () => {
  const output = fs.mkdtempSync(path.join(tmpdir(), "vize-fault-rolling-"));
  try {
    const seen = new Set<string>();
    for (let index = 0; index < 1000; index++) {
      const observation = beginProcessObservation(seen, "interval");
      retainProcessPacket(seen, String(index), packet(index), output, "interval");
      finishProcessObservation(observation);
      assert.ok(fs.readdirSync(path.join(output, "processes")).length <= processPacketLimit);
      assert.ok(seen.size <= processPacketLimit);
    }
    assert.equal(processPacketLimit, 128);
    assert.deepEqual(
      JSON.parse(fs.readFileSync(path.join(output, "processes/initial.json"), "utf8")),
      packet(0),
    );
    const summary = processRetention(seen);
    assert.equal(summary.observedPackets, 1000);
    assert.equal(summary.retainedPackets, 128);
    assert.equal(summary.droppedPackets, 872);
    assert.deepEqual(
      summary.retained.map((row) => row.sequence),
      [0, ...Array.from({ length: 127 }, (_, index) => index + 873)],
    );
    for (const row of summary.retained)
      assert.deepEqual(
        JSON.parse(fs.readFileSync(path.join(output, "processes", row.filename), "utf8")),
        packet(row.sequence),
      );
    assert.deepEqual(summary.retainedTimeCoverage, {
      first: packet(0).observedAt,
      last: packet(999).observedAt,
      tailFirst: packet(873).observedAt,
      tailLast: packet(999).observedAt,
      complete: false,
    });
    const terminal = beginProcessObservation(seen, "terminal");
    retainProcessPacket(seen, "terminal-state", packet(1000), output, "terminal");
    finishProcessObservation(terminal);
    const final = processRetention(seen);
    assert.equal(final.retainedPackets, 128);
    assert.equal(final.droppedPackets, 873);
    assert.equal(final.retained.at(-1)?.phase, "terminal");
    assert.equal(final.terminalObservation?.packetWrites, 1);
    assert.ok(final.terminalObservation?.finishedAt);
    assert.equal(final.terminalObservation?.partial, true);
    assert.deepEqual(
      JSON.parse(fs.readFileSync(path.join(output, "processes/initial.json"), "utf8")),
      packet(0),
    );
  } finally {
    fs.rmSync(output, { recursive: true, force: true });
  }
});

test("failed replacement and cleanup preserve the old whole packet and first failure", () => {
  const output = fs.mkdtempSync(path.join(tmpdir(), "vize-fault-persistence-"));
  try {
    const seen = new Set<string>();
    for (let index = 0; index < 128; index++)
      retainProcessPacket(seen, String(index), packet(index), output, "interval");
    const before = fs.readFileSync(path.join(output, "processes/tail-001.json"));
    const failure = new Error("synthetic first persistence failure");
    const rename = mock.method(fs, "renameSync", () => {
      throw failure;
    });
    const cleanup = mock.method(fs, "rmSync", () => {
      throw new Error("synthetic cleanup failure");
    });
    try {
      assert.throws(
        () => retainProcessPacket(seen, "next", packet(128), output, "interval"),
        (error) => error === failure,
      );
    } finally {
      rename.mock.restore();
      cleanup.mock.restore();
    }
    assert.deepEqual(fs.readFileSync(path.join(output, "processes/tail-001.json")), before);
    assert.equal(processRetention(seen).observedPackets, 128);
    assert.equal(processRetention(seen).droppedPackets, 0);
    assert.equal(processRetention(seen).persistenceFailures, 1);
    retainProcessPacket(seen, "next", packet(128), output, "interval");
    assert.equal(processRetention(seen).retainedPackets, 128);
    assert.equal(processRetention(seen).droppedPackets, 1);
    assert.deepEqual(
      JSON.parse(fs.readFileSync(path.join(output, "processes/tail-001.json"), "utf8")),
      packet(128),
    );
    assert.equal(fs.existsSync(path.join(output, "processes/.retained-packet.pending")), false);
  } finally {
    fs.rmSync(output, { recursive: true, force: true });
  }
});

test("an absent terminal actor records an explicit partial terminal attempt", () => {
  const output = fs.mkdtempSync(path.join(tmpdir(), "vize-fault-terminal-"));
  const proc = path.join(output, "empty-proc");
  fs.mkdirSync(proc);
  try {
    const seen = new Set<string>();
    observeDescendants(100, seen, new Map(), output, proc, "terminal");
    const summary = JSON.parse(fs.readFileSync(path.join(output, "process-sampling.json"), "utf8"));
    assert.deepEqual(summary.gaps, [
      { pid: 100, identity: "UNKNOWN", unavailable: "ProcessStatUnavailable" },
    ]);
    assert.equal(summary.retention.retainedPackets, 0);
    assert.equal(summary.retention.terminalObservation.packetWrites, 0);
    assert.ok(summary.retention.terminalObservation.finishedAt);
    assert.equal(summary.retention.terminalObservation.partial, true);
    assert.equal(summary.retention.retainedTimeCoverage.complete, false);
  } finally {
    fs.rmSync(output, { recursive: true, force: true });
  }
});

test("early command failure still makes one terminal attempt and keeps its original status", async () => {
  const output = fs.mkdtempSync(path.join(tmpdir(), "vize-fault-early-"));
  const values = {
    GITHUB_ACTIONS: "true",
    GITHUB_RUN_ID: "synthetic-retention",
    GITHUB_RUN_ATTEMPT: "1",
    GITHUB_JOB: "early",
    RUNNER_TEMP: output,
  };
  const previous = Object.fromEntries(Object.keys(values).map((key) => [key, process.env[key]]));
  Object.assign(process.env, values);
  const phases: string[] = [];
  const guest = mock.method(observers, "privateGuest", () => ({ privateFirecrackerGuest: false }));
  const source = mock.method(observers, "sourceSnapshot", () => ({ synthetic: true }));
  const observe = mock.method(
    observers,
    "observeDescendants",
    (...args: Parameters<typeof observeDescendants>) => {
      phases.push(args[5] ?? "interval");
      const error = new Error("synthetic observation persistence refusal");
      error.name = "PermissionError";
      throw error;
    },
  );
  try {
    const calls = path.join(output, "calls.txt");
    const program = `require('node:fs').appendFileSync(process.argv[1], 'once'); process.exit(23);`;
    assert.equal(await main(["--", process.execPath, "-e", program, calls], "linux"), 23);
    assert.equal(fs.readFileSync(calls, "utf8"), "once");
    assert.equal(phases.filter((phase) => phase === "terminal").length, 1);
    assert.equal(phases.at(-1), "terminal");
    const receipt = JSON.parse(
      fs.readFileSync(
        path.join(output, "js-native-fault-evidence-synthetic-retention-1-early/outcome.json"),
        "utf8",
      ),
    );
    assert.equal(receipt.commandReturnCode, 23);
    assert.equal(receipt.observationGaps["process-maps-terminal"], "PermissionError");
    assert.equal(receipt.processRetention.packetLimit, 128);
    assert.equal(receipt.processRetention.retainedPackets, 0);
  } finally {
    guest.mock.restore();
    source.mock.restore();
    observe.mock.restore();
    for (const [key, value] of Object.entries(previous)) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
    fs.rmSync(output, { recursive: true, force: true });
  }
});
