import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
const identify = (path: string) => {
  const bytes = readFileSync(path),
    version = spawnSync(path, ["--version"]);
  return {
    path,
    sha256: createHash("sha256").update(bytes).digest("hex"),
    version: {
      status: version.status,
      stdout: version.stdout.toString("base64"),
      stderr: version.stderr.toString("base64"),
    },
  };
};
mkdirSync("target/unicode-prop-packets", { recursive: true });
const fixture = readFileSync(
  "tests/_fixtures/differential/typechecker/unicode-prop-bindings-10573/cases.json",
);
const receipt = {
  source: process.env.HEAD_SHA,
  mapperSource: process.env.CONTENT_MAPPER_TYPESCRIPT_SHA,
  mapper: identify(`${process.env.RUNNER_TEMP}/tsgo`),
  native: identify("node_modules/@typescript/typescript-linux-x64/lib/tsc"),
  cli: identify("target/ci/vize"),
  fixtureSha256: createHash("sha256").update(fixture).digest("hex"),
};
writeFileSync(
  "target/unicode-prop-packets/providers.json",
  JSON.stringify(receipt, null, 2) + "\n",
);
