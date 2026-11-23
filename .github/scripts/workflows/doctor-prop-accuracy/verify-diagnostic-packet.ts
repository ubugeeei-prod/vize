import assert from "node:assert/strict";
import { join } from "node:path";
import { readJson, requiredEnv } from "../shared/files.ts";

const root = requiredEnv("VIZE_PROP_ACCURACY_ARTIFACTS");
const packet = readJson<{
  findings: { code: string }[];
  summary: { hasBlockingErrors: boolean; counts: { warnings: number } };
}>(join(root, "models-alias-fallthrough.json"));
const codes = new Set(packet.findings.map((finding) => finding.code));
for (const code of ["VIZE_DOCTOR_CF_PROP_TYPE_MISMATCH", "VIZE_DOCTOR_CF_UNDECLARED_PROP"])
  assert.ok(codes.has(code), code);
assert.ok(packet.summary.hasBlockingErrors);
