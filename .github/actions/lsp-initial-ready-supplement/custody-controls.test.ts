import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {
  BASE_SOURCE,
  CANDIDATE_SOURCE,
  INTERMEDIATE_SOURCE,
  PREVIOUS_WORKFLOW,
  SOURCE_PATHS,
  TOOLING_PATHS,
  checkAncestry,
  workloadPlan,
  modeEnvironment,
} from "./controller.ts";
import {
  checkAmbientOverrides,
  checkOutcome,
  parentFromCommit,
  verifyCaptures,
  completeFrames,
} from "./source-custody.ts";
type Obj = Record<string, unknown>;
const IMAGE = "a".repeat(64);
const hash = (bytes: Buffer | string): string => createHash("sha256").update(bytes).digest("hex");
const parents = {
  candidate: INTERMEDIATE_SOURCE,
  intermediate: BASE_SOURCE,
  previous: CANDIDATE_SOURCE,
  workflow: PREVIOUS_WORKFLOW,
};
const payload = Buffer.from('{"jsonrpc":"2.0","id":1}');
const frame = Buffer.concat([Buffer.from(`Content-Length: ${payload.length}\r\n\r\n`), payload]);
function capture(directory: string): Obj {
  const wire = { bytes: frame.length, sha256: hash(frame), framing: "complete" };
  return {
    directory,
    observationSha256: IMAGE,
    complete: true,
    sourceRevision: CANDIDATE_SOURCE,
    binarySha256: IMAGE,
    streams: { client: { ...wire }, server: { ...wire }, stderr: { bytes: 0, sha256: hash("") } },
    process: { exitStatus: 0, signal: null, error: null },
  };
}
function outcome(mode: "timing" | "observe" | "memory" = "timing"): Obj {
  const arrivals = mode === "timing" ? [0, 150, 650] : ["beforePrep", "duringPrep", "apiWait"];
  return {
    schemaVersion: 1,
    source: CANDIDATE_SOURCE,
    mode,
    attempt: 2,
    complete: true,
    errors: [],
    scope: "synthetic pure control, no native qualification",
    rows:
      mode === "memory"
        ? [{ milestones: [1, 4, 8, "closed", "reopened8"].map((milestone) => ({ milestone })) }]
        : ["huge4096", "vben2"].flatMap((fixture) =>
            arrivals.map((arrival) => ({ fixture, arrival })),
          ),
    rawCaptures: Array.from({ length: mode === "memory" ? 1 : 6 }, (_, index) =>
      capture(`raw/session-${index}`),
    ),
  };
}
function fixture(run: (directory: string, row: Obj, capture: Obj) => void): void {
  const output = fs.mkdtempSync(path.join(os.tmpdir(), "lsp-supplement-custody-"));
  try {
    const row = outcome("memory");
    const held = (row.rawCaptures as Obj[])[0]!;
    const directory = path.join(output, String(held.directory));
    fs.mkdirSync(directory, { recursive: true });
    const streams: Obj = {};
    for (const name of ["client", "server", "stderr"]) {
      const bytes = name === "stderr" ? Buffer.alloc(0) : frame;
      fs.writeFileSync(path.join(directory, `${name}.bin`), bytes);
      streams[name] = {
        path: `${name}.bin`,
        truncated: false,
        observedBytes: bytes.length,
        capturedBytes: bytes.length,
        sha256: hash(bytes),
        ...(name === "stderr" ? {} : { framing: { state: "complete" } }),
      };
    }
    const observation = {
      state: "process-closed",
      sourceRevision: CANDIDATE_SOURCE,
      binary: { sourceRevision: CANDIDATE_SOURCE, binarySha256: IMAGE },
      buildReceipt: {
        sourceRevision: CANDIDATE_SOURCE,
        binarySha256: IMAGE,
        recipe: "cargo build --release -p vize",
      },
      process: held.process,
      streams,
    };
    const file = path.join(directory, "observation.json");
    fs.writeFileSync(file, JSON.stringify(observation));
    held.observationSha256 = hash(fs.readFileSync(file));
    fs.writeFileSync(path.join(output, "outcome.json"), JSON.stringify(row));
    run(output, row, held);
  } finally {
    fs.rmSync(output, { recursive: true, force: true });
  }
}

