import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {
  BASE_SOURCE,
  CANDIDATE_SOURCE,
  PREVIOUS_WORKFLOW,
  SOURCE_PATHS,
  TOOLING_PATHS,
  checkAncestry,
  checkCapacityPaths,
  workloadPlan,
  modeEnvironment,
} from "./controller.ts";
import {
  checkAmbientOverrides,
  checkOutcome,
  parentFromCommit,
  verifyCaptures,
  completeFrames,
  cargoArtifact,
} from "./source-custody.ts";
import { checkSourceRow, decodePin } from "./foreground-source.ts";
import { imageRole } from "./resources.ts";
import { firstPair, type Session } from "./native-queries.ts";
import { hugeSource, hoverPacket, DOC } from "./fixtures.ts";
type Obj = Record<string, unknown>;
const IMAGE = "a".repeat(64);
const hash = (bytes: Buffer | string): string => createHash("sha256").update(bytes).digest("hex");
const parents = {
  candidate: BASE_SOURCE,
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
        : (mode === "timing" ? ["huge4096"] : ["huge4096", "vben2"]).flatMap((fixture) => {
            const arrivals =
              mode === "timing"
                ? [0]
                : ["beforePrep", "duringPrep", fixture === "vben2" ? "collectorWait" : "apiWait"];
            return arrivals.map((arrival) => ({ fixture, arrival }));
          }),
    rawCaptures: Array.from(
      { length: mode === "memory" ? 1 : mode === "timing" ? 1 : 6 },
      (_, index) => capture(`raw/session-${index}`),
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
  checkCapacityPaths("/runner/temp", "/runner/temp");
  for (const [actual, expected] of [
    ["/checkout", "/runner/temp"],
    ["relative", "relative"],
  ] as const)
    assert.throws(() => checkCapacityPaths(actual, expected), assert.AssertionError);
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
test("focused timing has exactly four ABBA attempts and four Huge sessions without any other arm", () => {
  assert.deepEqual(workloadPlan(), [
    { mode: "timing", role: "B", attempt: 1 },
    { mode: "timing", role: "C", attempt: 2 },
    { mode: "timing", role: "C", attempt: 3 },
    { mode: "timing", role: "B", attempt: 4 },
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
  (duplicate.rows as Obj[]).push((duplicate.rows as Obj[])[0]!);
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
  (row.rawCaptures as Obj[]).push(structuredClone((row.rawCaptures as Obj[])[0]!));
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

test("complete reviewed source row refuses absent, foreign mode/blob and same-size changed bytes", () => {
  const body = Buffer.from("source bytes\n");
  const gitBlob = createHash("sha1")
    .update(Buffer.from(`blob ${body.length}\0`))
    .update(body)
    .digest("hex");
  const expected = {
    mode: "100644",
    type: "blob",
    gitBlob,
    bytes: body.length,
    sha256: hash(body),
  };
  const actual = { mode: "100644", type: "blob", gitBlob, body };
  checkSourceRow(actual, expected);
  checkSourceRow(null, null);
  assert.throws(() => checkSourceRow(null, expected));
  assert.throws(() => checkSourceRow(actual, null));
  for (const changed of [
    { ...actual, mode: "100755" },
    { ...actual, type: "tree" },
    { ...actual, gitBlob: "b".repeat(40) },
    { ...actual, body: Buffer.alloc(body.length, 33) },
    { ...actual, body: body.subarray(1) },
  ])
    assert.throws(() => checkSourceRow(changed, expected));
});
test("focused timing refuses delayed arrivals, extra cells and omitted fixture", () => {
  const valid = outcome();
  checkOutcome(valid, "timing", CANDIDATE_SOURCE, 2);
  for (const arrival of [150, 650, "beforePrep"]) {
    const row = outcome();
    (row.rows as Obj[])[0]!.arrival = arrival;
    assert.throws(() => checkOutcome(row, "timing", CANDIDATE_SOURCE, 2));
  }
  const extra = outcome();
  (extra.rows as Obj[]).push({ fixture: "huge4096", arrival: 150 });
  assert.throws(() => checkOutcome(extra, "timing", CANDIDATE_SOURCE, 2));
});

test("source-derived one-separator whole hover still refuses unknown fields, wrong range and two separators", async () => {
  const uri = "file:///fixture/App.vue";
  const target = {
    uri,
    file: "/fixture/App.vue",
    invalid: hugeSource(false),
    repair: hugeSource(true),
  };
  const expected = hoverPacket(target.invalid, false);
  const completion = [
    {
      label: "invoiceTotal",
      kind: 5,
      sortText: "11",
      data: {
        vizeCompletion: {
          uri,
          revision: 1,
          requestUri: "opaque-native-input",
          item: { label: "invoiceTotal", kind: 5, sortText: "11", data: {} },
        },
      },
    },
  ];
  const session = (hover: unknown): Session =>
    ({
      request: async (method: string) => (method === "textDocument/hover" ? hover : completion),
    }) as unknown as Session;
  await firstPair(session(expected), target, false, () => {});
  for (const hover of [
    { ...expected, unexpected: true },
    { ...expected, range: { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } } },
    {
      ...expected,
      contents: {
        kind: "markdown",
        value: "```typescript\n(property) invoiceTotal: number\n```\n\n" + DOC,
      },
    },
    {
      ...expected,
      contents: { kind: "markdown", value: "```typescript\n(property) invoiceTotal: number\n```" },
    },
    {
      ...expected,
      contents: {
        kind: "markdown",
        value: "```typescript\n(property) invoiceTotal: string\n```\n" + DOC,
      },
    },
  ])
    await assert.rejects(
      () => firstPair(session(hover), target, false, () => {}),
      assert.AssertionError,
    );
});

test("encoded immutable source pins refuse malformed, empty and unsafe lengths", () => {
  const pin = decodePin(`${"a".repeat(40)}:12:${IMAGE}`);
  assert.deepEqual(pin, {
    mode: "100644",
    type: "blob",
    gitBlob: "a".repeat(40),
    bytes: 12,
    sha256: IMAGE,
  });
  assert.equal(decodePin(null), null);
  for (const value of [
    "",
    `${"a".repeat(40)}:0:${IMAGE}`,
    `${"a".repeat(40)}:9007199254740992:${IMAGE}`,
    `${"b".repeat(39)}:12:${IMAGE}`,
  ])
    assert.throws(() => decodePin(value), assert.AssertionError);
});
test("Huge timing cannot silently retain a Vben case or extra native session", () => {
  const row = outcome();
  (row.rows as Obj[])[0]!.fixture = "vben2";
  assert.throws(() => checkOutcome(row, "timing", CANDIDATE_SOURCE, 2), assert.AssertionError);
  const extra = outcome();
  (extra.rawCaptures as Obj[]).push(capture("raw/extra"));
  assert.throws(() => checkOutcome(extra, "timing", CANDIDATE_SOURCE, 2), assert.AssertionError);
});

test("native image joins require birth-bound physical inode and whole accepted image; early unknown never becomes native", () => {
  const first = { dev: "1", ino: "2", sha256: IMAGE };
  const native = { dev: "1", ino: "3", sha256: "b".repeat(64) };
  assert.equal(imageRole(first, first, native), "FIRST-source-CLI");
  assert.equal(imageRole(native, first, native), "accepted-native-provider");
  for (const actual of [
    {},
    { ...native, ino: "4" },
    { ...native, sha256: IMAGE },
    { ...native, dev: "2" },
  ])
    assert.equal(imageRole(actual, first, native), "UNKNOWN");
});

test("shipping CLI defaults do not require native while Canon and Maestro dependencies do", () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "huge-cargo-control-"));
  const file = path.join(dir, "cargo.jsonl"),
    executable = "/target/release/vize";
  const rows: Obj[] = [
    {
      reason: "compiler-artifact",
      target: { name: "vize", kind: ["bin"] },
      executable,
      profile: { test: false, opt_level: "3", debug_assertions: false },
      features: ["default", "glyph", "maestro"],
    },
    ...["vize_canon", "vize_maestro"].map((name) => ({
      reason: "compiler-artifact",
      target: { name },
      features: ["native"],
    })),
    ...[
      "corsa",
      "corsa_client",
      "corsa_core",
      "corsa_jsonrpc",
      "corsa_lsp",
      "corsa_orchestrator",
      "corsa_runtime",
    ].map((name) => ({
      reason: "compiler-artifact",
      target: { name, kind: ["lib"] },
      package_id: `registry+index#${name}@2.0.0-beta.4`,
      features: [],
    })),
    { reason: "build-finished", success: true },
  ];
  const check = (values: Obj[]): void => {
    fs.writeFileSync(file, values.map((row) => JSON.stringify(row)).join("\n") + "\n");
    cargoArtifact(file, executable);
  };
  try {
    check(rows);
    for (const index of [1, 2]) {
      const mutant = structuredClone(rows);
      mutant[index]!.features = [];
      assert.throws(() => check(mutant), assert.AssertionError);
    }
    const wrong = structuredClone(rows);
    wrong[0]!.features = ["default", "glyph", "maestro", "native"];
    assert.throws(() => check(wrong), assert.AssertionError);
  } finally {
    fs.rmSync(dir, { recursive: true });
  }
});
