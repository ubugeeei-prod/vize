import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import test from "node:test";
import { frame, compileWhole, executeWhole, semantic } from "./support/vue2-whole-sfc-oracle.ts";

import {
  observeRuntime,
  observedError,
  persistRuntimeObservations,
  type RuntimeObservation,
} from "./support/vue2-whole-sfc-observations.ts";

type Case = {
  id: string;
  source: string;
  expected: string;
  width: number;
  indent: number;
  lineEnding: string;
  runtimeCredit: boolean;
  semantic: unknown;
  events: string[];
  calls: unknown[];
  beforeFrame: ReturnType<typeof frame>;
  afterFrame: ReturnType<typeof frame>;
  beforeCompiler: ReturnType<typeof compileWhole>;
  afterCompiler: ReturnType<typeof compileWhole>;
  runtime: ReturnType<typeof executeWhole> | null;
};
const referenceBytes = readFileSync(
  new URL(
    "../../crates/vize_glyph/tests/fixtures/native-vue2-scriptless-sfc-2.7.16.json",
    import.meta.url,
  ),
);
const packet = JSON.parse(referenceBytes.toString("utf8")) as {
  schema: string;
  version: number;
  vueVersion: string;
  nativeCapture: null;
  authoredPrintSha256: string;
  cases: Case[];
};
type NativePacket = {
  schema: string;
  version: number;
  sourceHead: string | null;
  executionCommit: string | null;
  workflowRun: string | null;
  cases: Array<
    Record<string, unknown> & {
      id: string;
      format: { status: string; printed: string | null } | null;
    }
  >;
};
const path = process.env.VIZE_GLYPH_VUE2_SFC_CAPTURE;
let nativePacket: NativePacket | null = null;
const nativeLoad = {
  status: "absent",
  sha256: null as string | null,
  metadata: null as unknown,
  error: null as unknown,
};
if (path) {
  try {
    const bytes = readFileSync(path);
    nativeLoad.sha256 = createHash("sha256").update(bytes).digest("hex");
    const parsed = JSON.parse(bytes.toString("utf8")) as NativePacket;
    if (!Array.isArray(parsed?.cases)) throw new Error("actual native capture has no case array");
    nativePacket = parsed;
    nativeLoad.status = "loaded";
    nativeLoad.metadata = {
      schema: parsed.schema,
      version: parsed.version,
      sourceHead: parsed.sourceHead,
      executionCommit: parsed.executionCommit,
      workflowRun: parsed.workflowRun,
    };
  } catch (error) {
    nativeLoad.status = "errored";
    nativeLoad.error = observedError(error);
  }
}
const referenceBefore = packet.cases.map((row, index) =>
  observeRuntime({
    origin: "reference-before",
    index,
    id: row.id,
    source: row.source,
    runtimeRequested: row.runtimeCredit,
    unavailableReason: null,
  }),
);
const referenceAfter = packet.cases.map((row, index) =>
  observeRuntime({
    origin: "reference-after",
    index,
    id: row.id,
    source: row.expected,
    runtimeRequested: row.runtimeCredit,
    unavailableReason: null,
  }),
);
const nativeRuntime = (nativePacket?.cases ?? []).map((native, index) => {
  const id = typeof native?.id === "string" ? native.id : null;
  const row = packet.cases.find((candidate) => candidate.id === id);
  const source =
    native?.format?.status === "printed" && typeof native.format.printed === "string"
      ? native.format.printed
      : null;
  return observeRuntime({
    origin: "native",
    index,
    id,
    source,
    runtimeRequested: row?.runtimeCredit === true,
    unavailableReason:
      source === null
        ? "actual native attempt produced no printed source"
        : row
          ? null
          : "actual native case has no independent reference policy",
  });
});
// Persist all reference and actual-native dev/prod packets/errors before test assertions.
persistRuntimeObservations(createHash("sha256").update(referenceBytes).digest("hex"), nativeLoad, [
  ...referenceBefore,
  ...referenceAfter,
  ...nativeRuntime,
]);

const assertRuntime = (observation: RuntimeObservation | undefined, row: Case) => {
  assert.ok(observation);
  assert.deepEqual(
    observation.attempts.map((attempt) => attempt.environment),
    ["test", "production"],
  );
  const failures: string[] = [];
  for (const attempt of observation.attempts) {
    try {
      assert.ok(row.runtimeCredit);
      assert.ok(row.runtime);
      assert.equal(attempt.status, "completed", JSON.stringify(attempt));
      assert.ok(attempt.packet);
      assert.deepEqual(
        attempt.packet,
        row.runtime,
        `${row.id} complete primary recursive packet ${attempt.environment}`,
      );
      assert.deepEqual(
        semantic(attempt.packet.vnode),
        row.semantic,
        `${row.id} independent authored whole meaning`,
      );
      assert.deepEqual(
        attempt.packet.events,
        row.events,
        `${row.id} separate lookup/read/invocation order`,
      );
      assert.deepEqual(attempt.packet.calls, row.calls);
    } catch (error) {
      failures.push(`${attempt.environment}: ${String(error)}`);
    }
  }
  assert.deepEqual(
    failures,
    [],
    `${row.id} both actual environments were retained before judgment`,
  );
};

