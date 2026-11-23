import { requiredEnv } from "../shared/files.ts";
import { readFileSync, copyFileSync } from "node:fs";
import { join } from "node:path";
const directory = join(requiredEnv("RUNNER_TEMP"), "native");
const receipt = JSON.parse(
  readFileSync(join(directory, `build-receipt.${process.env.IMAGE_TARGET}.json`), "utf8"),
);
copyFileSync(join("npm/native", receipt.nodeFile), join(directory, receipt.nodeFile));
