import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {
  PROVIDER_SHA,
  INPUT_ROOT,
  readPacketInputs,
  TEST_NAME,
  TEST_SUFFIX,
  testExecutable,
  verifyAppendOnly,
  verifyPackets,
} from "./patina-plain-type-packets.mjs";

void test("baseline overlay preserves original test bytes and appends one natural module", () => {
  const original = Buffer.from("fn original_test() {}\n");
  const reviewed = Buffer.concat([original, Buffer.from(TEST_SUFFIX)]);
  assert.doesNotThrow(() => verifyAppendOnly(original, reviewed));
  for (const modified of [
    Buffer.from("fn changed_test() {}\n" + TEST_SUFFIX),
    Buffer.concat([reviewed, Buffer.from("fn extra() {}\n")]),
    original,
  ]) {
    assert.throws(() => verifyAppendOnly(original, modified));
  }
  assert.equal(TEST_SUFFIX, "\nmod packets;\n");
  assert.match(PROVIDER_SHA, /^[a-f0-9]{64}$/u);
});

void test("Cargo executable authority requires exactly one real Patina test artifact", () => {
  const artifact = {
    reason: "compiler-artifact",
    target: { name: "vize_patina" },
    profile: { test: true },
    executable: "/exact/source/target/patina-test",
  };
  const line = JSON.stringify(artifact);
  assert.equal(
    testExecutable(`${JSON.stringify({ reason: "build-finished", success: true })}\n${line}\n`),
    artifact.executable,
  );
  for (const bytes of [
    "",
    `${line}\n${line}`,
    JSON.stringify({ ...artifact, profile: { test: false } }),
    JSON.stringify({ ...artifact, target: { name: "other" } }),
  ]) {
    assert.throws(() => testExecutable(bytes), /one source-built/u);
  }
  assert.throws(() => testExecutable("truncated compiler JSON"));
});

void test("native source reads stay tied to the new pinned authored inputs", () => {
  const repository = path.resolve(new URL("../../../", import.meta.url).pathname);
  const inputs = readPacketInputs(repository);
  assert.equal(inputs.size, 13);
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "patina-packet-pins-"));
  try {
    fs.mkdirSync(path.join(root, INPUT_ROOT), { recursive: true });
    fs.copyFileSync(
      path.join(repository, "tests/_fixtures/differential/linter/manifest.json"),
      path.join(root, "tests/_fixtures/differential/linter/manifest.json"),
    );
    for (const name of inputs.keys())
      fs.copyFileSync(path.join(repository, INPUT_ROOT, name), path.join(root, INPUT_ROOT, name));
    fs.appendFileSync(path.join(root, INPUT_ROOT, "promise.tsx.txt"), "// changed\n");
    assert.throws(() => readPacketInputs(root), /pinned packet input changed/u);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
  const document = packets();
  assert.throws(
    () => verifyPackets(JSON.stringify(document), inputs),
    /replayed source must match pinned/u,
  );
});

function packets() {
  const cases = [
    "active",
    "javascript",
    "declaration-ts",
    "declaration-mts",
    "declaration-cts",
    "default",
    "host-disabled",
    "rule-disabled",
    "unavailable",
    "no-query",
  ];
  const rows = ["promise", "reactivity"].flatMap((kind) =>
    cases.map((name) => {
      const rule =
        name === "active"
          ? `type/${kind === "promise" ? "no-floating-promises" : "no-reactivity-loss"}`
          : name === "unavailable"
            ? "type/corsa-runtime"
            : undefined;
      const result = {
        filename: `/same/cwd/${kind}-${name}.tsx`,
        error_count: 0,
        warning_count: rule ? 1 : 0,
        diagnostics: rule
          ? [
              {
                rule_name: rule,
                severity: "warning",
                message: "complete message",
                start: 0,
                end: 6,
                help: "complete help",
                labels: [],
                fix: null,
              },
            ]
          : [],
      };
      return {
        kind,
        case: name,
        source: "complete authored source",
        expected: structuredClone(result),
        actual: result,
      };
    }),
  );
  return { provider_version: "7.0.2", packets: rows };
}

void test("full ordered native packets include all positive and negative fields", () => {
  const document = packets();
  assert.deepEqual(verifyPackets(Buffer.from(JSON.stringify(document))), document);
});

for (const [name, change] of [
  ["missing case", (d) => d.packets.pop()],
  ["reordered case", (d) => d.packets.reverse()],
  [
    "duplicate case",
    (d) => {
      d.packets[1] = structuredClone(d.packets[0]);
    },
  ],
  [
    "provider drift",
    (d) => {
      d.provider_version = "next";
    },
  ],
  [
    "message drift",
    (d) => {
      d.packets[0].actual.diagnostics[0].message = "different";
    },
  ],
  [
    "range drift",
    (d) => {
      d.packets[0].actual.diagnostics[0].end = 7;
    },
  ],
  [
    "negative false positive",
    (d) => {
      d.packets[1].actual.diagnostics = structuredClone(d.packets[0].actual.diagnostics);
    },
  ],
  [
    "skipped query counted as success",
    (d) => {
      d.packets[1].actual = structuredClone(d.packets[0].actual);
      d.packets[1].expected = structuredClone(d.packets[0].expected);
    },
  ],
  [
    "missing packet field on both sides",
    (d) => {
      delete d.packets[0].actual.filename;
      delete d.packets[0].expected.filename;
    },
  ],
  [
    "missing diagnostic field on both sides",
    (d) => {
      delete d.packets[0].actual.diagnostics[0].help;
      delete d.packets[0].expected.diagnostics[0].help;
    },
  ],
]) {
  void test(`packet authority rejects ${String(name)}`, () => {
    const document = packets();
    change(document);
    assert.throws(() => verifyPackets(JSON.stringify(document)));
  });
}

void test("native replay stays optional and follows all measurements without parallel source builds", () => {
  const workflow = fs.readFileSync(
    new URL("../../../.github/workflows/benchmark.yml", import.meta.url),
    "utf8",
  );
  const input = workflow.slice(
    workflow.indexOf("      patina_plain_type_packets:"),
    workflow.indexOf("\nrun-name:"),
  );
  assert.match(input, /default: false/u);
  assert.match(input, /type: boolean/u);
  const step = workflow.indexOf(
    "      - name: Replay conditional native type packets after observations",
  );
  for (const name of [
    "Compare base and head",
    "Qualify type-aware project batching on existing optimized binaries",
    "Observe CSS completion and hover on existing base and head CLIs",
    "Record benchmark build provenance",
  ]) {
    assert.ok(workflow.indexOf(`      - name: ${name}`) < step);
  }
  assert.match(
    workflow.slice(step),
    /github.event_name == 'workflow_dispatch' && inputs.patina_plain_type_packets/u,
  );
  assert.ok(workflow.includes("patina-plain-type-packets/*.stderr"));
  assert.ok(workflow.includes("if-no-files-found: error"));
  assert.match(TEST_NAME, /^linter::native_type_aware::plain_module_tests::packets::/u);
});
