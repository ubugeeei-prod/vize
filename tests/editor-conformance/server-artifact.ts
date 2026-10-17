// Bind downloaded editor test servers and host observations to the source build.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import fs from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";

type Artifact = { sourceSha: string; sha256: string };

export async function sha256(file: string): Promise<string> {
  const digest = createHash("sha256");
  for await (const chunk of createReadStream(file)) digest.update(chunk);
  return digest.digest("hex");
}

function sourceIdentity(sourceSha: string) {
  assert.match(sourceSha, /^[a-f0-9]{40}$/, "A complete source commit is required");
}

export async function writeArtifact(binary: string, receipt: string, sourceSha: string) {
  sourceIdentity(sourceSha);
  await fs.writeFile(receipt, JSON.stringify({ sourceSha, sha256: await sha256(binary) }, null, 2));
}

export async function verifyArtifact(binary: string, receipt: string, sourceSha: string) {
  sourceIdentity(sourceSha);
  const artifact = JSON.parse(await fs.readFile(receipt, "utf8")) as Artifact;
  assert.equal(artifact.sourceSha, sourceSha, "The server must come from this source commit");
  assert.equal(artifact.sha256, await sha256(binary), "The downloaded server digest must match");
  return artifact;
}

export async function verifyHost(artifactFile: string, hostFile: string, sourceSha: string) {
  sourceIdentity(sourceSha);
  const artifact = JSON.parse(await fs.readFile(artifactFile, "utf8")) as Artifact;
  const host = JSON.parse(await fs.readFile(hostFile, "utf8"));
  assert.equal(artifact.sourceSha, sourceSha);
  assert.equal(host.sourceSha, sourceSha, "Host evidence must describe this exact source commit");
  assert.equal(host.passed, true, "A skipped or failed host cannot qualify");
  assert.equal(host.binary.sha256, artifact.sha256, "The host must run the source-built server");
  assert.ok(Number.isInteger(host.binary.nativePid) && host.binary.nativePid > 0);
  assert.ok(Array.isArray(host.cases) && host.cases.length > 0, "Host cases must execute");
  const transcript = path.join(path.dirname(hostFile), path.basename(host.transcript.path));
  assert.equal(
    host.transcript.sha256,
    await sha256(transcript),
    "Keep the complete recorded packets",
  );
}

export async function verifyScorecard(artifactFile: string, resultFile: string, sourceSha: string) {
  const result = JSON.parse(await fs.readFile(resultFile, "utf8"));
  assert.equal(result.serverInfo?.source, "configured");
  assert.equal(result.serverInfo?.status, "ready");
  const binary = result.serverInfo.path;
  assert.ok(path.isAbsolute(binary), "The actual packaged host must identify its selected server");
  const artifact = await verifyArtifact(binary, artifactFile, sourceSha);
  await fs.writeFile(
    path.join(path.dirname(resultFile), "scorecard-receipt.json"),
    JSON.stringify(
      {
        passed: true,
        sourceSha,
        binary: { path: binary, sha256: artifact.sha256 },
        result: { path: resultFile, sha256: await sha256(resultFile) },
      },
      null,
      2,
    ),
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [command, first, second, sourceSha] = process.argv.slice(2);
  assert.ok(
    first && second && sourceSha,
    "Usage: server-artifact.ts write|verify|host FILE RECEIPT SHA",
  );
  if (command === "write") await writeArtifact(first, second, sourceSha);
  else if (command === "verify") await verifyArtifact(first, second, sourceSha);
  else if (command === "host") await verifyHost(first, second, sourceSha);
  else if (command === "scorecard") await verifyScorecard(first, second, sourceSha);
  else assert.fail(`Unknown artifact command: ${command}`);
}
