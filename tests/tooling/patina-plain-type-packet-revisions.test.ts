import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  packetTestContract,
  planPacketReplay,
  PACKET_MODULE_SUFFIX,
  verifyPacketTransition,
} from "../../tools/benchmarks/scripts/patina-plain-type-contracts.ts";
import type { RuntimePacketContract } from "../../tools/benchmarks/scripts/patina-plain-type-runtime-counts.ts";

const legacy = readFileSync(
  new URL(
    "../../tools/benchmarks/fixtures/patina-plain-type-packets/warning-v1.rs",
    import.meta.url,
  ),
);
const current = readFileSync(
  new URL(
    "../../crates/vize_patina/src/linter/native_type_aware/plain_module_tests/packets.rs",
    import.meta.url,
  ),
);
const parent = Buffer.from("fn original_test() {}\n" + PACKET_MODULE_SUFFIX);

test("each existing revision keeps its own authenticated original packet test without an overlay", () => {
  for (const [child, contract] of [
    [legacy, "runtime-warning-v1"],
    [current, "runtime-error-v2"],
  ] as const) {
    const plan = planPacketReplay(parent, child, legacy);
    assert.equal(plan.overlay, false);
    assert.equal(plan.contract, contract);
    assert.deepEqual(plan.parent, parent);
    assert.deepEqual(plan.child, child);
    assert.equal(packetTestContract(child), contract);
  }
  assert.throws(() =>
    planPacketReplay(parent, Buffer.concat([current, Buffer.from("\n")]), legacy),
  );
  assert.throws(() => planPacketReplay(Buffer.from("fn no_module() {}\n"), current, legacy));
  assert.throws(() =>
    planPacketReplay(Buffer.concat([parent, Buffer.from(PACKET_MODULE_SUFFIX)]), current, legacy),
  );
});

test("only a genuinely absent baseline child gets the original warning test append-only", () => {
  const original = Buffer.from("fn untouched_test() {}\n");
  const plan = planPacketReplay(original, undefined, legacy);
  assert.equal(plan.overlay, true);
  assert.equal(plan.contract, "runtime-warning-v1");
  assert.deepEqual(plan.parent, Buffer.concat([original, Buffer.from(PACKET_MODULE_SUFFIX)]));
  assert.deepEqual(plan.child, legacy);
  assert.deepEqual(original, Buffer.from("fn untouched_test() {}\n"));
  assert.throws(() => planPacketReplay(parent, undefined, legacy));
  assert.throws(() => planPacketReplay(original, undefined, current));
  assert.throws(() =>
    planPacketReplay(original, undefined, Buffer.concat([legacy, Buffer.from("\n")])),
  );
});

interface Diagnostic {
  rule_name: string;
  severity: string;
  message: string;
  start: number;
  end: number;
  help: string;
  labels: unknown[];
  fix: null | { replacement: string };
}
interface ResultPacket {
  filename: string;
  error_count: number;
  warning_count: number;
  diagnostics: Diagnostic[];
}
interface Cell {
  kind: string;
  case: string;
  source: string;
  expected: ResultPacket;
  actual: ResultPacket;
}
const root = "/same/cwd";
const inputs = new Map([
  ["promise.tsx.txt", 'const p = Promise.resolve("雪🌸{[]}");\n'],
  ["reactivity.tsx.txt", "const state = { count: 1 };\n"],
]);
const beforeHelp =
  "Type-aware lint rules were skipped because the native compiler could not be started. Install a native TypeScript runtime in this project or configure `typeChecker.corsaPath`, `CORSA_PATH`, or `TSGO_PATH` to its executable.";
const afterHelp =
  "Requested type-aware lint could not initialize its native runtime. Check the reported failure, install a compatible native TypeScript runtime, or configure `typeChecker.corsaPath`, `CORSA_PATH`, or `TSGO_PATH` to its executable.";