test("immutable source ancestry refuses missing, extra, foreign and merge parents", () => {
  checkAncestry(parents, [...SOURCE_PATHS], [...TOOLING_PATHS]);
  for (const key of Object.keys(parents) as (keyof typeof parents)[])
    assert.throws(() =>
      checkAncestry({ ...parents, [key]: "f".repeat(40) }, [...SOURCE_PATHS], [...TOOLING_PATHS]),
    );
  for (const paths of [SOURCE_PATHS.slice(1), [...SOURCE_PATHS, "Cargo.toml"]])
    assert.throws(() => checkAncestry(parents, [...paths], [...TOOLING_PATHS]));
  assert.throws(() => checkAncestry(parents, [...SOURCE_PATHS], TOOLING_PATHS.slice(1)));
  assert.equal(
    parentFromCommit(`tree ${IMAGE.slice(0, 40)}\nparent ${CANDIDATE_SOURCE}\nauthor x\n\nbody`),
    CANDIDATE_SOURCE,
  );
  for (const raw of [`tree x\n\nbody`, `parent ${CANDIDATE_SOURCE}\nparent ${BASE_SOURCE}\n\nbody`])
    assert.throws(() => parentFromCommit(raw));
});
test("timing ABBA precedes every observer and resource arm with no tiny rerun", () => {
  assert.deepEqual(workloadPlan(), [
    { mode: "timing", role: "B", attempt: 1 },
    { mode: "timing", role: "C", attempt: 2 },
    { mode: "timing", role: "C", attempt: 3 },
    { mode: "timing", role: "B", attempt: 4 },
    { mode: "observe", role: "B", attempt: 1 },
    { mode: "observe", role: "C", attempt: 2 },
    { mode: "memory", role: "B", attempt: 1 },
    { mode: "memory", role: "C", attempt: 2 },
  ]);
});
test("nonempty and empty ambient provider, trace and unknown dual overrides refuse", () => {
  checkAmbientOverrides({});
  for (const name of [
    "VIZE_LSP_BIN",
    "CORSA_PATH",
    "CORSA_EXECUTABLE",
    "VIZE_CORSA_PATH",
    "TSGO_PATH",
    "TSGO_EXECUTABLE",
    "VIZE_TSGO_PATH",
    "TSRS_PATH",
    "TSRS_EXECUTABLE",
    "VIZE_TSRS_PATH",
    "VIZE_TRACE_EDITOR_PREPARATION",
    "VIZE_LSP_CURRENT_BASELINE",
    "VIZE_LSP_BUILD_RECIPE",
    "VIZE_LSP_REQUIRE_SOURCE_BUILD",
    "VIZE_TEST_TSGO_PATH",
    "LSP_DUAL_UNKNOWN",
  ])
    for (const value of ["foreign", ""])
      assert.throws(() => checkAmbientOverrides({ [name]: value }));
});
test("finite rows and every closed raw capture must be complete and source bound", () => {
  for (const mode of ["timing", "observe", "memory"] as const)
    checkOutcome(outcome(mode), mode, CANDIDATE_SOURCE, 2);
  for (const key of ["source", "mode", "attempt", "complete", "errors"]) {
    const row = outcome();
    row[key] = key === "complete" ? false : key === "errors" ? ["missing"] : "foreign";
    assert.throws(() => checkOutcome(row, "timing", CANDIDATE_SOURCE, 2));
  }
  for (const key of ["rows", "rawCaptures"]) {
    const row = outcome();
    (row[key] as unknown[]).pop();
    assert.throws(() => checkOutcome(row, "timing", CANDIDATE_SOURCE, 2));
  }
  const duplicate = outcome();
  (duplicate.rows as Obj[])[5] = (duplicate.rows as Obj[])[0]!;
  assert.throws(() => checkOutcome(duplicate, "timing", CANDIDATE_SOURCE, 2));
  const missingMilestone = outcome("memory");
  ((missingMilestone.rows as Obj[])[0]!.milestones as unknown[]).pop();
  assert.throws(() => checkOutcome(missingMilestone, "memory", CANDIDATE_SOURCE, 2));
});
test("unknown, duplicate, escaped, empty-wire and incomplete capture metadata refuse", () => {
  for (const mutate of [
    (c: Obj) => {
      c.complete = false;
    },
    (c: Obj) => {
      c.sourceRevision = BASE_SOURCE;
    },
    (c: Obj) => {
      c.directory = "../outside";
    },
    (c: Obj) => {
      c.directory = "/outside";
    },
    (c: Obj) => {
      c.directory = "raw\\outside";
    },
    (c: Obj) => {
      c.process = { exitStatus: 1, signal: null, error: null };
    },
    (c: Obj) => {
      ((c.streams as Obj).client as Obj).bytes = 0;
    },
    (c: Obj) => {
      ((c.streams as Obj).server as Obj).framing = "partial";
    },
  ]) {
    const row = outcome();
    mutate((row.rawCaptures as Obj[])[0]!);
    assert.throws(() => checkOutcome(row, "timing", CANDIDATE_SOURCE, 2));
  }
  const row = outcome();
  (row.rawCaptures as Obj[])[1]!.directory = (row.rawCaptures as Obj[])[0]!.directory;
  assert.throws(() => checkOutcome(row, "timing", CANDIDATE_SOURCE, 2));
});
test("physical complete capture bytes and source receipt verify independently", () =>
  fixture((output, row) => {
    checkOutcome(row, "memory", CANDIDATE_SOURCE, 2);
    verifyCaptures(output, row, IMAGE);
  }));
