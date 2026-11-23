import assert from "node:assert/strict";
import { join } from "node:path";
import { readJson, requiredEnv } from "../shared/files.ts";

const root = requiredEnv("VIZE_BARREL_PROPS_ARTIFACTS");
const packet = readJson<{
  findings: { code: string }[];
  summary: { hasBlockingErrors: boolean; counts: { warnings: number } };
}>(join(root, "barrel.json"));
const codes = new Set(packet.findings.map((finding) => finding.code));
for (const code of ["VIZE_DOCTOR_CF_MISSING_REQUIRED_PROP"]) assert.ok(codes.has(code), code);
assert.ok(packet.summary.hasBlockingErrors);
