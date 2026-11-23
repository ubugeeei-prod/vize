import { requiredEnv } from "../shared/files.ts";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
const directory = requiredEnv("RUNNER_TEMP");
const receipt = {
  orchestration: execFileSync("git", ["rev-parse", "HEAD", "HEAD^{tree}"], { encoding: "utf8" })
    .trim()
    .split("\n"),
  helpers: [
    "native-fault-evidence.ts",
    "native-fault-core.ts",
    "native-fault-support.ts",
    "native-fault-retention.ts",
    "native-fault-resources.ts",
  ].map((file) => ({
    file,
    sha256: createHash("sha256")
      .update(readFileSync(path.join(directory, file)))
      .digest("hex"),
  })),
  reportedFailureHead: "0b82a31fc9dd8634786fc17ec3bd3eeb8427ae6e",
  physicalFailedSource: "0b82a31fc9dd8634786fc17ec3bd3eeb8427ae6e",
  physicalFailedTree: "b865787dc492550b568d7cea3bcf458f75b11cfb",
  originalRun: 41700735853,
  originalFailedJob: 126145257863,
};
writeFileSync(
  path.join(directory, "js-native-fault-evidence-orchestration/identity.json"),
  `${JSON.stringify(receipt, null, 2)}\n`,
);
