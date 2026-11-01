import assert from "node:assert/strict";
import { test } from "node:test";
import {
  stableWatchActor,
  observeStableSplitWatchLog,
} from "./support/native-beta-dto/stable-watcher-boundary.ts";
import { parseNativeIo } from "./support/native-beta-dto/native-io.ts";
import {
  at,
  fixture,
  inputs,
  observe,
  exitBoundFixture,
} from "./support/native-beta-dto/stable-watcher-mixed-fixture.ts";
test("same-thread terminal exit entry bounds EOF without a point-return or qualification claim", () => {
  const f = exitBoundFixture(),
    result = observe(f);
  assert.ok("eofReturnUpperBoundExitRequest" in result);
  assert.deepEqual(result.eofReturnUpperBoundExitRequest, f.owner.exitGroups[0]);
  assert.equal(result.nativeStdinEof.completion!.returnedAt, null);
  assert.deepEqual(
    result.stderrWrites.map((row) => ("eofReturnOrder" in row ? row.eofReturnOrder : null)),
    ["UNPROVEN", "UNPROVEN", "UNPROVEN", "UNPROVEN"],
  );
  for (const key of [
    "nativeQualification",
    "candidateFunctionalPass",
    "lifecyclePass",
    "registrationPass",
  ] as const)
    assert.equal(result[key], false);
  const value = inputs(f);
  assert.throws(() => stableWatchActor(value.client, value.server, f.context, value.io, f.eof));
});
test("wrong or missing exit owner, line, entry, result and attempted-write provenance refuse", () => {
  const changes: Array<(f: ReturnType<typeof fixture>) => void> = [
    (f) => {
      f.owner.exitGroups[0]!.tid = 43;
    },
    (f) => {
      f.owner.exitGroups[0]!.tid = 10;
    },
    (f) => {
      f.owner.exitGroups[0]!.line = 18;
    },
    (f) => {
      delete f.owner.exitGroups[0]!.line;
    },
    (f) => {
      f.owner.exitGroups[0]!.at = at(17);
    },
    (f) => {
      f.owner.exitGroups[0]!.code = 1;
    },
    (f) => {
      f.owner.exitGroups[0]!.rawResult = "-1 EINTR";
    },
    (f) => {
      f.owner.exitGroups[0]!.rawResult = null;
    },
    (f) => {
      f.owner.exitGroups = [];
    },
    (f) => {
      f.owner.exitGroups.push({ ...f.owner.exitGroups[0]! });
    },
    (f) => {
      f.eof[0]!.completion!.returnedBy!.line = 12;
    },
    (f) => {
      f.eof[0]!.completion!.returnedAt = at(16);
    },
    (f) => {
      f.context.role = "after";
    },
    (f) => {
      f.context.source = "0".repeat(40);
    },
    (f) => {
      f.context.sdkSha256 = "0".repeat(64);
    },
    (f) => {
      const write = parseNativeIo('1793423752.000010 write(2, "\\x61", 1) = ?')!;
      assert.equal(write.outcome, "unknown");
      f.owner.unknownWrites.push({ ...write, tid: 43, line: 29, file: "syscalls.43" });
    },
    (f) => {
      const write = parseNativeIo('1793423752.000010 write(2, "\\x61", 1) = -1 EAGAIN')!;
      assert.equal(write.outcome, "failed");
      f.owner.failedWrites.push({ ...write, tid: 43, line: 29, file: "syscalls.43" });
    },
  ];
  for (const change of changes) {
    const f = exitBoundFixture();
    change(f);
    assert.throws(() => observe(f));
  }
});
test("mixed return intervals are explicit observations with no qualification credit", () => {
  const f = fixture(),
    result = observe(f);
  assert.equal(result.schema, "vize.nativeStableWatcher.unprovenEofReturnObservation.v1");
  assert.deepEqual(
    result.stderrWrites.map((row) => ("eofReturnOrder" in row ? row.eofReturnOrder : null)),
    ["UNPROVEN", "UNPROVEN", "PROVEN", "PROVEN"],
  );
  for (const key of [
    "nativeQualification",
    "candidateFunctionalPass",
    "lifecyclePass",
    "registrationPass",
  ] as const)
    assert.equal(result[key], false);
  assert.equal(result.nativeStdinEof.completion!.returnedAt, null);
  assert.equal(
    Buffer.concat(result.stderrWrites.map((row) => Buffer.from(row.hex, "hex"))).length,
    182,
  );
  const value = inputs(f);
  assert.throws(() => stableWatchActor(value.client, value.server, f.context, value.io, f.eof));
  assert.throws(() =>
    observeStableSplitWatchLog(value.client, value.server, f.context, value.io, f.eof),
  );
});
test("fully proven strict witness retains its exact previous result and failed retry policy", () => {
  const f = fixture();
  f.eof[0]!.completion = { returnedAt: at(9), returnedBy: null };
  const failed = parseNativeIo('1793423752.000001 write(2, "\\x61", 1) = -1 EAGAIN')!;
  assert.equal(failed.outcome, "failed");
  f.owner.failedWrites.push({ ...failed, tid: 43, file: "syscalls.43", line: 1 });
  const value = inputs(f);
  assert.deepEqual(
    observe(f),
    observeStableSplitWatchLog(value.client, value.server, f.context, value.io, f.eof),
  );
});
test("wrong actors, bracket claims, incomplete frames or extra payload cannot be observations", () => {
  const cases: Array<(f: ReturnType<typeof fixture>) => void> = [
    (f) => {
      f.context.role = "after";
    },
    (f) => {
      f.context.source = "0".repeat(40);
    },
    (f) => {
      f.context.sdkVersion = "7.1.0-beta";
    },
    (f) => {
      f.context.sdkSha256 = "0".repeat(64);
    },
    (f) => {
      f.context.sdkExecutable = "/other/tsc";
    },
    (f) => {
      f.context.projectRoots = [];
    },
    (f) => {
      f.owner.nativeOwner = 99;
    },
    (f) => {
      f.owner.members.pop();
    },
    (f) => {
      f.owner.threadExits.pop();
    },
    (f) => {
      f.owner.threadExits[0]!.code = 1;
    },
    (f) => {
      f.eof[0]!.tid = 999;
    },
    (f) => {
      f.eof[0]!.fd = 7;
    },
    (f) => {
      f.eof[0]!.completion!.returnedAt = at(16);
    },
    (f) => {
      f.eof[0]!.completion!.returnedBy = null;
    },
    (f) => {
      f.eof[0]!.completion!.returnedBy!.basis = "explicit-resume";
    },
    (f) => {
      f.eof[0]!.completion!.returnedBy!.at = at(17);
    },
    (f) => {
      f.eof[0]!.completion!.returnedBy!.line = 15;
    },
    (f) => {
      f.parts.push("\n");
    },
    (f) => {
      f.parts[0] = "\t/tmp/foreign/**/*\n";
    },
    (f) => {
      f.client.pop();
    },
    (f) => {
      f.server.pop();
    },
    (f) => {
      f.server[4]!.packet.params = { type: 3, message: "wrong header" };
    },
  ];
  for (const change of cases) {
    const f = fixture();
    change(f);
    assert.throws(() => observe(f));
  }
  assert.throws(() =>
    observe(fixture(), (io) => {
      io.find((row) => row.fd === 2)!.at = at(8);
    }),
  );
  assert.throws(() =>
    observe(fixture(), (io) => {
      io.find((row) => row.fd === 2)!.tid = 999;
    }),
  );
  assert.throws(() =>
    observe(fixture(), (io) => {
      io.find((row) => row.fd === 1)!.data = Buffer.from("Content-Length: 99\r\n\r\n{}");
    }),
  );
});
test("actual unknown and failed write-attempt metadata refuses only the mixed observation", () => {
  for (const result of ["?", "-1 EAGAIN"]) {
    const f = fixture(),
      write = parseNativeIo(`1793423752.000010 write(2, "\\x61", 1) = ${result}`)!;
    if (write.outcome === "unknown")
      f.owner.unknownWrites.push({ ...write, tid: 43, file: "syscalls.43", line: 29 });
    else {
      assert.equal(write.outcome, "failed");
      f.owner.failedWrites.push({ ...write, tid: 43, file: "syscalls.43", line: 29 });
    }
    assert.throws(() => observe(f));
  }
});