function document(contract: RuntimePacketContract): { provider_version: string; packets: Cell[] } {
  const names = [
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
  return {
    provider_version: "7.0.2",
    packets: ["promise", "reactivity"].flatMap((kind) =>
      names.map((name) => {
        const unavailable = name === "unavailable";
        const failed = unavailable && contract === "runtime-error-v2";
        const rule = unavailable
          ? "type/corsa-runtime"
          : `type/${kind === "promise" ? "no-floating-promises" : "no-reactivity-loss"}`;
        const diagnostics: Diagnostic[] =
          name === "active" || unavailable
            ? [
                {
                  rule_name: rule,
                  severity: failed ? "error" : "warning",
                  message: unavailable
                    ? "Configured Corsa executable does not exist: /same/cwd/missing-provider (from configuration)"
                    : "complete ordinary finding",
                  start: 0,
                  end: unavailable ? 0 : 6,
                  help: unavailable ? (failed ? afterHelp : beforeHelp) : "complete ordinary help",
                  labels: [],
                  fix: null,
                },
              ]
            : [];
        const result = {
          filename: `${root}/${kind}-${name}.tsx`,
          error_count: failed ? 1 : 0,
          warning_count: diagnostics.length - (failed ? 1 : 0),
          diagnostics,
        };
        return {
          kind,
          case: name,
          source: inputs.get(`${kind}.tsx.txt`)!,
          expected: structuredClone(result),
          actual: result,
        };
      }),
    ),
  };
}
const bytes = (value: unknown): Buffer => Buffer.from(JSON.stringify(value, null, 2));

test("all twenty cells remain present with exactly two complete warning-to-error migrations", () => {
  const before = bytes(document("runtime-warning-v1"));
  const after = bytes(document("runtime-error-v2"));
  const result = verifyPacketTransition(
    before,
    after,
    "runtime-warning-v1",
    "runtime-error-v2",
    inputs,
    root,
  );
  assert.equal(result.packetCount, 20);
  assert.equal(result.cells.length, 20);
  assert.equal(result.unchanged, 18);
  assert.equal(result.changed, 2);
  assert.deepEqual(
    result.cells.filter((cell) => cell.changed).map((cell) => cell.id),
    ["promise/unavailable", "reactivity/unavailable"],
  );
  for (const cell of result.cells) {
    if (cell.changed) {
      assert.ok(cell.before);
      assert.ok(cell.after);
      assert.notEqual(cell.beforeSha256, cell.afterSha256);
    } else assert.equal(cell.beforeSha256, cell.afterSha256);
  }
  assert.deepEqual(before, bytes(document("runtime-warning-v1")));
  assert.deepEqual(after, bytes(document("runtime-error-v2")));
});

test("equal revision contracts require all twenty complete original bytes to match", () => {
  for (const contract of ["runtime-warning-v1", "runtime-error-v2"] as const) {
    const value = bytes(document(contract));
    const result = verifyPacketTransition(value, value, contract, contract, inputs, root);
    assert.equal(result.unchanged, 20);
    assert.equal(result.changed, 0);
  }
  assert.throws(() =>
    verifyPacketTransition(
      bytes(document("runtime-error-v2")),
      bytes(document("runtime-warning-v1")),
      "runtime-error-v2",
      "runtime-warning-v1",
      inputs,
      root,
    ),
  );
});

test("no positive or excluded cell can change its whole packet on either side", () => {
  for (let index = 0; index < 20; index++) {
    const after = document("runtime-error-v2");
    after.packets[index]!.actual.filename += ".changed";
    after.packets[index]!.expected.filename += ".changed";
    assert.throws(
      () =>
        verifyPacketTransition(
          bytes(document("runtime-warning-v1")),
          bytes(after),
          "runtime-warning-v1",
          "runtime-error-v2",
          inputs,
          root,
        ),
      String(index),
    );
  }
});

test("both unavailable contracts validate every original diagnostic field, not just counts", () => {
  const changes: Array<(cell: Cell) => void> = [
    (cell) => {
      cell.actual.error_count = 0;
    },
    (cell) => {
      cell.actual.warning_count = 1;
    },
    (cell) => {
      cell.actual.diagnostics = [];
    },
    (cell) => {
      cell.actual.diagnostics[0]!.severity = "warning";
    },
    (cell) => {
      cell.actual.diagnostics[0]!.rule_name = "type/no-floating-promises";
    },
    (cell) => {
      cell.actual.diagnostics[0]!.message += " changed";
    },
    (cell) => {
      cell.actual.diagnostics[0]!.help += " changed";
    },
    (cell) => {
      cell.actual.diagnostics[0]!.start = 1;
    },
    (cell) => {
      cell.actual.diagnostics[0]!.end = 1;
    },
    (cell) => {
      cell.actual.diagnostics[0]!.labels = [{ start: 0, end: 0 }];
    },
    (cell) => {
      cell.actual.diagnostics[0]!.fix = { replacement: "" };
    },
    (cell) => {
      cell.source += "changed";
    },
  ];
  for (const index of [8, 18])
    for (const change of changes) {
      const after = document("runtime-error-v2");
      const cell = after.packets[index]!;
      change(cell);
      cell.expected = structuredClone(cell.actual);
      assert.throws(() =>
        verifyPacketTransition(
          bytes(document("runtime-warning-v1")),
          bytes(after),
          "runtime-warning-v1",
          "runtime-error-v2",
          inputs,
          root,
        ),
      );
    }
});

test("raw byte comparison refuses whitespace normalization and changed envelope/order/population", () => {
  const before = bytes(document("runtime-warning-v1"));
  const after = bytes(document("runtime-error-v2"));
  const whitespace = after
    .toString()
    .replace(
      '"filename": "/same/cwd/promise-active.tsx"',
      '"filename" : "/same/cwd/promise-active.tsx"',
    );
  assert.notEqual(whitespace, after.toString());
  assert.throws(() =>
    verifyPacketTransition(
      before,
      Buffer.from(whitespace),
      "runtime-warning-v1",
      "runtime-error-v2",
      inputs,
      root,
    ),
  );
  for (const mutate of [
    (value: ReturnType<typeof document>) => {
      value.packets.pop();
    },
    (value: ReturnType<typeof document>) => {
      value.packets.reverse();
    },
    (value: ReturnType<typeof document>) => {
      value.packets[1] = structuredClone(value.packets[0]!);
    },
    (value: ReturnType<typeof document>) => {
      value.provider_version = "next";
    },
  ]) {
    const changed = document("runtime-error-v2");
    mutate(changed);
    assert.throws(() =>
      verifyPacketTransition(
        before,
        bytes(changed),
        "runtime-warning-v1",
        "runtime-error-v2",
        inputs,
        root,
      ),
    );
  }
  const value = document("runtime-error-v2");
  assert.throws(() =>
    verifyPacketTransition(
      before,
      bytes({ packets: value.packets, provider_version: value.provider_version }),
      "runtime-warning-v1",
      "runtime-error-v2",
      inputs,
      root,
    ),
  );
});
