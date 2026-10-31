import assert from "node:assert/strict";
import path from "node:path";
import fs from "node:fs";
import os from "node:os";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  loadLinterManifest,
  summarizeLinterAcceptance,
  validateLinterReport,
} from "../differential/linter-api.ts";
import {
  expectedNativeReason,
  nativeArgv,
  NATIVE_APIS,
  runNative,
} from "../differential/linter-native.ts";
import { sha256 } from "../differential/harness.mjs";
import {
  ORIGINAL_INPUT_LABEL_WIRES,
  expectedInputLabelReason,
} from "../differential/linter-input-label-contract.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const loaded = loadLinterManifest(
  path.join(root, "tests/_fixtures/differential/linter/manifest.json"),
  root,
);

void test("input-label native contract pins all six whole originals and rejects any wire drift", () => {
  for (const malformed of [
    null,
    undefined,
    true,
    0,
    "original",
    [],
    {},
    { ...ORIGINAL_INPUT_LABEL_WIRES[0], extra: "unproved" },
  ]) {
    assert.throws(() => expectedInputLabelReason(malformed, "--native-current-api"));
  }
  const originals = JSON.parse(
    fs.readFileSync(
      path.join(root, "crates/vize_patina/tests/fixtures/lint-history/cases.json"),
      "utf8",
    ),
  ).filter((wire: { rule: string }) => wire.rule === "a11y/form-control-has-label");
  assert.deepEqual(ORIGINAL_INPUT_LABEL_WIRES, originals);
  for (const original of ORIGINAL_INPUT_LABEL_WIRES) {
    const fixture = loaded.cases.find(
      (fixture: { id: string }) => fixture.id === `linter/current-api/${original.id}`,
    );
    assert(fixture);
    assert.deepEqual(JSON.parse(fixture.input.toString()), original);
    assert.equal(expectedNativeReason(fixture), null);
    assert.throws(() => expectedInputLabelReason(original, "--unprovided-native-api"));
    for (const [key, value] of Object.entries(original)) {
      const changed = {
        ...original,
        [key]: typeof value === "number" ? value + 1 : `${value}EXTRA`,
      };
      assert.throws(() => expectedInputLabelReason(changed, "--native-current-api"), key);
    }
  }
});

function probe(argv: string[], value: any) {
  const bytes = Buffer.from(`${JSON.stringify(value)}\n`);
  return { argv, exitStatus: 0, stdoutBase64: bytes.toString("base64"), sha256: sha256(bytes) };
}
const receipt = {
  source: { sourceRevision: "a".repeat(40) },
  probes: [
    probe(["--contract"], {
      schema: "vize.linter-history-observer",
      version: 3,
      apis: ["--current-api", "--report", "--static-class"],
      preset: "Incremental",
      locale: "En",
      help: "Full",
      native: "configured-template-and-sfc",
      nativeApis: NATIVE_APIS,
    }),
    probe(["--native-contract"], {
      schema: "vize.linter-native-observer",
      version: 3,
      apis: NATIVE_APIS,
      owners: {
        template: "NativeLintComponent",
        sfc: { setup: "NativeSfcLintOwner", template: "NativeSfcTemplateOwner" },
      },
      entries: ["template", "sfc"],
      wholeOutput: "Case+Observation",
      fallback: false,
    }),
  ],
};

function attempt(bytes: Buffer, reference?: any) {
  return {
    stdoutBase64: bytes.toString("base64"),
    stdoutSha256: sha256(bytes),
    stderrBase64: "",
    stderrSha256: sha256(Buffer.alloc(0)),
    exitStatus: 0,
    signal: null,
    processError: null,
    ...(reference ? { referenceComparison: reference } : {}),
  };
}
function provenance() {
  return {
    scope: "whole-product",
    sourceRevision: receipt.source.sourceRevision,
    buildReceiptSha256: sha256(Buffer.from(JSON.stringify(receipt))),
    contributions: ["parse", "lint", "output"].map((stage) => ({
      stage,
      implementation: "native",
      factOrigin: "native",
      fallback: false,
    })),
  };
}