test("independent whole Vue2 SFC frames compiler results and bounded dev/prod meanings are complete", () => {
  assert.equal(packet.schema, "vize.native-vue2-scriptless-sfc-reference");
  assert.equal(packet.version, 1);
  assert.equal(packet.vueVersion, "2.7.16");
  assert.equal(packet.nativeCapture, null);
  assert.equal(
    packet.authoredPrintSha256,
    "4a32485f52077b8b31297f8a277561a9fd024766e594d4d94e1685f498af71f6",
  );
  const authored = {
    schema: packet.schema,
    version: packet.version,
    vueVersion: packet.vueVersion,
    nativeCapture: packet.nativeCapture,
    cases: packet.cases.map((row) => ({
      id: row.id,
      source: row.source,
      expected: row.expected,
      width: row.width,
      indent: row.indent,
      lineEnding: row.lineEnding,
      runtimeCredit: row.runtimeCredit,
      semantic: row.semantic,
      events: row.events,
      calls: row.calls,
    })),
  };
  assert.equal(
    createHash("sha256")
      .update(JSON.stringify(authored, null, 2) + "\n")
      .digest("hex"),
    packet.authoredPrintSha256,
    "independently fixed print/meaning rows precede every native capture",
  );
  assert.equal(packet.cases.length, 32);
  assert.equal(new Set(packet.cases.map((row) => row.id)).size, 32);
  assert.equal(packet.cases.filter((row) => row.runtimeCredit).length, 29);
  assert.deepEqual(
    packet.cases.filter((row) => !row.runtimeCredit).map((row) => row.id),
    [
      "empty-primary-control",
      "multiple-root-primary-control",
      "attribute-interpolation-primary-control",
    ],
  );
  for (const [index, row] of packet.cases.entries()) {
    assert.deepEqual(frame(row.source), row.beforeFrame, row.id);
    assert.deepEqual(frame(row.expected), row.afterFrame, row.id);
    assert.deepEqual(compileWhole(row.source), row.beforeCompiler, row.id);
    assert.deepEqual(compileWhole(row.expected), row.afterCompiler, row.id);
    if (row.runtimeCredit) {
      assert.deepEqual(row.beforeCompiler.errors, []);
      assert.deepEqual(row.afterCompiler.errors, []);
      assertRuntime(referenceBefore[index], row);
      assertRuntime(referenceAfter[index], row);
    } else {
      assert.equal(
        row.runtime,
        null,
        "fallback/warning compilation gives no mounted/native ABI credit",
      );
      assert.equal(row.semantic, null);
      assert.deepEqual(row.events, []);
      assert.deepEqual(row.calls, []);
    }
  }
});

test(
  "fresh original whole Vue2 Docs join every complete primary reference and physical frame",
  {
    skip: path ? false : "dedicated source/protected hook requires fresh native output",
  },
  () => {
    assert.ok(path);
    assert.equal(nativeLoad.status, "loaded", JSON.stringify(nativeLoad));
    assert.ok(nativePacket);
    const actual = nativePacket;
    assert.equal(actual.schema, "vize.native-vue2-scriptless-sfc-capture");
    assert.equal(actual.version, 2);
    assert.equal(actual.sourceHead, process.env.VIZE_GLYPH_VUE2_SFC_SOURCE_HEAD ?? null);
    assert.equal(actual.executionCommit, process.env.GITHUB_SHA ?? null);
    assert.equal(actual.workflowRun, process.env.GITHUB_RUN_ID ?? null);
    assert.deepEqual(
      actual.cases.map((row) => row.id),
      packet.cases.map((row) => row.id),
    );
    const failures: string[] = [];
    const check = (label: string, run: () => void) => {
      try {
        run();
      } catch (error) {
        failures.push(`${label}: ${String(error)}`);
      }
    };
    for (const [index, native] of actual.cases.entries()) {
      const row = packet.cases[index];
      assert.ok(row);
      const open = Buffer.byteLength(row.source.slice(0, row.source.indexOf("<template") + 1));
      const close = Buffer.byteLength(
        row.source.slice(0, row.source.lastIndexOf("</template>") + 2),
      );
      const printed = {
        status: "printed",
        printed: row.expected,
        changed: row.source !== row.expected,
        refusal: null,
      };
      check(`${row.id} complete actual attempt`, () =>
        assert.deepEqual(
          native,
          {
            id: row.id,
            source: row.source,
            width: row.width,
            indent: row.indent,
            lineEnding: row.lineEnding,
            status: "completed",
            panic: null,
            format: printed,
            repeat: printed,
            fixedPoint: { ...printed, changed: false },
            beforeUtf8: row.beforeFrame.utf8,
            afterUtf8: row.afterFrame.utf8,
            byteLength: row.beforeFrame.byteLength,
            containerIndex: 0,
            openingName: { start: open, end: open + 8 },
            closingName: { start: close, end: close + 8 },
            selectionRefusal: null,
            afterSelectionRefusal: null,
          },
          row.id,
        ),
      );
      if (native.format?.status === "printed" && typeof native.format.printed === "string") {
        const source = native.format.printed;
        check(`${row.id} actual physical frame`, () =>
          assert.deepEqual(frame(source), row.afterFrame),
        );
        check(`${row.id} actual compiler`, () =>
          assert.deepEqual(compileWhole(source), row.afterCompiler),
        );
        if (row.runtimeCredit)
          check(`${row.id} actual dev/prod meaning`, () =>
            assertRuntime(nativeRuntime[index], row),
          );
        else check(`${row.id} reference-only control`, () => assert.equal(row.runtime, null));
      }
    }
    assert.deepEqual(
      failures,
      [],
      "all complete attempts are judged after their raw capture exists",
    );
  },
);
