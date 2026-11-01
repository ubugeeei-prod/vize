import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  acquireSfcEntryReference,
  captureWholeValue,
  CASES_SHA256,
  isRecord,
  loadSfcEntryCases,
  REFERENCE_VERSION,
  type SfcEntryCase,
} from "./support/sfc-entry-validation-reference.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

void test("whole observations retain nonenumerable unknown errors, undefined, symbols and cyclic collections", () => {
  const error = new Error("complete message");
  Object.defineProperty(error, "unknown", {
    value: { absent: undefined, negativeZero: -0 },
    enumerable: false,
  });
  const symbol = Symbol("original"),
    map = new Map<unknown, unknown>();
  map.set(symbol, error);
  map.set(map, symbol);
  const actual = captureWholeValue({ error, symbol, map });
  const encoded = JSON.stringify(actual);
  for (const required of [
    "complete message",
    "unknown",
    "absent",
    "undefined",
    "-0",
    "symbolRef",
    "ref",
    "stack",
    "enumerable",
  ])
    assert(encoded.includes(required), required);
  assert.deepEqual(JSON.parse(encoded), actual, "the retained tagged graph is wholly JSON-safe");
});

void test(
  "official rc14 retains all20 whole parse/script outcomes and the original eight error contracts",
  { timeout: 300_000 },
  async (t) => {
    const cases = loadSfcEntryCases(root);
    const phases: Array<{ phase: string; value: unknown }> = [];
    const outcomes: Array<{
      fixture: SfcEntryCase;
      parse: unknown;
      script: unknown;
      descriptorAfter: unknown;
      parseErrorCount: number | null;
      parseMessages: string[];
      scriptStatus: string;
      scriptMessage: string | null;
    }> = [];
    const record = (phase: string, value: unknown) => {
      phases.push({ phase, value });
    };
    const traceLimit = Error.stackTraceLimit;
    Error.stackTraceLimit = 0;
    let failure: unknown;
    try {
      const { compiler, loadedSourceReceipt } = await acquireSfcEntryReference(record);
      for (const fixture of cases) {
        let parsed: unknown,
          parse: unknown,
          script: unknown = { status: "not-invoked", reason: "parse did not return a descriptor" },
          scriptStatus = "not-invoked",
          scriptMessage: string | null = null;
        let parseErrorCount: number | null = null,
          parseMessages: string[] = [];
        try {
          parsed = compiler.parse(fixture.source, { filename: fixture.filename, sourceMap: false });
          parse = { status: "returned", value: captureWholeValue(parsed) };
        } catch (error) {
          parse = { status: "threw", value: captureWholeValue(error) };
        }
        if (isRecord(parsed) && Array.isArray(parsed.errors)) {
          parseErrorCount = parsed.errors.length;
          parseMessages = parsed.errors.map((error: unknown) =>
            error instanceof Error ? error.message : String(error),
          );
          try {
            const value = compiler.compileScript(parsed.descriptor, {
              id: "sfc-entry-validation",
              sourceMap: false,
            });
            scriptStatus = "returned";
            script = { status: scriptStatus, value: captureWholeValue(value) };
          } catch (error) {
            scriptStatus = "threw";
            scriptMessage = error instanceof Error ? error.message : String(error);
            script = { status: scriptStatus, value: captureWholeValue(error) };
          }
        }
        outcomes.push({
          fixture,
          parse,
          script,
          descriptorAfter: captureWholeValue(parsed),
          parseErrorCount,
          parseMessages,
          scriptStatus,
          scriptMessage,
        });
      }
      loadedSourceReceipt();
    } catch (error) {
      failure = captureWholeValue(error);
    } finally {
      Error.stackTraceLimit = traceLimit;
      const observation = {
        schema: "vize.sfc-entry-official-reference",
        version: 1,
        referenceVersion: REFERENCE_VERSION,
        catalogSha256: CASES_SHA256,
        recipe: {
          parse: "all20 filename/sourceMap:false",
          compileScript: "all20 parsed descriptors, id:sfc-entry-validation/sourceMap:false",
          errorStackTraceLimit: 0,
        },
        phases,
        outcomes,
        failure: captureWholeValue(failure),
        scope:
          "Whole official outcomes; Vize type-specifier/type-all/ambient acceptance is a separate policy, not assumed reference parity. Functions retain source/own properties, not callable closures.",
      };
      const directory = path.join(root, "target/ci/sfc-entry-validation-reference");
      fs.mkdirSync(directory, { recursive: true });
      fs.writeFileSync(
        path.join(directory, "observation.json"),
        JSON.stringify(observation, null, 2) + "\n",
        { flag: "wx" },
      );
      t.diagnostic(`sfc-entry-official-reference ${JSON.stringify(observation, null, 2)}`);
    }
    assert.equal(failure, undefined, "provider/capture failure; whole receipt retained above");
    assert.equal(outcomes.length, 20);
    assert.deepEqual(
      outcomes.map((entry) => entry.fixture.id),
      cases.map((entry) => entry.id),
    );
    const missing = "At least one <template> or <script> is required in a single file component.";
    for (const actual of outcomes) {
      if (actual.fixture.expect === "missing-block") {
        assert.equal(actual.parseErrorCount, 1, actual.fixture.id);
        assert.deepEqual(actual.parseMessages, [`${missing} ${actual.fixture.filename}`]);
      } else assert.equal(actual.parseErrorCount, 0, actual.fixture.id);
      if (actual.fixture.expect === "export-error") {
        assert.equal(actual.scriptStatus, "threw", actual.fixture.id);
        assert(
          actual.scriptMessage?.includes("<script setup> cannot contain ES module exports."),
          actual.fixture.id,
        );
      }
      if (
        ["type-alias", "type-interface", "normal-script", "setup-comments"].includes(
          actual.fixture.id,
        )
      )
        assert.equal(actual.scriptStatus, "returned", actual.fixture.id);
      if (actual.fixture.id === "template-only") {
        assert.equal(actual.scriptStatus, "threw");
        assert.equal(actual.scriptMessage, "[@vue/compiler-sfc] SFC contains no <script> tags.");
      }
    }
  },
);
