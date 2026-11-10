import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { requireVizeAndCorsaBins } from "../_helpers/apps.ts";
import { qualifyEcosystemProducerDrift } from "../_helpers/ecosystem-producer-qualification.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test(
  "ecosystem producer drift is qualified before accepting snapshots",
  {
    timeout: 300_000,
  },
  () => {
    requireVizeAndCorsaBins();
    qualifyEcosystemProducerDrift(repoRoot);
  },
);
