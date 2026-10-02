import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  advisoryId,
  assertCleanReport,
  assertRemediatedReport,
  verifyInstalledForge,
} from "../../support/security/node-forge-proof.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const result = spawnSync(
  "vp",
  ["exec", "pnpm", "audit", "--prod", "--audit-level", "moderate", "--json"],
  {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  },
);
process.stdout.write(result.stdout ?? "");
process.stderr.write(result.stderr ?? "");
if (result.error || result.signal || result.status === null) {
  console.error("npm-audit: transport/process failure", result.error ?? result.signal);
  process.exit(1);
}
if (result.status !== 0 && result.status !== 1) process.exit(result.status);
try {
  const report = JSON.parse(result.stdout);
  if (result.status !== 0) assertRemediatedReport(report);
  else assertCleanReport(report);
  const proof = verifyInstalledForge(root);
  console.log(
    "npm-audit: verified Node listhen source remediation",
    JSON.stringify({ advisory: advisoryId, ...proof }),
  );
  console.log(
    "npm-audit: upstream version remains 1.4.0; browser dist is outside this Node-only proof",
  );
} catch (error) {
  console.error("npm-audit: unresolved finding or invalid source attestation:", error);
  process.exit(result.status || 1);
}
