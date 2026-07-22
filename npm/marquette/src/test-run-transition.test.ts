import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import type { TestRunAdmissionDecision } from "./test-run-admission.js";
import {
  canonicalTestRunTransitionJson,
  testRunTransitionFingerprint,
  validateTestRunTransition,
  verifyTestRunTransition,
  type TestRunTransition,
} from "./test-run-transition.js";

interface SharedTransitionCase {
  readonly name: string;
  readonly family: string;
  readonly current: string;
  readonly previous: string | null;
  readonly decision: TestRunAdmissionDecision;
}

interface SharedTransitionDocument {
  readonly transitions: Readonly<Record<string, TestRunTransition>>;
  readonly cases: readonly SharedTransitionCase[];
}

function readDocument(): SharedTransitionDocument {
  return JSON.parse(
    readFileSync(
      new URL(
        "../../../tests/fixtures/test-run-evidence/transition-decisions.json",
        import.meta.url,
      ),
      "utf8",
    ),
  ) as SharedTransitionDocument;
}

function transitionNamed(document: SharedTransitionDocument, name: string): TestRunTransition {
  const transition = document.transitions[name];
  assert.ok(transition, `fixture transition ${name} must exist`);
  return transition;
}

void test("the fixture chain links by canonical fingerprint", async () => {
  const document = readDocument();
  const genesis = transitionNamed(document, "genesis");
  const second = transitionNamed(document, "second");
  assert.deepEqual(validateTestRunTransition(genesis), []);
  assert.equal(second.previous, await testRunTransitionFingerprint(genesis));

  const shuffled = { ...second, accepted: [...second.accepted].reverse() };
  assert.equal(canonicalTestRunTransitionJson(shuffled), canonicalTestRunTransitionJson(second));
});

void test("reproduces every shared transition decision", async () => {
  const document = readDocument();
  assert.ok(document.cases.length > 0);

  for (const sharedCase of document.cases) {
    const current = transitionNamed(document, sharedCase.current);
    const previous =
      sharedCase.previous === null ? null : transitionNamed(document, sharedCase.previous);
    const decision = await verifyTestRunTransition(current, previous);
    assert.deepEqual(
      JSON.parse(JSON.stringify(decision)),
      sharedCase.decision,
      `decision mismatch for case ${sharedCase.name}`,
    );
  }
});
