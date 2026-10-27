// Live negative control: a genuine historical release must not be upgraded into new producer authority.
import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { readReleaseIdentity } from "../../../tools/support/release/release/quota_ownership.ts";
import {
  CRATE_INVENTORY_PATH,
  decodeCrateInventoryFile,
  releaseCrateInventory,
} from "../../../tools/support/release/release/quota/inventory.ts";
import { githubJson, boundedBytes } from "../../../tools/support/release/release/quota/http.ts";
import {
  MissingProducerAttestation,
  readProducedCandidate,
} from "../../../tools/support/release/release/quota/producer.ts";
import { readOwnedCrateSource } from "../../../tools/support/release/release/quota/crate_source.ts";

async function main(output: string): Promise<void> {
  const temp = process.env.RUNNER_TEMP;
  if (!temp || !path.resolve(output).startsWith(path.resolve(temp) + path.sep))
    throw new Error("owned live-refusal output required");
  const version = "0.602.0",
    candidate = "92223735e9c41cefbfe584d7f364e4847569c92b",
    run = "40550456626";
  const identity = await readReleaseIdentity(version, candidate, run, githubJson);
  const inventory = decodeCrateInventoryFile(
    await githubJson(`contents/${CRATE_INVENTORY_PATH}?ref=${candidate}`),
  );
  const owned = { ...identity, inventoryGitBlob: inventory.gitBlob };
  let refusal: MissingProducerAttestation | null = null;
  try {
    await readProducedCandidate(owned);
  } catch (error) {
    if (!(error instanceof MissingProducerAttestation)) throw error;
    refusal = error;
  }
  if (refusal === null)
    throw new Error("legacy recovery must refuse; never synthesize a producer artifact");
  const names = releaseCrateInventory(inventory.source),
    publicSourceProofs = [];
  for (let index = 0; index < names.length; index += 4) {
    const batch = await Promise.all(
      names.slice(index, index + 4).map(async (crate) => {
        const response = await fetch(
          `https://crates.io/api/v1/crates/${crate}/${version}?vize_legacy_source=${Date.now()}`,
          {
            headers: {
              "User-Agent": "vize-release-quota (https://github.com/ubugeeei-prod/vize)",
              "Cache-Control": "no-cache",
            },
            redirect: "error",
            signal: AbortSignal.timeout(15_000),
          },
        );
        if (response.status !== 200) throw new Error("legacy exact crate metadata unavailable");
        const value: unknown = JSON.parse(
          (await boundedBytes(response, 2 * 1024 * 1024)).toString("utf8"),
        );
        if (!value || typeof value !== "object" || Array.isArray(value))
          throw new Error("legacy crate object required");
        const item = (value as Record<string, unknown>).version;
        if (!item || typeof item !== "object" || Array.isArray(item))
          throw new Error("legacy exact version required");
        const v = item as Record<string, unknown>,
          trust = v.trustpub_data as Record<string, unknown> | undefined;
        if (
          v.crate !== crate ||
          v.num !== version ||
          typeof v.checksum !== "string" ||
          trust?.provider !== "github" ||
          trust.repository !== "ubugeeei-prod/vize" ||
          String(trust.run_id) !== run ||
          trust.sha !== identity.workflowHead
        )
          throw new Error("legacy public producer metadata differs");
        return { crate, ...(await readOwnedCrateSource(crate, version, v.checksum, candidate)) };
      }),
    );
    publicSourceProofs.push(...batch);
  }
  const receipt = {
    schema: "vize.releaseQuotaLegacyRefusal.v1",
    version,
    identity: owned,
    expectedRefusalObserved: true,
    admissionAccepted: false,
    producerAttestationSynthesized: false,
    errorCode: refusal.code,
    error: refusal.message,
    originalArtifactInventory: refusal.witness,
    publicSourceProofs,
    publicSourceProofsDoNotGrantRecoveryAdmission: true,
  };
  mkdirSync(path.dirname(output), { recursive: true });
  writeFileSync(output, JSON.stringify(receipt, null, 2) + "\n");
  console.log(
    JSON.stringify({
      version,
      admission: "REFUSED",
      expectedRefusalObserved: true,
      publicCrateSourceProofs: publicSourceProofs.length,
      producerArtifactSynthesized: false,
    }),
  );
}
if (import.meta.main) await main(process.argv[2] ?? "");
