import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";

import { assertPublicKitAudit } from "../../tools/support/security/nuxt-kit-consumer.ts";
import { record } from "../../tools/support/security/braces-lock.ts";

function packet(version: string): unknown {
  return JSON.parse(
    fs.readFileSync(
      new URL(`./fixtures/nuxt-kit-consumer/public-${version}-audit.json.txt`, import.meta.url),
      "utf8",
    ),
  );
}

test("the complete real public 3.11.2 audit remains refused without workspace exceptions", () => {
  const original = packet("3.11.2");
  const before = structuredClone(original);
  assert.throws(() => assertPublicKitAudit(original, 1));
  // A forged success exit cannot admit the unchanged complete report either.
  assert.throws(() => assertPublicKitAudit(original, 0));
  assert.deepEqual(original, before);
});

test("the complete fresh candidate audit passes; errors and contradictory packets cannot", () => {
  const original = packet("3.21.8");
  const before = structuredClone(original);
  assertPublicKitAudit(original, 0);
  assert.deepEqual(original, before);
  for (const status of [null, 1, 2]) assert.throws(() => assertPublicKitAudit(original, status));
  const counts = (report: Record<string, unknown>) =>
    record(record(report.metadata).vulnerabilities);
  const mutations: Array<(report: Record<string, unknown>) => void> = [
    (report) => {
      report.error = { code: "EAI_AGAIN" };
    },
    (report) => {
      delete report.metadata;
    },
    (report) => {
      delete record(report.metadata).vulnerabilities;
    },
    (report) => {
      delete counts(report).critical;
    },
    (report) => {
      counts(report).high = 1;
    },
    (report) => {
      counts(report).low = 1;
    },
    (report) => {
      report.advisories = { hidden: { severity: "critical" } };
    },
    (report) => {
      delete report.advisories;
    },
  ];
  for (const mutate of mutations) {
    const changed = record(structuredClone(original));
    mutate(changed);
    assert.throws(() => assertPublicKitAudit(changed, 0));
  }
});
