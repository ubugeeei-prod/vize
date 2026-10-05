import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import test from "node:test";
import {
  observe,
  referenceIdentity,
  sha256,
  type Observation,
  type Scope,
} from "./support/vue1-pinned-getter.ts";
import {
  utf8Slice,
  verifyGetter,
  verifyMap,
  verifyNativeIdentity,
  verifyOutcome,
  type Case,
  type Control,
  type NativePacket,
} from "./support/vue1-text-document-judge.ts";

const fixtureUrl = new URL(
  "../../crates/vize_glyph/tests/fixtures/native-vue1-text-doc-1.0.28.json",
  import.meta.url,
);
const fixtureBytes = readFileSync(fixtureUrl);
const packet = JSON.parse(fixtureBytes.toString("utf8")) as {
  schema: string;
  version: number;
  vueVersion: string;
  nativeCapture: null;
  scopes: Record<string, Scope>;
  cases: Case[];
  primaryControls: Control[];
};
const environments = ["test", "production"] as const;
const scope = (name: string) => {
  const declaration = packet.scopes[name];
  assert.ok(declaration, `independently declared scope ${name}`);
  return declaration;
};
const callbackObservations = packet.cases.map((entry) => ({
  id: entry.id,
  environments: Object.fromEntries(
    environments.map((environment) => [
      environment,
      {
        before: observe(entry.primary.before.input, environment, scope(entry.primary.scope)),
        after: observe(entry.primary.after.input, environment, scope(entry.primary.scope)),
      },
    ]),
  ),
}));
const controlObservations = packet.primaryControls.map((entry) => ({
  id: entry.id,
  environments: Object.fromEntries(
    environments.map((environment) => [
      environment,
      observe(entry.input, environment, scope(entry.scope)),
    ]),
  ),
}));

const capturePath = process.env.VIZE_GLYPH_VUE1_TEXT_CAPTURE;
const requireCapture = process.env.VIZE_GLYPH_VUE1_TEXT_REQUIRE_CAPTURE === "1";
let native: NativePacket | null = null;
let nativeBytes: Buffer | null = null;
let nativeReadError: string | null = null;
let executableSha256: string | null = null;
if (capturePath) {
  try {
    nativeBytes = readFileSync(capturePath);
    native = JSON.parse(nativeBytes.toString("utf8")) as NativePacket;
    executableSha256 = sha256(readFileSync(native.executablePath));
  } catch (error) {
    nativeReadError = String(error);
  }
}

// Record every actual primary observation, including generated getter strings,
// complete warnings, undefined/noop outcomes and own Error fields/stacks BEFORE
// assertions. An unexpected native refusal remains in the whole Rust packet.
// This report never creates an expected fixture from a first native result.
const nativeObservations =
  native?.cases?.map((current, index) => {
    const entry = packet.cases[index];
    if (!entry) return { id: current.id, error: "unmatched native row" };
    return {
      id: current.id,
      environments: Object.fromEntries(
        environments.map((environment) => [
          environment,
          {
            rawCallback:
              typeof current.printed === "string" && entry.expected === entry.primary.after.input
                ? observe(current.printed, environment, scope(entry.primary.scope))
                : null,
            originalPrepared:
              typeof current.preparedText === "string"
                ? observe(
                    current.preparedText,
                    environment,
                    scope(entry.primary.scope),
                    "original-L1-prepared-expression",
                  )
                : null,
            formattedPrepared:
              typeof current.formattedPreparedText === "string"
                ? observe(
                    current.formattedPreparedText,
                    environment,
                    scope(entry.primary.scope),
                    "original-L1-prepared-expression",
                  )
                : null,
          },
        ]),
      ),
    };
  }) ?? [];