function threadExitFixture() {
  const f = exitBoundFixture();
  f.owner.exitGroups = [];
  const exit = f.owner.threadExits.find((row) => row.tid === 38)!;
  Object.assign(exit, { at: at(16), line: 17 });
  return f;
}
test("nonfragment EOF terminal thread exit retains all unknown relations and zero credits", () => {
  const f = threadExitFixture(),
    result = observe(f);
  assert.ok("eofReturnUpperBoundThreadExit" in result);
  assert.deepEqual(result.eofReturnUpperBoundThreadExit, f.owner.threadExits[1]);
  assert.equal(result.nativeStdinEof.completion!.returnedAt, null);
  assert.deepEqual(
    result.stderrWrites.map((row) => ("eofReturnOrder" in row ? row.eofReturnOrder : null)),
    Array(4).fill("UNPROVEN"),
  );
  for (const key of [
    "nativeQualification",
    "candidateFunctionalPass",
    "lifecyclePass",
    "registrationPass",
  ] as const)
    assert.equal(result[key], false);
  const value = inputs(f);
  assert.throws(() => stableWatchActor(value.client, value.server, f.context, value.io, f.eof));
  assert.throws(() =>
    observeStableSplitWatchLog(value.client, value.server, f.context, value.io, f.eof),
  );
});
test("thread-exit provenance refuses missing, duplicate, foreign, reversed and incomplete evidence", () => {
  const cases: Array<(f: ReturnType<typeof fixture>) => void> = [
    (f) => {
      f.owner.threadExits[1]!.tid = 52;
    },
    (f) => {
      f.owner.threadExits[1]!.code = 1;
    },
    (f) => {
      f.owner.threadExits[1]!.at = at(17);
    },
    (f) => {
      f.owner.threadExits[1]!.line = 18;
    },
    (f) => {
      delete f.owner.threadExits[1]!.line;
    },
    (f) => {
      f.owner.threadExits.splice(1, 1);
    },
    (f) => {
      f.owner.threadExits.push({ ...f.owner.threadExits[1]! });
    },
    (f) => {
      f.owner.members.pop();
    },
    (f) => {
      f.eof[0]!.completion!.returnedAt = at(16);
    },
    (f) => {
      f.eof[0]!.completion!.returnedBy = null;
    },
    (f) => {
      f.eof[0]!.completion!.returnedBy!.line = 12;
    },
    (f) => {
      f.eof[0]!.completion!.returnedBy!.at = at(8);
    },
    (f) => {
      f.context.role = "after";
    },
    (f) => {
      f.context.source = "0".repeat(40);
    },
    (f) => {
      f.context.sdkSha256 = "0".repeat(64);
    },
    (f) => {
      f.client.pop();
    },
    (f) => {
      f.server.pop();
    },
    (f) => {
      f.parts[0] = "foreign";
    },
    (f) => {
      const write = parseNativeIo('1793423752.000010 write(2, "\\x61", 1) = ?');
      assert.ok(write && write.outcome === "unknown");
      f.owner.unknownWrites.push({ ...write, tid: 43, line: 29, file: "syscalls.43" });
    },
  ];
  for (const change of cases) {
    const f = threadExitFixture();
    change(f);
    assert.throws(() => observe(f));
  }
  assert.throws(() =>
    observe(threadExitFixture(), (io) => {
      io.find((row) => row.fd === 2)!.at = at(8);
    }),
  );
  assert.throws(() =>
    observe(threadExitFixture(), (io) => {
      io.find((row) => row.fd === 2)!.at = at(20);
    }),
  );
});

