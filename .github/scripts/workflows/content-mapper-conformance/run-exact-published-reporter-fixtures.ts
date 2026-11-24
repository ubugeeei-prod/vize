import { requiredEnv } from "../shared/files.ts";
import { pathToFileURL } from "node:url";
import { resolve as resolveWorkflowInput } from "node:path";
import fs from "node:fs";
const { officialActorInputs } = (await import(
  pathToFileURL(resolveWorkflowInput("./tools/support/compat/content-mapper/official-fixtures.ts"))
    .href
)) as typeof import("../../../../tools/support/compat/content-mapper/official-fixtures.ts");
const keys = [
  "PUBLIC_CUT",
  "PUBLIC_HEAD",
  "PUBLIC_TAG",
  "PUBLIC_SOURCE_PR",
  "PUBLIC_RELEASE_RUN",
  "PUBLIC_ACTOR",
];
const inputs = Object.fromEntries(keys.map((key) => [key, process.env[key]]));
officialActorInputs(inputs);
fs.writeFileSync(requiredEnv("PUBLIC_INPUTS"), JSON.stringify(inputs, null, 2) + "\n");