const oracleReport = {
  schema: "vize.native-vue1-text-document-getter-capture",
  version: 1,
  sourceHead: process.env.VIZE_GLYPH_VUE1_TEXT_SOURCE_HEAD ?? null,
  executionCommit: process.env.GITHUB_SHA ?? null,
  workflowRun: process.env.GITHUB_RUN_ID ?? null,
  oracleExecutable: {
    path: process.execPath,
    sha256: sha256(readFileSync(process.execPath)),
    version: process.version,
    processId: process.pid,
    arguments: process.argv,
  },
  scope:
    "Actual pinned parser/getter observations; L1 prepared inputs are not browser decoding or mounted runtime proof.",
  reference: referenceIdentity,
  sourceFiles: [
    fixtureUrl,
    new URL(import.meta.url),
    new URL("./support/vue1-pinned-getter.ts", import.meta.url),
    new URL("./support/vue1-text-document-judge.ts", import.meta.url),
  ].map((url) => ({
    path: url.pathname,
    sha256: sha256(readFileSync(url)),
  })),
  fixtureSha256: sha256(fixtureBytes),
  nativeCapturePath: capturePath ?? null,
  nativeCaptureSha256: nativeBytes === null ? null : sha256(nativeBytes),
  executableSha256,
  nativeReadError,
  nativePacket: native,
  callbackObservations,
  controlObservations,
  nativeObservations,
};
if (process.env.VIZE_GLYPH_VUE1_TEXT_ORACLE_CAPTURE) {
  writeFileSync(
    process.env.VIZE_GLYPH_VUE1_TEXT_ORACLE_CAPTURE,
    `${JSON.stringify(oracleReport, null, 2)}\n`,
  );
}

test("independent complete Vue1 LF goldens retain actual official getter values and effects", () => {
  assert.equal(packet.schema, "vize.native-vue1-text-document-reference");
  assert.equal(packet.version, 1);
  assert.equal(packet.vueVersion, "1.0.28");
  assert.equal(packet.nativeCapture, null, "no execution receipt is fabricated into the reference");
  assert.equal(packet.cases.length, 64);
  assert.equal(new Set(packet.cases.map((entry) => entry.id)).size, 64);
  for (const [index, entry] of packet.cases.entries()) {
    assert.equal(utf8Slice(entry.source, entry.callbackSpan), entry.callback);
    utf8Slice(entry.source, entry.blockSpan);
    assert.ok(
      entry.callbackSpan[0] >= entry.blockSpan[0] && entry.callbackSpan[1] <= entry.blockSpan[1],
    );
    assert.ok(entry.path.length > 0 && entry.path.every((n) => Number.isSafeInteger(n) && n >= 0));
    assert.equal(entry.lineEnding, "Lf");
    assert.ok(!entry.expected.includes("\r"));
    const observed = callbackObservations[index];
    assert.equal(observed.id, entry.id);
    for (const environment of environments) {
      const actual = observed.environments[environment] as {
        before: Observation;
        after: Observation;
      };
      for (const which of ["before", "after"] as const) {
        const expected = entry.primary[which];
        const current = actual[which];
        const label = `${entry.id} ${which} ${environment}`;
        assert.equal(current.stage, "decoded-callback");
        assert.equal(current.input, expected.input);
        assert.deepEqual(current.tokens, expected.tokens, label);
        assert.equal(current.expression, expected.expression, label);
        verifyGetter(current, expected.getterBody, label);
        verifyOutcome(current, entry.primary.outcomes[environment], label);
        assert.equal(current.noop, false, "equal noop is never semantic acceptance");
        assert.equal(current.result?.kind, "return");
        if (current.result?.kind === "return")
          assert.notEqual(
            current.result.value.kind,
            "undefined",
            "equal undefined is never semantic acceptance",
          );
        if (entry.primary.scope !== "constants") assert.ok(current.trace.length > 0);
      }
    }
  }
});

test("whole official Vue1 malformed, comment, Unicode and thrown outcomes remain labelled controls", () => {
  assert.equal(packet.primaryControls.length, 8);
  for (const [index, entry] of packet.primaryControls.entries()) {
    assert.equal(entry.semanticCredit, false);
    const observed = controlObservations[index];
    assert.equal(observed.id, entry.id);
    for (const environment of environments) {
      const current = observed.environments[environment] as Observation;
      const label = `${entry.id} ${environment}`;
      assert.equal(current.input, entry.input);
      assert.deepEqual(current.tokens, entry.tokens, label);
      assert.equal(current.expression, entry.expression, label);
      verifyGetter(current, entry.getterBody, label);
      verifyOutcome(current, entry.outcomes[environment], label);
    }
  }
});

