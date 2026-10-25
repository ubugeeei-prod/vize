import { observeSourceLspSelection } from "./lsp.mjs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export async function observeSharedApiCampaigns(observe = observeSourceLspSelection) {
  const failures = [];
  for (const campaign of ["shared-api-reopen-original-702", "shared-api-reopen-702"]) {
    try {
      await observe({ campaign });
    } catch (error) {
      failures.push(error);
    }
  }
  if (failures.length !== 0)
    throw new AggregateError(failures, "every independent shared API campaign must qualify");
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await observeSharedApiCampaigns();