test("same-reader split root ACK retains terminal EOF uncertainty without widening strict proof", () => {
  const f = threadExitFixture();
  const result = observe(f, (io) => {
    const index = io.findIndex((row) => row.op === "read" && row.line === 12);
    const row = io[index]!;
    const end = Buffer.from(row.data).indexOf("\r\n\r\n") + 4;
    assert.ok(end > 4);
    io.splice(
      index,
      1,
      { ...row, line: 11, data: Buffer.from(row.data).subarray(0, end) },
      { ...row, data: Buffer.from(row.data).subarray(end) },
    );
  });
  assert.equal(result.watcherAck.parts.length, 2);
  assert.ok("eofReturnUpperBoundThreadExit" in result);
  assert.deepEqual(result.eofReturnUpperBoundThreadExit, f.owner.threadExits[1]);
  assert.equal("rootAckFragmentOrder" in result, false);
  assert.deepEqual(
    result.stderrWrites.map((row) => ("eofReturnOrder" in row ? row.eofReturnOrder : null)),
    Array(4).fill("UNPROVEN"),
  );
  for (const key of [
    "nativeQualification",
    "candidateFunctionalPass",
    "lifecyclePass",
    "registrationPass",
  ] as const)
    assert.equal(result[key], false);
  assert.throws(() =>
    observe(threadExitFixture(), (io) => {
      io.find((row) => row.op === "read" && row.line === 12)!.tid = 52;
    }),
  );
});
