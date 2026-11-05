import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { asRecord } from "../../../tools/support/json-value.ts";

const successor = asRecord(
  JSON.parse(
    fs.readFileSync(
      new URL(
        "../../_fixtures/differential/lint/setup-export-default-8090/require-expose-current-packet.json",
        import.meta.url,
      ),
      "utf8",
    ),
  ),
);
assert.equal(successor.schema, "vize.setup-export.require-expose-successor");
assert.equal(successor.version, 1);

const digest = (value: unknown): string =>
  createHash("sha256").update(JSON.stringify(value)).digest("hex");

// #19f7ff0de8 added vue/require-expose for an Options API object without
// expose. Only this all-rules route contains an export-default object.
// The original 70 packets remain immutable; actual diagnostics are untouched.
export function setupExportCurrentPacket<Route extends { readonly id: string }>(
  entry: { readonly id: string; readonly sourceSha256: string },
  route: Route,
  expected: unknown,
): unknown {
  if (entry.id !== successor.case || route.id !== successor.route) return expected;
  assert.equal(entry.sourceSha256, successor.sourceSha256, "whole UTF8/CRLF input identity");
  assert.equal(digest(route), successor.routeSha256, "whole original route identity");
  assert.equal(
    digest(expected),
    successor.beforePacketSha256,
    "whole prior diagnostic packet identity",
  );
  return structuredClone(successor.packet);
}
