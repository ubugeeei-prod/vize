import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { asRecord } from "../../tools/support/json-value.ts";
import { currentRuleDocumentationPacket } from "./support/current-rule-documentation.ts";
import { withoutHelp } from "./support/lint-help-policy.mjs";
import { setupExportCurrentPacket } from "./support/setup-export-current-packet.ts";

const controls = JSON.parse(
  fs.readFileSync(
    new URL(
      "../_fixtures/differential/lint/setup-export-default-8090/default-route-controls.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as {
  cases: { id: string; sourceSha256: string; routes: { id: string; afterCliPacket: unknown }[] }[];
};

test("the full setup-export successor excludes follow-on component warnings", () => {
  const changed: string[] = [];
  for (const entry of controls.cases)
    for (const route of entry.routes) {
      const prior = currentRuleDocumentationPacket(withoutHelp(route.afterCliPacket));
      const current = setupExportCurrentPacket(entry, route, prior);
      if (current === prior) continue;
      changed.push(`${entry.id}/${route.id}`);
      assert.ok(Array.isArray(current));
      assert.equal(current.length, 1);
      const row = asRecord(current[0]);
      assert.equal(row.file, prior[0].file);
      assert.equal(row.errorCount, prior[0].errorCount);
      assert.equal(row.warningCount, 0);
      assert.ok(Array.isArray(row.messages));
      assert.deepEqual(row.messages, prior[0].messages);
      assert.deepEqual(current, prior);
    }
  assert.deepEqual(changed, ["runtime-two-utf8-crlf/all"]);
});

test("changed inputs, routes and prior diagnostic packets are refused", () => {
  const entry = controls.cases.find((entry) => entry.id === "runtime-two-utf8-crlf")!;
  const route = entry.routes.find((route) => route.id === "all")!;
  const prior = currentRuleDocumentationPacket(withoutHelp(route.afterCliPacket));
  for (const [input, request, expected] of [
    [{ ...entry, sourceSha256: "changed" }, route, prior],
    [entry, { ...route, extra: true }, prior],
    [entry, route, []],
  ] as const)
    assert.throws(() => setupExportCurrentPacket(input, request, expected), {
      code: "ERR_ASSERTION",
    });
});
