/**
 * `@vizejs/vite-plugin` vs `@vitejs/plugin-vue` option parity gate (#3227).
 *
 * The drop-in claim is only worth as much as the option surface behind it, so
 * this test enumerates the installed `@vitejs/plugin-vue` surface and requires
 * every option, `Api` member, and plugin hook to be either proven honored by a
 * behavioral probe below, or recorded as an explicit gap in
 * `tests/_fixtures/vite-plugin-vue-option-parity.json`.
 */

import assert from "node:assert/strict";
import { test } from "node:test";

import {
  honoredEvidence,
  readLedger,
  upstreamSurface,
  validateLedger,
} from "./_helpers/vite-plugin-vue-parity.ts";
import { probeHookImplemented, probes } from "./_helpers/vite-plugin-vue-behavior-probes.ts";

test("the parity ledger stays exhaustive over the pinned @vitejs/plugin-vue surface", () => {
  const surface = upstreamSurface();
  const ledger = readLedger();
  validateLedger(ledger, surface);

  assert.ok(ledger.summary.honored > 0, "the ledger must record the surface Vize does honor");
  assert.equal(ledger.summary.unimplemented, 0, "no unchecked plugin-vue parity gaps may remain");
});

test("every option the ledger calls honored is backed by a behavioral probe", async () => {
  const evidence = honoredEvidence(readLedger());
  assert.notEqual(evidence.size, 0, "at least one entry must be proven honored");

  const executed = new Set<string>();
  for (const [entry, evidenceId] of evidence) {
    if (evidenceId === "hook-implemented") {
      probeHookImplemented(entry.slice("hooks.".length));
      continue;
    }
    const probe = probes.get(evidenceId);
    assert.ok(probe, `${entry} names unknown evidence ${JSON.stringify(evidenceId)}`);
    if (!executed.has(evidenceId)) {
      await probe();
      executed.add(evidenceId);
    }
  }

  assert.deepEqual(
    [...probes.keys()].filter((id) => !executed.has(id)),
    [],
    "every behavioral probe must back at least one honored ledger entry",
  );
});
