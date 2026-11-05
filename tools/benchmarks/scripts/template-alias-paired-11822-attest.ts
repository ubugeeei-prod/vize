import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { childEnvironment, write } from "./template-alias-paired-11822-custody.ts";
import { recordedStage } from "./template-alias-paired-11822-process.ts";
import type { ProbeIdentity, SourceReceipt } from "./template-alias-paired-11822-types.ts";

export function attestSource(directory: string, id: string, source: SourceReceipt) {
  recordedStage({
    directory,
    id: "native-probe-" + id,
    command: process.execPath,
    args: [
      join(dirname(fileURLToPath(import.meta.url)), "template-alias-paired-11822-probe.ts"),
      source.cliPackage,
    ],
    cwd: source.cliPackage,
    environment: childEnvironment(),
    sourceSha: source.source.sha,
    maxMilliseconds: 30_000,
  });
  const records = readFileSync(join(directory, "native-probe-" + id + ".stderr"), "utf8")
    .split("\n")
    .filter((line) => line.startsWith("TEMPLATE_ALIAS_NATIVE_IDENTITY "));
  assert.equal(records.length, 1);
  const identity = JSON.parse(
    records[0].slice("TEMPLATE_ALIAS_NATIVE_IDENTITY ".length),
  ) as ProbeIdentity;
  assert.deepEqual(identity.images, [{ path: source.image.path, sha256: source.image.sha256 }]);
  assert.equal(identity.token, source.token);
  assert.ok(identity.sharedObjects.includes(source.image.path));
  assert.equal(identity.nativeEntrypoint, join(source.nativePackage, "index.js"));
  write(join(directory, "native-probe-" + id + ".qualified.json"), {
    timingEligible: false,
    identity,
    physicalDeclarations: source.physicalDeclarations,
    scope:
      "normal capsule SDK load and version probe; actual mapper image observation is separately required in memory runs",
  });
}