// Deliberately synthetic protocol bytes test rejection contracts only. They are
// not execution evidence, source-built artifacts, fixture capture or credit.
function baseline() {
  const rows = loaded.cases.map((fixture: any) => {
    const reason = expectedNativeReason(fixture);
    const bytes = Buffer.from(
      `${JSON.stringify(reason ? { state: "unsupported", reason } : { state: "handled", observation: fixture.expected.toString() })}\n`,
    );
    const attempts = [
      attempt(bytes, reason ? null : { state: "equal" }),
      attempt(bytes, reason ? null : { state: "equal" }),
    ];
    const native: any = {
      state: reason ? "unsupported" : "completed",
      argv: nativeArgv(fixture),
      inputSha256: sha256(fixture.input),
      attempts,
    };
    if (reason) native.reason = reason;
    else {
      native.verdict = "matched-reference";
      native.provenance = provenance();
      native.observation = { argv: native.argv, inputSha256: native.inputSha256, attempts };
    }
    return {
      id: fixture.id,
      target: "lint",
      legacy: {
        state: "completed",
        verdict: "matched-reference",
        argv: fixture.argv,
        inputSha256: sha256(fixture.input),
        attempts: [
          attempt(fixture.expected, { state: "equal" }),
          attempt(fixture.expected, { state: "equal" }),
        ],
      },
      native,
      comparison: reason ? { state: "not-compared" } : { state: "equal" },
    };
  });
  return {
    schema: "vize.differential.result",
    version: 1,
    product: "linter",
    sourceRevision: receipt.source.sourceRevision,
    manifestSha256: loaded.manifestSha256,
    buildReceipt: receipt,
    rows,
    summary: {
      plannedCases: loaded.cases.length,
      legacyMatches: loaded.cases.length,
      legacyFailures: 0,
      baselineDrift: 0,
      nativeUnsupported: loaded.cases.length - 17,
      nativeFailures: 0,
      nativeHandled: 17,
      nativeEquivalent: 17,
      pairedComparisons: 17,
    },
  };
}
function changePacket(row: any, mutate: (packet: any) => void) {
  for (const raw of row.native.attempts) {
    const packet = JSON.parse(Buffer.from(raw.stdoutBase64, "base64").toString());
    mutate(packet);
    const bytes = Buffer.from(`${JSON.stringify(packet)}\n`);
    raw.stdoutBase64 = bytes.toString("base64");
    raw.stdoutSha256 = sha256(bytes);
  }
}

void test("shared acceptance counts only validated whole-product linter observations", () => {
  const acceptance = summarizeLinterAcceptance(loaded, baseline(), receipt);
  assert.deepEqual(acceptance.total, {
    planned: loaded.cases.length,
    nativeHandled: 17,
    nativeEquivalent: 17,
    unsupported: loaded.cases.length - 17,
    legacyBacked: 0,
    unverified: 0,
  });
  assert.deepEqual(acceptance.targets.lint, acceptance.total);
});