test("missing raw directory refuses after the terminal outcome is retained", () =>
  fixture((output, row, held) => {
    fs.rmSync(path.join(output, String(held.directory)), { recursive: true });
    assert.throws(() => verifyCaptures(output, row, IMAGE));
    assert.deepEqual(JSON.parse(fs.readFileSync(path.join(output, "outcome.json"), "utf8")), row);
  }));
test("same-size raw corruption and truncation are not complete", () => {
  for (const bytes of [Buffer.alloc(frame.length, 33), frame.subarray(0, frame.length - 1)])
    fixture((output, row, held) => {
      fs.writeFileSync(path.join(output, String(held.directory), "server.bin"), bytes);
      assert.throws(() => verifyCaptures(output, row, IMAGE));
    });
});
test("foreign image, actual source receipt and truncated observation refuse", () => {
  fixture((output, row) => assert.throws(() => verifyCaptures(output, row, "b".repeat(64))));
  for (const mutate of [
    (x: Obj) => {
      x.sourceRevision = BASE_SOURCE;
    },
    (x: Obj) => {
      ((x.streams as Obj).stderr as Obj).truncated = true;
    },
  ])
    fixture((output, row, held) => {
      const file = path.join(output, String(held.directory), "observation.json");
      const x = JSON.parse(fs.readFileSync(file, "utf8")) as Obj;
      mutate(x);
      fs.writeFileSync(file, JSON.stringify(x));
      held.observationSha256 = hash(fs.readFileSync(file));
      assert.throws(() => verifyCaptures(output, row, IMAGE));
    });
});
test("symlink raw file and oversized sparse file refuse before reading", () => {
  fixture((output, row, held) => {
    const directory = path.join(output, String(held.directory));
    const file = path.join(directory, "client.bin");
    fs.unlinkSync(file);
    fs.symlinkSync("server.bin", file);
    assert.throws(() => verifyCaptures(output, row, IMAGE));
  });
  fixture((output, row, held) => {
    fs.truncateSync(path.join(output, String(held.directory), "client.bin"), 16 * 1024 * 1024 + 1);
    assert.throws(() => verifyCaptures(output, row, IMAGE));
  });
});

test("raw framing refuses partial bodies, duplicate lengths, foreign JSON and empty streams", () => {
  assert.equal(completeFrames(Buffer.concat([frame, frame])), 2);
  for (const bytes of [
    Buffer.alloc(0),
    frame.subarray(0, frame.length - 1),
    Buffer.from("Content-Length: 1\r\nContent-Length: 1\r\n\r\n0"),
    Buffer.from("Content-Length: 2\r\n\r\n[]"),
    Buffer.from("Content-Length: 2\r\n\r\n{}"),
  ])
    assert.throws(() => completeFrames(bytes));
});

test("first timing is trace-free and actual owner/resource observers are explicitly traced", () => {
  assert.equal(
    modeEnvironment({ CORSA_PATH: "owned" }, "timing").VIZE_TRACE_EDITOR_PREPARATION,
    undefined,
  );
  for (const mode of ["observe", "memory"] as const)
    assert.equal(modeEnvironment({}, mode).VIZE_TRACE_EDITOR_PREPARATION, "1");
  for (const value of ["", "0", "1"])
    assert.throws(() => modeEnvironment({ VIZE_TRACE_EDITOR_PREPARATION: value }, "timing"));
});
test("source-correct receipt with missing or foreign FIRST image refuses", () => {
  for (const value of [undefined, "b".repeat(64)])
    fixture((output, row, held) => {
      const file = path.join(output, String(held.directory), "observation.json");
      const observation = JSON.parse(fs.readFileSync(file, "utf8")) as Obj;
      if (value === undefined) delete (observation.buildReceipt as Obj).binarySha256;
      else (observation.buildReceipt as Obj).binarySha256 = value;
      fs.writeFileSync(file, JSON.stringify(observation));
      held.observationSha256 = hash(fs.readFileSync(file));
      assert.throws(() => verifyCaptures(output, row, IMAGE));
    });
});
