import { requiredEnv } from "../../../../tools/ci/lib/scripts/files.ts";
import { writeFileSync } from "node:fs";
import { join } from "node:path";
writeFileSync(
  join(requiredEnv("VIZE_DEFAULT_PROP_CAPTURE"), "imported-runtime-props.runtime.json"),
  JSON.stringify({ sourceSha: process.env.SOURCE_SHA, nativeBinary: process.env.CORSA_PATH }) +
    "\n",
);
