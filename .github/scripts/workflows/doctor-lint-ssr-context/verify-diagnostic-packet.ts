import assert from "node:assert/strict";
import { join } from "node:path";
import { readJson, requiredEnv } from "../shared/files.ts";

const root = requiredEnv("VIZE_SSR_CONTEXT_ARTIFACTS");
const packet = readJson<{
  findings: { code: string }[];
  summary: { hasBlockingErrors: boolean; counts: { warnings: number } };
}>(join(root, "client-only.doctor.json"));
const codes = new Set(packet.findings.map((finding) => finding.code));
for (const code of ["VIZE_DOCTOR_CF_BROWSER_API_SSR"]) assert.ok(codes.has(code), code);
assert.equal(packet.summary.counts.warnings, 2);
const lint = readJson<{ messages: { ruleId: string }[] }[]>(join(root, "client-only.lint.json"));
assert.deepEqual(
  lint.flatMap((file) => file.messages.map((message) => message.ruleId)),
  Array(2).fill("ssr/no-browser-globals-in-ssr"),
);