void test("whole native linter report rejects forged output, refusal, repeat and source custody", () => {
  const valid = baseline();
  validateLinterReport(loaded, valid, receipt);
  const handled = valid.rows.findIndex((row: any) => row.native.state === "completed");
  const unprovided = valid.rows.findIndex(
    (row: any) => row.native.reason?.kind === "UnprovidedRule",
  );
  const unavailable = valid.rows.findIndex(
    (row: any) => row.native.reason?.kind === "ApiUnavailable",
  );
  const entry = valid.rows.findIndex((row: any) => row.native.reason?.kind === "EntryUnavailable");
  const mutations = [
    (r: any) => r.rows[handled].native.attempts.pop(),
    (r: any) => r.rows[unprovided].native.attempts.pop(),
    (r: any) => {
      r.rows[handled].native.inputSha256 = "0".repeat(64);
    },
    (r: any) => {
      r.rows[handled].native.argv = ["--current-api"];
    },
    (r: any) => {
      r.rows[handled].native.attempts[0].exitStatus = 1;
    },
    (r: any) => {
      r.rows[unprovided].native.attempts[0].exitStatus = 1;
    },
    (r: any) => {
      r.rows[handled].native.attempts[0].signal = "SIGTERM";
    },
    (r: any) => {
      r.rows[handled].native.attempts[0].processError = "timeout";
    },
    (r: any) => {
      r.rows[handled].native.attempts[0].stdoutSha256 = "0".repeat(64);
    },
    (r: any) =>
      changePacket(r.rows[unprovided], (packet) => {
        packet.reason.kind = "Recovered";
        packet.reason.detail = "Recovered { offset: 0 }";
      }),
    (r: any) =>
      changePacket(r.rows[unprovided], (packet) => {
        packet.reason.detail = 'UnprovidedRule { rule: "wrong/rule" }';
      }),
    (r: any) =>
      changePacket(r.rows[entry], (packet) => {
        packet.reason.entry = "template";
      }),
    (r: any) =>
      changePacket(r.rows[unavailable], (packet) => {
        packet.reason.detail = "native whole report API is not provided but clean";
      }),
    (r: any) =>
      changePacket(r.rows[handled], (packet) => {
        packet.observation = packet.observation.replace("Case {", "WrongCase {");
      }),
    (r: any) =>
      changePacket(r.rows[handled], (packet) => {
        packet.observation = "Case {}\nObservation {}\n";
      }),
    (r: any) => {
      const raw = r.rows[handled].native.attempts[1];
      raw.stdoutBase64 = Buffer.from("{}\n").toString("base64");
      raw.stdoutSha256 = sha256(Buffer.from("{}\n"));
    },
    (r: any) => {
      r.rows[handled].native.provenance.sourceRevision = "b".repeat(40);
    },
    (r: any) => {
      r.rows[handled].native.provenance.buildReceiptSha256 = "0".repeat(64);
    },
    (r: any) => {
      r.rows[handled].native.provenance.contributions[0].fallback = true;
    },
    (r: any) => {
      r.rows[handled].native.provenance.contributions[0].stage = "L2";
    },
    (r: any) => {
      r.rows[handled].native.provenance.contributions.pop();
    },
    (r: any) => {
      r.rows[handled].native.verdict = "baseline-drift";
    },
    (r: any) => {
      r.rows[handled].comparison.state = "different";
    },
    (r: any) => {
      r.rows[entry].comparison.state = "equal";
    },
    (r: any) => {
      r.rows[unprovided].native.state = "completed";
      changePacket(r.rows[unprovided], (packet) => {
        delete packet.reason;
        packet.state = "handled";
        packet.observation = "Case {}\nObservation {}\n";
      });
    },
    (r: any) => {
      r.rows[handled].native.state = "unsupported";
      changePacket(r.rows[handled], (packet) => {
        delete packet.observation;
        packet.state = "unsupported";
        packet.reason = {
          api: "--native-current-api",
          entry: "template",
          kind: "UnprovidedRule",
          detail: 'UnprovidedRule { rule: "vue/component-definition-name-casing" }',
        };
      });
    },
    (r: any) => {
      r.buildReceipt.probes.pop();
    },
  ];
  for (const mutate of mutations) {
    const report = structuredClone(valid);
    mutate(report);
    assert.throws(() => validateLinterReport(loaded, report, receipt));
  }
});

void test("a failed native process still retains both fresh failed attempts", () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-linter-failure-"));
  try {
    const binary = path.join(dir, "failed-observer");
    const count = path.join(dir, "attempts");
    fs.writeFileSync(
      binary,
      `#!/bin/sh\nprintf 'attempt\\n' >> '${count}'\nprintf 'malformed outcome\\n'\nexit 1\n`,
      { mode: 0o755 },
    );
    const actual = runNative(binary, loaded.cases[0], receipt);
    assert.equal(actual.state, "failed");
    assert.equal(actual.attempts.length, 2);
    assert.deepEqual(
      actual.attempts.map((attempt: any) => attempt.exitStatus),
      [1, 1],
    );
    assert.equal(fs.readFileSync(count, "utf8"), "attempt\nattempt\n");
    assert.equal(actual.provenance, undefined);
    assert.equal(actual.reason, undefined);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});