test(
  "fresh original Vue1 TextView Docs join whole callback goldens and actual once-prepared getter inputs",
  {
    skip:
      capturePath || requireCapture ? false : "optional all-scripts run has no fresh Rust capture",
  },
  () => {
    assert.ok(capturePath, "dedicated qualification requires VIZE_GLYPH_VUE1_TEXT_CAPTURE");
    assert.ok(
      process.env.VIZE_GLYPH_VUE1_TEXT_ORACLE_CAPTURE,
      "capture qualification retains actual getter observations before assertions",
    );
    assert.equal(nativeReadError, null);
    assert.ok(native && nativeBytes);
    verifyNativeIdentity(native, executableSha256, oracleReport.nativeCaptureSha256);
    assert.deepEqual(
      native.cases.map((entry) => entry.id),
      packet.cases.map((entry) => entry.id),
    );
    for (const [index, current] of native.cases.entries()) {
      const entry = packet.cases[index];
      assert.deepEqual(
        Object.keys(current).sort(),
        [
          "blockSpan",
          "callback",
          "callbackSpan",
          "formattedPreparedMap",
          "formattedPreparedSpan",
          "formattedPreparedText",
          "id",
          "idempotent",
          "indentWidth",
          "lineEnding",
          "path",
          "preparedMap",
          "preparedSpan",
          "preparedText",
          "printed",
          "source",
          "width",
        ].sort(),
        entry.id,
      );
      assert.deepEqual(
        {
          id: current.id,
          source: current.source,
          blockSpan: current.blockSpan,
          path: current.path,
          callbackSpan: current.callbackSpan,
          callback: current.callback,
          width: current.width,
          indentWidth: current.indentWidth,
          lineEnding: current.lineEnding,
          printed: current.printed,
          idempotent: current.idempotent,
        },
        {
          id: entry.id,
          source: entry.source,
          blockSpan: entry.blockSpan,
          path: entry.path,
          callbackSpan: entry.callbackSpan,
          callback: entry.callback,
          width: entry.width,
          indentWidth: entry.indentWidth,
          lineEnding: "Lf",
          printed: entry.expected,
          idempotent: entry.expected,
        },
        entry.id,
      );
      assert.equal(utf8Slice(current.source, current.callbackSpan), current.callback);
      assert.equal(current.preparedText, entry.primary.before.expression);
      assert.equal(current.formattedPreparedText, entry.primary.after.expression);
      assert.deepEqual(current.preparedSpan, entry.primary.before.authoredPreparedSpan);
      assert.deepEqual(current.formattedPreparedSpan, entry.primary.after.authoredPreparedSpan);
      verifyMap(current.source, current.preparedSpan, current.preparedText, current.preparedMap);
      verifyMap(
        current.printed,
        current.formattedPreparedSpan,
        current.formattedPreparedText,
        current.formattedPreparedMap,
      );
      const observed = nativeObservations[index] as {
        id: string;
        environments: Record<
          string,
          {
            rawCallback: Observation | null;
            originalPrepared: Observation;
            formattedPrepared: Observation;
          }
        >;
      };
      assert.equal(observed.id, entry.id);
      for (const environment of environments) {
        const actual = observed.environments[environment];
        if (entry.expected === entry.primary.after.input) {
          assert.ok(actual.rawCallback);
          assert.equal(actual.rawCallback.input, current.printed);
          assert.deepEqual(actual.rawCallback.tokens, entry.primary.after.tokens);
          assert.equal(actual.rawCallback.expression, entry.primary.after.expression);
          verifyGetter(actual.rawCallback, entry.primary.after.getterBody, entry.id);
          verifyOutcome(actual.rawCallback, entry.primary.outcomes[environment], entry.id);
        } else assert.equal(actual.rawCallback, null, "no invented browser decoder");
        for (const [which, prepared] of [
          ["before", actual.originalPrepared],
          ["after", actual.formattedPrepared],
        ] as const) {
          assert.equal(prepared.stage, "original-L1-prepared-expression");
          assert.equal(prepared.tokens, null);
          assert.equal(prepared.input, entry.primary[which].expression);
          assert.equal(prepared.expression, entry.primary[which].expression);
          verifyGetter(prepared, entry.primary[which].getterBody, entry.id);
          verifyOutcome(prepared, entry.primary.outcomes[environment], entry.id);
        }
      }
    }
  },
);
